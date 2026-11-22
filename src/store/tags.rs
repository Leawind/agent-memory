//! Tag and scope data operations: create / views / rename (memberships follow the id) / delete
//! (detach and purge), plus the name resolution every write path goes through.
//!
//! A tag name is either bare (`rust`) or scoped (`system::pc1sw0`); the scope is an entity of its
//! own (`tag_scopes`) holding the description and the "required on every write" contract. Scopes
//! and tags are separate namespaces, so one name can be both a bare tag and a scope.

use crate::model::{tag_scope_of, tag_value_of, RESERVED_TAG};
use crate::sql;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use std::collections::HashMap;

use super::{is_unique_violation, Store};

/// One tag carried by a memory: its internal id and full display name (`rust` / `system::pc1sw0`).
#[derive(Debug)]
pub struct TagRow {
    pub tag_id: i64,
    pub name: String,
}

/// Link result for a batch of tag names (see `resolve_tags`).
#[derive(Debug)]
pub struct TagLinkage {
    /// Tag ids, aligned with the deduplicated input names in first-appearance order.
    pub links: Vec<i64>,
    /// Tags created by this call.
    pub autocreated: Vec<String>,
    /// Tags that already existed and were reused.
    pub reused: Vec<String>,
    /// Reused tags with an empty description (hinting to fill it in).
    pub missing_description: Vec<String>,
}

/// A scoped name whose scope does not exist: scopes are entities, never created as a side effect
/// of tagging (the caller creates the scope first).
fn unknown_scope(scope: &str, tag: &str) -> String {
    format!("unknown scope '{scope}' of tag '{tag}': create the scope first")
}

impl Store {
    pub fn tag_exists(&self, name: &str) -> Result<bool, String> {
        self.conn
            .query_row(sql::TAG_EXISTS, [name], |_| Ok(()))
            .optional()
            .map(|row| row.is_some())
            .map_err(|e| e.to_string())
    }

    pub fn scope_exists(&self, name: &str) -> Result<bool, String> {
        self.conn
            .query_row(sql::SCOPE_EXISTS, [name], |_| Ok(()))
            .optional()
            .map(|row| row.is_some())
            .map_err(|e| e.to_string())
    }

    /// Find existing tags differing from the given name only in case (compared on the Rust side with consistent Unicode semantics).
    pub fn find_tag_case_insensitive(&self, name: &str) -> Result<Option<String>, String> {
        let mut st = self
            .conn
            .prepare_cached(sql::TAG_ALL_NAMES)
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

    /// The scope's internal id and whether it is required on every write; None when it is missing.
    fn scope_row(&self, name: &str) -> Result<Option<(i64, bool)>, String> {
        self.conn
            .query_row(sql::SCOPE_BY_NAME, [name], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? != 0))
            })
            .optional()
            .map_err(|e| e.to_string())
    }

    /// The scope id a tag name belongs to (None for a bare name), refusing a name whose scope does
    /// not exist — a rename that moves a tag to another scope lands here too.
    fn scope_id_for(&self, name: &str) -> Result<Option<i64>, String> {
        let Some(scope) = tag_scope_of(name) else {
            return Ok(None);
        };
        match self.scope_row(scope)? {
            Some((id, _)) => Ok(Some(id)),
            None => Err(unknown_scope(scope, name)),
        }
    }

    /// Find existing scopes differing from the given name only in case (the did-you-mean source
    /// for a query that names a scope).
    pub fn find_scope_case_insensitive(&self, name: &str) -> Result<Option<String>, String> {
        let mut st = self
            .conn
            .prepare_cached(sql::SCOPE_ALL_NAMES)
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

    /// Create the scope when it is missing and return its id: the first tag of a scope, and a
    /// write allowed to extend the vocabulary, both bring their scope into being.
    fn scope_ensure(&self, scope: &str) -> Result<i64, String> {
        self.conn
            .execute(
                sql::SCOPE_ENSURE,
                params![scope, "", crate::model::now() as i64],
            )
            .map_err(|e| e.to_string())?;
        match self.scope_row(scope)? {
            Some((id, _)) => Ok(id),
            None => Err(format!("cannot create scope '{scope}'")),
        }
    }

    /// Create a tag; errors when it already exists (making the unique constraint explicit). A
    /// scoped name brings its scope into being when the scope is new; a bare name never consults
    /// the scopes.
    pub fn tag_create(&self, name: &str, description: &str) -> Result<(), String> {
        let scope_id = match tag_scope_of(name) {
            Some(scope) => Some(self.scope_ensure(scope)?),
            None => None,
        };
        self.conn
            .execute(
                sql::TAG_CREATE,
                params![name, scope_id, description, crate::model::now() as i64],
            )
            .map(|_| ())
            .map_err(|e| {
                if is_unique_violation(&e) {
                    format!(
                        "tag '{name}' already exists (rename it with tag_update, or see tag_list)"
                    )
                } else {
                    e.to_string()
                }
            })
    }

    /// Create a scope; errors when it already exists. The new scope starts out not required.
    pub fn scope_create(&self, name: &str, description: &str) -> Result<(), String> {
        self.conn
            .execute(
                sql::SCOPE_CREATE,
                params![name, description, crate::model::now() as i64],
            )
            .map(|_| ())
            .map_err(|e| {
                if is_unique_violation(&e) {
                    format!("scope '{name}' already exists")
                } else {
                    e.to_string()
                }
            })
    }

    /// Update a scope in place — never an upsert, the scope must already exist. `required: Some(true)`
    /// starts the write contract now and `Some(false)` clears it. Returns (description written,
    /// requirement toggled), so a caller can tell a no-op from a change.
    pub fn scope_update(
        &self,
        name: &str,
        description: Option<&str>,
        required: Option<bool>,
    ) -> Result<(bool, bool), String> {
        if !self.scope_exists(name)? {
            return Err(format!("scope '{name}' not found (see tag_list)"));
        }
        let mut description_updated = false;
        if let Some(text) = description {
            description_updated = self
                .conn
                .execute(sql::SCOPE_SET_DESCRIPTION, params![text, name])
                .map_err(|e| e.to_string())?
                > 0;
        }
        let mut required_updated = false;
        if let Some(flag) = required {
            required_updated = self
                .conn
                .execute(
                    sql::SCOPE_SET_REQUIRED,
                    params![name, i64::from(flag), crate::model::now() as i64],
                )
                .map_err(|e| e.to_string())?
                > 0;
        }
        Ok((description_updated, required_updated))
    }

    /// One sparse view per scope, ordered by memory count descending then name ascending.
    pub fn scope_views(&self) -> Result<Vec<Value>, String> {
        let mut st = self
            .conn
            .prepare_cached(sql::SCOPE_VIEW_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], row_to_scope_view)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Scopes declared on every memory write, in name order. Their declaration is the write
    /// contract: a memory states a value, or null meaning it takes no value in that scope (the
    /// statement itself is not stored — absence is the record, and each scope's `required_since`
    /// is what separates a declared absence from an unclassified older memory).
    pub fn required_scopes(&self) -> Result<Vec<String>, String> {
        let mut st = self
            .conn
            .prepare_cached(sql::SCOPE_REQUIRED_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Whether the scope must be declared on every write (false for an unknown name).
    /// Set the write contract with an explicit start time: a backup carries the cutoff, so a
    /// restored store keeps the same boundary between declared and unclassified memories instead
    /// of re-classifying everything from the restore moment.
    pub fn scope_set_required_since(&self, name: &str, since: i64) -> Result<(), String> {
        self.conn
            .execute(sql::SCOPE_SET_REQUIRED_SINCE, params![name, since])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Single tag view: description, memory count, and the scope the tag belongs to.
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
    /// for browsing the taxonomy).
    pub fn tag_views(&self) -> Result<Vec<Value>, String> {
        let mut st = self
            .conn
            .prepare_cached(sql::TAG_VIEW_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], row_to_tag_view)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Update a tag: rename and/or change the description. Memberships point at the internal id, so
    /// a rename only touches the name (the scope column follows it); renaming onto an existing name
    /// errors like exact-duplicate creation. The requirement is a property of the scope, so it
    /// stays where it is whatever tags come and go. Returns (whether a rename happened, whether a
    /// description was written).
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
            let scope_id = self.scope_id_for(final_name)?;
            self.conn
                .execute(sql::TAG_SET_NAME, params![final_name, scope_id, name])
                .map_err(|e| e.to_string())?;
            renamed = true;
        }
        let mut description_updated = false;
        if let Some(text) = description {
            self.conn
                .execute(sql::TAG_SET_DESCRIPTION, params![text, final_name])
                .map_err(|e| e.to_string())?;
            description_updated = true;
        }
        Ok((renamed, description_updated))
    }

    /// The scope a tag belongs to and whether that scope is required (None for a bare or an unknown
    /// tag).
    fn tag_scope_requirement(&self, name: &str) -> Result<Option<(String, bool)>, String> {
        self.conn
            .query_row(sql::TAG_SCOPE_REQUIREMENT, [name], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? != 0))
            })
            .optional()
            .map_err(|e| e.to_string())
    }

    fn tag_memory_count(&self, name: &str) -> Result<u64, String> {
        self.conn
            .query_row(sql::TAG_MEMORY_COUNT, [name], |r| r.get::<_, i64>(0))
            .map(|n| n as u64)
            .map_err(|e| e.to_string())
    }

    /// detach: delete the tag, cascading away its memberships, returning the number of memories
    /// that carried it.
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

    /// Ids (as "m{n}") of every memory carrying the tag — the purge preview's impact list.
    pub fn tag_memory_ids(&self, name: &str) -> Result<Vec<String>, String> {
        let ids = self.ids_with_tag(name)?;
        Ok(ids.iter().map(|i| Self::format_id(*i)).collect())
    }

    /// purge: also delete every memory carrying the tag, returning the deleted memory ids.
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
            .prepare_cached(sql::MEMORY_IDS_WITH_TAG)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([name], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Resolve tag names to linkable ids. Duplicate names collapse to one link (the first
    /// occurrence keeps its position), so `links` is what the caller writes to `memory_tags`.
    ///
    /// A missing bare tag is created when `create_missing`; a scoped tag needs its scope, which is
    /// never created here. A required scope only accepts values that already exist — auto-creation
    /// would let the write contract rot through a typo. `autocreated` / `reused` /
    /// `missing_description` classify the names for the caller's response.
    pub fn resolve_tags(
        &self,
        names: &[String],
        create_missing: bool,
    ) -> Result<TagLinkage, String> {
        let mut distinct: Vec<&str> = Vec::with_capacity(names.len());
        for name in names {
            if !distinct.contains(&name.as_str()) {
                distinct.push(name);
            }
        }

        // Phase 1 refuses everything before anything is written, so a failed call creates nothing.
        let mut scope_ids: Vec<Option<i64>> = Vec::with_capacity(distinct.len());
        let mut unknown: Vec<&str> = Vec::new();
        {
            let mut scope_stmt = self
                .conn
                .prepare_cached(sql::SCOPE_BY_NAME)
                .map_err(|e| e.to_string())?;
            let mut exists_stmt = self
                .conn
                .prepare_cached(sql::TAG_EXISTS)
                .map_err(|e| e.to_string())?;
            for name in &distinct {
                let mut required = false;
                let scope_id = match tag_scope_of(name) {
                    None => None,
                    Some(scope) => {
                        match scope_stmt.query_row([scope], |r| {
                            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)? != 0))
                        }) {
                            Ok((id, flag)) => {
                                required = flag;
                                Some(id)
                            }
                            Err(rusqlite::Error::QueryReturnedNoRows) => {
                                // A write allowed to extend the vocabulary brings its scope
                                // into being; anything else refuses the unknown scope.
                                if !create_missing {
                                    return Err(unknown_scope(scope, name));
                                }
                                Some(self.scope_ensure(scope)?)
                            }
                            Err(e) => return Err(e.to_string()),
                        }
                    }
                };
                scope_ids.push(scope_id);
                let exists = exists_stmt
                    .query_row([name], |_| Ok(()))
                    .optional()
                    .map_err(|e| e.to_string())?
                    .is_some();
                if exists {
                    continue;
                }
                if required {
                    return Err(format!(
                        "value '{}' of required scope '{}' is not an existing tag",
                        tag_value_of(name),
                        tag_scope_of(name).unwrap_or_default()
                    ));
                }
                if !create_missing {
                    unknown.push(name);
                }
            }
        }
        if !unknown.is_empty() {
            let mut described = Vec::with_capacity(unknown.len());
            for name in unknown {
                described.push(match self.find_tag_case_insensitive(name)? {
                    Some(similar) => format!("'{name}' (did you mean '{similar}'?)"),
                    None => format!("'{name}'"),
                });
            }
            return Err(format!(
                "unknown tags: {}; create them explicitly with tag_create, reuse the existing names, or pass create_missing_tags: true to auto-create them empty",
                described.join(", ")
            ));
        }

        // Phase 2 links, creating whatever phase 1 allowed to be created.
        let mut out = TagLinkage {
            links: Vec::with_capacity(distinct.len()),
            autocreated: Vec::new(),
            reused: Vec::new(),
            missing_description: Vec::new(),
        };
        let mut autocreate = self
            .conn
            .prepare_cached(sql::TAG_AUTOCREATE)
            .map_err(|e| e.to_string())?;
        let mut by_name = self
            .conn
            .prepare_cached(sql::TAG_BY_NAME)
            .map_err(|e| e.to_string())?;
        for (index, name) in distinct.iter().enumerate() {
            let inserted = autocreate
                .execute(params![name, scope_ids[index], crate::model::now() as i64])
                .map_err(|e| e.to_string())?;
            let (id, description): (i64, String) = by_name
                .query_row([name], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| e.to_string())?;
            out.links.push(id);
            if inserted > 0 {
                out.autocreated.push((*name).to_string());
            } else {
                out.reused.push((*name).to_string());
                if description.is_empty() {
                    out.missing_description.push((*name).to_string());
                }
            }
        }
        Ok(out)
    }

    /// All tag ids with their current names (id→name view source).
    pub fn tag_id_names(&self) -> Result<HashMap<i64, String>, String> {
        let mut statement = self
            .conn
            .prepare_cached(sql::TAG_EXPORT_ALL)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Resolve existing tag names to internal ids; nonexistent names are skipped outright (matching the
    /// silent no-op semantics of unlink-by-name).
    pub fn tag_ids_by_names(&self, names: &[String]) -> Result<Vec<i64>, String> {
        let mut by_name = self
            .conn
            .prepare_cached(sql::TAG_BY_NAME)
            .map_err(|e| e.to_string())?;
        let mut ids = Vec::with_capacity(names.len());
        for n in names {
            let id = match by_name.query_row([n], |r| r.get::<_, i64>(0)) {
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

    /// The memory's tag rows, name-ordered.
    pub fn tag_rows_for(&self, id: i64) -> Result<Vec<TagRow>, String> {
        let mut statement = self
            .conn
            .prepare_cached(sql::MEMORY_TAG_ORIGINS)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([id], |row| {
                Ok(TagRow {
                    tag_id: row.get(0)?,
                    name: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Replace a memory's tags wholesale, enforcing the model invariant first: on one memory, a
    /// scope carries at most one value, so two tags of the same scope cannot coexist.
    pub fn set_memory_tags(&self, memory_id: i64, tag_ids: &[i64]) -> Result<(), String> {
        self.validate_one_tag_per_scope(tag_ids)?;
        self.conn
            .execute(sql::MEMORY_CLEAR_TAGS, [memory_id])
            .map_err(|e| e.to_string())?;
        let mut link = self
            .conn
            .prepare_cached(sql::MEMORY_LINK_TAG)
            .map_err(|e| e.to_string())?;
        for tag_id in tag_ids {
            link.execute(params![memory_id, tag_id])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// The per-scope uniqueness invariant, checked on the final id set (so it covers every write
    /// path: create, update, merge, import). Names are resolved only when the check fails.
    fn validate_one_tag_per_scope(&self, tag_ids: &[i64]) -> Result<(), String> {
        if tag_ids.len() < 2 {
            return Ok(());
        }
        let id_set = json!(tag_ids).to_string();
        let mut st = self
            .conn
            .prepare_cached(sql::TAG_SCOPE_NAMES_FOR_IDS)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([id_set], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let mut by_scope: HashMap<String, String> = HashMap::new();
        for row in rows {
            let (tag, scope) = row.map_err(|e| e.to_string())?;
            if let Some(first) = by_scope.insert(scope.clone(), tag.clone()) {
                return Err(format!(
                    "at most one tag per scope: '{scope}' has both '{}' and '{}'",
                    tag_value_of(&first),
                    tag_value_of(&tag)
                ));
            }
        }
        Ok(())
    }

    /// A tag can be deleted only when it is not reserved, its scope is not required on every write,
    /// and no named predicate binds its id (decay rules reference individual tags, not groups).
    pub fn assert_tag_deletable(&self, name: &str) -> Result<(), String> {
        if name == RESERVED_TAG {
            return Err(format!(
                "tag '{name}' is reserved for the standing conventions and cannot be deleted"
            ));
        }
        if let Some((scope, required)) = self.tag_scope_requirement(name)? {
            if required {
                return Err(format!(
                    "tag '{name}' belongs to scope '{scope}', which must be declared on every write; clear the requirement first"
                ));
            }
        }
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

/// Sparse agent-facing shape: tokens are the currency, so fields carrying no information are
/// dropped outright — undescribed tags omit `description`, bare tags omit `scope`, and `reserved`
/// appears only on the reserved tag. Timestamps are not exposed at all: they cost several tokens
/// per row and rarely change what the agent does next.
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
    if let Some(scope) = r.get::<_, Option<String>>(3)? {
        v["scope"] = json!(scope);
    }
    if name == crate::model::RESERVED_TAG {
        v["reserved"] = json!(true);
    }
    Ok(v)
}

/// Same sparse rules as a tag view: undescribed scopes omit `description`, and `required` appears
/// only when the scope is required on every write.
fn row_to_scope_view(r: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let name: String = r.get(0)?;
    let count: i64 = r.get(2)?;
    let tag_count: i64 = r.get(3)?;
    let mut v = json!({
        "name": name,
        "count": count,
        "tag_count": tag_count,
    });
    let description: String = r.get(1)?;
    if !description.is_empty() {
        v["description"] = json!(description);
    }
    if r.get::<_, i64>(4)? != 0 {
        v["required"] = json!(true);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    fn names(tags: &[&str]) -> Vec<String> {
        tags.iter().map(|tag| (*tag).to_string()).collect()
    }

    /// Resolve names and attach them to a fresh memory (the production path resolves ids in the
    /// handler, so the test keeps the two steps apart).
    fn insert_with_tags(st: &Store, summary: &str, content: &str, tags: &[&str], at: u64) -> i64 {
        let links = st.resolve_tags(&names(tags), true).unwrap().links;
        let id = st.insert_memory(summary, content, &[], at, at).unwrap();
        st.set_memory_tags(id, &links).unwrap();
        id
    }

    /// resolve_tags' three-way classification: created / reused / reused-missing-description;
    /// a duplicate name collapses to one link and ids stay stable across calls.
    #[test]
    fn resolve_tags_classifies_autocreated_reused_and_missing_description() {
        let path = temp_db("link-tags");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("described", "has description").unwrap();

        let first = st
            .resolve_tags(&names(&["described", "fresh", "fresh"]), true)
            .unwrap();
        assert_eq!(first.autocreated, vec!["fresh".to_string()]);
        assert_eq!(first.reused, vec!["described".to_string()]);
        assert!(first.missing_description.is_empty());
        assert_eq!(
            first.links.len(),
            2,
            "a duplicate name collapses to one link"
        );

        let second = st
            .resolve_tags(&names(&["described", "fresh"]), true)
            .unwrap();
        assert!(second.autocreated.is_empty());
        assert_eq!(
            second.reused,
            vec!["described".to_string(), "fresh".to_string()]
        );
        assert_eq!(second.missing_description, vec!["fresh".to_string()]);
        assert_eq!(first.links, second.links, "tag ids are stable");
        cleanup(&path);
    }

    /// Without create_missing an unknown name is an error naming it and any case-variant already in
    /// the store; nothing is created on a refused call.
    #[test]
    fn resolve_tags_refuses_unknown_names_without_auto_creation() {
        let path = temp_db("link-unknown");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("rust", "").unwrap();

        let err = st
            .resolve_tags(&names(&["Rust", "新标签"]), false)
            .unwrap_err();
        assert!(
            err.contains("'Rust'") && err.contains("did you mean 'rust'"),
            "got: {err}"
        );
        assert!(err.contains("create_missing_tags: true"), "got: {err}");
        assert_eq!(st.tag_count_unreserved().unwrap(), 1, "nothing was created");
        cleanup(&path);
    }

    /// A scoped name needs its scope to exist, a required scope needs its value to exist, and a
    /// name may be both a bare tag and a scope (the namespaces are separate).
    #[test]
    fn scopes_gate_scoped_tags_and_are_never_created_implicitly() {
        let path = temp_db("scopes-gate");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.scope_create("system", "").unwrap();
        assert!(
            st.scope_create("system", "").is_err(),
            "scope names are unique"
        );
        assert!(st.tag_create("system::pc1sw0", "laptop").is_ok());
        assert!(!st.tag_exists("system").unwrap(), "a scope is not a tag");
        assert!(st
            .tag_create("system", "same name, other namespace")
            .is_ok());

        // A scoped tag brings its scope into being; the two namespaces stay independent.
        assert!(st.tag_create("ghost::value", "").is_ok());
        assert!(st.scope_exists("ghost").unwrap());
        assert!(!st.tag_exists("ghost").unwrap());
        // A write that may not extend the vocabulary refuses an unknown scope ...
        let err = st
            .resolve_tags(&names(&["phantom::value"]), false)
            .unwrap_err();
        assert!(err.starts_with("unknown scope 'phantom'"), "got: {err}");
        assert!(!st.scope_exists("phantom").unwrap());
        // ... while a write that may extend it creates the scope on the way in.
        assert!(st.resolve_tags(&names(&["phantom::value"]), true).is_ok());
        assert!(st.scope_exists("phantom").unwrap());

        // A required scope only accepts values that already exist, even with create_missing
        let (_, required_updated) = st.scope_update("system", None, Some(true)).unwrap();
        assert!(required_updated);
        assert_eq!(st.required_scopes().unwrap(), vec!["system".to_string()]);
        let err = st
            .resolve_tags(&names(&["system::pc1sw1"]), true)
            .unwrap_err();
        assert_eq!(
            err,
            "value 'pc1sw1' of required scope 'system' is not an existing tag"
        );
        assert!(st.resolve_tags(&names(&["system::pc1sw0"]), true).is_ok());
        cleanup(&path);
    }

    /// One tag per scope on a memory: the second value of a scope is refused by name, and a refused
    /// write leaves the previous set untouched.
    #[test]
    fn set_memory_tags_enforces_one_tag_per_scope() {
        let path = temp_db("one-per-scope");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.scope_create("system", "").unwrap();
        st.scope_create("project", "").unwrap();
        let links = st
            .resolve_tags(
                &names(&["system::a", "system::b", "project::p", "rust"]),
                true,
            )
            .unwrap()
            .links;
        let id = st.insert_memory("s", "c", &[], 1, 1).unwrap();
        st.set_memory_tags(id, &[links[0], links[2], links[3]])
            .unwrap();
        assert_eq!(
            st.tag_memory_ids("system::a").unwrap(),
            vec![Store::format_id(id)]
        );

        let err = st.set_memory_tags(id, &[links[0], links[1]]).unwrap_err();
        assert_eq!(
            err,
            "at most one tag per scope: 'system' has both 'a' and 'b'"
        );
        assert_eq!(
            st.tag_rows_for(id).unwrap().len(),
            3,
            "the refused write changed nothing"
        );
        cleanup(&path);
    }

    /// Rename and description are the two halves of an update; a rename may move a tag to another
    /// scope, and memberships follow the id instead of the name.
    #[test]
    fn tag_update_renames_across_scopes_and_keeps_memberships() {
        let path = temp_db("rename");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.scope_create("project", "").unwrap();
        st.scope_create("topic", "").unwrap();
        st.tag_create("project::nmmt", "toolchain").unwrap();
        st.tag_create("lang", "").unwrap();
        let id = insert_with_tags(&st, "a", "ca", &["project::nmmt"], 1);

        let err = st
            .tag_update("project::nmmt", Some("lang"), None)
            .unwrap_err();
        assert!(err.contains("already exists"), "got: {err}");
        let err = st
            .tag_update("project::nmmt", Some("ghost::x"), None)
            .unwrap_err();
        assert!(err.starts_with("unknown scope 'ghost'"), "got: {err}");

        let (renamed, described) = st
            .tag_update("project::nmmt", Some("topic::nmmt"), None)
            .unwrap();
        assert!(renamed && !described);
        assert!(!st.tag_exists("project::nmmt").unwrap());
        assert_eq!(
            st.tag_view("topic::nmmt").unwrap()["scope"],
            "topic",
            "the scope column follows the name"
        );
        assert_eq!(
            st.tag_memory_ids("topic::nmmt").unwrap(),
            vec![Store::format_id(id)]
        );

        let (renamed, described) = st
            .tag_update("topic::nmmt", None, Some("toolchain"))
            .unwrap();
        assert!(!renamed && described);
        assert_eq!(
            st.tag_view("topic::nmmt").unwrap()["description"],
            "toolchain"
        );
        assert!(st.tag_update("nope", Some("x"), None).is_err());
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

    /// detach drops the tag and frees its memories, reporting how many carried it; purge deletes
    /// those memories too and returns their ids.
    #[test]
    fn tag_delete_detach_vs_purge() {
        let path = temp_db("delete");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("x", "").unwrap();
        st.tag_create("keep", "").unwrap();
        let a = insert_with_tags(&st, "a", "ca", &["x"], 1);
        let _b = insert_with_tags(&st, "b", "cb", &["x", "keep"], 2);
        let c = insert_with_tags(&st, "c", "cc", &["keep"], 3);

        assert_eq!(st.tag_delete_detach("x").unwrap(), 2);
        assert!(!st.tag_exists("x").unwrap());
        assert!(
            st.tag_rows_for(a).unwrap().is_empty(),
            "the membership died with the tag"
        );
        assert!(st.tag_delete_detach("x").unwrap_err().contains("not found"));

        // purge: recreate x on one memory, then the tag and that memory go together
        st.tag_create("x", "").unwrap();
        let links = st.tag_ids_by_names(&names(&["x"])).unwrap();
        st.set_memory_tags(c, &links).unwrap();
        assert_eq!(st.tag_delete_purge("x").unwrap(), vec![Store::format_id(c)]);
        assert!(!st.tag_exists("x").unwrap());
        assert!(
            st.tag_rows_for(c).unwrap().is_empty(),
            "the memory is gone too"
        );
        cleanup(&path);
    }

    /// Scope views report each scope's membership (distinct memories carrying any of its tags) and
    /// tag count, in count-desc then name order; a tag view names its scope.
    #[test]
    fn scope_views_report_membership_and_requirement() {
        let path = temp_db("scope-views");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.scope_create("system", "machines").unwrap();
        st.scope_create("project", "").unwrap();
        st.scope_create("unused", "").unwrap();
        assert_eq!(
            st.scope_update("unused", Some("one tag planned"), None)
                .unwrap(),
            (true, false)
        );
        assert!(st.scope_update("ghost", None, Some(true)).is_err());
        st.scope_update("system", None, Some(true)).unwrap();
        // A required scope only accepts values that already exist, so the vocabulary comes first.
        st.tag_create("system::pc1sw0", "").unwrap();
        st.tag_create("system::pc1sw1", "").unwrap();
        insert_with_tags(&st, "a", "ca", &["system::pc1sw0", "project::nmmt"], 1);
        insert_with_tags(&st, "b", "cb", &["system::pc1sw1"], 2);

        let views = st.scope_views().unwrap();
        let ordered: Vec<&str> = views
            .iter()
            .map(|view| view["name"].as_str().unwrap())
            .collect();
        assert_eq!(ordered, vec!["system", "project", "unused"]);
        assert_eq!(views[0]["count"], 2);
        assert_eq!(views[0]["tag_count"], 2);
        assert_eq!(views[0]["required"], true);
        assert_eq!(views[0]["description"], "machines");
        assert_eq!(views[1]["count"], 1);
        assert!(views[1].get("required").is_none());
        assert!(views[1].get("description").is_none());
        assert_eq!(views[2]["count"], 0, "a scope without tags counts as empty");

        assert_eq!(st.tag_view("system::pc1sw0").unwrap()["scope"], "system");
        // Browse order: count desc, then name asc
        let tags = st.tag_views().unwrap();
        let ordered: Vec<&str> = tags
            .iter()
            .map(|view| view["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            ordered,
            vec![
                "project::nmmt",
                "system::pc1sw0",
                "system::pc1sw1",
                RESERVED_TAG
            ]
        );
        let reserved = tags.last().unwrap();
        assert!(reserved.get("scope").is_none(), "a bare tag has no scope");
        assert_eq!(reserved["reserved"], true, "the reserved marker survives");
        cleanup(&path);
    }

    /// Deletion guards: the reserved tag never goes away, and a tag of a required scope is refused
    /// until the requirement is cleared.
    #[test]
    fn deletion_refuses_reserved_tag_and_required_scope() {
        let path = temp_db("delete-guards");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        assert!(st
            .tag_delete_detach(RESERVED_TAG)
            .unwrap_err()
            .contains("reserved"));

        st.scope_create("system", "").unwrap();
        st.tag_create("system::pc1sw0", "").unwrap();
        st.scope_update("system", None, Some(true)).unwrap();
        assert_eq!(
            st.tag_delete_detach("system::pc1sw0").unwrap_err(),
            "tag 'system::pc1sw0' belongs to scope 'system', which must be declared on every write; clear the requirement first"
        );
        st.scope_update("system", None, Some(false)).unwrap();
        assert_eq!(st.tag_delete_detach("system::pc1sw0").unwrap(), 0);
        cleanup(&path);
    }
}
