//! The key registry: structured `key:value` tags are disciplined by registered keys.
//!
//! Key types: `enum` values live in the tags table as `key:value` names (full vocabulary
//! hygiene reused); `string` values are free per-memory literals in `memory_attributes`
//! (never listed); `bool` keys have no value — writing one means presence. Keys may be
//! lazily created on first attribute write; this module holds the explicit management paths.

use super::Store;
use crate::sql;
use rusqlite::params;
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct KeyRow {
    pub name: String,
    pub key_type: String,
    pub multi: bool,
    pub description: String,
    pub created_at: u64,
}

pub const MAX_KEY_DESC_CHARS: usize = 512;

fn decode(row: &rusqlite::Row<'_>) -> rusqlite::Result<KeyRow> {
    Ok(KeyRow {
        name: row.get(0)?,
        key_type: row.get(1)?,
        multi: row.get::<_, i64>(2)? != 0,
        description: row.get(3)?,
        created_at: row.get::<_, i64>(4)? as u64,
    })
}

impl Store {
    pub fn key_rows(&self) -> Result<Vec<KeyRow>, String> {
        let mut statement = self.conn.prepare(sql::KEY_ALL).map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], decode)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn key_get(&self, name: &str) -> Result<Option<KeyRow>, String> {
        match self.conn.query_row(sql::KEY_GET, [name], decode) {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Register or reconfigure a key. `bool` keys reject `multi` (presence is inherently
    /// single-valued). Type changes apply to future writes; existing data is grandfathered.
    pub fn key_set(
        &self,
        raw_name: &str,
        key_type: &str,
        multi: bool,
        description: &str,
    ) -> Result<KeyRow, String> {
        let name = crate::model::normalize_key_name(raw_name)?;
        if !matches!(key_type, "enum" | "string" | "bool") {
            return Err("key type must be enum, string or bool".into());
        }
        if description.chars().count() > MAX_KEY_DESC_CHARS {
            return Err(format!(
                "key description exceeds {MAX_KEY_DESC_CHARS} characters"
            ));
        }
        let multi = if key_type == "bool" { false } else { multi };
        let existing = self.key_get(&name)?;
        let created_at = existing.as_ref().map_or_else(crate::model::now, |k| k.created_at);
        self.conn
            .execute(
                sql::KEY_PUT,
                params![name, key_type, multi as i64, description, created_at as i64],
            )
            .map_err(|e| e.to_string())?;
        Ok(KeyRow {
            name,
            key_type: key_type.to_string(),
            multi,
            description: description.to_string(),
            created_at,
        })
    }

    /// Delete a key (detach semantics): enum values stop being linked and their `key:value`
    /// tag rows are removed via the regular tag delete path; string/bool attribute rows are
    /// cleared. The vocabulary discipline goes with the key — values become unusable as such.
    pub fn key_delete(&self, raw_name: &str) -> Result<Value, String> {
        let name = crate::model::normalize_key_name(raw_name)?;
        let key = self
            .key_get(&name)?
            .ok_or_else(|| format!("key '{name}' not found (see key_list)"))?;
        let mut detached_tags = 0u64;
        if key.key_type == "enum" {
            for row in self.key_value_tags(&name)? {
                detached_tags += self.tag_delete_detach(&row)?;
            }
        }
        self.clear_attributes_for_key(&name)?;
        self.conn
            .execute(sql::KEY_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(json!({"deleted": true, "memories_updated": detached_tags}))
    }

    /// Tag names belonging to an enum key (`key:value` rows currently in the registry).
    pub fn key_value_tags(&self, key: &str) -> Result<Vec<String>, String> {
        let mut statement = self
            .conn
            .prepare("SELECT name FROM tags WHERE name LIKE ?1 || ':%' ESCAPE '\\' ORDER BY name")
            .map_err(|e| e.to_string())?;
        let escaped = format!("{}%", key.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"));
        let rows = statement
            .query_map([escaped], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    #[test]
    fn keys_roundtrip_with_validation_and_defaults() {
        let path = temp_db("keys");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let key = st.key_set("project", "enum", false, "工作项目").unwrap();
        assert_eq!(key.name, "project");
        assert_eq!(key.key_type, "enum");
        assert!(!key.multi);
        // Re-set keeps creation time but updates fields
        let created = key.created_at;
        let key = st.key_set("project", "enum", true, "").unwrap();
        assert_eq!(key.created_at, created);
        assert!(key.multi);
        assert_eq!(st.key_rows().unwrap().len(), 1);
        // bool keys cannot be multi; bad types and names rejected
        assert!(!st.key_set("flag", "bool", true, "").unwrap().multi);
        assert!(st.key_set("x", "number", false, "").is_err());
        assert!(st.key_set("a:b", "enum", false, "").is_err());
        assert!(st.key_set("ok", "enum", false, &"x".repeat(513)).is_err());
        assert!(st.key_get("missing").unwrap().is_none());
        cleanup(&path);
    }

    #[test]
    fn enum_key_delete_detaches_value_tags_and_keeps_vocabulary_gone() {
        let path = temp_db("key-delete");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.key_set("project", "enum", false, "").unwrap();
        st.tag_create("project:alpha", "").unwrap();
        st.tag_create("project:beta", "").unwrap();
        st.tag_create("unrelated", "").unwrap();
        // Attach project:alpha to a memory through the regular link path
        let ids = st
            .link_tags(&["project:alpha".to_string()], true)
            .unwrap()
            .ids;
        let memory = st
            .insert_memory("s", "c", &ids, crate::model::now(), crate::model::now())
            .unwrap();
        assert_eq!(st.key_value_tags("project").unwrap().len(), 2);
        let out = st.key_delete("project").unwrap();
        assert_eq!(out["memories_updated"].as_u64(), Some(1));
        assert!(st.key_get("project").unwrap().is_none());
        assert!(st.key_value_tags("project").unwrap().is_empty());
        assert!(st.tag_exists("unrelated").unwrap());
        let tags = st.original_tag_ids(memory).unwrap();
        assert!(tags.is_empty(), "value links detached with the key");
        assert!(st.key_delete("project").is_err());
        cleanup(&path);
    }
}

impl Store {
    /// Remove every attribute row of one key (used on key deletion).
    pub fn clear_attributes_for_key(&self, key: &str) -> Result<(), String> {
        self.conn
            .execute(sql::ATTRIBUTE_CLEAR_FOR_KEY, [key])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
