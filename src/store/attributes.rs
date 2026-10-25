//! Query binding and the typed attribute view, both derived from stored annotation rows alone:
//! a row is a bare tag (no target) or an edge (source → target), and target names that parse as
//! canonical numbers render as JSON numbers.

use super::Store;
use crate::{
    model::Memory,
    tag_expr::{TagAtom, TagExpr},
};
use serde_json::{json, Map, Value};

impl Store {
    /// Rewrite query atoms to the model's matching semantics:
    /// - A bare word naming an existing tag becomes a presence regex — the tag counts as carried
    ///   when it is on the memory in any state (bare, edge source `w:…`, edge target `…:w`).
    ///   Display forms carry at most one colon, so those three positions are exactly the roles.
    ///   Unknown words stay literal tags, and leaf validation downstream reports them with the
    ///   usual did-you-mean.
    /// - `key=value` becomes the displayed `key:value` edge leaf — a number canonicalizes (so
    ///   `key=1.0` finds `key:1`), an identifier word goes in verbatim, anything else is a bind
    ///   error.
    pub fn bind_query_atoms(&self, expr: TagExpr) -> Result<TagExpr, String> {
        expr.try_map(&mut |atom| match atom {
            TagAtom::Tag(name) => {
                if !self.tag_exists(&name)? {
                    return Ok(TagAtom::Tag(name));
                }
                let escaped = regex::escape(&name);
                let pattern = format!("^{escaped}$|^{escaped}:|:{escaped}$");
                Ok(TagAtom::Regex(Box::new(
                    regex::Regex::new(&pattern).expect("escaped presence regex"),
                )))
            }
            TagAtom::KeyEq { key, value } => {
                let stored = match crate::model::parse_number_text(&value) {
                    Some(canonical) => crate::model::compose_edge(&key, &canonical),
                    None => {
                        crate::model::normalize_tag_name(&value).map_err(|_| {
                            format!(
                                "edge target '{value}' for source '{key}' must be an identifier word (letters, digits, '_', '-', '.') or a number"
                            )
                        })?;
                        crate::model::compose_edge(&key, &value)
                    }
                };
                Ok(TagAtom::Tag(stored))
            }
            other => Ok(other),
        })
    }

    /// Typed view of a memory's attribute values, derived from its edges alone: a target name
    /// renders as a JSON number when it parses as a canonical number, otherwise as a string.
    /// One edge per source by construction; bare tags are not attributes.
    pub fn attributes_view(&self, m: &Memory) -> Result<Value, String> {
        let mut out = Map::new();
        let id = Store::parse_id(&m.id).unwrap_or(0);
        for row in self.tag_rows_for(id)? {
            let Some(target) = row.target else { continue };
            let rendered = match crate::model::parse_number_text(&target) {
                Some(canonical) => {
                    serde_json::from_str(&canonical).unwrap_or_else(|_| json!(target))
                }
                None => json!(target),
            };
            out.insert(row.source, rendered);
        }
        Ok(Value::Object(out))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    /// Case sensitivity: tag names are case-sensitive, and the presence regex must not fold
    /// ASCII case across them.
    #[test]
    fn presence_matching_does_not_fold_case() {
        let path = temp_db("presence-case");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        for name in ["project", "Project"] {
            let edges: Vec<crate::model::TagEdge> =
                vec![(name.to_string(), Some("alpha".to_string()))];
            let links = st.resolve_tags(&edges, true).unwrap().links;
            st.insert_memory("s", "c", &links, crate::model::now(), crate::model::now())
                .unwrap();
        }
        // Both spellings exist as distinct tags; each matches only its own display form
        let expr = st
            .bind_query_atoms(crate::tag_expr::parse("project").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Regex(_))));
        cleanup(&path);
    }
}

#[cfg(test)]
mod query_tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;
    use crate::tag_expr;

    fn insert(st: &Store, edges: &[(&str, Option<&str>)]) {
        let edges: Vec<crate::model::TagEdge> = edges
            .iter()
            .map(|(s, t)| ((*s).to_string(), t.map(|t| t.to_string())))
            .collect();
        let links = st.resolve_tags(&edges, true).unwrap().links;
        st.insert_memory("s", "c", &links, crate::model::now(), crate::model::now())
            .unwrap();
    }

    #[test]
    fn query_atoms_rewrite_to_presence_and_edge_leaves() {
        let path = temp_db("query-atoms");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        insert(
            &st,
            &[
                ("project", Some("alpha")),
                ("count", Some("1")),
                ("rust", None),
            ],
        );

        // equality rewrites to the composed display leaf, the target verbatim
        let expr = st
            .bind_query_atoms(tag_expr::parse("project=alpha").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["project:alpha"]);
        // numbers canonicalize: 1.0 finds the stored 1
        let expr = st
            .bind_query_atoms(tag_expr::parse("count=1.0").unwrap())
            .unwrap();
        assert_eq!(expr.tag_names(), vec!["count:1"]);
        // an existing tag becomes a presence regex: matches bare, source and target states
        let expr = st
            .bind_query_atoms(tag_expr::parse("project").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Regex(_))));
        assert!(
            expr.eval(&["project:alpha".to_string()]),
            "source state matches"
        );
        assert!(
            expr.eval(&["other:project".to_string()]),
            "target state matches"
        );
        assert!(expr.eval(&["project".to_string()]), "bare state matches");
        assert!(
            !expr.eval(&["project-x".to_string()]),
            "substring is not presence"
        );
        assert!(!expr.eval(&["xproject".to_string()]));
        // an unknown bare name stays a literal tag (did-you-mean fires downstream)
        let expr = st
            .bind_query_atoms(tag_expr::parse("nmmtp").unwrap())
            .unwrap();
        assert!(expr.eval_with(&|a| matches!(a, TagAtom::Tag(n) if n == "nmmtp")));
        // malformed targets error at bind time
        assert!(st
            .bind_query_atoms(tag_expr::parse("project='has space'").unwrap())
            .is_err());
        cleanup(&path);
    }

    /// The presence regex matches all three states and nothing else: a word inside another
    /// tag's name (prefix or suffix without the colon boundary) must not match. Evaluation is
    /// pure over display forms, so only the vocabulary needs to exist for the rewrite.
    #[test]
    fn presence_regex_is_boundary_exact() {
        let path = temp_db("presence-boundaries");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.resolve_tags(
            &[
                ("nmmt".into(), None),
                ("project".into(), Some("nmmt-lang".into())),
            ],
            true,
        )
        .unwrap();
        let expr = st
            .bind_query_atoms(tag_expr::parse("nmmt").unwrap())
            .unwrap();
        let cases: &[(&[&str], bool)] = &[
            (&["nmmt"], true),                      // bare
            (&["project:nmmt"], true),              // target
            (&["nmmt:rust"], true),                 // source
            (&["project:nmmt-lang"], false),        // substring only
            (&["xnmmt"], false),                    // substring only
            (&["project:nmmt", "unrelated"], true), // among others
        ];
        for (tags, expect) in cases {
            let owned: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
            assert_eq!(expr.eval(&owned), *expect, "tags {tags:?}");
        }
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
        let edges: Vec<crate::model::TagEdge> = [
            ("project", Some("alpha")),
            ("count", Some("1")),
            ("plain", None),
        ]
        .iter()
        .map(|(s, t)| ((*s).to_string(), t.map(|t| t.to_string())))
        .collect();
        let links = st.resolve_tags(&edges, true).unwrap().links;
        let id = st
            .insert_memory("s", "c", &links, crate::model::now(), crate::model::now())
            .unwrap();
        let memory = &st.get_memories(&[id]).unwrap().0[0];
        let view = st.attributes_view(memory).unwrap();
        assert_eq!(view["project"], serde_json::json!("alpha"));
        // number-shaped targets are JSON numbers, not strings
        assert_eq!(view["count"], serde_json::json!(1));
        assert!(view.get("plain").is_none(), "bare tags are not attributes");
        cleanup(&path);
    }
}
