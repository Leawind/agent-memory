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
    /// the id already encodes creation order.
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

/// Tag name normalization: trim surrounding whitespace and cap length.
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
    Ok(t.to_string())
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
