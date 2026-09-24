//! 纯数据模型：标签、记忆条目、id / 标签名归一化。
//!
//! 该层不含任何 I/O；持久化见 `store`，API 限制常量也定义在这里，
//! 由工具层（`tools`）引用，保证各处限制一致。

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// API 层限制：摘要最大字符数。
pub const MAX_SUMMARY_CHARS: usize = 512;
/// API 层限制：正文最大字符数。
pub const MAX_CONTENT_CHARS: usize = 200_000;
/// API 层限制：标签描述最大字符数。
pub const MAX_TAG_DESC_CHARS: usize = 500;
/// API 层限制：标签名最大字符数。
pub const MAX_TAG_NAME_CHARS: usize = 100;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tag {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub created_at: u64,
}

impl Tag {
    pub fn new(name: String) -> Self {
        Tag {
            name,
            description: String::new(),
            created_at: now(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Memory {
    pub id: String,
    pub summary: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Memory {
    /// 渐进式披露第一层：摘要视图（不含正文）。
    pub fn summary_view(&self) -> Value {
        json!({
            "id": self.id,
            "tags": self.tags,
            "summary": self.summary,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        })
    }

    /// 渐进式披露第二层：完整视图（含正文）。
    pub fn full_view(&self) -> Value {
        let mut v = self.summary_view();
        v["content"] = json!(self.content);
        v
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 记忆 id 归一化：容忍 "1" / "m1" 两种写法，统一为 "m1"。
pub fn normalize_id(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return t.to_string();
    }
    if let Some(rest) = t.strip_prefix('m') {
        if !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()) {
            return t.to_string();
        }
    }
    if t.bytes().all(|b| b.is_ascii_digit()) {
        return format!("m{}", t);
    }
    t.to_string()
}

/// 标签名归一化：去除首尾空白，限制长度。
pub fn normalize_tag_name(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err("tag name cannot be empty".into());
    }
    if t.chars().count() > MAX_TAG_NAME_CHARS {
        return Err(format!(
            "tag name is too long (max {} characters)",
            MAX_TAG_NAME_CHARS
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
        assert_eq!(normalize_id("12"), "m12");
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
