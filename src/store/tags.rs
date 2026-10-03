//! Tag data operations: create / views / rename (foreign-key cascade) / delete (detach and purge).

use crate::sql;
use rusqlite::params;
use serde_json::{json, Value};

use super::{is_unique_violation, Store};

/// Link result for a batch of tag names (see `link_tags`).
pub struct TagLinkage {
    /// Internal tag ids aligned one-to-one with the input names (invisible externally).
    pub ids: Vec<i64>,
    /// Tag names created by this call.
    pub autocreated: Vec<String>,
    /// Tag names that already existed and were reused.
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

    /// Existing tag names matching the regex (returned as stored, for tag_set filtering via json_each).
    pub fn tag_names_matching(&self, re: &regex::Regex) -> Result<Vec<String>, String> {
        let mut st = self
            .conn
            .prepare(sql::TAG_ALL_NAMES)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| {
            let n = match r {
                Ok(n) => n,
                Err(e) => return Some(Err(e.to_string())),
            };
            if re.is_match(&n) {
                Some(Ok(n))
            } else {
                None
            }
        })
        .collect()
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

    /// Single tag view: description, memory count, last-used time.
    pub fn tag_view(&self, name: &str) -> Result<Value, String> {
        match self.conn.query_row(sql::TAG_VIEW, [name], row_to_tag_view) {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(format!("tag '{name}' not found (see tag_list)"))
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// All tag views, ordered by memory count descending then name ascending (the default order for browsing the taxonomy).
    pub fn tag_views(&self) -> Result<Vec<Value>, String> {
        let mut st = self
            .conn
            .prepare(sql::TAG_VIEW_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], row_to_tag_view)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Update a tag: rename and/or change the description. Tags link to memories by internal id, so renaming touches only name;
    /// references follow automatically. Errors when the target name already exists (same semantics as exact-duplicate tag creation).
    /// Returns (whether a rename happened, whether a description was written).
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

    /// detach: delete the tag (cascading away all references), returning the number of memories affected.
    pub fn tag_delete_detach(&self, name: &str) -> Result<u64, String> {
        if !self.tag_exists(name)? {
            return Err(format!("tag '{name}' not found (see tag_list)"));
        }
        let affected = self.tag_memory_count(name)?;
        self.conn
            .execute(sql::TAG_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(affected)
    }

    /// Ids (as "m{n}") of every memory carrying the tag — the purge preview's impact list.
    pub fn tag_memory_ids(&self, name: &str) -> Result<Vec<String>, String> {
        let ids = self.ids_with_tag(name)?;
        Ok(ids.iter().map(|i| Self::format_id(*i)).collect())
    }

    /// purge: also delete every memory carrying the tag, returning the deleted memory ids.
    pub fn tag_delete_purge(&self, name: &str) -> Result<Vec<String>, String> {
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

    /// Resolve tag names to linkable tags. With `create_missing` the missing names are auto-created
    /// (empty description) and classified into `autocreated` / `reused` / `missing_description`;
    /// without it, unknown names are an error that names each one and any case-variant already in
    /// the store — the agent decides between reusing, an explicit tag_create, or opting into
    /// auto-creation, so the taxonomy cannot rot through casual tagging.
    ///
    /// `ids` corresponds one-to-one with the input names; `autocreated` holds tags created this
    /// time, `reused` the already-existing ones, and `missing_description` the reused tags with an
    /// empty description (usually left over from earlier auto-creation — left to the caller whether
    /// to remind about filling them in).
    pub fn link_tags(&self, names: &[String], create_missing: bool) -> Result<TagLinkage, String> {
        if !create_missing {
            let mut unknown: Vec<String> = Vec::new();
            for n in names {
                if !self.tag_exists(n)? {
                    let mut entry = format!("'{n}'");
                    if let Some(similar) = self.find_tag_case_insensitive(n)? {
                        entry.push_str(&format!(" (did you mean '{similar}'?)"));
                    }
                    unknown.push(entry);
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
            ids: Vec::with_capacity(names.len()),
            autocreated: Vec::new(),
            reused: Vec::new(),
            missing_description: Vec::new(),
        };
        for n in names {
            let inserted = self
                .conn
                .execute(sql::TAG_AUTOCREATE, params![n, crate::model::now() as i64])
                .map_err(|e| e.to_string())?;
            let (id, description): (i64, String) = self
                .conn
                .query_row(sql::TAG_BY_NAME, [n], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| e.to_string())?;
            out.ids.push(id);
            if inserted > 0 {
                out.autocreated.push(n.clone());
            } else {
                out.reused.push(n.clone());
                if description.is_empty() {
                    out.missing_description.push(n.clone());
                }
            }
        }
        Ok(out)
    }

    /// Resolve existing tag names to internal ids; nonexistent names are skipped outright (matching the old
    /// silent no-op semantics of unlink-by-name).
    pub fn tag_ids_for_names(&self, names: &[String]) -> Result<Vec<i64>, String> {
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
}

fn row_to_tag_view(r: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let count: i64 = r.get(2)?;
    let last: Option<i64> = r.get(3)?;
    let created: i64 = r.get(4)?;
    let name: String = r.get(0)?;
    let reserved = name == crate::model::RESERVED_TAG;
    Ok(json!({
        "name": name,
        "description": r.get::<_, String>(1)?,
        "memory_count": count,
        "last_used_at": last.map(|t| json!(t)).unwrap_or(Value::Null),
        "created_at": created,
        "reserved": reserved,
    }))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    /// Test helper: link tags by name and insert a memory (the production path resolves ids in the handler).
    fn insert_with_tags(st: &Store, summary: &str, content: &str, tags: &[&str], at: u64) -> i64 {
        let names: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        let ids = st.link_tags(&names, true).unwrap().ids;
        st.insert_memory(summary, content, &ids, at, at).unwrap()
    }

    /// link_tags' three-way classification: created / reused / reused-missing-description; ids align with the input name by name.
    #[test]
    fn link_tags_classifies_autocreated_reused_and_missing_description() {
        let path = temp_db("link-tags");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("described", "has description").unwrap();

        let first = st
            .link_tags(&["described".into(), "fresh".into()], true)
            .unwrap();
        assert_eq!(first.autocreated, vec!["fresh".to_string()]);
        assert_eq!(first.reused, vec!["described".to_string()]);
        assert!(first.missing_description.is_empty());
        assert_eq!(first.ids.len(), 2);

        // Second call reuses everything; fresh, auto-created earlier with an empty description → lands in missing_description
        let second = st
            .link_tags(&["described".into(), "fresh".into()], true)
            .unwrap();
        assert!(second.autocreated.is_empty());
        assert_eq!(
            second.reused,
            vec!["described".to_string(), "fresh".to_string()]
        );
        assert_eq!(second.missing_description, vec!["fresh".to_string()]);
        // Same name → same id (tag ids are stable; renaming never changes the id)
        assert_eq!(first.ids, second.ids);
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

    #[test]
    fn tag_delete_detach_vs_purge() {
        let path = temp_db("delete");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("x", "").unwrap();
        st.tag_create("keep", "").unwrap();
        insert_with_tags(&st, "a", "ca", &["x"], 1);
        insert_with_tags(&st, "b", "cb", &["x", "keep"], 1);
        insert_with_tags(&st, "c", "cc", &["keep"], 1);

        let updated = st.tag_delete_detach("x").unwrap();
        assert_eq!(updated, 2);
        assert!(!st.tag_exists("x").unwrap());
        assert_eq!(st.all_memories().unwrap().len(), 3);

        // Recreate x and purge: the memory is deleted along with it
        st.tag_create("x", "").unwrap();
        let all = st.all_memories().unwrap();
        let b_id = Store::parse_id(&all[1].id).unwrap();
        let x_id = st.tag_ids_for_names(&["x".into()]).unwrap();
        st.update_memory(b_id, None, None, &x_id, &[]).unwrap();
        let deleted = st.tag_delete_purge("x").unwrap();
        assert_eq!(deleted.len(), 1);
        assert_eq!(st.all_memories().unwrap().len(), 2);
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
