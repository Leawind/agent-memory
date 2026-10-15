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
    /// Attribute rows for registered string/bool keys: (key, value). An empty value marks
    /// bool presence. Enum-key values are NOT here, they are ordinary `key:value` tags.
    pub attrs: Vec<(String, String)>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Memory {
    /// Keys a memory carries: the prefix of every `key:value` tag plus every attribute key.
    /// Bare tags carry no key.
    pub fn tag_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = Vec::new();
        for tag in &self.tags {
            if let Some((key, _)) = tag.split_once(':') {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
        for (key, _) in &self.attrs {
            let key = key.as_str();
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys
    }

    /// Structured attribute view: empty value = bool presence (`true`), multiple rows for a
    /// key = array, otherwise the scalar string. Enum-key values are not included (they show
    /// up in `tags` as `key:value` names).
    pub fn attributes_view(&self) -> Value {
        let mut out = serde_json::Map::new();
        let mut seen: Vec<&str> = Vec::new();
        for (key, _) in &self.attrs {
            if seen.contains(&key.as_str()) {
                continue;
            }
            seen.push(key);
            let rows: Vec<&String> = self
                .attrs
                .iter()
                .filter(|(k, _)| k == key)
                .map(|(_, v)| v)
                .collect();
            let rendered = if rows.iter().all(|v| v.is_empty()) {
                json!(true)
            } else if rows.len() == 1 {
                json!(rows[0])
            } else {
                json!(rows)
            };
            out.insert(key.clone(), rendered);
        }
        Value::Object(out)
    }

    /// First layer of progressive disclosure: summary view (no content). One compact server-local
    /// `updated` time; creation time is not exposed at this level — browsing runs on recency, and
    /// the id already encodes creation order.
    pub fn summary_view(&self) -> Value {
        let mut view = json!({
            "id": self.id,
            "tags": self.tags,
            "summary": self.summary,
            "updated": crate::util::format_local_compact(self.updated_at),
        });
        let attributes = self.attributes_view();
        if !attributes.as_object().expect("object").is_empty() {
            view["attributes"] = attributes;
        }
        view
    }

    /// Second layer of progressive disclosure: full view (with content), both timestamps as the
    /// compact server-local wall clock.
    pub fn full_view(&self) -> Value {
        let mut view = json!({
            "id": self.id,
            "tags": self.tags,
            "summary": self.summary,
            "created": crate::util::format_local_compact(self.created_at),
            "updated": crate::util::format_local_compact(self.updated_at),
            "content": self.content,
        });
        let attributes = self.attributes_view();
        if !attributes.as_object().expect("object").is_empty() {
            view["attributes"] = attributes;
        }
        view
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

/// Tag name normalization: trim surrounding whitespace, cap length, and enforce the tag-name
/// format. Names are tag_expr atoms — the format keeps expression parsing unambiguous (no
/// quoting needed, no collisions with operators/quotes/regex delimiters): Unicode letters and
/// numbers (CJK included) plus `_`, `-`, `.` and `:` (the key:value form stores an enum value
/// under its key), starting with a letter, number or `_`.
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
    let valid = t.chars().enumerate().all(|(i, c)| match i {
        0 => c.is_alphanumeric() || c == '_',
        _ => c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'),
    });
    if !valid {
        return Err(format!(
            "invalid tag name '{t}': use letters, digits, '_', '-', '.' and ':' only, starting with a letter or digit (no spaces, quotes or expression operators)"
        ));
    }
    Ok(t.to_string())
}

/// Key name normalization: the tag-name charset minus `:` — a key prefixes the stored
/// `key:value` tag names, so a colon inside a key would make qualification ambiguous.
pub fn normalize_key_name(raw: &str) -> Result<String, String> {
    let key = normalize_tag_name(raw)?;
    if key.contains(':') {
        return Err("key name cannot contain ':'".into());
    }
    Ok(key)
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
        // The tag_expr atom charset: letters/digits (Unicode included), '_' '-' '.' and ':' (the
        // key:value separator); no spaces, quotes or operators; must start with a letter, digit or '_'
        for ok in [
            "rust",
            "_private",
            "v1.2",
            "multi-hyphen_underscore.ok",
            "方案-A.1",
            "project:agent-memory",
            "项目:记忆库",
        ] {
            assert!(normalize_tag_name(ok).is_ok(), "{ok} should be valid");
        }
        for bad in [
            "rust & life", // expression operator with spaces
            "a&b",         // glued operator
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
    fn key_names_reject_colons_but_use_the_tag_charset() {
        assert_eq!(normalize_key_name(" project "), Ok("project".to_string()));
        assert!(normalize_key_name("a:b").is_err());
        assert!(normalize_key_name("").is_err());
        assert!(normalize_key_name("has space").is_err());
    }

    #[test]
    fn tag_keys_come_from_colon_tags() {
        let m = Memory {
            id: "m1".into(),
            summary: "s".into(),
            content: String::new(),
            tags: vec!["project:a".into(), "project:b".into(), "rust".into()],
            attrs: Vec::new(),
            created_at: 1,
            updated_at: 2,
        };
        assert_eq!(m.tag_keys(), vec!["project"]);
        let mut m = m;
        m.attrs = vec![("status".into(), String::new())];
        assert_eq!(m.tag_keys(), vec!["project", "status"]);
    }

    #[test]
    fn attributes_render_bool_scalar_and_array() {
        let m = Memory {
            id: "m1".into(),
            summary: "s".into(),
            content: String::new(),
            tags: vec![],
            attrs: vec![
                ("pinned".into(), String::new()),
                ("project".into(), "alpha".into()),
                ("url".into(), "https://a".into()),
                ("url".into(), "https://b".into()),
            ],
            created_at: 1,
            updated_at: 2,
        };
        let view = m.attributes_view();
        assert_eq!(view["pinned"], json!(true));
        assert_eq!(view["project"], json!("alpha"));
        assert_eq!(view["url"], json!(["https://a", "https://b"]));
        let empty = Memory {
            id: "m2".into(),
            summary: "s".into(),
            content: String::new(),
            tags: vec![],
            attrs: Vec::new(),
            created_at: 1,
            updated_at: 2,
        };
        assert!(empty.summary_view().get("attributes").is_none());
        assert!(empty.full_view().get("attributes").is_none());
    }

    #[test]
    fn views_control_content_disclosure() {
        let m = Memory {
            id: "m1".into(),
            summary: "s".into(),
            content: "secret body".into(),
            tags: vec!["t".into()],
            attrs: Vec::new(),
            created_at: 1,
            updated_at: 2,
        };
        assert!(m.summary_view().get("content").is_none());
        assert_eq!(m.full_view()["content"], "secret body");
    }
}
