//! Tag data operations: create / views / rename (foreign-key cascade) / delete (detach and purge).

use crate::sql;
use rusqlite::params;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::{is_unique_violation, Store};

/// One annotation row of a memory: the source tag (id + name) and, when the row carries an
/// edge, the target tag (id + name). `target_id`/`target` are `None` for bare tags.
#[derive(Debug)]
pub struct TagRow {
    pub tag_id: i64,
    pub source: String,
    pub target_id: Option<i64>,
    pub target: Option<String>,
}

/// Link result for a batch of tag edges (see `resolve_tags`).
#[derive(Debug)]
pub struct TagLinkage {
    /// Internal links aligned one-to-one with the input edges (invisible externally):
    /// (source tag id, target tag id or None).
    pub links: Vec<(i64, Option<i64>)>,
    /// Tags created by this call.
    pub autocreated: Vec<String>,
    /// Tags that already existed and were reused.
    pub reused: Vec<String>,
    /// Reused tags with an empty description (hinting to fill it in).
    pub missing_description: Vec<String>,
}

impl Store {
    pub fn tag_exists(&self, name: &str) -> Result<bool, String> {
        match self.conn.query_row(sql::TAG_EXISTS, [name], |_| Ok(())) {
            Ok(()) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Find existing tags differing from the given name only in case (compared on the Rust side with consistent Unicode semantics).
    pub fn find_tag_case_insensitive(&self, name: &str) -> Result<Option<String>, String> {
        let mut st = self
            .conn
            .prepare(sql::TAG_ALL_NAMES)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let fold = name.to_lowercase();
        for r in rows {
            let n = r.map_err(|e| e.to_string())?;
            if n != name && n.to_lowercase() == fold {
                return Ok(Some(n));
            }
        }
        Ok(None)
    }

    /// Create a tag; errors when it already exists (making the unique constraint explicit).
    pub fn tag_create(&self, name: &str, description: &str) -> Result<(), String> {
        self.conn
            .execute(
                sql::TAG_CREATE,
                params![name, description, crate::model::now() as i64],
            )
            .map(|_| ())
            .map_err(|e| {
                if is_unique_violation(&e) {
                    format!(
                        "tag '{name}' already exists (rename it with tag_rename, or see tag_list)"
                    )
                } else {
                    e.to_string()
                }
            })
    }

    /// Single tag view: description, memory count (presence in any role), last-used time.
    pub fn tag_view(&self, name: &str) -> Result<Value, String> {
        match self.conn.query_row(sql::TAG_VIEW, [name], row_to_tag_view) {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(format!("tag '{name}' not found (see tag_list)"))
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// All tag views, ordered by memory count descending then name ascending (the default order
    /// for browsing the taxonomy). Each view is paired with its `target_only` flag — the tag's
    /// only presence is as an edge target — so navigation faces can apply the default set
    /// (organizing tags only) or list everything on an explicit filter.
    pub fn tag_views(&self) -> Result<Vec<(Value, bool)>, String> {
        let mut st = self
            .conn
            .prepare(sql::TAG_VIEW_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| Ok((row_to_tag_view(r)?, r.get::<_, i64>(3)? != 0)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Update a tag: rename and/or change the description. Tags link to memories by internal id
    /// in both edge roles, so renaming touches only name; references follow automatically in
    /// every role. Errors when the target name already exists (same semantics as exact-duplicate
    /// tag creation). Returns (whether a rename happened, whether a description was written).
    pub fn tag_update(
        &self,
        name: &str,
        new_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<(bool, bool), String> {
        if !self.tag_exists(name)? {
            return Err(format!("tag '{name}' not found (see tag_list)"));
        }
        let final_name = new_name.unwrap_or(name);
        let mut renamed = false;
        if final_name != name {
            if self.tag_exists(final_name)? {
                return Err(format!("tag '{final_name}' already exists"));
            }
            self.conn
                .execute(sql::TAG_SET_NAME, params![final_name, name])
                .map_err(|e| e.to_string())?;
            renamed = true;
        }
        let mut description_updated = false;
        if let Some(d) = description {
            self.conn
                .execute(sql::TAG_SET_DESCRIPTION, params![d, final_name])
                .map_err(|e| e.to_string())?;
            description_updated = true;
        }
        Ok((renamed, description_updated))
    }

    fn tag_memory_count(&self, name: &str) -> Result<u64, String> {
        self.conn
            .query_row(sql::TAG_MEMORY_COUNT, [name], |r| r.get::<_, i64>(0))
            .map(|n| n as u64)
            .map_err(|e| e.to_string())
    }

    /// detach: delete the tag (cascading away its annotations in every role — source, target and
    /// bare), returning the number of memories affected.
    pub fn tag_delete_detach(&self, name: &str) -> Result<u64, String> {
        self.assert_tag_deletable(name)?;
        if !self.tag_exists(name)? {
            return Err(format!("tag '{name}' not found (see tag_list)"));
        }
        let affected = self.tag_memory_count(name)?;
        self.conn
            .execute(sql::TAG_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(affected)
    }

    /// Ids (as "m{n}") of every memory carrying the tag in any role — the purge preview's impact list.
    pub fn tag_memory_ids(&self, name: &str) -> Result<Vec<String>, String> {
        let ids = self.ids_with_tag(name)?;
        Ok(ids.iter().map(|i| Self::format_id(*i)).collect())
    }

    /// purge: also delete every memory carrying the tag in any role, returning the deleted memory ids.
    pub fn tag_delete_purge(&self, name: &str) -> Result<Vec<String>, String> {
        self.assert_tag_deletable(name)?;
        if !self.tag_exists(name)? {
            return Err(format!("tag '{name}' not found (see tag_list)"));
        }
        let id_strs = self.tag_memory_ids(name)?;
        self.conn
            .execute(sql::PURGE_MEMORIES_WITH_TAG, [name])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(sql::TAG_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(id_strs)
    }

    fn ids_with_tag(&self, name: &str) -> Result<Vec<i64>, String> {
        let mut st = self
            .conn
            .prepare(sql::MEMORY_IDS_WITH_TAG)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([name], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Resolve tag edges to linkable rows: every named tag — source and target halves alike —
    /// is a vocabulary entry. With `create_missing` unknown names are auto-created (empty
    /// description) and classified into `autocreated` / `reused` / `missing_description`;
    /// without it, unknown names are an error that names each one and any case-variant already
    /// in the store — the agent decides between reusing, an explicit tag_create, or opting into
    /// auto-creation, so the taxonomy cannot rot through casual tagging.
    ///
    /// `links` corresponds one-to-one with the input edges; `autocreated` holds tags created
    /// this time, `reused` the already-existing ones, and `missing_description` the reused tags
    /// with an empty description (usually left over from earlier auto-creation — left to the
    /// caller whether to remind about filling them in).
    pub fn resolve_tags(
        &self,
        edges: &[crate::model::TagEdge],
        create_missing: bool,
    ) -> Result<TagLinkage, String> {
        let mut names: Vec<&str> = Vec::new();
        for (source, target) in edges {
            for name in [source.as_str(), target.as_deref().unwrap_or_default()] {
                if !name.is_empty() && !names.contains(&name) {
                    names.push(name);
                }
            }
        }
        if !create_missing {
            let mut unknown: Vec<String> = Vec::new();
            for name in &names {
                if !self.tag_exists(name)? {
                    match self.find_tag_case_insensitive(name)? {
                        Some(similar) => {
                            unknown.push(format!("'{name}' (did you mean '{similar}'?)"))
                        }
                        None => unknown.push(format!("'{name}'")),
                    }
                }
            }
            if !unknown.is_empty() {
                return Err(format!(
                    "unknown tags: {}; create them explicitly with tag_create, reuse the existing names, or pass create_missing_tags: true to auto-create them empty",
                    unknown.join(", ")
                ));
            }
        }
        let mut out = TagLinkage {
            links: Vec::with_capacity(edges.len()),
            autocreated: Vec::new(),
            reused: Vec::new(),
            missing_description: Vec::new(),
        };
        let mut id_by_name: HashMap<&str, i64> = HashMap::with_capacity(names.len());
        for name in &names {
            let inserted = self
                .conn
                .execute(
                    sql::TAG_AUTOCREATE,
                    params![name, crate::model::now() as i64],
                )
                .map_err(|e| e.to_string())?;
            let (id, description): (i64, String) = self
                .conn
                .query_row(sql::TAG_BY_NAME, [name], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| e.to_string())?;
            id_by_name.insert(name, id);
            if inserted > 0 {
                out.autocreated.push((*name).to_string());
            } else {
                out.reused.push((*name).to_string());
                if description.is_empty() {
                    out.missing_description.push((*name).to_string());
                }
            }
        }
        for (source, target) in edges {
            out.links.push((
                id_by_name[source.as_str()],
                target.as_deref().map(|t| id_by_name[t]),
            ));
        }
        Ok(out)
    }

    /// All tag ids with their current names (id→name view source).
    pub fn tag_id_names(&self) -> Result<HashMap<i64, String>, String> {
        let mut statement = self
            .conn
            .prepare(sql::TAG_EXPORT_ALL)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Resolve existing tag names to internal ids; nonexistent names are skipped outright (matching the
    /// silent no-op semantics of unlink-by-name).
    pub fn tag_ids_by_names(&self, names: &[String]) -> Result<Vec<i64>, String> {
        let mut ids = Vec::with_capacity(names.len());
        for n in names {
            let id = match self
                .conn
                .query_row(sql::TAG_BY_NAME, [n], |r| r.get::<_, i64>(0))
            {
                Ok(id) => Some(id),
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(e) => return Err(e.to_string()),
            };
            if let Some(id) = id {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    /// Count tags excluding the reserved one: the reserved tag is a permanent fixture seeded at
    /// open, so "does the store hold any user-created tag" must not count it (import's
    /// fresh-database check is the caller that cares).
    pub fn tag_count_unreserved(&self) -> Result<i64, String> {
        self.conn
            .query_row(
                sql::TAG_COUNT_UNRESERVED,
                [crate::model::RESERVED_TAG],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())
    }

    /// The memory's annotation rows — one per source tag, the target `None` for bare tags.
    pub fn tag_rows_for(&self, id: i64) -> Result<Vec<TagRow>, String> {
        let mut statement = self
            .conn
            .prepare(sql::MEMORY_TAG_ORIGINS)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([id], |row| {
                Ok(TagRow {
                    tag_id: row.get(0)?,
                    source: row.get(1)?,
                    target_id: row.get(2)?,
                    target: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Replace a memory's annotation rows wholesale, enforcing the model invariant first:
    /// on one memory, a tag carrying an edge target must not have its own row (a tag is
    /// either an edge source/bare or an edge target, never both).
    pub fn set_memory_tags(&self, id: i64, links: &[(i64, Option<i64>)]) -> Result<(), String> {
        self.validate_tag_rows(links)?;
        self.conn
            .execute(sql::MEMORY_CLEAR_TAGS, [id])
            .map_err(|e| e.to_string())?;
        for (tag_id, target_id) in links {
            self.conn
                .execute(sql::MEMORY_LINK_TAG, params![id, tag_id, target_id])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// The role-exclusivity invariant, checked on the final row set (ids, so it covers every
    /// write path: create, update, merge, import). The error names the tags involved —
    /// resolved on demand, since the happy path must stay lookup-free.
    fn validate_tag_rows(&self, rows: &[(i64, Option<i64>)]) -> Result<(), String> {
        let own: std::collections::HashSet<i64> = rows.iter().map(|r| r.0).collect();
        let Some(target_id) = rows.iter().filter_map(|r| r.1).find(|t| own.contains(t)) else {
            return Ok(());
        };
        let name_of = |id: i64, fallback: &str| -> String {
            self.tag_id_names()
                .ok()
                .and_then(|names| names.get(&id).cloned())
                .unwrap_or_else(|| fallback.to_string())
        };
        let target_name = name_of(target_id, "#");
        let (owner_id, owner_target) = rows
            .iter()
            .find(|r| r.0 == target_id)
            .map(|r| (r.0, r.1))
            .unwrap_or((target_id, None));
        let owner_desc = match owner_target {
            Some(t) => format!(
                "the source of '{}:{}'",
                name_of(owner_id, "#"),
                name_of(t, "#")
            ),
            None => "a bare tag".to_string(),
        };
        Err(format!(
            "tag '{target_name}' cannot be both an edge target and {owner_desc} on one memory (no nesting) — drop or retarget one side first"
        ))
    }

    /// A tag can be deleted only when no named predicate binds its id (decay rules reference
    /// individual tags, not groups, so they no longer block tag deletion).
    pub fn assert_tag_deletable(&self, name: &str) -> Result<(), String> {
        let ids = self.tag_ids_by_names(&[name.to_string()])?;
        if let Some(id) = ids.first() {
            if self
                .predicate_rows()?
                .iter()
                .any(|d| d.predicate.references_tag(*id))
            {
                return Err(format!(
                    "tag '{name}' is referenced by a named predicate; remove that reference first"
                ));
            }
        }
        Ok(())
    }
}

/// Sparse agent-facing shape: tokens are the currency, so tags carrying no information are
/// dropped outright — undescribed tags omit `description`, and `reserved` appears only when
/// true (everything except 'convention'). Timestamps are not exposed at all: they cost
/// several tokens per row and rarely change what the agent does next.
fn row_to_tag_view(r: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let name: String = r.get(0)?;
    let count: i64 = r.get(2)?;
    let mut v = json!({
        "name": name,
        "count": count,
    });
    let description: String = r.get(1)?;
    if !description.is_empty() {
        v["description"] = json!(description);
    }
    if name == crate::model::RESERVED_TAG {
        v["reserved"] = json!(true);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    /// Test helper: resolve tag edges and insert a memory (the production path resolves ids in the handler).
    pub fn insert_with_tags(
        st: &Store,
        summary: &str,
        content: &str,
        tags: &[&str],
        at: u64,
    ) -> i64 {
        let edges: Vec<crate::model::TagEdge> = tags
            .iter()
            .map(|t| match t.split_once(':') {
                Some((s, v)) => (s.to_string(), Some(v.to_string())),
                None => (t.to_string(), None),
            })
            .collect();
        let links = st.resolve_tags(&edges, true).unwrap().links;
        st.insert_memory(summary, content, &links, at, at).unwrap()
    }

    /// resolve_tags' three-way classification: created / reused / reused-missing-description;
    /// links align with the input edges one-to-one, and both halves resolve to vocabulary ids.
    #[test]
    fn resolve_tags_classifies_autocreated_reused_and_missing_description() {
        let path = temp_db("link-tags");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("described", "has description").unwrap();

        let first = st
            .resolve_tags(
                &[
                    ("described".into(), None),
                    ("fresh".into(), Some("target".into())),
                ],
                true,
            )
            .unwrap();
        assert_eq!(
            first.autocreated,
            vec!["fresh".to_string(), "target".to_string()]
        );
        assert_eq!(first.reused, vec!["described".to_string()]);
        assert!(first.missing_description.is_empty());
        assert_eq!(first.links.len(), 2);
        let (source_id, target_id) = first.links[1];
        assert_ne!(Some(source_id), target_id);

        // Second call reuses everything; fresh, auto-created earlier with an empty description → lands in missing_description
        let second = st
            .resolve_tags(
                &[
                    ("described".into(), None),
                    ("fresh".into(), Some("target".into())),
                ],
                true,
            )
            .unwrap();
        assert!(second.autocreated.is_empty());
        assert_eq!(
            second.reused,
            vec![
                "described".to_string(),
                "fresh".to_string(),
                "target".to_string()
            ]
        );
        assert_eq!(
            second.missing_description,
            vec!["fresh".to_string(), "target".to_string()]
        );
        // Same name → same id (tag ids are stable; renaming never changes the id)
        assert_eq!(first.links, second.links);
        cleanup(&path);
    }

    /// Role exclusivity: an edge onto a tag that carries its own annotation on the same memory
    /// is rejected with the tag named — whether the own row is an edge (nesting) or bare.
    #[test]
    fn set_memory_tags_rejects_role_conflicts() {
        let path = temp_db("role-conflicts");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let ids = |names: &[&str]| {
            st.tag_ids_by_names(&names.iter().map(|n| n.to_string()).collect::<Vec<_>>())
                .unwrap()
        };
        // Seed vocabulary
        st.resolve_tags(
            &[
                ("project".into(), Some("nmmt".into())),
                ("rust".into(), None),
            ],
            true,
        )
        .unwrap();
        let got = ids(&["project", "nmmt", "rust"]);
        let (project, nmmt, rust) = (got[0], got[1], got[2]);
        let id = st.insert_memory("s", "c", &[], 1, 1).unwrap();

        // Nesting: project:nmmt while nmmt:rust is on the memory
        let err = st
            .set_memory_tags(id, &[(project, Some(nmmt)), (nmmt, Some(rust))])
            .unwrap_err();
        assert!(
            err.contains("nmmt") && err.contains("no nesting"),
            "got: {err}"
        );

        // Bare + target: rust bare while project:rust is on the memory
        let err = st
            .set_memory_tags(id, &[(project, Some(rust)), (rust, None)])
            .unwrap_err();
        assert!(
            err.contains("rust") && err.contains("bare tag"),
            "got: {err}"
        );

        // Self-edge
        let err = st.set_memory_tags(id, &[(nmmt, Some(nmmt))]).unwrap_err();
        assert!(err.contains("nmmt"), "got: {err}");

        // The valid shape still writes: edge + target-only tag + unrelated bare tag
        st.set_memory_tags(id, &[(project, Some(nmmt)), (rust, None)])
            .unwrap();
        let memory = &st.get_memories(&[id]).unwrap().0[0];
        assert_eq!(
            memory.tags,
            vec!["project:nmmt".to_string(), "rust".to_string()]
        );
        cleanup(&path);
    }

    #[test]
    fn tag_update_renames_and_sets_description() {
        let path = temp_db("rename");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("rust", "language").unwrap();
        st.tag_create("lang", "").unwrap();
        let a = insert_with_tags(&st, "a", "ca", &["rust"], 1);
        let _b = insert_with_tags(&st, "b", "cb", &["rust"], 1);

        // Renaming onto an existing tag: errors out
        let err = st.tag_update("rust", Some("lang"), None).unwrap_err();
        assert!(err.contains("already exists"), "got: {err}");

        // Normal rename: memory references follow the new name (internal id unchanged)
        let (renamed, description_updated) = st.tag_update("rust", Some("systems"), None).unwrap();
        assert!(renamed);
        assert!(!description_updated);
        assert!(!st.tag_exists("rust").unwrap());
        assert!(st.tag_exists("systems").unwrap());
        let (found, _) = st.get_memories(&[a]).unwrap();
        assert_eq!(found[0].tags, vec!["systems".to_string()]);

        // Description-only change: no rename (new_name omitted), the return value distinguishes both parts
        let (renamed, description_updated) =
            st.tag_update("systems", None, Some("programming")).unwrap();
        assert!(!renamed);
        assert!(description_updated);
        assert_eq!(
            st.tag_view("systems").unwrap()["description"],
            "programming"
        );

        // new_name equal to the current name: not a rename; the description is written as usual
        let (renamed, _) = st
            .tag_update("systems", Some("systems"), Some("langs"))
            .unwrap();
        assert!(!renamed);
        assert_eq!(st.tag_view("systems").unwrap()["description"], "langs");

        // Nonexistent tag errors out
        assert!(st.tag_update("nope", Some("x"), None).is_err());
        cleanup(&path);
    }

    /// A rename follows a tag through every role: edge targets reference the same vocabulary id
    /// as sources, so one rename updates all display forms at once.
    #[test]
    fn rename_follows_targets_and_sources_alike() {
        let path = temp_db("rename-roles");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let a = insert_with_tags(&st, "a", "ca", &["project:nmmt", "topic:nmmt"], 1);
        let b = insert_with_tags(&st, "b", "cb", &["nmmt"], 1);
        let c = insert_with_tags(&st, "c", "cc", &["nmmt:rust"], 1);

        st.tag_update("nmmt", Some("renamed-project"), None)
            .unwrap();
        for (id, expected) in [
            (
                a,
                vec![
                    "project:renamed-project".to_string(),
                    "topic:renamed-project".to_string(),
                ],
            ),
            (b, vec!["renamed-project".to_string()]),
            (c, vec!["renamed-project:rust".to_string()]),
        ] {
            let (found, _) = st.get_memories(&[id]).unwrap();
            assert_eq!(found[0].tags, expected);
        }
        cleanup(&path);
    }

    #[test]
    fn tag_delete_detach_vs_purge() {
        let path = temp_db("delete");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("x", "").unwrap();
        st.tag_create("keep", "").unwrap();
        let a = insert_with_tags(&st, "a", "ca", &["x"], 1);
        let _b = insert_with_tags(&st, "b", "cb", &["x", "keep"], 1);
        let _c = insert_with_tags(&st, "c", "cc", &["keep"], 1);
        // d carries x only as an edge target: detach must still count (and free) it
        let d = insert_with_tags(&st, "d", "cd", &["ref:x"], 1);

        let updated = st.tag_delete_detach("x").unwrap();
        assert_eq!(updated, 3, "a and b carry x directly, d as a target");
        assert!(!st.tag_exists("x").unwrap());
        assert_eq!(st.all_memories().unwrap().len(), 4);
        let (found, _) = st.get_memories(&[d]).unwrap();
        assert_eq!(
            found[0].tags,
            Vec::<String>::new(),
            "the edge died with its target"
        );
        let (found, _) = st.get_memories(&[a]).unwrap();
        assert_eq!(found[0].tags, Vec::<String>::new());

        // Recreate x and purge: the memory is deleted along with it
        st.tag_create("x", "").unwrap();
        let all = st.all_memories().unwrap();
        let b_id = Store::parse_id(&all[1].id).unwrap();
        let x_id = st.tag_ids_by_names(&["x".into()]).unwrap();
        st.update_memory(b_id, None, None, &[(x_id[0], None)], &[])
            .unwrap();
        let deleted = st.tag_delete_purge("x").unwrap();
        assert_eq!(deleted.len(), 1);
        assert_eq!(st.all_memories().unwrap().len(), 3);
        assert!(!st.tag_exists("x").unwrap());
        cleanup(&path);
    }

    #[test]
    fn tag_create_unique_and_case_hint() {
        let path = temp_db("case");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("rust", "").unwrap();
        let err = st.tag_create("rust", "");
        assert!(err.unwrap_err().contains("already exists"));
        assert_eq!(
            st.find_tag_case_insensitive("Rust").unwrap(),
            Some("rust".to_string())
        );
        assert_eq!(st.find_tag_case_insensitive("rust").unwrap(), None);
        cleanup(&path);
    }
}
