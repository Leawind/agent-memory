//! Value-bearing tags: a tag is a `key:value` pair whose key may carry no value, and a key is
//! unique per memory (one value each). The vocabulary is the keys (the `tags` table); values
//! ride the relation rows, so queries and the typed view derive from stored relations alone.

use super::Store;
use crate::{
    model::Memory,
    sql,
    tag_expr::{TagAtom, TagExpr},
};
use serde_json::{json, Map, Value};

impl Store {
    /// Values a key carries across the store (the value halves of its relations, bare relations
    /// excluded). The key is compared exactly (case-sensitive): LIKE would fold ASCII case
    /// across case-sensitive key names.
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
    /// else is a bind error. A bare word that is not an existing key but carries valued relations
    /// somewhere in the store becomes a value-existence regex; everything else stays a literal
    /// tag, and leaf validation downstream reports unknown names with the usual did-you-mean.
    pub fn bind_query_atoms(&self, expr: TagExpr) -> Result<TagExpr, String> {
        expr.try_map(&mut |atom| match atom {
            TagAtom::Tag(name) => {
                if self.key_value_tags(&name)?.is_empty() {
                    return Ok(TagAtom::Tag(name));
                }
                // The word is a key with valued relations somewhere: existence means the memory
                // carries the key bare or with any value
                let pattern = format!("^{}(:|$)", regex::escape(&name));
                Ok(TagAtom::Regex(Box::new(
                    regex::Regex::new(&pattern).expect("escaped prefix regex"),
                )))
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

    /// Typed view of a memory's attribute values, derived from its valued relations alone: a
    /// value renders as a JSON number when it parses as a canonical number, otherwise as a
    /// string. One value per key by construction.
    pub fn attributes_view(&self, m: &Memory) -> Result<Value, String> {
        let mut out = Map::new();
        let id = Store::parse_id(&m.id).unwrap_or(0);
        for (_, key, value) in self.tag_rows_for(id)? {
            let Some(value) = value else { continue };
            let rendered = match crate::model::parse_number_text(&value) {
                Some(canonical) => {
                    serde_json::from_str(&canonical).unwrap_or_else(|_| json!(value))
                }
                None => json!(value),
            };
            out.insert(key, rendered);
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
        for key in ["project", "Project"] {
            let pairs: Vec<crate::model::TagPair> =
                vec![(key.to_string(), Some("alpha".to_string()))];
            let ids = st.link_keys(&pairs, true).unwrap().ids;
            let links: Vec<(i64, Option<String>)> = ids
                .into_iter()
                .zip(pairs.into_iter().map(|(_, v)| v))
                .collect();
            st.insert_memory("s", "c", &links, crate::model::now(), crate::model::now())
                .unwrap();
        }
        assert_eq!(st.key_value_tags("project").unwrap(), vec!["alpha"]);
        assert_eq!(st.key_value_tags("Project").unwrap(), vec!["alpha"]);
        cleanup(&path);
    }
}

#[cfg(test)]
mod query_tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;
    use crate::tag_expr;

    fn link(st: &Store, pairs: &[(&str, Option<&str>)]) {
        let pairs: Vec<crate::model::TagPair> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.map(|v| v.to_string())))
            .collect();
        let ids = st.link_keys(&pairs, true).unwrap().ids;
        let links: Vec<(i64, Option<String>)> = ids
            .into_iter()
            .zip(pairs.iter().map(|(_, v)| v.as_ref().map(|v| v.to_string())))
            .collect();
        st.insert_memory("s", "c", &links, crate::model::now(), crate::model::now())
            .unwrap();
    }

    #[test]
    fn query_atoms_rewrite_to_tag_leaves() {
        let path = temp_db("query-atoms");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        link(
            &st,
            &[
                ("project", Some("alpha")),
                ("count", Some("1")),
                ("rust", None),
            ],
        );

        // equality rewrites to the composed display leaf, the value verbatim
        let expr = st
            .bind_query_atoms(tag_expr::parse("project=alpha").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["project:alpha"]);
        // numbers canonicalize: 1.0 finds the stored 1
        let expr = st
            .bind_query_atoms(tag_expr::parse("count=1.0").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["count:1"]);
        // a bare word that is not an existing key but carries valued relations tests existence
        let expr = st
            .bind_query_atoms(tag_expr::parse("project").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Regex(_))));
        // a word that is both a bare tag and a key matches either form
        link(&st, &[("count", None)]);
        let expr = st
            .bind_query_atoms(tag_expr::parse("count").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Regex(_))));
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
        let pairs: Vec<crate::model::TagPair> = [
            ("project", Some("alpha")),
            ("count", Some("1")),
            ("plain", None),
        ]
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.map(|v| v.to_string())))
        .collect();
        let ids = st.link_keys(&pairs, true).unwrap().ids;
        let links: Vec<(i64, Option<String>)> = ids
            .into_iter()
            .zip(pairs.into_iter().map(|(_, v)| v))
            .collect();
        let id = st
            .insert_memory("s", "c", &links, crate::model::now(), crate::model::now())
            .unwrap();
        let memory = &st.get_memories(&[id]).unwrap().0[0];
        let view = st.attributes_view(memory).unwrap();
        assert_eq!(view["project"], serde_json::json!("alpha"));
        // number-shaped values are JSON numbers, not strings
        assert_eq!(view["count"], serde_json::json!(1));
        assert!(view.get("plain").is_none(), "bare tags are not attributes");
        cleanup(&path);
    }
}
