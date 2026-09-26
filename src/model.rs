//! 纯数据模型：记忆条目、id / 标签名归一化。
//!
//! 该层不含任何 I/O；持久化见 `store`（SQLite），API 限制常量也定义在这里，
//! 由工具层（`tools`）引用，保证各处限制一致。

use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// API 层限制：摘要最大字符数。
pub const MAX_SUMMARY_CHARS: usize = 512;
/// API 层限制：正文最大字符数。
pub const MAX_CONTENT_CHARS: usize = 262_144;
/// API 层限制：标签描述最大字符数。
pub const MAX_TAG_DESC_CHARS: usize = 512;
/// API 层限制：标签名最大字符数。
pub const MAX_TAG_NAME_CHARS: usize = 100;
/// API 层限制：身份名最大字符数。
pub const MAX_IDENTITY_NAME_CHARS: usize = 100;
/// API 层限制：自定义 initialize 提示词最大字符数。
pub const MAX_INSTRUCTIONS_CHARS: usize = 20_000;

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

/// 记忆 id 归一化：仅去首尾空白。权威格式是 "m{n}"，省略 m 前缀的写法
/// 不被容忍——裸数字按原样保留，后续解析失败归入 missing/NotFound。
pub fn normalize_id(raw: &str) -> String {
    raw.trim().to_string()
}

/// 标签名归一化：去除首尾空白，限制长度。
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

/// 身份名归一化：去除首尾空白，限制长度（规则与标签名一致）。
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
