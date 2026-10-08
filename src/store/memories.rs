//! Memory data operations: insert / query / paged browsing / update / delete.

use crate::model::Memory;
use crate::sql;
use rusqlite::params;
use std::collections::HashMap;

use super::Store;

/// The AND-composed optional filters of a list query: `tag` is one exact tag name (internal use:
/// the memory:// resource face); `id_set` is a JSON array text of internal memory ids (the
/// resolved tag expression, regex atoms included). `None` = no constraint.
#[derive(Default)]
pub struct ListFilter<'a> {
    pub tag: Option<&'a str>,
    pub id_set: Option<&'a str>,
}

impl Store {
    /// Summarize entry ids that normalize to the same value as an existing memory (compared on the Rust side with consistent Unicode semantics).
    pub fn find_duplicates_by_summary(&self, summary: &str) -> Result<Vec<String>, String> {
        let mut st = self
            .conn
            .prepare(sql::MEMORY_SUMMARIES)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        let norm = summary.to_lowercase();
        let mut out = Vec::new();
        for r in rows {
            let (id, s) = r.map_err(|e| e.to_string())?;
            if s.to_lowercase() == norm {
                out.push(Self::format_id(id));
            }
        }
        Ok(out)
    }

    /// Insert a memory and link it by internal tag ids (the handler resolves ids via `link_tags` first).
    pub fn insert_memory(
        &self,
        summary: &str,
        content: &str,
        tag_ids: &[i64],
        created_at: u64,
        updated_at: u64,
    ) -> Result<i64, String> {
        self.validate_tag_constraints(tag_ids)?;
        self.conn
            .execute(
                sql::MEMORY_INSERT,
                params![summary, content, created_at as i64, updated_at as i64],
            )
            .map_err(|e| e.to_string())?;
        let id = self.conn.last_insert_rowid();
        for tag_id in tag_ids {
            self.conn
                .execute(sql::MEMORY_LINK_TAG, params![id, tag_id])
                .map_err(|e| e.to_string())?;
        }
        Ok(id)
    }

    fn memory_by_id(&self, id: i64) -> Result<Option<Memory>, String> {
        let row = self.conn.query_row(sql::MEMORY_BY_ID, [id], |r| {
            let created: i64 = r.get(3)?;
            let updated: i64 = r.get(4)?;
            Ok(Memory {
                id: Self::format_id(r.get(0)?),
                summary: r.get(1)?,
                content: r.get(2)?,
                tags: Vec::new(),
                created_at: created as u64,
                updated_at: updated as u64,
            })
        });
        let mut m = match row {
            Ok(m) => m,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        m.tags = self.tags_of(id)?;
        Ok(Some(m))
    }

    fn tags_of(&self, id: i64) -> Result<Vec<String>, String> {
        let mut st = self
            .conn
            .prepare(sql::MEMORY_TAGS_OF)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([id], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// All memories (with tags, id ascending), for the in-memory searcher.
    pub fn all_memories(&self) -> Result<Vec<Memory>, String> {
        let mut st = self
            .conn
            .prepare(sql::MEMORY_ALL)
            .map_err(|e| e.to_string())?;
        let mut out: Vec<Memory> = st
            .query_map([], |r| {
                let created: i64 = r.get(3)?;
                let updated: i64 = r.get(4)?;
                Ok(Memory {
                    id: Self::format_id(r.get(0)?),
                    summary: r.get(1)?,
                    content: r.get(2)?,
                    tags: Vec::new(),
                    created_at: created as u64,
                    updated_at: updated as u64,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut tag_stmt = self
            .conn
            .prepare(sql::MEMORY_TAG_PAIRS)
            .map_err(|e| e.to_string())?;
        let pairs = tag_stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut by_memory: HashMap<i64, Vec<String>> = HashMap::new();
        for (mid, name) in pairs {
            by_memory.entry(mid).or_default().push(name);
        }
        for m in &mut out {
            let id = Self::parse_id(&m.id).unwrap_or(0);
            if let Some(tags) = by_memory.get(&id) {
                m.tags = tags.clone();
            }
        }
        Ok(out)
    }

    /// Paged browsing with AND-composed optional filters (`filter.tag` is one exact tag;
    /// `filter.id_set` is a JSON array text of internal memory ids, the resolved memory set of a
    /// caller-side tag expression), returns (total count, current page).
    ///
    /// Fully static SQL (see sql/memory_list_page.sql): no filtering while the bound filters are NULL;
    /// id-set filtering is expanded with json_each (SQLite's built-in JSON1); the sort column is chosen among `?3` via CASE;
    /// `?4` carries ±1 for ascending/descending (all columns are integers).
    /// `sort` only accepts values whitelisted by the handler.
    pub fn list_memories(
        &self,
        filter: ListFilter<'_>,
        sort: &str,
        asc: bool,
        offset: u64,
        limit: u64,
    ) -> Result<(u64, Vec<Memory>), String> {
        let ListFilter { tag, id_set } = filter;
        let dir: i64 = if asc { 1 } else { -1 };
        let total: u64 = self
            .conn
            .query_row(sql::MEMORY_LIST_COUNT, params![tag, id_set], |r| {
                r.get::<_, i64>(0)
            })
            .map(|n| n as u64)
            .map_err(|e| e.to_string())?;
        let mut st = self
            .conn
            .prepare(sql::MEMORY_LIST_PAGE)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map(
                params![tag, id_set, sort, dir, limit as i64, offset as i64],
                |r| {
                    let created: i64 = r.get(3)?;
                    let updated: i64 = r.get(4)?;
                    Ok(Memory {
                        id: Self::format_id(r.get(0)?),
                        summary: r.get(1)?,
                        content: r.get(2)?,
                        tags: Vec::new(),
                        created_at: created as u64,
                        updated_at: updated as u64,
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        let mut page: Vec<Memory> = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for m in &mut page {
            let id = Self::parse_id(&m.id).unwrap_or(0);
            m.tags = self.tags_of(id)?;
        }
        Ok((total, page))
    }

    /// Fetch full memories by id in bulk, returning (found, missing).
    pub fn get_memories(&self, ids: &[i64]) -> Result<(Vec<Memory>, Vec<String>), String> {
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for id in ids {
            match self.memory_by_id(*id)? {
                Some(m) => found.push(m),
                None => missing.push(Self::format_id(*id)),
            }
        }
        Ok((found, missing))
    }

    /// Update memory fields and tags; returns whether anything changed. updated_at tracks the
    /// *content* timeline: it is written only when summary/content change (inside
    /// MEMORY_UPDATE_FIELDS), so curating tags never pushes an old memory back to the top of the
    /// default newest-first browse — the field values agents read did not move.
    /// Tags are added/removed by internal id (add is resolved by the handler via `link_tags` first, remove via
    /// `tag_ids_for_names` — nonexistent names are skipped silently).
    pub fn update_memory(
        &self,
        id: i64,
        summary: Option<&str>,
        content: Option<&str>,
        add_tag_ids: &[i64],
        remove_tag_ids: &[i64],
    ) -> Result<bool, String> {
        if !self.memory_exists(id)? {
            return Err(format!(
                "memory '{}' not found (use memory_list or memory_search first)",
                Self::format_id(id)
            ));
        }
        let mut changed = false;
        let names = self.tags_of(id)?;
        let mut final_tags = self.tag_ids_for_names(&names)?;
        for tag_id in add_tag_ids {
            if !final_tags.contains(tag_id) {
                final_tags.push(*tag_id);
            }
        }
        final_tags.retain(|id| !remove_tag_ids.contains(id));
        self.validate_tag_constraints(&final_tags)?;
        if summary.is_some() || content.is_some() {
            self.conn
                .execute(
                    sql::MEMORY_UPDATE_FIELDS,
                    params![summary, content, crate::model::now() as i64, id],
                )
                .map_err(|e| e.to_string())?;
            // Changed fields invalidate the vector: delete it so it falls back into the backfill queue (the write hook re-embeds
            // right away; when the embedding service is unavailable it simply stays empty and the update itself is not blocked)
            self.embedding_delete(id)?;
            changed = true;
        }
        // Contract (defs.rs): add_tags runs before remove_tags; when both lists contain the same
        // tag the final result is removal (the explicit remove intent wins). Tag-only changes do
        // not touch updated_at — see the doc above.
        for tag_id in add_tag_ids {
            let n = self
                .conn
                .execute(sql::MEMORY_LINK_TAG, params![id, tag_id])
                .map_err(|e| e.to_string())?;
            changed = changed || n > 0;
        }
        for tag_id in remove_tag_ids {
            let n = self
                .conn
                .execute(sql::MEMORY_UNLINK_TAG, params![id, tag_id])
                .map_err(|e| e.to_string())?;
            changed = changed || n > 0;
        }
        Ok(changed)
    }

    pub fn memory_exists(&self, id: i64) -> Result<bool, String> {
        match self.conn.query_row(sql::MEMORY_EXISTS, [id], |_| Ok(())) {
            Ok(()) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Delete memories, returning (deleted ids, missing ids).
    pub fn delete_memories(&self, ids: &[i64]) -> Result<(Vec<String>, Vec<String>), String> {
        let mut deleted = Vec::new();
        let mut missing = Vec::new();
        for id in ids {
            let n = self
                .conn
                .execute(sql::MEMORY_DELETE, [id])
                .map_err(|e| e.to_string())?;
            if n > 0 {
                deleted.push(Self::format_id(*id));
            } else {
                missing.push(Self::format_id(*id));
            }
        }
        Ok((deleted, missing))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;
    use serde_json::json;

    /// Test helper: link tags by name and insert a memory (the production path resolves ids in the handler).
    fn insert_with_tags(st: &Store, summary: &str, content: &str, tags: &[&str], at: u64) -> i64 {
        let names: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        let ids = st.link_tags(&names, true).unwrap().ids;
        st.insert_memory(summary, content, &ids, at, at).unwrap()
    }

    #[test]
    fn memory_lifecycle_with_tags() {
        let path = temp_db("lifecycle");
        cleanup(&path);
        let st = Store::open(&path).unwrap();

        let dup = st.find_duplicates_by_summary("Rust notes").unwrap();
        assert!(dup.is_empty());
        let id = insert_with_tags(&st, "Rust notes", "borrow checker", &["rust", "notes"], 10);
        assert_eq!(Store::format_id(id), "m1");

        // Duplicate detection (case-insensitive)
        let dup = st.find_duplicates_by_summary("rust NOTES").unwrap();
        assert_eq!(dup, vec!["m1".to_string()]);

        // Full read with tags
        let all = st.all_memories().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].tags, vec!["notes".to_string(), "rust".to_string()]);

        // Update: change fields + add/remove tags (name → id resolution goes through the same path as the handler)
        let add = st.link_tags(&["study".into()], true).unwrap();
        let remove = st.tag_ids_for_names(&["notes".into()]).unwrap();
        let changed = st
            .update_memory(id, Some("new summary"), None, &add.ids, &remove)
            .unwrap();
        assert!(changed);
        let (found, missing) = st.get_memories(&[id]).unwrap();
        assert!(missing.is_empty());
        assert_eq!(found[0].summary, "new summary");
        assert_eq!(found[0].tags, vec!["rust".to_string(), "study".to_string()]);

        // Nonexistent memory errors out
        assert!(st.update_memory(999, Some("x"), None, &[], &[]).is_err());

        // Delete
        let (deleted, missing) = st.delete_memories(&[id, 999]).unwrap();
        assert_eq!(deleted, vec!["m1".to_string()]);
        assert_eq!(missing, vec!["m999".to_string()]);
        assert_eq!(st.stats().unwrap()["memories"], 0);
        cleanup(&path);
    }

    #[test]
    fn list_paginates_and_filters() {
        let path = temp_db("list");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        for i in 0..5 {
            insert_with_tags(&st, &format!("s{i}"), "c", &["t1"], i);
        }
        let (total, page) = st
            .list_memories(Default::default(), "updated_at", false, 0, 3)
            .unwrap();
        assert_eq!(total, 5);
        assert_eq!(page.len(), 3);
        assert_eq!(page[0].summary, "s4");
        let (total2, page2) = st
            .list_memories(Default::default(), "updated_at", false, 3, 3)
            .unwrap();
        assert_eq!(total2, 5);
        assert_eq!(page2.len(), 2);
        let (total3, page3) = st
            .list_memories(
                ListFilter {
                    tag: Some("t1"),
                    ..Default::default()
                },
                "updated_at",
                true,
                0,
                200,
            )
            .unwrap();
        assert_eq!(total3, 5);
        assert_eq!(page3[0].summary, "s0");
        let (total4, _) = st
            .list_memories(
                ListFilter {
                    tag: Some("t1"),
                    ..Default::default()
                },
                "id",
                false,
                0,
                200,
            )
            .unwrap();
        assert_eq!(total4, 5);
        // Id-set filtering (id_set is JSON array text of memory ids, expanded with json_each; empty set = no results)
        insert_with_tags(&st, "s-other", "c", &["t2"], 9);
        let id_set_raw = |ids: &[i64]| json!(ids).to_string();
        let five_ids = [1i64, 2, 3, 4, 5];
        let (total5, page5) = st
            .list_memories(
                ListFilter {
                    id_set: Some(&id_set_raw(&five_ids)),
                    ..Default::default()
                },
                "updated_at",
                false,
                0,
                200,
            )
            .unwrap();
        assert_eq!(total5, 5);
        assert!(page5.iter().all(|m| m.summary != "s-other"));
        let (total6, _) = st
            .list_memories(
                ListFilter {
                    id_set: Some(&id_set_raw(&[1, 2, 3, 4, 5, 6])),
                    ..Default::default()
                },
                "updated_at",
                false,
                0,
                200,
            )
            .unwrap();
        assert_eq!(total6, 6);
        let (total7, _) = st
            .list_memories(
                ListFilter {
                    id_set: Some("[]"),
                    ..Default::default()
                },
                "updated_at",
                false,
                0,
                200,
            )
            .unwrap();
        assert_eq!(total7, 0);
        // Exact tag and id-set used together = AND (id 6 carries t2, so the intersection is {m6})
        let (total8, _) = st
            .list_memories(
                ListFilter {
                    tag: Some("t2"),
                    id_set: Some(&id_set_raw(&[6])),
                },
                "updated_at",
                false,
                0,
                200,
            )
            .unwrap();
        assert_eq!(total8, 1);
        cleanup(&path);
    }

    /// Contract (defs.rs): add_tags runs before remove_tags;
    /// when the same tag appears in both lists the final result is removal.
    #[test]
    fn update_memory_add_before_remove_ends_removed() {
        let path = temp_db("add-remove");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let id = insert_with_tags(&st, "s", "c", &["a"], 1);
        let add = st.link_tags(&["a".into(), "b".into()], true).unwrap();
        let remove = st.tag_ids_for_names(&["a".into()]).unwrap();
        st.update_memory(id, None, None, &add.ids, &remove).unwrap();
        let (found, _) = st.get_memories(&[id]).unwrap();
        assert_eq!(found[0].tags, vec!["b".to_string()]);
        cleanup(&path);
    }

    /// updated_at tracks the content timeline: tag-only adjustments report changed=true but leave
    /// the timestamp (and therefore the default newest-first browse order) alone; only a summary
    /// or content write moves it.
    #[test]
    fn tag_only_updates_do_not_bump_updated_at() {
        let path = temp_db("touch");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let old = insert_with_tags(&st, "old", "old body", &["a"], 100);
        let fresh = insert_with_tags(&st, "fresh", "fresh body", &["a"], 200);

        // Tag-only change on the older memory: changed, but its updated_at is untouched, so the
        // default updated_at-desc listing keeps the genuinely newer memory first
        let add = st.link_tags(&["b".into()], true).unwrap();
        let changed = st.update_memory(old, None, None, &add.ids, &[]).unwrap();
        assert!(changed);
        let (_, page) = st
            .list_memories(Default::default(), "updated_at", false, 0, 10)
            .unwrap();
        assert_eq!(page[0].summary, "fresh");

        // A content write does move it back to the top
        st.update_memory(old, None, Some("rewritten"), &[], &[])
            .unwrap();
        let (_, page) = st
            .list_memories(Default::default(), "updated_at", false, 0, 10)
            .unwrap();
        assert_eq!(page[0].summary, "old");

        // ...and a no-op change (existing tag re-added) touches nothing at all
        let (before, _) = st.get_memories(&[fresh]).unwrap();
        let existing = st.link_tags(&["a".into()], true).unwrap();
        let changed = st
            .update_memory(fresh, None, None, &existing.ids, &[])
            .unwrap();
        assert!(!changed);
        let (after, _) = st.get_memories(&[fresh]).unwrap();
        assert_eq!(before[0].updated_at, after[0].updated_at);

        cleanup(&path);
    }
}
