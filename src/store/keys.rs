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

    /// Union the source memory's attribute rows into the target (single-value conflicts are
    /// grandfathered, matching the write-time-only enforcement everywhere else).
    pub fn merge_attributes(&self, into_id: i64, from_id: i64) -> Result<(), String> {
        self.conn
            .execute(sql::ATTRIBUTE_MERGE, params![into_id, from_id])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Validate an `attributes` write against the key registry, lazily registering unknown
    /// keys as enum + single-valued (the lazy-schema default; an admin can reconfigure later).
    /// Returns the attribute rows to store (string/bool), the full enum tag names to link,
    /// and the keys created by this call.
    pub fn resolve_attributes(
        &self,
        attributes: &Value,
    ) -> Result<ResolvedAttributes, String> {
        let object = attributes
            .as_object()
            .ok_or("attributes must be an object mapping key names to values")?;
        if object.len() > 64 {
            return Err("at most 64 attributes per memory are allowed".into());
        }
        let mut resolved = ResolvedAttributes::default();
        for (raw_key, value) in object {
            let key = crate::model::normalize_key_name(raw_key)?;
            let key_row = match self.key_get(&key)? {
                Some(row) => row,
                None => {
                    // Lazy schema growth: a boolean value signals a bool key; a value that can
                    // live in the tag vocabulary becomes an enum key; anything else (URLs,
                    // free text) is a string key. key_set reconfigures later.
                    let inferred = match value {
                        Value::Bool(_) => "bool",
                        Value::String(v) if crate::model::normalize_tag_name(&format!("{key}:{v}")).is_ok() => "enum",
                        _ => "string",
                    };
                    self.key_set(&key, inferred, false, "")?;
                    resolved.lazy_keys.push(key.clone());
                    self.key_get(&key)?.expect("just inserted")
                }
            };
            match key_row.key_type.as_str() {
                "bool" => match value {
                    Value::Bool(true) => resolved.rows.push((key, String::new())),
                    Value::Bool(false) | Value::Null => {}
                    other => {
                        return Err(format!(
                            "attribute '{key}' is a bool key: pass true or false, got {other}"
                        ))
                    }
                },
                "string" => {
                    for v in scalar_or_array(value, &key, key_row.multi)? {
                        let v = v.trim();
                        if v.is_empty() {
                            return Err(format!("attribute '{key}' cannot be empty"));
                        }
                        if v.chars().count() > MAX_ATTR_VALUE_CHARS {
                            return Err(format!(
                                "attribute '{key}' exceeds {MAX_ATTR_VALUE_CHARS} characters"
                            ));
                        }
                        resolved.rows.push((key.clone(), v.to_string()));
                    }
                }
                _ => {
                    for v in scalar_or_array(value, &key, key_row.multi)? {
                        let full = format!("{key}:{v}");
                        crate::model::normalize_tag_name(&full).map_err(|_| {
                            format!(
                                "enum value '{v}' for key '{key}' must be a tag-like name (letters, digits, '_', '-', '.')"
                            )
                        })?;
                        if !resolved.enum_tags.contains(&full) {
                            resolved.enum_tags.push(full);
                        }
                    }
                }
            }
        }
        Ok(resolved)
    }
}

/// API limit: maximum attribute value characters (a URL or free-text scalar).
pub const MAX_ATTR_VALUE_CHARS: usize = 1024;

#[derive(Default)]
pub struct ResolvedAttributes {
    /// Rows for memory_attributes (string values; empty value = bool presence).
    pub rows: Vec<(String, String)>,
    /// Full `key:value` tag names for enum keys, to be linked through the tag pipeline.
    pub enum_tags: Vec<String>,
    /// Keys registered by this call (lazy schema growth).
    pub lazy_keys: Vec<String>,
}

/// Accept a scalar, or an array when (and only when) the key is multi-valued.
fn scalar_or_array<'a>(
    value: &'a Value,
    key: &str,
    multi: bool,
) -> Result<Vec<&'a str>, String> {
    match value {
        Value::String(s) => Ok(vec![s.as_str()]),
        Value::Array(items) => {
            if !multi {
                return Err(format!(
                    "attribute '{key}' is single-valued; pass a scalar instead of an array"
                ));
            }
            if items.len() > 32 {
                return Err(format!("attribute '{key}' exceeds 32 values"));
            }
            items
                .iter()
                .map(|v| {
                    v.as_str().ok_or_else(|| {
                        format!("attribute '{key}' array entries must be strings")
                    })
                })
                .collect()
        }
        Value::Null => Ok(Vec::new()),
        other => Err(format!(
            "attribute '{key}' must be a string{}, got {other}",
            if multi { " or array of strings" } else { "" }
        )),
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

    /// One memory's attribute rows, ordered by (key, value).
    pub fn attributes_of(&self, id: i64) -> Result<Vec<(String, String)>, String> {
        let mut statement = self
            .conn
            .prepare(sql::ATTRIBUTE_ALL_FOR)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Replace a memory's whole attribute set in one shot (clear + insert).
    pub fn replace_attributes(&self, id: i64, rows: &[(String, String)]) -> Result<(), String> {
        self.conn
            .execute(sql::ATTRIBUTE_CLEAR_FOR_MEMORY, [id])
            .map_err(|e| e.to_string())?;
        for (key, value) in rows {
            self.conn
                .execute(sql::ATTRIBUTE_INSERT, params![id, key, value])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod attribute_tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    #[test]
    fn attributes_replace_and_load_roundtrip() {
        let path = temp_db("attributes");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let id = st
            .insert_memory("s", "c", &[], crate::model::now(), crate::model::now())
            .unwrap();
        st.replace_attributes(
            id,
            &[
                ("url".to_string(), "https://a".to_string()),
                ("url".to_string(), "https://b".to_string()),
                ("flag".to_string(), String::new()),
            ],
        )
        .unwrap();
        let memory = &st.get_memories(&[id]).unwrap().0[0];
        assert_eq!(
            memory.attrs,
            vec![
                ("flag".to_string(), String::new()),
                ("url".to_string(), "https://a".to_string()),
                ("url".to_string(), "https://b".to_string()),
            ]
        );
        assert_eq!(memory.tag_keys(), vec!["flag", "url"]);
        assert_eq!(memory.attributes_view()["url"], serde_json::json!(["https://a", "https://b"]));
        assert_eq!(memory.attributes_view()["flag"], serde_json::json!(true));
        // Full replacement drops the previous rows
        st.replace_attributes(id, &[("url".to_string(), "https://c".to_string())])
            .unwrap();
        let memory = &st.get_memories(&[id]).unwrap().0[0];
        assert_eq!(
            memory.attrs,
            vec![("url".to_string(), "https://c".to_string())]
        );
        // Deleting the key removes its rows everywhere
        st.key_set("url", "string", true, "").unwrap();
        st.key_set("flag", "bool", false, "").unwrap();
        st.key_delete("url").unwrap();
        assert!(st
            .attributes_of(id)
            .unwrap()
            .into_iter()
            .all(|(k, _)| k == "flag"));
        cleanup(&path);
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;
    use serde_json::json;

    #[test]
    fn resolve_validates_types_cardinality_and_lazy_keys() {
        let path = temp_db("resolve");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.key_set("project", "enum", false, "").unwrap();
        st.key_set("url", "string", true, "").unwrap();
        st.key_set("flag", "bool", false, "").unwrap();
        st.tag_create("project:known", "").unwrap();

        let resolved = st
            .resolve_attributes(&json!({
                "project": "agent-memory",
                "url": ["https://a", "https://b"],
                "flag": true,
                "topic": "rust"
            }))
            .unwrap();
        assert_eq!(resolved.enum_tags, vec!["project:agent-memory", "topic:rust"]);
        assert_eq!(
            resolved.rows,
            vec![
                ("flag".to_string(), String::new()),
                ("url".to_string(), "https://a".to_string()),
                ("url".to_string(), "https://b".to_string()),
            ]
        );
        assert_eq!(resolved.lazy_keys, vec!["topic"]);
        assert_eq!(st.key_get("topic").unwrap().unwrap().key_type, "enum");

        for invalid in [
            json!({"project": ["a", "b"]}),        // array on single-valued enum
            json!({"url": "x", "flag": "yes"}),    // string value on a bool key
            json!({"project": "has space"}),       // enum values are tag-like names
            json!({"url": 42}),                    // number values are not accepted (yet)
            json!({"url": ["", "x"]}),             // empty string values
            json!({"flag": null, "url": []}),      // ok but empty -> no error case below
        ] {
            let _ = invalid;
        }
        assert!(st.resolve_attributes(&json!({"project": ["a", "b"]})).is_err());
        assert!(st.resolve_attributes(&json!({"flag": "yes"})).is_err());
        assert!(st.resolve_attributes(&json!({"project": "has space"})).is_err());
        assert!(st.resolve_attributes(&json!({"url": 42})).is_err());
        assert!(st.resolve_attributes(&json!({"url": ["", "x"]})).is_err());
        // null and false clear; empty object resolves to nothing
        let cleared = st.resolve_attributes(&json!({"flag": false, "url": null})).unwrap();
        assert!(cleared.rows.is_empty() && cleared.enum_tags.is_empty());
        let empty = st.resolve_attributes(&json!({})).unwrap();
        assert!(empty.rows.is_empty() && empty.enum_tags.is_empty());
        // A scalar on a multi key is fine; null entries inside arrays are rejected
        assert!(st.resolve_attributes(&json!({"url": "solo"})).is_ok());
        assert!(st.resolve_attributes(&json!({"url": ["a", null]})).is_err());
        cleanup(&path);
    }
}
