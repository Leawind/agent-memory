//! Aggregate queries and ops entry points: hygiene checks, statistics, export/import.

use crate::sql;
use crate::tag_expr::TagExpr;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};

use super::Store;

/// One named hygiene check. `id` is a stable machine identifier (the UI localizes it);
/// `issues` holds raw operator-facing lines, printed verbatim by the CLI.
pub struct HygieneCheck {
    pub id: &'static str,
    pub issues: Vec<String>,
}

/// Behavior selectors for `import_dump`.
#[derive(Debug)]
pub struct ImportOptions<'a> {
    /// Merge into a non-empty target instead of requiring an empty one: memories whose
    /// (summary, content) already exist are skipped, present tag rows keep their name and
    /// description (missing ones are created), and predicate definitions plus the lifecycle
    /// policy in the file are ignored — merge moves data, not configuration, and never
    /// modifies existing data.
    pub merge: bool,
    /// Import only memories matching this tag expression, evaluated inside the dump file:
    /// tags and `@name` references resolve against the file's own vocabulary and predicate
    /// definitions, so the same file imports the same subset on any target.
    pub predicate: Option<&'a str>,
}

impl ImportOptions<'_> {
    /// Default behavior: restore into an empty target with no filter (test convenience).
    #[cfg(test)]
    pub const RESTORE: ImportOptions<'static> = ImportOptions {
        merge: false,
        predicate: None,
    };
}

/// What `import_dump` materialized: inserted memories (duplicates skipped in merge mode are
/// not counted), tag rows created (present ones reused untouched) and how many memories were
/// skipped as already present.
#[derive(Debug)]
pub struct ImportReport {
    pub memories: usize,
    pub tags: usize,
    pub skipped: usize,
}

impl Store {
    /// Health check: a fixed list of named data-hazard probes (read-only). Covers dirty data
    /// that could slip in while foreign keys were off, timestamp/duplicate data-quality
    /// hazards, and derived-data staleness (embedding coverage). The REST doctor endpoint
    /// returns these as a checklist so the UI can render each verdict;
    /// `hygiene_issues` flattens the same probes for the CLI.
    pub fn hygiene_checks(&self) -> Result<Vec<HygieneCheck>, String> {
        let probes = [
            ("integrity", self.check_integrity()),
            ("foreign_keys", self.check_foreign_keys()),
            ("tag_refs", self.check_tag_refs()),
            ("memory_refs", self.check_memory_refs()),
            ("tag_case", self.check_tag_case()),
            ("empty_summary", self.check_empty_summaries()),
            ("timestamps", self.check_timestamps()),
            ("duplicates", self.check_duplicates()),
            ("orphan_embeddings", self.check_orphan_embeddings()),
            ("embedding_coverage", self.check_embedding_coverage()),
        ];
        probes
            .into_iter()
            .map(|(id, result)| result.map(|issues| HygieneCheck { id, issues }))
            .collect()
    }

    /// Flattened issue lines across every check (the CLI's output shape).
    pub fn hygiene_issues(&self) -> Result<Vec<String>, String> {
        Ok(self
            .hygiene_checks()?
            .into_iter()
            .flat_map(|c| c.issues)
            .collect())
    }

    /// PRAGMA quick_check: page-level corruption and broken indexes.
    fn check_integrity(&self) -> Result<Vec<String>, String> {
        let rows: Vec<String> = self
            .conn
            .prepare_cached(sql::HYGIENE_INTEGRITY)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let bad: Vec<String> = rows.into_iter().filter(|row| row != "ok").collect();
        if bad.is_empty() {
            Ok(vec![])
        } else {
            Ok(vec![format!(
                "database integrity check failed: {}",
                bad.join("; ")
            )])
        }
    }

    /// PRAGMA foreign_key_check: every foreign-key violation in the file, regardless of the
    /// connection's foreign_keys setting (the check itself never depends on it).
    fn check_foreign_keys(&self) -> Result<Vec<String>, String> {
        let rows = self
            .conn
            .prepare_cached(sql::HYGIENE_FOREIGN_KEYS)
            .map_err(|e| e.to_string())?
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .map(|(table, rowid, parent, parent_id)| {
                format!(
                    "foreign key violation: {table} row {} references missing {parent} row {}",
                    Self::display_row(&table, rowid),
                    Self::display_row(&parent, parent_id),
                )
            })
            .collect())
    }

    /// Row ids read the way the API names them: memory rows carry the m<N> prefix, everything
    /// else (tags, identities) is a bare internal id.
    fn display_row(table: &str, id: i64) -> String {
        if table == "memories" {
            Self::format_id(id)
        } else {
            id.to_string()
        }
    }

    /// Orphan references: link rows pointing at nonexistent tag ids (possible while foreign keys are off)
    fn check_tag_refs(&self) -> Result<Vec<String>, String> {
        let orphan_ids: Vec<i64> = self
            .conn
            .prepare_cached(sql::HYGIENE_ORPHANS)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if orphan_ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(vec![format!(
            "memory_tags rows reference missing tags (schema corruption): tag ids {}",
            orphan_ids
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )])
    }

    /// Reverse orphans: link rows pointing at nonexistent memories (possible while foreign keys are off)
    fn check_memory_refs(&self) -> Result<Vec<String>, String> {
        let reverse_ids: Vec<i64> = self
            .conn
            .prepare_cached(sql::HYGIENE_REVERSE_ORPHANS)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if reverse_ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(vec![format!(
            "database contains join rows pointing to missing memories (schema corruption): {}",
            reverse_ids
                .iter()
                .map(|id| Self::format_id(*id))
                .collect::<Vec<_>>()
                .join(", ")
        )])
    }

    /// Groups of tags differing only in case
    fn check_tag_case(&self) -> Result<Vec<String>, String> {
        let names: Vec<String> = self
            .conn
            .prepare_cached(sql::TAG_ALL_NAMES)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut groups: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        for n in &names {
            groups.entry(n.to_lowercase()).or_default().push(n.clone());
        }
        Ok(groups
            .into_values()
            .filter(|group| group.len() > 1)
            .map(|group| {
                format!(
                    "case-conflicting tag group: {} (keep one and merge the rest with tag_update)",
                    group.join(" / ")
                )
            })
            .collect())
    }

    /// Empty summaries (content is optional: summary-only memories are a normal shape, not a hazard)
    fn check_empty_summaries(&self) -> Result<Vec<String>, String> {
        let rows = self
            .conn
            .prepare_cached(sql::HYGIENE_MEMORIES)
            .map_err(|e| e.to_string())?
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows
            .into_iter()
            .filter(|(_, summary)| summary.trim().is_empty())
            .map(|(id, _)| format!("memory {} has an empty summary", Self::format_id(id)))
            .collect())
    }

    /// Timestamp sanity: non-epoch timestamps, updated-before-created inversions, and dates
    /// too far in the future (agents write on behalf of machines whose clocks can be wrong).
    fn check_timestamps(&self) -> Result<Vec<String>, String> {
        let rows = self
            .conn
            .prepare_cached(sql::HYGIENE_TIMESTAMPS)
            .map_err(|e| e.to_string())?
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let now = crate::model::now() as i64;
        // Tolerate modest clock skew between agents; flag anything beyond a full day ahead
        const FUTURE_SLACK_SECS: i64 = 86_400;
        let mut issues = Vec::new();
        for (id, created_at, updated_at) in rows {
            if created_at <= 0 || updated_at <= 0 {
                issues.push(format!(
                    "memory {} has a non-positive timestamp (created {created_at}, updated {updated_at})",
                    Self::format_id(id)
                ));
            } else if updated_at < created_at {
                issues.push(format!(
                    "memory {} was updated before it was created",
                    Self::format_id(id)
                ));
            } else if created_at > now + FUTURE_SLACK_SECS || updated_at > now + FUTURE_SLACK_SECS {
                issues.push(format!(
                    "memory {} is dated more than a day into the future (clock skew?)",
                    Self::format_id(id)
                ));
            }
        }
        Ok(issues)
    }

    /// Duplicate memories: same summary AND same content — redundant copies that waste the
    /// context budget of whichever agent pulls them and noise up search results.
    fn check_duplicates(&self) -> Result<Vec<String>, String> {
        let rows = self
            .conn
            .prepare_cached(sql::HYGIENE_DUPLICATES)
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
        let mut groups: std::collections::BTreeMap<(String, String), Vec<String>> =
            Default::default();
        for (id, summary, content) in rows {
            groups
                .entry((summary, content))
                .or_default()
                .push(Self::format_id(id));
        }
        Ok(groups
            .into_iter()
            .filter(|(_, ids)| ids.len() > 1)
            .map(|((summary, _), ids)| {
                format!(
                    "duplicate memories with identical summary and content: {} ({})",
                    ids.join(", "),
                    elide(&summary, 80)
                )
            })
            .collect())
    }

    /// Embedding rows whose memory is gone (cascade deletes make this impossible with foreign
    /// keys on; rows written while they were off would linger invisibly and poison vectors).
    fn check_orphan_embeddings(&self) -> Result<Vec<String>, String> {
        let ids: Vec<i64> = self
            .conn
            .prepare_cached(sql::HYGIENE_ORPHAN_EMBEDDINGS)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(vec![format!(
            "embedding rows point to missing memories (schema corruption): {}",
            ids.iter()
                .map(|id| Self::format_id(*id))
                .collect::<Vec<_>>()
                .join(", ")
        )])
    }

    /// Semantic search coverage: for every enabled, fully specified candidate whose identity has
    /// memories lacking vectors, suggest a backfill. Derived-data staleness rather than corruption,
    /// but it belongs here so doctor stays the single health-check entry point.
    fn check_embedding_coverage(&self) -> Result<Vec<String>, String> {
        let mut issues = Vec::new();
        for cfg in self.embedding_configs()? {
            let pending =
                self.embedding_pending_count(&cfg.vector_key(), Some(&cfg.fingerprint()))?;
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
        // Embedding section: the head entry's numbers stay top-level for convenience (the UI and
        // CLI read them), while `models` carries every candidate with its own cache coverage.
        // Cache coverage is keyed by identity: two entries may share a model name and still have
        // separate caches (each has a user-assigned ID).
        let entries = self.embedding_entries()?;
        let configs: Vec<_> = entries.iter().filter_map(|e| e.usable()).collect();
        let embedding = if configs.is_empty() {
            json!({ "enabled": false })
        } else {
            let head = &configs[0];
            let models: Vec<Value> = entries
                .iter()
                .map(|e| {
                    let mut item = json!({
                        "key": e.vector_key(),
                        "name": e.name,
                        "model": e.model,
                        "enabled": e.enabled,
                    });
                    if let Some(cfg) = e.usable() {
                        item["embedded"] = json!(self.embedding_embedded_count(
                            &cfg.vector_key(),
                            Some(&cfg.fingerprint())
                        )?);
                        item["pending"] = json!(self.embedding_pending_count(
                            &cfg.vector_key(),
                            Some(&cfg.fingerprint())
                        )?);
                    }
                    Ok::<_, String>(item)
                })
                .collect::<Result<Vec<_>, _>>()?;
            json!({
                "enabled": true,
                "model": head.model,
                "embedded": self.embedding_embedded_count(&head.vector_key(), Some(&head.fingerprint()))?,
                "pending": self.embedding_pending_count(&head.vector_key(), Some(&head.fingerprint()))?,
                "models": models,
            })
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
    ///
    /// With `filter`, only memories whose display tag names satisfy the expression are exported,
    /// together with the closure of the tags their annotations reference; the dump then carries
    /// data only — predicate definitions and the lifecycle policy are configuration and stay
    /// out. A full export (no filter) keeps the complete shape.
    pub fn export_dump(&self, filter: Option<&TagExpr>) -> Result<Value, String> {
        // All tag rows; a filtered export keeps only the closure collected below.
        let mut all_tags: Vec<(i64, String, String, i64)> = Vec::new();
        let mut st = self
            .conn
            .prepare_cached(sql::TAG_EXPORT_ALL)
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
            all_tags.push(row.map_err(|e| e.to_string())?);
        }

        // Memory ↔ tag annotations as (memory id, source tag id, target tag id) pairs — the
        // target half rides the annotation row (null for bare tags), memory_id then tag_id ascending.
        let mut pairs: Vec<(i64, i64, Option<i64>)> = Vec::new();
        let mut st = self
            .conn
            .prepare_cached(sql::MEMORY_TAG_ID_PAIRS)
            .map_err(|e| e.to_string())?;
        let pair_rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, Option<i64>>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for pair in pair_rows {
            pairs.push(pair.map_err(|e| e.to_string())?);
        }

        // Memory rows collected first: matching needs each memory's display tag names, which
        // are composed from the pairs and the tag rows above.
        let mut rows: Vec<(i64, String, String, i64, i64)> = Vec::new();
        let mut st = self
            .conn
            .prepare_cached(sql::MEMORY_ALL)
            .map_err(|e| e.to_string())?;
        let memory_rows = st
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
        for row in memory_rows {
            rows.push(row.map_err(|e| e.to_string())?);
        }

        // Matched memory ids when filtering, evaluated on the same display forms the
        // list/search paths see.
        let matched: Option<HashSet<i64>> = match filter {
            None => None,
            Some(expr) => {
                let names: HashMap<i64, &str> = all_tags
                    .iter()
                    .map(|(id, name, ..)| (*id, name.as_str()))
                    .collect();
                let mut display: HashMap<i64, Vec<String>> = HashMap::new();
                for (memory_id, tag_id, target_id) in &pairs {
                    let source = names
                        .get(tag_id)
                        .ok_or("corrupt store: annotation references unknown tag")?;
                    let composed = match target_id.map(|t| names.get(&t)) {
                        Some(Some(target)) => crate::model::compose_edge(source, target),
                        Some(None) | None => (*source).to_string(),
                    };
                    display.entry(*memory_id).or_default().push(composed);
                }
                Some(
                    rows.iter()
                        .filter(|(id, ..)| {
                            expr.eval(display.get(id).map(Vec::as_slice).unwrap_or(&[]))
                        })
                        .map(|(id, ..)| *id)
                        .collect(),
                )
            }
        };

        // Annotation refs for the exported memories; a filtered export also collects the tag
        // closure (source halves and edge targets of matched annotations only).
        let mut refs: HashMap<i64, Vec<Value>> = HashMap::new();
        let mut closure: HashSet<i64> = HashSet::new();
        for (memory_id, tag_id, target_id) in pairs {
            if matched
                .as_ref()
                .is_some_and(|set| !set.contains(&memory_id))
            {
                continue;
            }
            closure.insert(tag_id);
            if let Some(t) = target_id {
                closure.insert(t);
            }
            refs.entry(memory_id).or_default().push(json!([
                tag_id.to_string(),
                target_id
                    .map(|t| Value::String(t.to_string()))
                    .unwrap_or(Value::Null),
            ]));
        }

        let mut tags = Map::new();
        for (id, name, description, created_at) in all_tags {
            if matched.is_some() && !closure.contains(&id) {
                continue;
            }
            tags.insert(
                id.to_string(),
                json!({
                    "name": name,
                    "description": description,
                    "created_at": created_at,
                }),
            );
        }

        let mut memories = Map::new();
        for row in rows {
            let (id, summary, content, created_at, updated_at) = row;
            if matched.as_ref().is_some_and(|set| !set.contains(&id)) {
                continue;
            }
            let tags = refs.remove(&id).unwrap_or_default();
            memories.insert(
                Self::format_id(id),
                json!({
                    "summary": summary,
                    "content": content,
                    "tags": tags,
                    "created_at": created_at,
                    "updated_at": updated_at,
                    "access": self.access_export(id)?,
                }),
            );
        }
        let mut dump = json!({
            "exported_at": crate::model::now(),
            "tags": Value::Object(tags),
            "memories": Value::Object(memories),
        });
        if matched.is_none() {
            dump["predicates"] = self.predicates_view()?;
            dump["lifecycle_policy"] = json!(self.lifecycle_policy()?);
        }
        Ok(dump)
    }

    /// Restore or merge data from JSON produced by `export_dump`. Restore mode (default) targets an
    /// empty database — import is a "restore/migration", not a merge, to avoid ambiguity with existing
    /// data's ids and tag descriptions. Merge mode (`options.merge`) instead imports into a populated
    /// target: verbatim-duplicate memories are skipped, present tag rows stay untouched, and the
    /// file's predicates / lifecycle policy are ignored.
    /// Memories keep their exported created_at / updated_at; ids are not preserved (renumbered),
    /// and the id keys in the export file exist only to express memory→tag references — referencing a tag id
    /// absent from the export errors out outright. Legacy array-shaped exports are always rejected; no compatibility shims.
    pub fn import_dump(
        &self,
        dump: &Value,
        options: &ImportOptions,
    ) -> Result<ImportReport, String> {
        if !options.merge {
            let empty = self.stats()?;
            // The reserved tag is a permanent fixture seeded at open, not user data: a fresh
            // database always carries exactly that one row, so it does not block import.
            let unreserved_tags = self.tag_count_unreserved()?;
            if empty["memories"].as_u64().unwrap_or(0) != 0
                || unreserved_tags != 0
                || !self.predicate_rows()?.is_empty()
            {
                return Err("target database is not empty; import is a restore, use merge mode for an incremental import or point --db at a fresh database".into());
            }
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

        // Normalize dump tag names up front (duplicates within the export file are rejected too —
        // name is unique in the database): the name map feeds both tag creation and the import
        // filter's dump-local vocabulary. Dump tag names are vocabulary names; both edge halves
        // reference this same table.
        let mut name_by_key: HashMap<&str, String> = HashMap::with_capacity(tags.len());
        let mut description_by_key: HashMap<&str, String> = HashMap::with_capacity(tags.len());
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
            name_by_key.insert(key.as_str(), name);
            description_by_key.insert(key.as_str(), description);
        }

        // Optional import filter, evaluated inside the dump: a memory is kept when its display
        // tag names (composed from the file's own vocabulary) satisfy the expression. This runs
        // before any write, so a bad filter leaves the store untouched.
        let keep: Option<HashSet<&str>> = match options.predicate {
            None => None,
            Some(raw) => {
                let vocabulary: HashSet<String> = name_by_key.values().cloned().collect();
                let predicates: HashMap<String, String> = dump
                    .get("predicates")
                    .and_then(Value::as_array)
                    .map(|rows| {
                        rows.iter()
                            .filter_map(|row| {
                                let name = row.get("name")?.as_str()?.to_string();
                                let expression = row.get("predicate")?.as_str()?.to_string();
                                Some((name, expression))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let expr = super::predicates::resolve_dump_filter(raw, &vocabulary, &predicates)?;
                let mut keep = HashSet::with_capacity(memories.len());
                for (key, m) in memories {
                    let display = parse_memory_tag_pairs(key, m)?
                        .into_iter()
                        .map(|(source, target)| {
                            let source = name_by_key.get(source).ok_or_else(|| {
                                format!("invalid export: memory {key} references unknown tag id '{source}'")
                            })?;
                            match target {
                                None => Ok(source.clone()),
                                Some(target) => {
                                    let target = name_by_key.get(target).ok_or_else(|| {
                                        format!("invalid export: memory {key} references unknown target tag id '{target}'")
                                    })?;
                                    Ok(crate::model::compose_edge(source, target))
                                }
                            }
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    if expr.eval(&display) {
                        keep.insert(key.as_str());
                    }
                }
                Some(keep)
            }
        };

        // Create tags with an export tag id → new database id mapping for translating memory
        // references. Present rows are reused as-is: the reserved tag is seeded at open in every
        // database, and merge mode never rewrites existing vocabulary.
        let mut tag_id_by_key: HashMap<&str, i64> = HashMap::with_capacity(tags.len());
        let mut created_tags = 0usize;
        for key in tags.keys() {
            let name = &name_by_key[key.as_str()];
            let id = match self
                .tag_ids_by_names(std::slice::from_ref(name))?
                .into_iter()
                .next()
            {
                Some(existing) => existing,
                None => {
                    self.tag_create(name, &description_by_key[key.as_str()])?;
                    created_tags += 1;
                    self.tag_ids_by_names(std::slice::from_ref(name))?
                        .into_iter()
                        .next()
                        .ok_or_else(|| format!("tag '{name}' vanished right after creation"))?
                }
            };
            tag_id_by_key.insert(key.as_str(), id);
        }

        // Predicates and the lifecycle policy are configuration, not data: a restore materializes
        // them into a fresh database, a merge leaves the target's own configuration alone.
        if !options.merge {
            self.import_predicates(dump.get("predicates").unwrap_or(&json!([])))?;
            if let Some(value) = dump.get("lifecycle_policy") {
                let policy = crate::lifecycle::Policy::parse(value)?;
                self.settings_put(
                    Self::SETTING_LIFECYCLE_POLICY,
                    &serde_json::to_string(&policy).map_err(|e| e.to_string())?,
                )?;
            }
        }
        let mut inserted = 0usize;
        let mut skipped = 0usize;
        for (key, m) in memories {
            if keep.as_ref().is_some_and(|set| !set.contains(key.as_str())) {
                continue;
            }
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
            let content = validate_max_len(content, "content", crate::model::MAX_CONTENT_CHARS)?;
            // Merge skips memories the target already carries verbatim; import never modifies
            // present data, so an evolved copy arrives as a new memory instead.
            if options.merge && self.memory_id_by_text(&summary, &content)?.is_some() {
                skipped += 1;
                continue;
            }
            let created_at = m.get("created_at").and_then(Value::as_u64).unwrap_or(0);
            let updated_at = m
                .get("updated_at")
                .and_then(Value::as_u64)
                .unwrap_or(created_at);
            let mut links: Vec<(i64, Option<i64>)> = Vec::new();
            for (s, target) in parse_memory_tag_pairs(key, m)? {
                let id = tag_id_by_key.get(s).ok_or_else(|| {
                    format!("invalid export: memory {key} references unknown tag id '{s}'")
                })?;
                let target_id = match target {
                    None => None,
                    Some(text) => Some(*tag_id_by_key.get(text).ok_or_else(|| {
                        format!("invalid export: memory {key} references unknown target tag id '{text}'")
                    })?),
                };
                links.push((*id, target_id));
            }
            let id = self.insert_memory(&summary, &content, &links, created_at, updated_at)?;
            if let Some(value) = m.get("access") {
                let history = serde_json::from_value(value.clone())
                    .map_err(|e| format!("invalid exported access history: {e}"))?;
                self.access_import(id, &history)?;
            }
            inserted += 1;
        }
        self.access_rebuild()?;
        self.bump_search_revision()?;
        Ok(ImportReport {
            memories: inserted,
            tags: created_tags,
            skipped,
        })
    }
}

/// Parse one memory's tag pairs in the export-file shape: `[source tag id, target tag id|null]`
/// strings, both halves referencing the file's tags map.
fn parse_memory_tag_pairs<'a>(
    key: &str,
    m: &'a Value,
) -> Result<Vec<(&'a str, Option<&'a str>)>, String> {
    let Some(tag_refs) = m.get("tags").and_then(Value::as_array) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::with_capacity(tag_refs.len());
    for r in tag_refs {
        let pair = r
            .as_array()
            .ok_or("invalid export: memory tags must be [source tag id, target tag id] pairs")?;
        let s = pair
            .first()
            .and_then(Value::as_str)
            .ok_or("invalid export: memory tag pairs start with the source tag id string")?;
        let target = match pair.get(1) {
            None | Some(Value::Null) => None,
            Some(Value::String(text)) => Some(text.as_str()),
            Some(other) => {
                return Err(format!(
                    "invalid export: memory {key} has a non-string target tag id: {other}"
                ))
            }
        };
        out.push((s, target));
    }
    Ok(out)
}

/// Non-empty + length validation (a backstop on the import side; the normal API path is validated by tools::params).
fn validate_nonempty_len(s: &str, what: &str, max: usize) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err(format!("invalid export: {what} must not be empty"));
    }
    validate_max_len(t, what, max)
}

/// Truncate a long text for issue-line display (the full text stays in the database).
fn elide(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{truncated}…")
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
        let rust_id = st
            .resolve_tags(&[("rust".into(), None)], true)
            .unwrap()
            .links[0]
            .0;
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
        // Summary-only memories (empty content) are a normal shape and must not be flagged
        st.conn
            .execute(
                "INSERT INTO memories(id, summary, content, created_at, updated_at) \
                 VALUES (2, 'summary only', '', 1, 1)",
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
        assert!(
            !issues.contains("empty content"),
            "summary-only memory must not be flagged: {issues}"
        );
        cleanup(&path);
    }

    /// The doctor endpoint exposes a named checklist (the UI renders every verdict), so a
    /// healthy database must still yield one entry per probe — and the richer probes must
    /// flag timestamp inversions, far-future clocks, duplicate memories and orphan embeddings.
    #[test]
    fn hygiene_checks_is_a_named_checklist_and_flags_rich_hazards() {
        let path = temp_db("hygiene-checks");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let checks = st.hygiene_checks().unwrap();
        let ids: Vec<&str> = checks.iter().map(|c| c.id).collect();
        assert_eq!(
            ids,
            [
                "integrity",
                "foreign_keys",
                "tag_refs",
                "memory_refs",
                "tag_case",
                "empty_summary",
                "timestamps",
                "duplicates",
                "orphan_embeddings",
                "embedding_coverage",
            ]
        );
        assert!(
            checks.iter().all(|c| c.issues.is_empty()),
            "clean db must pass every check"
        );

        // Inject the hazards (foreign keys off, as in the real corruption paths)
        st.conn.execute("PRAGMA foreign_keys = OFF", []).unwrap();
        st.conn
            .execute(
                "INSERT INTO memories(id, summary, content, created_at, updated_at) \
                 VALUES (1, 'dup', 'same body', 1, 2)",
                [],
            )
            .unwrap();
        st.conn
            .execute(
                "INSERT INTO memories(id, summary, content, created_at, updated_at) \
                 VALUES (2, 'dup', 'same body', 1, 1)",
                [],
            )
            .unwrap();
        st.conn
            .execute(
                "INSERT INTO memories(id, summary, content, created_at, updated_at) \
                 VALUES (3, 'skew', 'c', 100, 50)",
                [],
            )
            .unwrap();
        let future = crate::model::now() as i64 + 7 * 86_400;
        st.conn
            .execute(
                "INSERT INTO memories(id, summary, content, created_at, updated_at) \
                 VALUES (4, 'future', 'c', ?1, ?1)",
                [future],
            )
            .unwrap();
        st.conn
            .execute(
                "INSERT INTO memory_embeddings(memory_id, model, dim, vec, updated_at) \
                 VALUES (999, 'x', 2, x'00000000', 1)",
                [],
            )
            .unwrap();

        fn by_id(st: &Store, id: &str) -> Vec<String> {
            st.hygiene_checks()
                .unwrap()
                .into_iter()
                .find(|c| c.id == id)
                .unwrap()
                .issues
        }
        let dups = by_id(&st, "duplicates");
        assert_eq!(dups.len(), 1);
        assert!(
            dups[0].contains("m1, m2") && dups[0].contains("dup"),
            "got: {}",
            dups[0]
        );
        let stamps = by_id(&st, "timestamps");
        assert!(
            stamps
                .iter()
                .any(|i| i.contains("m3") && i.contains("updated before it was created")),
            "missing inversion: {stamps:?}"
        );
        assert!(
            stamps
                .iter()
                .any(|i| i.contains("m4") && i.contains("future")),
            "missing far-future: {stamps:?}"
        );
        let orphans = by_id(&st, "orphan_embeddings");
        assert_eq!(orphans.len(), 1);
        assert!(orphans[0].contains("m999"), "got: {}", orphans[0]);
        // The generic foreign-key probe sees the orphan embedding row too
        assert!(
            by_id(&st, "foreign_keys")[0].contains("memory_embeddings"),
            "fk probe must cover memory_embeddings"
        );
        // The flat CLI view carries the same lines
        assert!(st.hygiene_issues().unwrap().len() >= 4);
        cleanup(&path);
    }

    #[test]
    fn export_dump_keys_by_id_and_carries_no_derived_fields() {
        let path = temp_db("export");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("t", "desc").unwrap();
        let links = st.resolve_tags(&[("t".into(), None)], true).unwrap().links;
        let tag_key = links[0].0.to_string();
        st.insert_memory("s", "body", &links, 1, 1).unwrap();
        let dump = st.export_dump(None).unwrap();

        // Derived info never enters the export (all of it is derivable from the export file itself)
        assert!(dump.get("total_memories").is_none());
        assert!(dump.get("total_tags").is_none());
        // tags keyed by internal auto-increment id, primary data only
        let tag = &dump["tags"][&tag_key];
        assert_eq!(tag["name"], "t");
        assert_eq!(tag["description"], "desc");
        assert!(tag.get("memory_count").is_none());
        assert!(tag.get("last_used_at").is_none());
        // memories keyed by "m<N>", tag references as [tag id string, value] pairs
        let memory = &dump["memories"]["m1"];
        assert_eq!(memory["summary"], "s");
        assert_eq!(memory["content"], "body");
        assert_eq!(memory["tags"], json!([[tag_key, null]]));
        assert_eq!(memory["created_at"], 1);
        assert_eq!(memory["updated_at"], 1);
        assert!(dump["exported_at"].as_u64().is_some());
        // A memory without tags exports as an empty array
        st.insert_memory("s2", "body2", &[], 2, 2).unwrap();
        let dump = st.export_dump(None).unwrap();
        assert_eq!(dump["memories"]["m2"]["tags"], json!([]));
        cleanup(&path);
    }

    /// The unified tag vocabulary rides the dump and the restore: without it a restored store cannot
    /// resolve `key=value` queries or render typed attributes for the `key:value` display forms.
    #[test]
    fn export_import_round_trips_value_tags_and_queries() {
        let src = temp_db("keys-export-src");
        let dst = temp_db("keys-export-dst");
        cleanup(&src);
        cleanup(&dst);
        let dump = {
            let st = Store::open(&src).unwrap();
            let edges: Vec<crate::model::TagEdge> = vec![
                ("project".to_string(), Some("apollo".to_string())),
                ("count".to_string(), Some("3".to_string())),
            ];
            let links = st.resolve_tags(&edges, true).unwrap().links;
            st.insert_memory("s", "body", &links, 1, 1).unwrap();
            st.export_dump(None).unwrap()
        };
        // The dump carries raw (source id, target id) pairs with the promoted target tags as
        // vocabulary rows; display composition happens on read
        let memory_tags = dump["memories"]["m1"]["tags"].as_array().unwrap();
        assert_eq!(memory_tags.len(), 2);
        for pair in memory_tags {
            let pair = pair.as_array().unwrap();
            assert_eq!(pair.len(), 2);
            assert!(
                pair[0].is_string() && pair[1].is_string(),
                "ids travel as strings: {pair:?}"
            );
            assert!(dump["tags"][pair[0].as_str().unwrap()].is_object());
            assert!(
                dump["tags"][pair[1].as_str().unwrap()].is_object(),
                "targets are vocabulary rows too: {pair:?}"
            );
        }

        Store::open(&dst)
            .unwrap()
            .import_dump(&dump, &ImportOptions::RESTORE)
            .unwrap();
        let st = Store::open(&dst).unwrap();
        let memory = &st.get_memories(&[1]).unwrap().0[0];
        assert_eq!(
            memory.tags,
            vec!["count:3".to_string(), "project:apollo".to_string()]
        );
        assert_eq!(st.attributes_view(memory).unwrap()["count"], 3);
        // The restored store resolves keyed queries
        let expr = st
            .bind_query_atoms(crate::tag_expr::parse("project=apollo").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["project:apollo"]);
        cleanup(&src);
        cleanup(&dst);
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
            let links = st.resolve_tags(&[("t".into(), None)], true).unwrap().links;
            st.insert_memory("s1", "body1", &links, 100, 200).unwrap();
            st.insert_memory("s2", "body2", &[], 300, 400).unwrap();
            st.export_dump(None).unwrap()
        };

        // Restore into an empty database: counts and timestamps preserved; references translate automatically after tag ids are renumbered
        let report = Store::open(&dst)
            .unwrap()
            .import_dump(&dump, &ImportOptions::RESTORE)
            .unwrap();
        assert_eq!(report.memories, 2);
        assert_eq!(report.tags, 1);
        let st = Store::open(&dst).unwrap();
        assert_eq!(st.stats().unwrap()["memories"], 2);
        let all = st.all_memories().unwrap();
        assert_eq!(all[0].created_at, 100);
        assert_eq!(all[0].updated_at, 200);
        assert_eq!(all[0].tags, vec!["t".to_string()]);
        assert_eq!(st.tag_view("t").unwrap()["description"], "带描述的标签");

        // Non-empty target database → rejected, with the merge mode named as the alternative
        let err = Store::open(&dst)
            .unwrap()
            .import_dump(&dump, &ImportOptions::RESTORE)
            .unwrap_err();
        assert!(
            err.contains("not empty") && err.contains("merge"),
            "got: {err}"
        );

        // Structurally corrupt export file → error
        let broken = Store::open(&temp_db("import-broken-dst"));
        drop(broken);
        let dst2 = temp_db("import-broken");
        cleanup(&dst2);
        let st2 = Store::open(&dst2).unwrap();
        let err = st2
            .import_dump(&json!({"memories": {}}), &ImportOptions::RESTORE)
            .unwrap_err();
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
        let err = st
            .import_dump(&legacy, &ImportOptions::RESTORE)
            .unwrap_err();
        assert!(err.contains("must be an object"), "got: {err}");
        // Each direction names its own case
        let err = st
            .import_dump(
                &json!({"tags": {}, "memories": []}),
                &ImportOptions::RESTORE,
            )
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
            // A memory's tag reference is not a [source tag id, target tag id] pair
            (
                "non-pair-ref",
                json!({"tags": {}, "memories": {"m1": {"summary": "s", "content": "c", "tags": [42]}}}),
                "must be [source tag id, target tag id] pairs",
            ),
            // The target half of a pair is a string or null
            (
                "non-string-value",
                json!({"tags": {"1": {"name": "t", "description": ""}}, "memories": {"m1": {"summary": "s", "content": "c", "tags": [["1", 3]]}}}),
                "non-string target tag id",
            ),
            // The target half references a tag absent from the export → error (strict about values)
            (
                "unknown-target-ref",
                json!({"tags": {"1": {"name": "t", "description": ""}}, "memories": {"m1": {"summary": "s", "content": "c", "tags": [["1", "2"]]}}}),
                "references unknown target tag id '2'",
            ),
            // References a tag id absent from the export → error (strict about values)
            (
                "unknown-ref",
                json!({"tags": {"1": {"name": "t", "description": ""}}, "memories": {"m1": {"summary": "s", "content": "c", "tags": [["2", null]]}}}),
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
            let err = st.import_dump(dump, &ImportOptions::RESTORE).unwrap_err();
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
            "memories": {"m1": {"summary": "s", "content": "c", "tags": [["3", null]]}},
        });
        let err = crate::store::with_db_in(&path, crate::store::TxMode::Write, |st: &Store| {
            st.import_dump(&dump, &ImportOptions::RESTORE).map(|_| ())
        })
        .unwrap_err();
        assert!(err.contains("unknown tag id '3'"), "got: {err}");
        let st = Store::open(&path).unwrap();
        // Only the seeded reserved tag remains — the import's own tags rolled back
        assert_eq!(st.tag_count_unreserved().unwrap(), 0, "tags must roll back");
        assert_eq!(st.stats().unwrap()["memories"], 0);
        cleanup(&path);
    }

    /// A filter narrows the dump to matching memories plus their tag closure, and drops the
    /// configuration facets; the unfiltered export keeps the complete shape.
    #[test]
    fn export_dump_filters_by_predicate_and_carries_data_only() {
        let path = temp_db("export-filter");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let links = st
            .resolve_tags(
                &[
                    ("project".to_string(), Some("apollo".to_string())),
                    ("unrelated".to_string(), None),
                ],
                true,
            )
            .unwrap()
            .links;
        st.insert_memory("apollo note", "body a", &links[..1], 1, 1)
            .unwrap();
        st.insert_memory("other note", "body b", &links[1..], 2, 2)
            .unwrap();
        let expr = st.resolve_query_expr("project=apollo").unwrap();
        let dump = st.export_dump(Some(&expr)).unwrap();
        let memories = dump["memories"].as_object().unwrap();
        assert_eq!(memories.len(), 1);
        assert_eq!(memories["m1"]["summary"], "apollo note");
        let names: HashSet<String> = dump["tags"]
            .as_object()
            .unwrap()
            .values()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect();
        // The tag closure carries both edge halves, nothing else
        assert_eq!(
            names,
            HashSet::from(["project".to_string(), "apollo".to_string()])
        );
        assert!(dump.get("predicates").is_none());
        assert!(dump.get("lifecycle_policy").is_none());
        let full = st.export_dump(None).unwrap();
        assert_eq!(full["memories"].as_object().unwrap().len(), 2);
        assert!(full.get("predicates").is_some());
        assert!(full.get("lifecycle_policy").is_some());
        cleanup(&path);
    }

    /// Merge import: verbatim duplicates are skipped, new memories and tags are added, existing
    /// tag rows stay untouched, and the file's predicates are ignored.
    #[test]
    fn import_dump_merges_into_non_empty_target() {
        let src = temp_db("merge-src");
        let dst = temp_db("merge-dst");
        cleanup(&src);
        cleanup(&dst);
        let dump = {
            let st = Store::open(&src).unwrap();
            st.tag_create("t", "dump description").unwrap();
            st.tag_create("fresh", "").unwrap();
            let links = st.resolve_tags(&[("t".into(), None)], true).unwrap().links;
            st.insert_memory("s1", "body1", &links, 100, 200).unwrap();
            st.insert_memory("new", "new body", &[], 300, 400).unwrap();
            st.predicate_set("active", "t", "").unwrap();
            st.export_dump(None).unwrap()
        };
        // The target already carries the dump's first memory verbatim, its own description for
        // the shared tag, and no predicates
        {
            let st = Store::open(&dst).unwrap();
            st.tag_create("t", "target description").unwrap();
            st.insert_memory("s1", "body1", &[], 1, 1).unwrap();
        }
        let report = Store::open(&dst)
            .unwrap()
            .import_dump(
                &dump,
                &ImportOptions {
                    merge: true,
                    predicate: None,
                },
            )
            .unwrap();
        assert_eq!(report.memories, 1);
        assert_eq!(report.skipped, 1);
        assert_eq!(report.tags, 1);
        let st = Store::open(&dst).unwrap();
        assert_eq!(st.stats().unwrap()["memories"], 2);
        // Existing tag description untouched; the new tag arrived
        assert_eq!(
            st.tag_view("t").unwrap()["description"],
            "target description"
        );
        assert!(st.tag_view("fresh").is_ok());
        // The new memory kept its dump timestamps
        let all = st.all_memories().unwrap();
        let new_memory = all.iter().find(|m| m.summary == "new").unwrap();
        assert_eq!((new_memory.created_at, new_memory.updated_at), (300, 400));
        // Configuration in the file is ignored on merge
        assert!(st.predicate_rows().unwrap().is_empty());
        cleanup(&src);
        cleanup(&dst);
    }

    /// The import filter evaluates inside the dump file: `@named` references resolve from the
    /// file's own predicate definitions, and unknown leaves error instead of silently matching
    /// nothing.
    #[test]
    fn import_dump_filters_memories_inside_the_dump() {
        let dst = temp_db("import-filter");
        cleanup(&dst);
        let dump = json!({
            "exported_at": 1,
            "tags": {
                "1": {"name": "project", "description": ""},
                "2": {"name": "apollo", "description": ""},
                "3": {"name": "other", "description": ""},
            },
            "predicates": [{"name": "active", "predicate": "project", "description": ""}],
            "memories": {
                "m1": {"summary": "hit", "content": "c1", "tags": [["1", "2"]]},
                "m2": {"summary": "miss", "content": "c2", "tags": [["3", null]]},
            }
        });
        let report = Store::open(&dst)
            .unwrap()
            .import_dump(
                &dump,
                &ImportOptions {
                    merge: false,
                    predicate: Some("@active"),
                },
            )
            .unwrap();
        assert_eq!(report.memories, 1);
        let st = Store::open(&dst).unwrap();
        let all = st.all_memories().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].summary, "hit");
        assert_eq!(all[0].tags, vec!["project:apollo".to_string()]);
        // The file's vocabulary materializes even where its memories were filtered out
        assert!(st.tag_view("other").is_ok());

        // An unknown leaf is a hard error (a typo must not silently import nothing)
        let fresh = temp_db("import-filter-fresh");
        cleanup(&fresh);
        let err = Store::open(&fresh)
            .unwrap()
            .import_dump(
                &dump,
                &ImportOptions {
                    merge: false,
                    predicate: Some("nosuchtag"),
                },
            )
            .unwrap_err();
        assert!(err.contains("unknown tags in tag_expr"), "got: {err}");
        cleanup(&dst);
        cleanup(&fresh);
    }
}
