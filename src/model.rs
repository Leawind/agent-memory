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
    /// Keys a memory carries: the prefix of every `key:value` tag. Bare tags carry no key.
    /// Enum and number attribute values are both stored as such tags, so this is the single
    /// source of the memory's keyed dimension.
    pub fn tag_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = Vec::new();
        for tag in &self.tags {
            if let Some((key, _)) = tag.split_once(':') {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        keys
    }

    /// First layer of progressive disclosure: summary view (no content). One compact server-local
    /// `updated` time; creation time is not exposed at this level — browsing runs on recency, and
    /// the id already encodes creation order. The typed `attributes` view is attached by the
    /// caller (it needs the key registry; see `store::Store::attach_attributes`).
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

/// The one identifier rule shared by bare tag names, attribute key names and enum attribute
/// values: one word of Unicode identifier characters (XID_Continue, so letters, digits and `_`
/// in every script including CJK), with `-` and `.` allowed after the first character. No
/// spaces, no colon (the `key:value` separator), no quotes or expression operators — which
/// keeps tag_expr parsing unambiguous and lets the compact display form stay quote-free.
fn ident_regex() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"^[\p{XID_Continue}][\p{XID_Continue}\-.]*$")
            .expect("static ident regex")
    })
}

/// Tag name normalization: trim surrounding whitespace, cap length, enforce the identifier
/// rule. Colons are reserved for the internally composed `key:value` value tags — input paths
/// (tags arrays, tag_create, key names) never accept them; `key=value` in tag_expr is the
/// way to qualify values.
pub fn normalize_tag_name(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err("tag name cannot be empty".into());
    }
    if t.chars().count() > MAX_TAG_NAME_CHARS {
        return Err(format!(
            "tag name is too long (max {MAX_TAG_NAME_CHARS} characters)"
        ));
    }
    if !ident_regex().is_match(t) {
        return Err(format!(
            "invalid tag name '{t}': use one identifier word of letters, digits, '_', '-' or '.' (Unicode), starting with a letter, digit or '_' — no spaces, colons, quotes or expression operators"
        ));
    }
    Ok(t.to_string())
}

/// Key name normalization: the same identifier rule as tag names (a colon inside a key would
/// make the stored `key:value` qualification ambiguous). Kept as a named alias so call sites
/// read as the schema concept they enforce.
pub fn normalize_key_name(raw: &str) -> Result<String, String> {
    normalize_tag_name(raw)
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

/// Compose the stored tag name for a validated (key, value) pair. Both parts are pre-validated
/// (identifier rule, or canonical number on the value side), so the result carries exactly one
/// colon. Internal: input paths qualify values via `attributes`, never by naming this directly.
pub fn compose_value_tag(key: &str, value: &str) -> String {
    format!("{key}:{value}")
}

/// Validate a stored tag name read back from a dump: a bare identifier, or the internally
/// composed `key:value` form with an identifier key and an identifier or canonical-number
/// value. Input paths reject colons; this is the import-side counterpart for names the store
/// composes itself.
pub fn validate_stored_tag_name(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    match t.split_once(':') {
        None => normalize_tag_name(t),
        Some((key, value)) => {
            normalize_tag_name(key)?;
            if normalize_tag_name(value).is_ok() || parse_number_text(value).is_some() {
                Ok(t.to_string())
            } else {
                Err(format!(
                    "invalid stored tag name '{t}': the value part must be an identifier word or a number"
                ))
            }
        }
    }
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
        // The shared identifier rule for tags, keys and enum values: XID_Continue characters
        // with '-' and '.' after the first; no colon (reserved for key:value), spaces, quotes
        // or operators
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
            "project:agent-memory", // colon is the key:value separator, input paths reject it
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
    fn key_names_follow_the_same_identifier_rule() {
        assert_eq!(normalize_key_name(" project "), Ok("project".to_string()));
        assert!(normalize_key_name("a:b").is_err());
        assert!(normalize_key_name("").is_err());
        assert!(normalize_key_name("has space").is_err());
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
        // Composed names carry exactly one colon; import accepts what the store composes
        assert_eq!(
            validate_stored_tag_name("count:42"),
            Ok("count:42".to_string())
        );
        assert_eq!(
            validate_stored_tag_name("count:-3"),
            Ok("count:-3".to_string())
        );
        assert!(validate_stored_tag_name("count:abc:def").is_err());
        assert!(validate_stored_tag_name("has space:1").is_err());
        assert_eq!(validate_stored_tag_name(" plain "), Ok("plain".to_string()));
    }

    #[test]
    fn tag_keys_come_from_colon_tags() {
        let m = Memory {
            id: "m1".into(),
            summary: "s".into(),
            content: String::new(),
            tags: vec!["project:a".into(), "project:b".into(), "rust".into()],
            created_at: 1,
            updated_at: 2,
        };
        assert_eq!(m.tag_keys(), vec!["project"]);
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
