use super::Store;
use crate::{
    access::{self, Kind, Projection},
    sql,
};
use rusqlite::params;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};

fn projection(row: &rusqlite::Row<'_>, start: usize) -> rusqlite::Result<Projection> {
    Ok(Projection {
        score: row.get(start)?,
        as_of: row.get::<_, i64>(start + 1)? as u64,
        reads: row.get(start + 2)?,
        uses: row.get(start + 3)?,
    })
}

impl Store {
    pub fn access_export(&self, id: i64) -> Result<access::History, String> {
        let mut statement = self
            .conn
            .prepare(sql::ACCESS_EVENTS_EXPORT)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([id], |row| {
                let kind: String = row.get(2)?;
                Ok(access::Event {
                    actor: row.get(0)?,
                    event_key: row.get(1)?,
                    kind: if kind == "read" {
                        Kind::Read
                    } else {
                        Kind::Use
                    },
                    occurred_at: row.get::<_, i64>(3)? as u64,
                    weight: row.get(4)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let events = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut statement = self
            .conn
            .prepare(sql::ACCESS_BUCKETS_FOR)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([id], |row| {
                Ok(access::Bucket {
                    end: row.get::<_, i64>(0)? as u64,
                    score: row.get(1)?,
                    reads: row.get(2)?,
                    uses: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let buckets = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(access::History {
            scoring_version: access::SCORING_VERSION,
            events,
            buckets,
        })
    }

    /// Restore frozen event weights, not a replay through current credit-window policy.
    pub fn access_import(&self, id: i64, history: &access::History) -> Result<(), String> {
        history.validate()?;
        for event in &history.events {
            let inserted = self
                .conn
                .execute(
                    sql::ACCESS_EVENT_INSERT,
                    params![
                        id,
                        event.actor,
                        event.event_key,
                        event.kind.as_str(),
                        event.occurred_at as i64,
                        event.weight
                    ],
                )
                .map_err(|e| e.to_string())?;
            if inserted != 1 {
                return Err("duplicate exported access event key".into());
            }
        }
        for bucket in &history.buckets {
            self.conn
                .execute(
                    sql::ACCESS_BUCKET_PUT,
                    params![
                        id,
                        bucket.end as i64,
                        bucket.score,
                        bucket.reads,
                        bucket.uses
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn access_projection(&self, id: i64) -> Result<Projection, String> {
        match self
            .conn
            .query_row(sql::ACCESS_PROJECTION_GET, [id], |row| projection(row, 0))
        {
            Ok(state) => Ok(state),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Projection::default()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn access_projection_all(&self) -> Result<HashMap<i64, Projection>, String> {
        let mut statement = self
            .conn
            .prepare(sql::ACCESS_PROJECTION_ALL)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, projection(row, 1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    fn access_projection_put(&self, id: i64, state: &Projection) -> Result<(), String> {
        self.conn
            .execute(
                sql::ACCESS_PROJECTION_PUT,
                params![id, state.score, state.as_of as i64, state.reads, state.uses],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Event and materialized projection are committed together by the caller's write transaction.
    pub fn access_append(
        &self,
        id: i64,
        actor: &str,
        kind: Kind,
        key: Option<&str>,
        now: u64,
    ) -> Result<(bool, bool), String> {
        let recent: bool = self
            .conn
            .query_row(
                sql::ACCESS_RECENT_CREDIT,
                params![
                    id,
                    actor,
                    kind.as_str(),
                    now.saturating_sub(access::CREDIT_WINDOW_SECONDS) as i64
                ],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let weight = if recent { 0.0 } else { kind.weight() };
        let inserted = self
            .conn
            .execute(
                sql::ACCESS_EVENT_INSERT,
                params![id, actor, key, kind.as_str(), now as i64, weight],
            )
            .map_err(|e| e.to_string())?;
        if inserted == 0 {
            return Ok((false, false));
        }
        let mut state = self.access_projection(id)?;
        state.add(
            weight,
            now,
            i64::from(matches!(kind, Kind::Read)),
            i64::from(matches!(kind, Kind::Use)),
        );
        self.access_projection_put(id, &state)?;
        Ok((true, weight > 0.0))
    }

    pub fn access_compact(&self, now: u64) -> Result<usize, String> {
        let cutoff = now.saturating_sub(access::RAW_RETENTION_SECONDS) / 3600 * 3600;
        let mut statement = self
            .conn
            .prepare(sql::ACCESS_COMPACT_BATCH)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([cutoff as i64], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)? as u64,
                    row.get::<_, f64>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        let events = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut buckets: BTreeMap<(i64, u64), Projection> = BTreeMap::new();
        let mut ids = Vec::new();
        for (event_id, id, kind, time, weight) in events {
            let end = time / 3600 * 3600 + 3600;
            let state = buckets.entry((id, end)).or_default();
            state.add(
                access::decay(weight, time, end),
                end,
                i64::from(kind == "read"),
                i64::from(kind == "use"),
            );
            ids.push(event_id);
        }
        for ((id, end), state) in buckets {
            self.conn
                .execute(
                    sql::ACCESS_BUCKET_PUT,
                    params![id, end as i64, state.score, state.reads, state.uses],
                )
                .map_err(|e| e.to_string())?;
        }
        self.conn
            .execute(
                sql::ACCESS_DELETE_BATCH,
                [serde_json::to_string(&ids).map_err(|e| e.to_string())?],
            )
            .map_err(|e| e.to_string())?;
        Ok(ids.len())
    }

    pub fn access_rebuild(&self) -> Result<usize, String> {
        let mut statement = self
            .conn
            .prepare(sql::ACCESS_MEMORY_IDS)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;
        let ids = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(sql::ACCESS_PROJECTION_CLEAR, [])
            .map_err(|e| e.to_string())?;
        for id in &ids {
            let mut state = Projection::default();
            let mut statement = self
                .conn
                .prepare(sql::ACCESS_BUCKETS_FOR)
                .map_err(|e| e.to_string())?;
            let rows = statement
                .query_map([id], |row| {
                    Ok((
                        row.get::<_, i64>(0)? as u64,
                        row.get::<_, f64>(1)?,
                        row.get::<_, i64>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                let (at, score, reads, uses) = row.map_err(|e| e.to_string())?;
                state.add(score, at, reads, uses);
            }
            let mut statement = self
                .conn
                .prepare(sql::ACCESS_EVENTS_FOR)
                .map_err(|e| e.to_string())?;
            let rows = statement
                .query_map([id], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, i64>(1)? as u64,
                        row.get::<_, f64>(2)?,
                    ))
                })
                .map_err(|e| e.to_string())?;
            for row in rows {
                let (kind, at, weight) = row.map_err(|e| e.to_string())?;
                state.add(
                    weight,
                    at,
                    i64::from(kind == "read"),
                    i64::from(kind == "use"),
                );
            }
            self.access_projection_put(*id, &state)?;
        }
        Ok(ids.len())
    }

    pub fn access_stats(&self) -> Result<Value, String> {
        let total: f64 = self
            .access_projection_all()?
            .values()
            .map(|state| state.value(crate::model::now()))
            .sum();
        self.conn.query_row(sql::ACCESS_STATS, [], |row| Ok(json!({
            "total_reinforcement": total,
            "raw_events": row.get::<_, i64>(0)?, "hourly_buckets": row.get::<_, i64>(1)?, "tracked_memories": row.get::<_, i64>(2)?,
            "scoring_version": access::SCORING_VERSION, "half_life_seconds": access::HALF_LIFE_SECONDS,
            "raw_retention_seconds": access::RAW_RETENTION_SECONDS,
            "dropped_read_events": access::DROPPED_READ_EVENTS.load(std::sync::atomic::Ordering::Relaxed),
        }))).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{
        self,
        test_support::{cleanup, temp_db},
        TxMode,
    };

    #[test]
    fn access_credit_idempotency_compaction_and_restore_preserve_scores() {
        let path = temp_db("access-history");
        let restored = temp_db("access-restored");
        cleanup(&path);
        cleanup(&restored);
        let at = 7200;
        let later = at + 2 * access::RAW_RETENTION_SECONDS;
        let dump = store::with_db_in(&path, TxMode::Write, |st| -> Result<Value, String> {
            let id = st.insert_memory("Access test", "private body", &[], 1, 1)?;
            assert_eq!(
                st.access_append(id, "alice", Kind::Use, Some("first"), at)?,
                (true, true)
            );
            assert_eq!(
                st.access_append(id, "alice", Kind::Use, Some("first"), at + 61)?,
                (false, false)
            );
            assert_eq!(
                st.access_append(id, "alice", Kind::Use, Some("repeat"), at + 10)?,
                (true, false)
            );
            assert_eq!(
                st.access_append(id, "bob", Kind::Use, Some("first"), at + 10)?,
                (true, true)
            );
            assert_eq!(
                st.access_append(id, "alice", Kind::Read, None, at + 10)?,
                (true, true)
            );
            let before = st.access_projection(id)?;
            assert_eq!((before.reads, before.uses), (1, 3));
            assert_eq!(st.access_compact(later)?, 4);
            assert_eq!(st.access_compact(later)?, 0);
            assert_eq!(st.access_rebuild()?, 1);
            let after = st.access_projection(id)?;
            assert!((before.value(later) - after.value(later)).abs() < 1e-12);
            assert_eq!((after.reads, after.uses), (1, 3));
            st.access_append(id, "alice", Kind::Use, Some("new"), later)?;
            let dump = st.export_dump()?;
            assert_eq!(
                dump["memories"]["m1"]["access"]["buckets"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(
                dump["memories"]["m1"]["access"]["events"]
                    .as_array()
                    .unwrap()
                    .len(),
                1
            );
            Ok(dump)
        })
        .unwrap();
        store::with_db_in(&restored, TxMode::Write, |st| -> Result<(), String> {
            st.import_dump(&dump)?;
            let original = Store::open(&path)?.access_projection(1)?;
            let restored = st.access_projection(1)?;
            assert!((original.value(later) - restored.value(later)).abs() < 1e-12);
            assert_eq!((restored.reads, restored.uses), (1, 4));
            st.delete_memories(&[1])?;
            assert_eq!(st.access_stats()?["raw_events"], 0);
            assert_eq!(st.access_stats()?["hourly_buckets"], 0);
            assert_eq!(st.access_stats()?["tracked_memories"], 0);
            Ok(())
        })
        .unwrap();
        cleanup(&path);
        cleanup(&restored);
    }

    #[test]
    fn access_write_rollback_and_best_effort_read_deadline() {
        let path = temp_db("access-rollback");
        cleanup(&path);
        store::with_db_in(&path, TxMode::Write, |st| {
            st.insert_memory("Test", "body", &[], 1, 1)
        })
        .unwrap();
        let failed: Result<(), String> = store::with_db_in(&path, TxMode::Write, |st| {
            st.access_append(1, "alice", Kind::Use, Some("rollback"), 1)?;
            Err("rollback".into())
        });
        assert!(failed.is_err());
        assert_eq!(
            Store::open(&path).unwrap().access_stats().unwrap()["raw_events"],
            0
        );
        store::with_db_in(&path, TxMode::Write, |_st| -> Result<(), String> {
            let before = access::DROPPED_READ_EVENTS.load(std::sync::atomic::Ordering::Relaxed);
            let start = std::time::Instant::now();
            access::record_reads(&path, "reader", &[1]);
            assert!(start.elapsed() < std::time::Duration::from_secs(1));
            assert!(
                access::DROPPED_READ_EVENTS.load(std::sync::atomic::Ordering::Relaxed) > before
            );
            Ok(())
        })
        .unwrap();
        cleanup(&path);
    }

    #[test]
    fn invalid_access_backup_rolls_back_memory_and_history() {
        let path = temp_db("access-invalid-backup");
        cleanup(&path);
        let dump = json!({"tags":{}, "memories":{"m1":{
            "summary":"Restore", "content":"", "access":{"scoring_version":1,"buckets":[],"events":[
                {"actor":"alice","event_key":"key","kind":"use","occurred_at":100,"weight":1.0},
                {"actor":"alice","event_key":"key","kind":"use","occurred_at":101,"weight":1.0}
            ]}
        }}});
        let result = store::with_db_in(&path, TxMode::Write, |st| st.import_dump(&dump));
        assert!(result.is_err());
        let st = Store::open(&path).unwrap();
        assert_eq!(st.stats().unwrap()["memories"], 0);
        assert_eq!(st.access_stats().unwrap()["raw_events"], 0);
        drop(st);
        cleanup(&path);
    }
}
