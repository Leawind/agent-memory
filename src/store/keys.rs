//! Value-bearing tags: a tag is a `key:value` pair whose key may carry no value, and a key is
//! unique per memory (one value each). There is no key registry: values are identifier words or
//! numbers in canonical decimal form, stored verbatim as the value half of the composed name, so
//! the whole tag pipeline (rename, merge, case hints, counts, listing) applies unchanged.

use super::Store;
use crate::{
    model::Memory,
    sql,
    tag_expr::{TagAtom, TagExpr},
};
use serde_json::{json, Map, Value};

/// API limit: at most this many attributes on one memory.
pub const MAX_ATTRIBUTES: usize = 64;

/// The query atom meaning "memory carries this key with any value": a prefix regex over tag
/// names (the composed `key:value` tags are ordinary tags, so existence is a namespace match).
fn key_existence_atom(key: &str) -> TagAtom {
    let pattern = format!("^{}:", regex::escape(key));
    TagAtom::Regex(Box::new(
        regex::Regex::new(&pattern).expect("escaped prefix regex"),
    ))
}

impl Store {
    /// Tag names belonging to a key (the stored `key:value` rows with that key half). The key
    /// half is compared with substr (exact, case-sensitive): LIKE would fold ASCII case across
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

    /// Rewrite query atoms: `key=value` becomes the stored `key:value` tag leaf — a number
    /// canonicalizes (so `key=1.0` finds `key:1`), an identifier word goes in verbatim, anything
    /// else is a bind error. A bare word that is not an existing tag but prefixes stored
    /// `key:value` tags becomes a value-existence regex; everything else stays a literal tag,
    /// and leaf validation downstream reports unknown names with the usual did-you-mean.
    pub fn bind_query_atoms(&self, expr: TagExpr) -> Result<TagExpr, String> {
        expr.try_map(&mut |atom| match atom {
            TagAtom::Tag(name) => {
                if self.tag_exists(&name)? || self.key_value_tags(&name)?.is_empty() {
                    return Ok(TagAtom::Tag(name));
                }
                Ok(key_existence_atom(&name))
            }
            TagAtom::KeyEq { key, value } => {
                let stored = match crate::model::parse_number_text(&value) {
                    Some(canonical) => crate::model::compose_value_tag(&key, &canonical),
                    None => {
                        crate::model::normalize_tag_name(&value).map_err(|_| {
                            format!(
                                "attribute value '{value}' for key '{key}' must be an identifier word (letters, digits, '_', '-', '.') or a number"
                            )
                        })?;
                        crate::model::compose_value_tag(&key, &value)
                    }
                };
                Ok(TagAtom::Tag(stored))
            }
            other => Ok(other),
        })
    }

    /// Validate an `attributes` write: keys are identifier names (unique per memory — one value
    /// each), values are identifier words or numbers. Returns the composed `key:value` tag names
    /// to link through the tag pipeline like any other tag.
    pub fn resolve_attributes(&self, attributes: &Value) -> Result<Vec<String>, String> {
        let object = attributes
            .as_object()
            .ok_or("attributes must be an object mapping key names to values")?;
        if object.len() > MAX_ATTRIBUTES {
            return Err(format!(
                "at most {MAX_ATTRIBUTES} attributes per memory are allowed"
            ));
        }
        let mut value_tags = Vec::with_capacity(object.len());
        for (raw_key, value) in object {
            if value.is_null() {
                continue; // explicit no-op: a value is cleared by not being present
            }
            let key = crate::model::normalize_key_name(raw_key)?;
            let stored = match value {
                Value::String(text) => {
                    crate::model::normalize_tag_name(text).map_err(|_| {
                        format!(
                            "attribute '{key}': string values must be identifier words (letters, digits, '_', '-', '.'), got '{text}'"
                        )
                    })?;
                    crate::model::compose_value_tag(&key, text)
                }
                Value::Number(n) => {
                    let canonical = crate::model::canonical_number(n).ok_or_else(|| {
                        format!("attribute '{key}': {n} is not a finite number")
                    })?;
                    crate::model::compose_value_tag(&key, &canonical)
                }
                other => {
                    return Err(format!(
                        "attribute '{key}' must be a string or number, got {other} — a key holds one value per memory; bools are not values (presence flags are plain tags)"
                    ))
                }
            };
            value_tags.push(stored);
        }
        Ok(value_tags)
    }

    /// Typed view of a memory's attribute values, derived from its `key:value` tags alone: a
    /// value renders as a JSON number when it parses as a canonical number, otherwise as a
    /// string; one value per key is a scalar, several (legacy data) an array.
    pub fn attributes_view(&self, m: &Memory) -> Result<Value, String> {
        let mut out = Map::new();
        for key in m.tag_keys() {
            let values: Vec<Value> = self
                .key_value_tags(key)?
                .into_iter()
                .filter_map(|tag| {
                    let (_, value) = tag.split_once(':')?;
                    match crate::model::parse_number_text(value) {
                        Some(canonical) => {
                            Some(serde_json::from_str(&canonical).unwrap_or_else(|_| json!(value)))
                        }
                        None => Some(json!(value)),
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

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

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
    fn resolve_validates_values_without_a_registry() {
        let path = temp_db("resolve");
        cleanup(&path);
        let st = Store::open(&path).unwrap();

        // serde_json object iteration is alphabetical, so the composition order is too
        let resolved = st
            .resolve_attributes(&json!({"count": 2.5, "project": "agent-memory"}))
            .unwrap();
        assert_eq!(
            resolved,
            vec!["count:2.5".to_string(), "project:agent-memory".to_string()]
        );

        // Numbers canonicalize: 1 and 1.0 are the same stored name
        let resolved = st.resolve_attributes(&json!({"count": 1.0})).unwrap();
        assert_eq!(resolved, vec!["count:1".to_string()]);

        for (invalid, expected) in [
            (json!({"project": ["a", "b"]}), "one value per memory"),
            (json!({"project": "has space"}), "identifier words"),
            (json!({"flag": true}), "plain tags"),
            (json!({"project": "a:b"}), "identifier words"),
        ] {
            let err = st.resolve_attributes(&invalid).unwrap_err();
            assert!(err.contains(expected), "{invalid}: got {err}");
        }
        // null clears by absence; empty object resolves to nothing
        let cleared = st.resolve_attributes(&json!({"count": null})).unwrap();
        assert!(cleared.is_empty());
        let empty = st.resolve_attributes(&json!({})).unwrap();
        assert!(empty.is_empty());
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
        st.tag_create("project:alpha", "").unwrap();
        st.tag_create("count:1", "").unwrap();
        st.tag_create("rust", "").unwrap();

        // equality rewrites to the composed tag leaf, the value verbatim
        let expr = st
            .bind_query_atoms(tag_expr::parse("project=alpha").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["project:alpha"]);
        // numbers canonicalize: 1.0 finds the stored 1
        let expr = st
            .bind_query_atoms(tag_expr::parse("count=1.0").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["count:1"]);
        // a bare word that is not an existing tag but prefixes stored values tests existence
        let expr = st
            .bind_query_atoms(tag_expr::parse("project").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Regex(_))));
        // an existing bare tag stays literal even when it also prefixes values
        st.tag_create("count", "").unwrap();
        let expr = st
            .bind_query_atoms(tag_expr::parse("count").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Tag(n) if n == "count")));
        // unknown bare names stay literal tags (did-you-mean fires downstream)
        let expr = st
            .bind_query_atoms(tag_expr::parse("rust").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Tag(n) if n == "rust")));
        // malformed values error at bind time
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
        let names: Vec<String> = ["project:alpha", "count:1", "legacy:2", "legacy:3", "plain"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let ids = st.link_tags(&names, true).unwrap().ids;
        let id = st
            .insert_memory("s", "c", &ids, crate::model::now(), crate::model::now())
            .unwrap();
        let memory = &st.get_memories(&[id]).unwrap().0[0];
        let view = st.attributes_view(memory).unwrap();
        assert_eq!(view["project"], serde_json::json!("alpha"));
        // number-shaped values are JSON numbers, not strings
        assert_eq!(view["count"], serde_json::json!(1));
        // several values under one key (legacy data) render as an array
        assert_eq!(view["legacy"], serde_json::json!([2, 3]));
        assert!(view.get("plain").is_none(), "bare tags are not attributes");
        cleanup(&path);
    }
}
