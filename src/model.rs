//! Pure data model: memory entries and id / tag-name normalization.
//!
//! This layer has no I/O; persistence lives in `store` (SQLite). The API limit constants are also defined
//! here and referenced by the tool layer (`tools`), keeping limits consistent everywhere.

use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// API limit: maximum title/summary characters.
pub const MAX_SUMMARY_CHARS: usize = 512;
/// API limit: maximum content characters.
pub const MAX_CONTENT_CHARS: usize = 262_144;
/// API limit: maximum tag description characters.
pub const MAX_TAG_DESC_CHARS: usize = 512;
/// API limit: maximum tag name characters.
pub const MAX_TAG_NAME_CHARS: usize = 100;
/// API limit: maximum identity name characters.
pub const MAX_IDENTITY_NAME_CHARS: usize = 100;
/// API limit: maximum custom instructions characters (overrides the built-in discovery prompt,
/// served by both the modern discover and the legacy initialize).
pub const MAX_INSTRUCTIONS_CHARS: usize = 20_000;

/// The reserved tag marking operator-curated standing rules (the "resident convention"): memories
/// carrying it are exposed as resources in their own right, and the tag itself can never be
/// renamed or deleted. Attaching it to / detaching it from memories requires the admin capability
/// (guarded centrally in `tools::execute`).
pub const RESERVED_TAG: &str = "convention";
/// Description reported for the reserved tag when it is listed before anyone created it
/// (the synthesized `tag_list` row); once created, the tag's own description wins.
pub const RESERVED_TAG_DESCRIPTION: &str = "Reserved tag: anchors the operator-curated resident convention (exposed as memory:// resources). It cannot be renamed or deleted, and attaching it to memories requires the 'admin' permission.";

#[derive(Clone, Debug)]
pub struct Memory {
    pub id: String,
    pub summary: String,
    pub content: String,
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Memory {
    /// First layer of progressive disclosure: summary view (no content). One compact server-local
    /// `updated` time; creation time is not exposed at this level — browsing runs on recency, and
    /// the id already encodes creation order. The typed `attributes` view is attached by the
    /// caller (see `store::Store::attributes_view`).
    pub fn summary_view(&self) -> Value {
        json!({
            "id": self.id,
            "tags": self.tags,
            "summary": self.summary,
            "updated": crate::util::format_local_compact(self.updated_at),
        })
    }

    /// Second layer of progressive disclosure: full view (with content), both timestamps as the
    /// compact server-local wall clock.
    pub fn full_view(&self) -> Value {
        json!({
            "id": self.id,
            "tags": self.tags,
            "summary": self.summary,
            "created": crate::util::format_local_compact(self.created_at),
            "updated": crate::util::format_local_compact(self.updated_at),
            "content": self.content,
        })
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Memory id normalization: trims surrounding whitespace only. The authoritative format is "m{n}";
/// omitting the m prefix is not tolerated — bare numbers are kept as-is and later fail parsing into missing/NotFound.
pub fn normalize_id(raw: &str) -> String {
    raw.trim().to_string()
}

/// The one identifier rule shared by tag names and edge targets: one word of Unicode identifier
/// characters (XID_Continue, so letters, digits and `_` in every script including CJK), with `-`
/// and `.` allowed after the first character. No spaces, no colon (the `source:target` display
/// separator), no quotes or expression operators — which keeps tag_expr parsing unambiguous and
/// lets the compact display form stay quote-free.
fn ident_regex() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"^[\p{XID_Continue}][\p{XID_Continue}\-.]*$")
            .expect("static ident regex")
    })
}

/// The separator between a tag's scope and its value: `system::pc1sw0`. A tag name carries at
/// most one separator; both halves are identifier words.
pub const TAG_SCOPE_SEPARATOR: &str = "::";

fn validate_ident(text: &str, what: &str) -> Result<(), String> {
    if !ident_regex().is_match(text) {
        return Err(format!(
            "invalid {what} '{text}': use one identifier word of letters, digits, '_', '-' or '.' (Unicode), starting with a letter, digit or '_' — no spaces, '{TAG_SCOPE_SEPARATOR}', quotes or expression operators"
        ));
    }
    Ok(())
}

fn check_length(text: &str, what: &str) -> Result<(), String> {
    if text.chars().count() > MAX_TAG_NAME_CHARS {
        return Err(format!(
            "{what} is too long (max {MAX_TAG_NAME_CHARS} characters)"
        ));
    }
    Ok(())
}

/// One identifier word: a scope name or a value. No separator inside.
pub fn normalize_ident_word(raw: &str, what: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(format!("{what} cannot be empty"));
    }
    check_length(t, what)?;
    validate_ident(t, what)?;
    Ok(t.to_string())
}

/// Tag name normalization: trim surrounding whitespace, cap length, enforce the identifier rule
/// on each part. A bare tag is one word (`rust`); a scoped tag is `scope::value` (`system::pc1sw0`).
pub fn normalize_tag_name(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err("tag name cannot be empty".into());
    }
    check_length(t, "tag name")?;
    match t.split_once(TAG_SCOPE_SEPARATOR) {
        None => validate_ident(t, "tag name")?,
        Some((scope, value)) => {
            if value.contains(TAG_SCOPE_SEPARATOR) {
                return Err(format!(
                    "invalid tag name '{t}': at most one '{TAG_SCOPE_SEPARATOR}' separator"
                ));
            }
            validate_ident(scope, "tag scope")?;
            validate_ident(value, "tag value")?;
        }
    }
    Ok(t.to_string())
}

/// The scope half of a tag name, or None for a bare tag.
pub fn tag_scope_of(name: &str) -> Option<&str> {
    name.split_once(TAG_SCOPE_SEPARATOR).map(|(scope, _)| scope)
}

/// The value half of a tag name (the whole name for a bare tag).
pub fn tag_value_of(name: &str) -> &str {
    match name.split_once(TAG_SCOPE_SEPARATOR) {
        Some((_, value)) => value,
        None => name,
    }
}

/// Compose a scoped tag name from its parts. Both halves are pre-validated identifier words.
pub fn compose_tag_name(scope: &str, value: &str) -> String {
    format!("{scope}{TAG_SCOPE_SEPARATOR}{value}")
}

/// Canonical decimal text for an attribute number: integers keep their exact form (`1` and
/// `1.0` are the same value and the same `key:1` tag), everything else takes the shortest
/// round-trip form (`0.1`, `1e300` — the exponent keeps huge magnitudes inside the tag-name
/// length cap); `-0` normalizes to `0`. Returns None for non-finite floats (not JSON anyway).
pub fn canonical_number(n: &serde_json::Number) -> Option<String> {
    if let Some(i) = n.as_i64() {
        return Some(i.to_string());
    }
    if let Some(u) = n.as_u64() {
        return Some(u.to_string());
    }
    let f = n.as_f64()?;
    if !f.is_finite() {
        return None;
    }
    if f == 0.0 {
        return Some("0".to_string());
    }
    if f.fract() == 0.0 {
        if f > 0.0 && f <= u64::MAX as f64 {
            return Some((f as u64).to_string());
        }
        if f < 0.0 && f >= i64::MIN as f64 {
            return Some((f as i64).to_string());
        }
    }
    // serde_json's float form writes positive exponents as "e+300"; '+' is outside the
    // identifier charset, so the canonical form drops it ("1e300") — JSON parses both alike.
    Some(n.to_string().replace("e+", "e"))
}

/// Parse a numeric attribute value from plain text (a `key=42` query right-hand side, a dump's
/// value half) into canonical form; anything that is not a plain JSON number is None.
pub fn parse_number_text(text: &str) -> Option<String> {
    let n: serde_json::Number = serde_json::from_str(text).ok()?;
    canonical_number(&n)
}

/// API limit: at most this many tags (annotations) on one memory per write.
pub const MAX_TAGS_PER_MEMORY: usize = 64;

/// One memory write's tag declaration after normalization.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct TagWrite {
    /// Tag names the write adds: bare tags as-is, scoped tags composed as `scope::value`.
    pub tags: Vec<String>,
    /// Scopes the caller declared with a null value — the memory takes no tag in them. A create
    /// stores nothing for them (absence *is* the declaration); an incremental update drops the
    /// tag it currently carries in each of these scopes, which is why they are reported apart.
    pub cleared: Vec<String>,
}

/// Normalize one memory write's tags. `bare` lists the tags that carry no value; `attributes`
/// maps a scope to the value the memory takes in it, and a scope carries at most one value:
/// - bare tag: one word, listed in `bare` (`rust`);
/// - scoped tag: a key of `attributes` with a string or number value (`{system: "pc1sw0"}`
///   composes the name `system::pc1sw0`);
/// - no value in a scope: a key of `attributes` with null, accepted only for a scope that must
///   be declared on every write (`required`).
///
/// A null on any other key is an error rather than a silent no-op: the caller meant something by
/// it, and ignoring it would let the write look successful without doing what was asked.
///
/// `require_all` demands a declaration for every required scope (fresh memory); an incremental
/// update passes false and validates only the declarations it carries.
pub fn normalize_tag_write(
    bare: &[String],
    attributes: &serde_json::Map<String, Value>,
    required_scopes: &[String],
    require_all: bool,
) -> Result<TagWrite, String> {
    let is_required = |scope: &str| required_scopes.iter().any(|r| r == scope);
    // Every name appears once: the attribute object enforces that structurally, the array does not.
    let mut tags: Vec<String> = Vec::with_capacity(bare.len() + attributes.len());
    let mut cleared: Vec<String> = Vec::new();
    for raw in bare {
        let name = normalize_tag_name(raw)?;
        if let Some(scope) = tag_scope_of(&name) {
            return Err(format!(
                "tag '{name}' carries a value: declare it in attributes as {{\"{scope}\": \"{}\"}}",
                tag_value_of(&name)
            ));
        }
        if is_required(&name) {
            return Err(format!(
                "scope '{name}' must be declared in attributes with a value or null (it applies to every write), not listed as a bare tag"
            ));
        }
        if tags.contains(&name) {
            return Err(format!("tag '{name}' is listed twice in tags"));
        }
        tags.push(name);
    }

    let mut declared: Vec<String> = Vec::with_capacity(attributes.len());
    for (raw_scope, value) in attributes {
        let scope = normalize_ident_word(raw_scope, "scope name")?;
        // Keys are compared after trimming, so "system" and " system" name one scope
        if declared.contains(&scope) {
            return Err(format!(
                "attributes names scope '{scope}' more than once (scope names are compared after trimming)"
            ));
        }
        if tags.iter().any(|tag| tag == &scope) {
            return Err(format!(
                "'{scope}' is used both as a bare tag and as a scope in this write — one name, one role"
            ));
        }
        declared.push(scope.clone());
        match value {
            Value::Null => {
                if !is_required(&scope) {
                    return Err(format!(
                        "attributes.{scope} is null, which declares that the memory takes no value in that scope; only a scope required on every write accepts that declaration — drop the key instead"
                    ));
                }
                cleared.push(scope);
            }
            Value::String(text) => {
                let value = normalize_ident_word(text, "scope value").map_err(|_| {
                    format!(
                        "attributes.{scope}: the value must be an identifier word (letters, digits, '_', '-', '.'), got '{text}'"
                    )
                })?;
                tags.push(compose_tag_name(&scope, &value));
            }
            Value::Number(n) => {
                let canonical = canonical_number(n)
                    .ok_or_else(|| format!("attributes.{scope}: {n} is not a finite number"))?;
                tags.push(compose_tag_name(&scope, &canonical));
            }
            other => {
                return Err(format!(
                    "attributes.{scope} must be a string, a number or null, got {other} — booleans are not values (presence flags belong in tags)"
                ))
            }
        }
    }
    if tags.len() > MAX_TAGS_PER_MEMORY {
        return Err(format!(
            "at most {MAX_TAGS_PER_MEMORY} tags per memory are allowed"
        ));
    }
    if require_all {
        let mut missing: Vec<&str> = required_scopes
            .iter()
            .map(String::as_str)
            .filter(|scope| !declared.iter().any(|name| name == scope))
            .collect();
        if !missing.is_empty() {
            missing.sort_unstable();
            return Err(format!(
                "missing required scopes in attributes: {}; declare each one with its value, or null when this memory takes no value in it",
                missing.join(", ")
            ));
        }
    }
    Ok(TagWrite { tags, cleared })
}

/// Identity name normalization: trim surrounding whitespace and cap length (same rules as tag names).
pub fn normalize_identity_name(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err("identity name cannot be empty".into());
    }
    if t.chars().count() > MAX_IDENTITY_NAME_CHARS {
        return Err(format!(
            "identity name is too long (max {MAX_IDENTITY_NAME_CHARS} characters)"
        ));
    }
    Ok(t.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_id_variants() {
        assert_eq!(normalize_id(" m12 "), "m12");
        assert_eq!(normalize_id("12"), "12");
        assert_eq!(normalize_id("m0"), "m0");
        assert_eq!(normalize_id("abc"), "abc");
        assert_eq!(normalize_id(""), "");
    }

    #[test]
    fn tag_name_validation() {
        assert!(normalize_tag_name("  rust  ") == Ok("rust".to_string()));
        assert!(normalize_tag_name("  ").is_err());
        assert!(normalize_tag_name(&"x".repeat(MAX_TAG_NAME_CHARS + 1)).is_err());
        assert!(normalize_tag_name(&"x".repeat(MAX_TAG_NAME_CHARS)).is_ok());
        assert!(normalize_tag_name("中文标签").is_ok());
    }

    #[test]
    fn tag_name_format_is_enforced() {
        // The shared identifier rule for tags and edge targets: XID_Continue characters
        // with '-' and '.' after the first; no colon (reserved for source:target), spaces,
        // quotes or operators
        for ok in [
            "rust",
            "_private",
            "3d",
            "v1.2",
            "multi-hyphen_underscore.ok",
            "方案-A.1",
        ] {
            assert!(normalize_tag_name(ok).is_ok(), "{ok} should be valid");
        }
        for bad in [
            "project:agent-memory", // colon is the source:target display separator
            "rust & life",          // expression operator with spaces
            "a&b",                  // glued operator
            "a|b",
            "!not",
            "(paren)",
            "a/b", // regex delimiter
            "has space",
            "quo\"te",
            "quo'te",
            ",",
            "-leading",
            ".leading",
            "/leading",
        ] {
            assert!(normalize_tag_name(bad).is_err(), "{bad} should be invalid");
        }
    }

    #[test]
    fn numbers_canonicalize_for_storage_and_queries() {
        let n = |text: &str| {
            canonical_number(&serde_json::from_str::<serde_json::Number>(text).unwrap()).unwrap()
        };
        assert_eq!(n("42"), "42");
        // 1 and 1.0 are the same value and the same name
        assert_eq!(n("1.0"), "1");
        assert_eq!(n("-3"), "-3");
        assert_eq!(n("-3.0"), "-3");
        assert_eq!(n("0.5"), "0.5");
        assert_eq!(n("-0"), "0");
        // Beyond the exact integer range the exponent form keeps names short
        assert_eq!(n("1e300"), "1e300");
        // Query/dump text parses through the same canonicalization
        assert_eq!(parse_number_text("42"), Some("42".to_string()));
        // Leading zeros are not JSON numbers: the query side uses the canonical form
        assert_eq!(parse_number_text("042"), None);
        assert_eq!(parse_number_text("1.0e2"), Some("100".to_string()));
        assert_eq!(parse_number_text("abc"), None);
        assert_eq!(parse_number_text(""), None);
        assert_eq!(parse_number_text("1_000"), None);
    }

    fn attrs(body: &str) -> serde_json::Map<String, serde_json::Value> {
        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(body).unwrap()
    }

    #[test]
    fn tag_write_composes_scoped_names() {
        let bare = vec!["rust".to_string(), "notes".to_string()];
        let write = normalize_tag_write(
            &bare,
            &attrs(r#"{"system": "pc1sw0", "count": 2.5, "big": 1e300}"#),
            &[],
            true,
        )
        .unwrap();
        let mut tags = write.tags;
        tags.sort();
        assert_eq!(
            tags,
            vec![
                "big::1e300".to_string(),
                "count::2.5".to_string(),
                "notes".to_string(),
                "rust".to_string(),
                "system::pc1sw0".to_string(),
            ]
        );
        assert!(write.cleared.is_empty());
    }

    /// A null in `attributes` is the "no value" declaration: accepted for a required scope (and
    /// reported so an update can clear the tag it carries), rejected everywhere else where it
    /// would be a silent no-op.
    #[test]
    fn null_attributes_declare_absence_only_for_required_scopes() {
        let required = vec!["country".to_string()];
        let write =
            normalize_tag_write(&[], &attrs(r#"{"country": null}"#), &required, true).unwrap();
        assert!(write.tags.is_empty());
        assert_eq!(write.cleared, vec!["country".to_string()]);

        let write = normalize_tag_write(
            &["rust".to_string()],
            &attrs(r#"{"country": "cn"}"#),
            &required,
            true,
        )
        .unwrap();
        let mut tags = write.tags;
        tags.sort();
        assert_eq!(tags, vec!["country::cn".to_string(), "rust".to_string()]);

        for (bare, body, expected) in [
            // the declaration belongs to every write, so it cannot be a bare tag
            (
                &["country".to_string()][..],
                r#"{}"#,
                "listed as a bare tag",
            ),
            (
                &[][..],
                r#"{"country": null, "rust": null}"#,
                "only a scope required",
            ),
            (&[][..], r#"{"project": "has space"}"#, "identifier word"),
            // a value is one word: a scoped tag is declared through attributes
            (&[][..], r#"{"project": "a::b"}"#, "identifier word"),
            (&[][..], r#"{"project": true}"#, "booleans are not values"),
            (
                &[][..],
                r#"{"project": "a", " project ": "b"}"#,
                "more than once",
            ),
            // one name, one role
            (
                &["rust".to_string()][..],
                r#"{"rust": "x"}"#,
                "one name, one role",
            ),
        ] {
            let err = normalize_tag_write(bare, &attrs(body), &required, true).unwrap_err();
            assert!(err.contains(expected), "{body}: got {err}");
        }
    }

    /// A create must declare every required scope; an incremental update only validates what it
    /// carries, so classifying an older memory later stays possible.
    #[test]
    fn required_scopes_must_be_declared_on_create_only() {
        let required = vec!["country".to_string(), "planet".to_string()];
        let err =
            normalize_tag_write(&[], &attrs(r#"{"country": "cn"}"#), &required, true).unwrap_err();
        assert!(err.contains("planet"), "got {err}");
        assert!(
            normalize_tag_write(&[], &attrs(r#"{"country": "cn"}"#), &required, false).is_ok(),
            "an update may touch one required scope alone"
        );
    }

    #[test]
    fn duplicate_bare_names_are_rejected() {
        let err = normalize_tag_write(
            &["rust".to_string(), "rust".to_string()],
            &serde_json::Map::new(),
            &[],
            true,
        )
        .unwrap_err();
        assert!(err.contains("twice"), "got {err}");
    }

    #[test]
    fn views_control_content_disclosure() {
        let m = Memory {
            id: "m1".into(),
            summary: "s".into(),
            content: "secret body".into(),
            tags: vec!["t".into()],
            created_at: 1,
            updated_at: 2,
        };
        assert!(m.summary_view().get("content").is_none());
        assert_eq!(m.full_view()["content"], "secret body");
    }
}
