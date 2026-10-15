//! The key registry: structured `key:value` tags are disciplined by registered keys.
//!
//! Value types: `enum` values are reusable tag-vocabulary words; `number` values are numeric
//! quantities in canonical decimal form. Both are stored as ordinary `key:value` tags in the
//! tags table, so the whole tag pipeline (rename, merge, case hints, counts, listing) applies
//! to attribute values unchanged. Keys may be lazily created on first attribute write; this
//! module holds the explicit management paths.

use super::Store;
use crate::{
    model::Memory,
    sql,
    tag_expr::{TagAtom, TagExpr},
};
use rusqlite::params;
use serde_json::{json, Map, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct KeyRow {
    pub name: String,
    pub key_type: String,
    pub multi: bool,
    pub description: String,
    pub created_at: u64,
}

pub const MAX_KEY_DESC_CHARS: usize = 512;
/// API limit: at most this many attributes on one memory.
pub const MAX_ATTRIBUTES: usize = 64;
/// API limit: at most this many values on one multi-valued attribute.
pub const MAX_ATTRIBUTE_VALUES: usize = 32;

/// The query atom meaning "memory carries this key": a prefix regex over tag names (the
/// composed `key:value` tags are ordinary tags, so existence is a namespace match for both
/// value types).
fn key_existence_atom(key: &str) -> TagAtom {
    let pattern = format!("^{}:", regex::escape(key));
    TagAtom::Regex(Box::new(
        regex::Regex::new(&pattern).expect("escaped prefix regex"),
    ))
}

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
        let rows = statement.query_map([], decode).map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn key_get(&self, name: &str) -> Result<Option<KeyRow>, String> {
        match self.conn.query_row(sql::KEY_GET, [name], decode) {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Register or reconfigure a key. Type changes apply to future writes; existing value tags
    /// are grandfathered. Both value types are stored the same way, so reconfiguring never
    /// moves data.
    pub fn key_set(
        &self,
        raw_name: &str,
        key_type: &str,
        multi: bool,
        description: &str,
    ) -> Result<KeyRow, String> {
        let name = crate::model::normalize_key_name(raw_name)?;
        if !matches!(key_type, "enum" | "number") {
            return Err("key type must be enum or number".into());
        }
        if description.chars().count() > MAX_KEY_DESC_CHARS {
            return Err(format!(
                "key description exceeds {MAX_KEY_DESC_CHARS} characters"
            ));
        }
        let existing = self.key_get(&name)?;
        let created_at = existing
            .as_ref()
            .map_or_else(crate::model::now, |k| k.created_at);
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

    /// Restore a key from a dump verbatim (created_at included) — the import-side path.
    pub fn key_restore(&self, row: &KeyRow) -> Result<(), String> {
        let name = crate::model::normalize_key_name(&row.name)?;
        if !matches!(row.key_type.as_str(), "enum" | "number") {
            return Err(format!(
                "invalid export: key '{}' has an unknown type",
                row.name
            ));
        }
        self.conn
            .execute(
                sql::KEY_PUT,
                params![
                    name,
                    row.key_type,
                    row.multi as i64,
                    row.description,
                    row.created_at as i64
                ],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Delete a key: its `key:value` value tags are detached from memories and their tag rows
    /// removed via the regular tag delete path. The vocabulary discipline goes with the key —
    /// values become unusable as such.
    pub fn key_delete(&self, raw_name: &str) -> Result<Value, String> {
        let name = crate::model::normalize_key_name(raw_name)?;
        self.key_get(&name)?
            .ok_or_else(|| format!("key '{name}' not found (see key_list)"))?;
        let mut detached_tags = 0u64;
        for row in self.key_value_tags(&name)? {
            detached_tags += self.tag_delete_detach(&row)?;
        }
        self.conn
            .execute(sql::KEY_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(json!({"deleted": true, "memories_updated": detached_tags}))
    }

    /// Tag names belonging to a key (`key:value` rows currently in the registry). The key half
    /// is compared with substr (exact, case-sensitive): LIKE would fold ASCII case across
    /// case-sensitive tag names.
    pub fn key_value_tags(&self, key: &str) -> Result<Vec<String>, String> {
        let mut statement = self
            .conn
            .prepare(sql::KEY_VALUE_TAGS)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([key], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    /// Rewrite query atoms by the key registry: a bare registered key becomes a value-existence
    /// prefix regex, and `key=value` becomes the stored `key:value` tag leaf (enum: the value
    /// verbatim; number: canonicalized, so `key=1.0` finds `key:1`). Unregistered bare names
    /// stay literal tags; unknown attr keys error out. Leaf validation downstream then reports
    /// unknown values with the usual did-you-mean.
    pub fn bind_query_atoms(&self, expr: TagExpr) -> Result<TagExpr, String> {
        let keys = self.key_rows()?;
        expr.try_map(&mut |atom| match atom {
            TagAtom::Tag(name) => match keys.iter().find(|k| k.name == name) {
                None => Ok(TagAtom::Tag(name)),
                Some(k) => Ok(key_existence_atom(&k.name)),
            },
            TagAtom::KeyEq { key, value } => {
                let row = keys
                    .iter()
                    .find(|k| k.name == key)
                    .ok_or_else(|| format!("unknown attribute key '{key}' (see key_list)"))?;
                let stored = match row.key_type.as_str() {
                    "enum" => {
                        crate::model::normalize_tag_name(&value).map_err(|_| {
                            format!(
                                "enum value '{value}' for key '{key}' must be an identifier word (letters, digits, '_', '-', '.')"
                            )
                        })?;
                        crate::model::compose_value_tag(&key, &value)
                    }
                    _ => {
                        let canonical = crate::model::parse_number_text(&value).ok_or_else(|| {
                            format!("attribute '{key}' is a number key: '{value}' is not a number")
                        })?;
                        crate::model::compose_value_tag(&key, &canonical)
                    }
                };
                Ok(TagAtom::Tag(stored))
            }
            other => Ok(other),
        })
    }

    /// Validate an `attributes` write against the key registry, lazily registering unknown keys
    /// (a string value infers an enum key, a number a number key — an admin can reconfigure
    /// later). Returns the composed `key:value` tag names to link through the tag pipeline and
    /// the keys created by this call.
    pub fn resolve_attributes(&self, attributes: &Value) -> Result<ResolvedAttributes, String> {
        let object = attributes
            .as_object()
            .ok_or("attributes must be an object mapping key names to values")?;
        if object.len() > MAX_ATTRIBUTES {
            return Err(format!(
                "at most {MAX_ATTRIBUTES} attributes per memory are allowed"
            ));
        }
        let mut resolved = ResolvedAttributes::default();
        for (raw_key, value) in object {
            if value.is_null() {
                continue; // explicit no-op: a value is cleared by not being present
            }
            let key = crate::model::normalize_key_name(raw_key)?;
            let key_row = match self.key_get(&key)? {
                Some(row) => row,
                None => {
                    // Lazy schema growth reflects the first write: a string value infers an
                    // enum key, a number a number key, and an array marks the key multi-valued
                    // (peeking past the wrapper for the element type). key_set reconfigures later.
                    let peeked = match value {
                        Value::Array(items) => items.first().cloned().unwrap_or(Value::Null),
                        other => other.clone(),
                    };
                    let inferred = match peeked {
                        Value::String(_) => "enum",
                        Value::Number(_) => "number",
                        _ => {
                            return Err(format!(
                                "attribute '{key}': values are identifier words (enum) or numbers; bools are not attribute values — presence flags are plain tags"
                            ))
                        }
                    };
                    self.key_set(&key, inferred, value.is_array(), "")?;
                    resolved.lazy_keys.push(key.clone());
                    self.key_get(&key)?.expect("just inserted")
                }
            };
            let values = match value {
                Value::Array(items) => {
                    if !key_row.multi {
                        return Err(format!(
                            "attribute '{key}' is single-valued; pass a scalar instead of an array"
                        ));
                    }
                    if items.len() > MAX_ATTRIBUTE_VALUES {
                        return Err(format!(
                            "attribute '{key}' exceeds {MAX_ATTRIBUTE_VALUES} values"
                        ));
                    }
                    items.clone()
                }
                scalar @ (Value::String(_) | Value::Number(_)) => vec![scalar.clone()],
                other => {
                    return Err(format!(
                        "attribute '{key}' must be a string or number{}, got {other}",
                        if key_row.multi {
                            " or an array of them"
                        } else {
                            ""
                        }
                    ))
                }
            };
            for v in values {
                let stored = match key_row.key_type.as_str() {
                    "enum" => {
                        let text = v.as_str().ok_or_else(|| {
                            format!("attribute '{key}' is an enum key: values must be strings")
                        })?;
                        crate::model::normalize_tag_name(text).map_err(|_| {
                            format!(
                                "enum value '{text}' for key '{key}' must be an identifier word (letters, digits, '_', '-', '.')"
                            )
                        })?;
                        crate::model::compose_value_tag(&key, text)
                    }
                    _ => {
                        let n = v.as_number().ok_or_else(|| {
                            format!("attribute '{key}' is a number key: values must be numbers")
                        })?;
                        let canonical = crate::model::canonical_number(n).ok_or_else(|| {
                            format!("attribute '{key}': {n} is not a finite number")
                        })?;
                        crate::model::compose_value_tag(&key, &canonical)
                    }
                };
                if !resolved.value_tags.contains(&stored) {
                    resolved.value_tags.push(stored);
                }
            }
        }
        Ok(resolved)
    }

    /// Typed view of a memory's attribute values, derived from its `key:value` tags and the key
    /// registry: enum values render as strings, number values as JSON numbers. Unregistered
    /// keys are skipped. One value renders as a scalar, several as an array.
    pub fn attributes_view(&self, m: &Memory) -> Result<Value, String> {
        let mut out = Map::new();
        for key in m.tag_keys() {
            let Some(row) = self.key_get(key)? else {
                continue;
            };
            let values: Vec<Value> = self
                .key_value_tags(key)?
                .into_iter()
                .filter_map(|tag| {
                    let (_, value) = tag.split_once(':')?;
                    if row.key_type == "number" {
                        crate::model::parse_number_text(value).map(|canonical| {
                            serde_json::from_str(&canonical).expect("canonical number")
                        })
                    } else {
                        Some(json!(value))
                    }
                })
                .collect();
            let rendered = match values.len() {
                0 => continue,
                1 => values.into_iter().next().expect("one value"),
                _ => Value::Array(values),
            };
            out.insert(key.to_string(), rendered);
        }
        Ok(Value::Object(out))
    }
}

#[derive(Default, Debug)]
pub struct ResolvedAttributes {
    /// Composed `key:value` tag names for the written values, to be linked through the tag
    /// pipeline like any other tag.
    pub value_tags: Vec<String>,
    /// Keys registered by this call (lazy schema growth).
    pub lazy_keys: Vec<String>,
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
        // Bad types and names rejected
        assert!(st.key_set("x", "string", false, "").is_err());
        assert!(st.key_set("x", "bool", false, "").is_err());
        assert!(st.key_set("a:b", "enum", false, "").is_err());
        assert!(st.key_set("ok", "enum", false, &"x".repeat(513)).is_err());
        assert!(st.key_get("missing").unwrap().is_none());
        cleanup(&path);
    }

    #[test]
    fn key_delete_detaches_value_tags_for_both_types() {
        let path = temp_db("key-delete");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.key_set("project", "enum", false, "").unwrap();
        st.key_set("count", "number", false, "").unwrap();
        st.tag_create("project:alpha", "").unwrap();
        st.tag_create("count:42", "").unwrap();
        st.tag_create("unrelated", "").unwrap();
        // Attach both values to a memory through the regular link path
        let ids = st
            .link_tags(&["project:alpha".to_string(), "count:42".to_string()], true)
            .unwrap()
            .ids;
        let memory = st
            .insert_memory("s", "c", &ids, crate::model::now(), crate::model::now())
            .unwrap();
        assert_eq!(st.key_value_tags("project").unwrap().len(), 1);
        let out = st.key_delete("project").unwrap();
        assert_eq!(out["memories_updated"].as_u64(), Some(1));
        assert!(st.key_get("project").unwrap().is_none());
        assert!(st.key_value_tags("project").unwrap().is_empty());
        assert!(st.tag_exists("unrelated").unwrap());
        assert!(st.tag_exists("count:42").unwrap());
        let tags = st.original_tag_ids(memory).unwrap();
        assert_eq!(tags.len(), 1, "only the deleted key's value detached");
        assert!(st.key_delete("project").is_err());
        cleanup(&path);
    }

    /// Case sensitivity: keys are case-sensitive names, and the value lookup must not fold
    /// ASCII case across them (LIKE would have).
    #[test]
    fn key_value_tags_do_not_fold_case() {
        let path = temp_db("key-value-tags-case");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("project:alpha", "").unwrap();
        st.tag_create("Project:alpha", "").unwrap();
        assert_eq!(st.key_value_tags("project").unwrap(), vec!["project:alpha"]);
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
        st.key_set("count", "number", true, "").unwrap();
        st.tag_create("project:known", "").unwrap();

        let resolved = st
            .resolve_attributes(&json!({
                "project": "agent-memory",
                "count": [1, 2.5],
                "topic": "rust"
            }))
            .unwrap();
        // serde_json object iteration is alphabetical, so the composition order is too
        assert_eq!(
            resolved.value_tags,
            vec!["count:1", "count:2.5", "project:agent-memory", "topic:rust"]
        );
        assert_eq!(resolved.lazy_keys, vec!["topic"]);
        assert_eq!(st.key_get("topic").unwrap().unwrap().key_type, "enum");

        // Numbers canonicalize: 1 and 1.0 are the same stored name
        let resolved = st.resolve_attributes(&json!({"count": 1.0})).unwrap();
        assert_eq!(resolved.value_tags, vec!["count:1"]);

        for (invalid, expected) in [
            (json!({"project": ["a", "b"]}), "single-valued"),
            (json!({"project": "has space"}), "identifier word"),
            (json!({"count": "42"}), "number key"),
            (json!({"flag": true}), "plain tags"),
            (json!({"project": "a:b"}), "identifier word"),
        ] {
            let err = st.resolve_attributes(&invalid).unwrap_err();
            assert!(err.contains(expected), "{invalid}: got {err}");
        }
        // A string value on a number key is rejected, as is a number on an enum key
        assert!(st.resolve_attributes(&json!({"project": 1})).is_err());
        // null clears by absence; empty object resolves to nothing
        let cleared = st.resolve_attributes(&json!({"count": null})).unwrap();
        assert!(cleared.value_tags.is_empty() && cleared.lazy_keys.is_empty());
        let empty = st.resolve_attributes(&json!({})).unwrap();
        assert!(empty.value_tags.is_empty() && empty.lazy_keys.is_empty());
        // A scalar on a multi key is fine; null entries inside arrays are rejected
        assert!(st.resolve_attributes(&json!({"count": 7})).is_ok());
        assert!(st.resolve_attributes(&json!({"count": [1, null]})).is_err());
        cleanup(&path);
    }
}

#[cfg(test)]
mod query_tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;
    use crate::tag_expr;

    #[test]
    fn query_atoms_rewrite_to_tag_leaves() {
        let path = temp_db("query-atoms");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.key_set("project", "enum", false, "").unwrap();
        st.key_set("count", "number", false, "").unwrap();
        st.tag_create("project:alpha", "").unwrap();

        // enum equality rewrites to the composed tag leaf
        let expr = st
            .bind_query_atoms(tag_expr::parse("project=alpha").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["project:alpha"]);
        // number equality canonicalizes: 1.0 finds the stored 1
        let expr = st
            .bind_query_atoms(tag_expr::parse("count=1.0").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["count:1"]);
        // existence on a registered key becomes the prefix regex, for both types
        for name in ["project", "count"] {
            let expr = st.bind_query_atoms(tag_expr::parse(name).unwrap()).unwrap();
            assert!(
                expr.eval_with(&|a| matches!(a, TagAtom::Regex(_))),
                "{name}"
            );
        }
        // Unregistered bare names stay literal tags; unknown attr keys and malformed values error
        let expr = st
            .bind_query_atoms(tag_expr::parse("rust").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Tag(n) if n == "rust")));
        assert!(st
            .bind_query_atoms(tag_expr::parse("nokey=x").unwrap())
            .is_err());
        assert!(st
            .bind_query_atoms(tag_expr::parse("count=notanumber").unwrap())
            .is_err());
        assert!(st
            .bind_query_atoms(tag_expr::parse("project='has space'").unwrap())
            .is_err());
        cleanup(&path);
    }
}

#[cfg(test)]
mod view_tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    #[test]
    fn attributes_view_renders_typed_values() {
        let path = temp_db("attributes-view");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let names: Vec<String> = ["project:alpha", "count:1", "scores:2", "scores:3", "plain"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let ids = st.link_tags(&names, true).unwrap().ids;
        let id = st
            .insert_memory("s", "c", &ids, crate::model::now(), crate::model::now())
            .unwrap();
        st.key_set("project", "enum", false, "").unwrap();
        st.key_set("count", "number", false, "").unwrap();
        st.key_set("scores", "number", true, "").unwrap();
        let memory = &st.get_memories(&[id]).unwrap().0[0];
        let view = st.attributes_view(memory).unwrap();
        assert_eq!(view["project"], serde_json::json!("alpha"));
        // number values are JSON numbers, not strings
        assert_eq!(view["count"], serde_json::json!(1));
        assert_eq!(view["scores"], serde_json::json!([2, 3]));
        assert!(view.get("plain").is_none(), "bare tags are not attributes");
        // Unregistered keys are skipped, not guessed
        let names: Vec<String> = vec!["ghost:x".to_string()];
        let ids = st.link_tags(&names, true).unwrap().ids;
        st.insert_memory("s2", "c2", &ids, 1, 1).unwrap();
        let memory = &st.get_memories(&[id]).unwrap().0[0];
        assert!(st.attributes_view(memory).unwrap().get("ghost").is_none());
        cleanup(&path);
    }
}
