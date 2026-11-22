//! Query validation and the typed attribute view, both derived from stored tag names alone: a
//! name is a bare word or `scope::value`, and a scoped value that parses as a canonical number
//! renders as a JSON number.

use super::Store;
use crate::{
    model::{tag_scope_of, Memory},
    tag_expr::TagExpr,
};
use serde_json::{json, Map, Value};

impl Store {
    /// Typed view of a memory's scoped tags: `system::pc1sw0` renders as `{"system": "pc1sw0"}`,
    /// and a value that parses as a canonical number renders as a JSON number. Bare tags are not
    /// attributes.
    pub fn attributes_view(&self, m: &Memory) -> Result<Value, String> {
        let mut out = Map::new();
        for name in &m.tags {
            let Some(scope) = tag_scope_of(name) else {
                continue;
            };
            let value = crate::model::tag_value_of(name);
            let rendered = match crate::model::parse_number_text(value) {
                Some(canonical) => {
                    serde_json::from_str(&canonical).unwrap_or_else(|_| json!(value))
                }
                None => json!(value),
            };
            out.insert(scope.to_string(), rendered);
        }
        Ok(Value::Object(out))
    }
}

/// Leaf existence check shared by the query-resolution entry points. Leaves name tags, so a bare
/// leaf must exist as a tag, a `scope::value` leaf only needs its scope to exist (a value that is
/// absent simply matches nothing), and `scope::*` needs the scope. A bare word naming a scope is
/// reported as such instead of as an unknown tag: the fix is a different expression, not a
/// different name.
pub(crate) fn validate_query_leaves(
    expr: &TagExpr,
    tag_exists: &dyn Fn(&str) -> Result<bool, String>,
    scope_exists: &dyn Fn(&str) -> Result<bool, String>,
    hint: &dyn Fn(&str) -> Result<Option<String>, String>,
) -> Result<(), String> {
    let mut unknown: Vec<String> = Vec::new();
    for name in expr.tag_names() {
        match tag_scope_of(name) {
            None => {
                if tag_exists(name)? {
                    continue;
                }
                if scope_exists(name)? {
                    unknown.push(format!(
                        "'{name}' is a scope, not a tag: write {name}::* for any value, or {name}::value for one"
                    ));
                    continue;
                }
                unknown.push(match hint(name)? {
                    Some(similar) => format!("'{name}' (did you mean '{similar}'?)"),
                    None => format!("'{name}'"),
                });
            }
            Some(scope) => {
                if !scope_exists(scope)? {
                    unknown.push(match hint(scope)? {
                        Some(similar) => {
                            format!("scope '{scope}' (did you mean '{similar}'?)")
                        }
                        None => format!("scope '{scope}'"),
                    });
                }
            }
        }
    }
    for scope in expr.scope_names() {
        if !scope_exists(scope)? {
            unknown.push(match hint(scope)? {
                Some(similar) => format!("scope '{scope}' (did you mean '{similar}'?)"),
                None => format!("scope '{scope}'"),
            });
        }
    }
    if unknown.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "unknown names in tag_expr: {} (see tag_list for available tags and scopes)",
            unknown.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;
    use crate::tag_expr;

    fn insert(st: &Store, tags: &[&str]) {
        let names: Vec<String> = tags.iter().map(|tag| (*tag).to_string()).collect();
        let links = st.resolve_tags(&names, true).unwrap().links;
        st.insert_memory("s", "c", &links, crate::model::now(), crate::model::now())
            .unwrap();
    }

    fn validate(st: &Store, raw: &str) -> Result<(), String> {
        let expr = tag_expr::parse(raw).unwrap();
        validate_query_leaves(
            &expr,
            &|name| st.tag_exists(name),
            &|scope| st.scope_exists(scope),
            &|name| match st.find_tag_case_insensitive(name)? {
                Some(tag) => Ok(Some(tag)),
                None => st.find_scope_case_insensitive(name),
            },
        )
    }

    /// Typed view: scoped tags become attributes, number-shaped values become JSON numbers, and
    /// bare tags stay out of the view.
    #[test]
    fn attributes_view_renders_typed_values() {
        let path = temp_db("attributes-view");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.scope_create("project", "").unwrap();
        st.scope_create("count", "").unwrap();
        insert(&st, &["project::alpha", "count::1", "plain"]);
        let memory = &st.get_memories(&[1]).unwrap().0[0];
        let view = st.attributes_view(memory).unwrap();
        assert_eq!(view["project"], json!("alpha"));
        assert_eq!(view["count"], json!(1));
        assert!(view.get("plain").is_none(), "bare tags are not attributes");
        cleanup(&path);
    }

    /// A bare leaf must be a tag; naming a scope is reported as a scope, not as a typo; a scoped
    /// leaf only needs its scope; a typo gets the did-you-mean hint.
    #[test]
    fn leaves_validate_against_tags_and_scopes() {
        let path = temp_db("leaf-validation");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.scope_create("system", "").unwrap();
        st.tag_create("rust", "").unwrap();
        st.tag_create("system::pc1sw0", "").unwrap();

        assert!(validate(&st, "rust").is_ok());
        assert!(validate(&st, "system::pc1sw0").is_ok());
        assert!(
            validate(&st, "system::anything").is_ok(),
            "an unknown value matches nothing but is not an error"
        );
        assert!(validate(&st, "system::*").is_ok());

        let err = validate(&st, "system").unwrap_err();
        assert!(err.contains("is a scope, not a tag"), "got: {err}");
        let err = validate(&st, "System::pc1sw0").unwrap_err();
        assert!(err.contains("did you mean"), "got: {err}");
        let err = validate(&st, "rust::x").unwrap_err();
        assert!(err.contains("scope 'rust'"), "got: {err}");
        cleanup(&path);
    }
}
