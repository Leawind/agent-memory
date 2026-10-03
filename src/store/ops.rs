//! Aggregate queries and ops entry points: hygiene checks, statistics, export/import.

use crate::sql;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};

use super::Store;

impl Store {
    /// Health check: reports data hazards (read-only). Covers dirty data that could slip in while foreign keys are off.
    pub fn hygiene_issues(&self) -> Result<Vec<String>, String> {
        let mut issues = Vec::new();
        // Orphan references: link rows pointing at nonexistent tag ids (possible while foreign keys are off)
        let orphan_ids: Vec<i64> = self
            .conn
            .prepare(sql::HYGIENE_ORPHANS)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if !orphan_ids.is_empty() {
            issues.push(format!(
                "memory_tags rows reference missing tags (schema corruption): tag ids {}",
                orphan_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        // Reverse orphans: link rows pointing at nonexistent memories (possible while foreign keys are off)
        let reverse_ids: Vec<i64> = self
            .conn
            .prepare(
                "SELECT DISTINCT memory_id FROM memory_tags \
                 WHERE memory_id NOT IN (SELECT id FROM memories) ORDER BY memory_id",
            )
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if !reverse_ids.is_empty() {
            issues.push(format!(
                "database contains join rows pointing to missing memories (schema corruption): {}",
                reverse_ids
                    .iter()
                    .map(|id| Self::format_id(*id))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        // Groups of tags differing only in case
        let names: Vec<String> = self
            .conn
            .prepare(sql::TAG_ALL_NAMES)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut groups: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        for n in &names {
            groups.entry(n.to_lowercase()).or_default().push(n.clone());
        }
        for group in groups.values() {
            if group.len() > 1 {
                issues.push(format!(
                    "case-conflicting tag group: {} (keep one and merge the rest with tag_update)",
                    group.join(" / ")
                ));
            }
        }
        // Empty titles / empty content
        let rows = self
            .conn
            .prepare(sql::HYGIENE_MEMORIES)
            .map_err(|e| e.to_string())?
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for (id, summary, content) in rows {
            if summary.trim().is_empty() {
                issues.push(format!(
                    "memory {} has an empty summary",
                    Self::format_id(id)
                ));
            }
            if content.trim().is_empty() {
                issues.push(format!("memory {} has empty content", Self::format_id(id)));
            }
        }
        // Semantic search coverage: when enabled and memories lack vectors for the current model, suggest a backfill.
        // This is a derived-data issue rather than corruption, but it belongs here so doctor stays the single health-check entry point.
        if let Some(cfg) = self.embedding_config()? {
            let pending = self.embedding_pending_count(&cfg.model)?;
            if pending > 0 {
                issues.push(format!(
                    "{pending} memories lack up-to-date embeddings (model '{}'); \
                     run `agent-memory embed-backfill` or use the admin UI",
                    cfg.model
                ));
            }
        }
        Ok(issues)
    }

    /// Data statistics (JSON form, shared by the CLI and the API).
    pub fn stats(&self) -> Result<Value, String> {
        let memories: i64 = self
            .conn
            .query_row(sql::STATS_MEMORY_COUNT, [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let tags: i64 = self
            .conn
            .query_row(sql::STATS_TAG_COUNT, [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        // The real next id is authoritatively tracked by AUTOINCREMENT's sqlite_sequence: deleting the largest id
        // neither rolls back nor gets reused. SQLite only creates that internal table after the first insert into memories,
        // so a completely empty database lacks it — fall back to MAX(id)+1 then (which also yields 1).
        let has_sequence: i64 = self
            .conn
            .query_row(sql::STATS_HAS_SEQUENCE, [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let next_id: i64 = if has_sequence != 0 {
            self.conn
                .query_row(sql::STATS_NEXT_ID, [], |r| r.get(0))
                .map_err(|e| e.to_string())?
        } else {
            self.conn
                .query_row(sql::STATS_MAX_ID, [], |r| r.get::<_, i64>(0))
                .map_err(|e| e.to_string())?
                + 1
        };
        let newest = self
            .conn
            .query_row(sql::STATS_NEWEST, [], |r| {
                Ok(json!({
                    "id": Self::format_id(r.get::<_, i64>(0)?),
                    "updated_at": r.get::<_, i64>(1)?,
                }))
            })
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other.to_string()),
            })?;
        let file_size = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        let schema_version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let embedding = match self.embedding_config()? {
            Some(cfg) => {
                let embedded = self.embedding_embedded_count(&cfg.model)?;
                let pending = self.embedding_pending_count(&cfg.model)?;
                json!({
                    "enabled": true,
                    "model": cfg.model,
                    "embedded": embedded,
                    "pending": pending,
                })
            }
            None => json!({ "enabled": false }),
        };
        Ok(json!({
            "path": self.path.display().to_string(),
            "memories": memories,
            "tags": tags,
            "next_id": Self::format_id(next_id),
            "file_size": file_size,
            "newest_update": newest.unwrap_or(Value::Null),
            "schema_version": schema_version,
            "embedding": embedding,
        }))
    }

    /// Export content: full memories + the tag table, independent of the storage's internal schema (cross-platform migration can go through here too).
    ///
    /// Shape: `tags` / `memories` are both objects keyed by id — tag keys are the decimal string of the internal
    /// auto-increment id, memory keys are `m<N>` (consistent with `format_id`); a memory's `tags` array
    /// references tags by id string. Only primary data is included (names/descriptions/content/timestamps), not
    /// derived info such as memory_count, last_used_at or total counts (all derivable from the file itself).
    /// Import renumbers ids; the id keys act only as reference tokens within the file.
    pub fn export_dump(&self) -> Result<Value, String> {
        let mut tags = Map::new();
        let mut st = self
            .conn
            .prepare(sql::TAG_EXPORT_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, name, description, created_at) = row.map_err(|e| e.to_string())?;
            tags.insert(
                id.to_string(),
                json!({
                    "name": name,
                    "description": description,
                    "created_at": created_at,
                }),
            );
        }

        // Memory ↔ tag links are taken as id pairs (memory_id ascending, tag_id ascending).
        let mut refs: HashMap<i64, Vec<String>> = HashMap::new();
        let mut pairs = self
            .conn
            .prepare(sql::MEMORY_TAG_ID_PAIRS)
            .map_err(|e| e.to_string())?;
        let pair_rows = pairs
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        for pair in pair_rows {
            let (memory_id, tag_id) = pair.map_err(|e| e.to_string())?;
            refs.entry(memory_id).or_default().push(tag_id.to_string());
        }

        let mut memories = Map::new();
        let mut st = self
            .conn
            .prepare(sql::MEMORY_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, summary, content, created_at, updated_at) = row.map_err(|e| e.to_string())?;
            let tags = refs.remove(&id).unwrap_or_default();
            memories.insert(
                Self::format_id(id),
                json!({
                    "summary": summary,
                    "content": content,
                    "tags": tags,
                    "created_at": created_at,
                    "updated_at": updated_at,
                }),
            );
        }
        Ok(json!({
            "exported_at": crate::model::now(),
            "tags": Value::Object(tags),
            "memories": Value::Object(memories),
        }))
    }

    /// Restore data from JSON produced by `export_dump`. The target database must be empty — import is a "restore/
    /// migration", not a merge, to avoid ambiguity with existing data's ids and tag descriptions.
    /// Memories keep their exported created_at / updated_at; ids are not preserved (renumbered),
    /// and the id keys in the export file exist only to express memory→tag references — referencing a tag id
    /// absent from the export errors out outright. Legacy array-shaped exports are always rejected; no compatibility shims.
    /// Returns (memories imported, tags imported).
    pub fn import_dump(&self, dump: &Value) -> Result<(usize, usize), String> {
        let empty = self.stats()?;
        if empty["memories"].as_u64().unwrap_or(0) != 0 || empty["tags"].as_u64().unwrap_or(0) != 0
        {
            return Err(
                "target database is not empty; import refuses to merge -- point --db at a fresh database".into(),
            );
        }
        let tags = dump
            .get("tags")
            .and_then(Value::as_object)
            .ok_or("invalid export: 'tags' must be an object keyed by tag id")?;
        let memories = dump
            .get("memories")
            .and_then(Value::as_object)
            .ok_or("invalid export: 'memories' must be an object keyed by memory id")?;

        // Id keys are renumbered on import but still act as reference tokens, so first validate strictly against the export format:
        // tag keys are positive-integer decimal strings, memory keys are "m<N>".
        for key in tags.keys() {
            let valid = key.parse::<u64>().map(|n| n > 0).unwrap_or(false);
            if !valid {
                return Err(format!(
                    "invalid export: tag key '{key}' is not a positive integer id"
                ));
            }
        }
        for key in memories.keys() {
            if Store::parse_id(key).is_none() {
                return Err(format!(
                    "invalid export: memory key '{key}' does not follow the \"m<N>\" id format"
                ));
            }
        }

        // Create tags (duplicates within the export file are rejected too — name is unique in the database),
        // with a export tag id → new database id mapping for translating memory references.
        let mut tag_id_by_key: HashMap<&str, i64> = HashMap::with_capacity(tags.len());
        let mut seen_names = HashSet::with_capacity(tags.len());
        for (key, t) in tags {
            let name = t
                .get("name")
                .and_then(Value::as_str)
                .ok_or("invalid export: tag without name")?;
            let description = t.get("description").and_then(Value::as_str).unwrap_or("");
            let name = crate::model::normalize_tag_name(name)?;
            if !seen_names.insert(name.clone()) {
                return Err(format!("invalid export: duplicate tag name '{name}'"));
            }
            let description = validate_max_len(
                description,
                "tag description",
                crate::model::MAX_TAG_DESC_CHARS,
            )?;
            self.tag_create(&name, &description)?;
            let id = self
                .tag_ids_for_names(std::slice::from_ref(&name))?
                .into_iter()
                .next()
                .ok_or_else(|| format!("tag '{name}' vanished right after creation"))?;
            tag_id_by_key.insert(key.as_str(), id);
        }

        for (key, m) in memories {
            let summary = m
                .get("summary")
                .and_then(Value::as_str)
                .ok_or("invalid export: memory without summary")?;
            let content = m
                .get("content")
                .and_then(Value::as_str)
                .ok_or("invalid export: memory without content")?;
            let summary =
                validate_nonempty_len(summary, "summary", crate::model::MAX_SUMMARY_CHARS)?;
            let content =
                validate_nonempty_len(content, "content", crate::model::MAX_CONTENT_CHARS)?;
            let created_at = m.get("created_at").and_then(Value::as_u64).unwrap_or(0);
            let updated_at = m
                .get("updated_at")
                .and_then(Value::as_u64)
                .unwrap_or(created_at);
            let empty_tags: Vec<Value> = Vec::new();
            let tag_refs = m
                .get("tags")
                .and_then(Value::as_array)
                .unwrap_or(&empty_tags);
            let mut tag_ids = Vec::with_capacity(tag_refs.len());
            for r in tag_refs {
                let s = r
                    .as_str()
                    .ok_or("invalid export: memory tags must be tag id strings")?;
                let id = tag_id_by_key.get(s).ok_or_else(|| {
                    format!("invalid export: memory {key} references unknown tag id '{s}'")
                })?;
                tag_ids.push(*id);
            }
            self.insert_memory(&summary, &content, &tag_ids, created_at, updated_at)?;
        }
        Ok((memories.len(), tags.len()))
    }
}

/// Non-empty + length validation (a backstop on the import side; the normal API path is validated by tools::params).
fn validate_nonempty_len(s: &str, what: &str, max: usize) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err(format!("invalid export: {what} must not be empty"));
    }
    validate_max_len(t, what, max)
}

fn validate_max_len(s: &str, what: &str, max: usize) -> Result<String, String> {
    if s.chars().count() > max {
        return Err(format!(
            "invalid export: {what} is too long (max {max} characters)"
        ));
    }
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    #[test]
    fn next_id_never_reuses_deleted_max() {
        let path = temp_db("next-id");
        cleanup(&path);
        {
            let st = Store::open(&path).unwrap();
            // Fully empty database: sqlite_sequence does not exist yet, so the MAX+1 fallback applies and the next id is m1
            assert_eq!(st.stats().unwrap()["next_id"], "m1");

            let a = st.insert_memory("a", "ca", &[], 1, 1).unwrap();
            let b = st.insert_memory("b", "cb", &[], 2, 2).unwrap();
            let c = st.insert_memory("c", "cc", &[], 3, 3).unwrap();
            assert_eq!(st.stats().unwrap()["next_id"], "m4");

            // Delete the current largest id: the next id must not roll back (the AUTOINCREMENT sequence only moves forward)
            st.delete_memories(&[c]).unwrap();
            assert_eq!(st.stats().unwrap()["next_id"], "m4");
            assert_eq!(st.insert_memory("d", "cd", &[], 4, 4).unwrap(), 4);

            // After deleting every memory, ids are still not reused
            st.delete_memories(&[a, b]).unwrap();
            assert_eq!(st.stats().unwrap()["next_id"], "m5");
        }
        // The sequence persists across connections: reopening the database still does not roll back
        {
            let st = Store::open(&path).unwrap();
            assert_eq!(st.stats().unwrap()["next_id"], "m5");
            assert_eq!(st.insert_memory("e", "ce", &[], 5, 5).unwrap(), 5);
        }
        cleanup(&path);
    }

    #[test]
    fn hygiene_reports_real_problems_only() {
        let path = temp_db("hygiene");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        assert!(st.hygiene_issues().unwrap().is_empty());

        // With foreign keys off, inject orphan references, reverse orphans, case conflicts and an empty title
        st.conn.execute("PRAGMA foreign_keys = OFF", []).unwrap();
        let rust_id = st.link_tags(&["rust".into()], true).unwrap().ids[0];
        // Orphan reference: a link row pointing at nonexistent tag id 999
        st.conn
            .execute(
                "INSERT INTO memory_tags(memory_id, tag_id) VALUES (1, 999)",
                [],
            )
            .unwrap();
        // Reverse orphan: a link row pointing at nonexistent memory m42 (the rust tag itself exists)
        st.conn
            .execute(
                "INSERT INTO memory_tags(memory_id, tag_id) VALUES (42, ?1)",
                [rust_id],
            )
            .unwrap();
        st.conn
            .execute("INSERT INTO tags(name, created_at) VALUES ('Rust', 1)", [])
            .unwrap();
        st.conn
            .execute(
                "INSERT INTO memories(id, summary, content, created_at, updated_at) \
                 VALUES (1, '   ', 'c', 1, 1)",
                [],
            )
            .unwrap();
        let issues = st.hygiene_issues().unwrap().join("\n");
        assert!(issues.contains("999"), "missing orphan: {issues}");
        assert!(
            issues.contains("pointing to missing memories"),
            "missing reverse orphan: {issues}"
        );
        assert!(
            issues.contains("case-conflicting"),
            "missing case: {issues}"
        );
        assert!(issues.contains("empty summary"), "missing empty: {issues}");
        cleanup(&path);
    }

    #[test]
    fn export_dump_keys_by_id_and_carries_no_derived_fields() {
        let path = temp_db("export");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("t", "desc").unwrap();
        let ids = st.link_tags(&["t".into()], true).unwrap().ids;
        st.insert_memory("s", "body", &ids, 1, 1).unwrap();
        let dump = st.export_dump().unwrap();

        // Derived info never enters the export (all of it is derivable from the export file itself)
        assert!(dump.get("total_memories").is_none());
        assert!(dump.get("total_tags").is_none());
        // tags keyed by internal auto-increment id, primary data only
        let tag_key = ids[0].to_string();
        let tag = &dump["tags"][&tag_key];
        assert_eq!(tag["name"], "t");
        assert_eq!(tag["description"], "desc");
        assert!(tag.get("memory_count").is_none());
        assert!(tag.get("last_used_at").is_none());
        // memories keyed by "m<N>", tag references by tag id string
        let memory = &dump["memories"]["m1"];
        assert_eq!(memory["summary"], "s");
        assert_eq!(memory["content"], "body");
        assert_eq!(memory["tags"], json!([tag_key]));
        assert_eq!(memory["created_at"], 1);
        assert_eq!(memory["updated_at"], 1);
        assert!(dump["exported_at"].as_u64().is_some());
        // A memory without tags exports as an empty array
        st.insert_memory("s2", "body2", &[], 2, 2).unwrap();
        let dump = st.export_dump().unwrap();
        assert_eq!(dump["memories"]["m2"]["tags"], json!([]));
        cleanup(&path);
    }

    #[test]
    fn import_dump_restores_and_refuses_non_empty_target() {
        let src = temp_db("import-src");
        let dst = temp_db("import-dst");
        cleanup(&src);
        cleanup(&dst);
        let dump = {
            let st = Store::open(&src).unwrap();
            st.tag_create("t", "带描述的标签").unwrap();
            let ids = st.link_tags(&["t".into()], true).unwrap().ids;
            st.insert_memory("s1", "body1", &ids, 100, 200).unwrap();
            st.insert_memory("s2", "body2", &[], 300, 400).unwrap();
            st.export_dump().unwrap()
        };

        // Restore into an empty database: counts and timestamps preserved; references translate automatically after tag ids are renumbered
        let (memories, tags) = Store::open(&dst).unwrap().import_dump(&dump).unwrap();
        assert_eq!(memories, 2);
        assert_eq!(tags, 1);
        let st = Store::open(&dst).unwrap();
        assert_eq!(st.stats().unwrap()["memories"], 2);
        let all = st.all_memories().unwrap();
        assert_eq!(all[0].created_at, 100);
        assert_eq!(all[0].updated_at, 200);
        assert_eq!(all[0].tags, vec!["t".to_string()]);
        assert_eq!(st.tag_view("t").unwrap()["description"], "带描述的标签");

        // Non-empty target database → rejected
        let err = Store::open(&dst).unwrap().import_dump(&dump).unwrap_err();
        assert!(err.contains("not empty"), "got: {err}");

        // Structurally corrupt export file → error
        let broken = Store::open(&temp_db("import-broken-dst"));
        drop(broken);
        let dst2 = temp_db("import-broken");
        cleanup(&dst2);
        let st2 = Store::open(&dst2).unwrap();
        let err = st2.import_dump(&json!({"memories": {}})).unwrap_err();
        assert!(err.contains("'tags' must be an object"), "got: {err}");
        cleanup(&src);
        cleanup(&dst);
        cleanup(&dst2);
    }

    /// Legacy array-shaped exports are always rejected (no version compatibility); the error must state the expected shape.
    #[test]
    fn import_dump_rejects_legacy_array_format() {
        let dst = temp_db("import-legacy");
        cleanup(&dst);
        let st = Store::open(&dst).unwrap();
        let legacy = json!({
            "exported_at": 1,
            "total_memories": 0,
            "total_tags": 0,
            "tags": [],
            "memories": [],
        });
        let err = st.import_dump(&legacy).unwrap_err();
        assert!(err.contains("must be an object"), "got: {err}");
        // Each direction names its own case
        let err = st
            .import_dump(&json!({"tags": {}, "memories": []}))
            .unwrap_err();
        assert!(err.contains("'memories' must be an object"), "got: {err}");
        assert_eq!(st.stats().unwrap()["memories"], 0);
        cleanup(&dst);
    }

    /// Import honors the same contract as the API: blank titles, overlong tag names, non-string/unknown tag references,
    /// invalid id keys and duplicate tag names within the file are all rejected. Each case gets its own empty database (in the production
    /// path import runs in a single write transaction and rolls back wholesale on failure — the rollback itself is covered by the dedicated test below).
    #[test]
    fn import_dump_rejects_contract_violations() {
        let cases: Vec<(&str, Value, &str)> = vec![
            // Blank title
            (
                "blank-summary",
                json!({"tags": {}, "memories": {"m1": {"summary": "   ", "content": "c"}}}),
                "must not be empty",
            ),
            // Overlong tag name (>100 characters)
            (
                "oversized-tag",
                json!({"tags": {"1": {"name": "x".repeat(101)}}, "memories": {}}),
                "too long",
            ),
            // A memory's tag reference is not a string (tag ids are string tokens)
            (
                "non-string-ref",
                json!({"tags": {}, "memories": {"m1": {"summary": "s", "content": "c", "tags": [42]}}}),
                "must be tag id strings",
            ),
            // References a tag id absent from the export → error (strict about values)
            (
                "unknown-ref",
                json!({"tags": {"1": {"name": "t", "description": ""}}, "memories": {"m1": {"summary": "s", "content": "c", "tags": ["2"]}}}),
                "unknown tag id '2'",
            ),
            // Invalid id keys: a tag key that is not a positive integer, a memory key missing the m prefix
            (
                "bad-tag-key",
                json!({"tags": {"t1": {"name": "t", "description": ""}}, "memories": {}}),
                "tag key 't1' is not a positive integer",
            ),
            (
                "bad-memory-key",
                json!({"tags": {}, "memories": {"1": {"summary": "s", "content": "c"}}}),
                "memory key '1' does not follow",
            ),
            // Duplicate tag names within the export file → error (name is unique in the database)
            (
                "duplicate-tag",
                json!({"tags": {"1": {"name": "t", "description": ""}, "2": {"name": "t", "description": "other"}}, "memories": {}}),
                "duplicate tag name 't'",
            ),
        ];
        for (slug, dump, expect) in &cases {
            let path = temp_db(&format!("import-invalid-{slug}"));
            cleanup(&path);
            let st = Store::open(&path).unwrap();
            let err = st.import_dump(dump).unwrap_err();
            assert!(err.contains(expect), "{slug}: got: {err}");
            // A rejection must leave no memories in the database
            assert_eq!(st.stats().unwrap()["memories"], 0, "{slug}");
            cleanup(&path);
        }
    }

    /// In the production path import runs in a single write transaction: a mid-way failure rolls back the tags already
    /// created along with everything else — no half-imported data remains.
    #[test]
    fn import_dump_failure_rolls_back_whole_transaction() {
        let path = temp_db("import-rollback");
        cleanup(&path);
        let dump = json!({
            "tags": {
                "1": {"name": "kept", "description": ""},
                "2": {"name": "other", "description": ""},
            },
            "memories": {"m1": {"summary": "s", "content": "c", "tags": ["3"]}},
        });
        let err = crate::store::with_db_in(&path, crate::store::TxMode::Write, |st: &Store| {
            st.import_dump(&dump).map(|_| ())
        })
        .unwrap_err();
        assert!(err.contains("unknown tag id '3'"), "got: {err}");
        let st = Store::open(&path).unwrap();
        assert_eq!(st.stats().unwrap()["tags"], 0, "tags must roll back");
        assert_eq!(st.stats().unwrap()["memories"], 0);
        cleanup(&path);
    }
}
