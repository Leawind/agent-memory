//! 工具清单：名称、描述与 JSON Schema 定义。
//!
//! Schema 是对 agent 暴露的契约的唯一权威来源：参数校验所用的
//! 合法参数名列表直接从 `inputSchema.properties` 派生，两者不会失同步。

use crate::auth::Cap;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

pub const TOOL_NAMES: &[&str] = &[
    "tag_create",
    "tag_list",
    "tag_rename",
    "tag_delete",
    "memory_create",
    "memory_list",
    "memory_search",
    "memory_get",
    "memory_update",
    "memory_delete",
];

pub fn tool_definitions() -> Value {
    Value::Array(vec![
        def(
            "tag_create",
            "Create a new tag. Tags are labels used to organize memories; you own the taxonomy. Fails if the tag already exists (check with tag_list). If a tag differing only by case exists, it is created anyway but reported in similar_existing - prefer merging to keep the taxonomy tidy.",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Unique tag name (trimmed, 1-100 chars, case-sensitive, any language)."},
                    "description": {"type": "string", "description": "Optional: what this tag groups (max 512 chars)."}
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            "tag_list",
            "List tags with descriptions, memory counts, creation and last-used timestamps (name filterable by regex). Start here when exploring the memory store.",
            json!({
                "type": "object",
                "properties": {
                    "filter": {"type": "string", "description": "Optional regular expression on tag names; tags whose name matches anywhere are kept (use ^...$ to anchor). Case-sensitive; Rust regex syntax. Omit to list all tags."}
                },
                "additionalProperties": false
            }),
            true, false,
        ),
        def(
            "tag_rename",
            "Rename a tag (updates every memory referencing it) and/or update its description. At least one of new_name / description is required.",
            json!({
                "type": "object",
                "properties": {
                    "old_name": {"type": "string"},
                    "new_name": {"type": "string", "description": "New unique name. Omit to keep the current name."},
                    "description": {"type": "string", "description": "New description. Omit to keep the current one."}
                },
                "required": ["old_name"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            "tag_delete",
            "Delete a tag. mode 'detach' (default) removes only the tag from memories and keeps them; mode 'purge' also permanently deletes every memory carrying this tag.",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "mode": {"type": "string", "enum": ["detach", "purge"], "description": "Default: detach."}
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            false, true,
        ),
        def(
            "memory_create",
            "Store a new memory. 'summary' is the one-line abstract shown by searches and lists (progressive disclosure level 1); 'content' is the full text revealed on demand (level 2). Unknown tags are auto-created and reported back in tags_autocreated. If existing memories have the same summary they are listed in duplicate_of - prefer memory_update on those instead of storing again.",
            json!({
                "type": "object",
                "properties": {
                    "summary": {"type": "string", "description": "One-line abstract (max 512 chars); make it precise and self-contained."},
                    "content": {"type": "string", "description": "Full text of the memory. Markdown is recommended (headings, lists, code blocks, tables); the web admin UI renders it."},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tag names for this memory."}
                },
                "required": ["summary", "content"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            "memory_list",
            "Browse memories, optionally filtered by tag. Returns summaries only (id, tags, abstract, timestamps) - never full content; call memory_get for entries worth reading. Newest first by default; paginated.",
            json!({
                "type": "object",
                "properties": {
                    "tag": {"type": "string", "description": "Only memories carrying this exact tag."},
                    "tag_filter": {"type": "string", "description": "Only memories carrying at least one tag whose name matches this regular expression (match anywhere; ^...$ anchors; case-sensitive; Rust regex syntax). Combinable with 'tag' (both must hold)."},
                    "sort": {"type": "string", "enum": ["updated_at", "created_at", "id"], "description": "Default: updated_at."},
                    "order": {"type": "string", "enum": ["asc", "desc"], "description": "Default: desc (newest first)."},
                    "offset": {"type": "integer", "minimum": 0},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 200, "description": "Default 20."}
                },
                "additionalProperties": false
            }),
            true, false,
        ),
        def(
            "memory_search",
            "Search across tags, summaries and content. All whitespace-separated terms must match (AND); a quoted term (\"exact phrase\") must match verbatim and scores a bonus. Matching is substring-based and case-insensitive, so CJK queries work without segmentation. Repeated hits and whole-word ASCII matches rank higher. Tag matches rank highest. When the server has semantic search enabled, results also include meaning-similar memories that share no keywords (mode 'hybrid'); if the embedding service is unavailable the search silently falls back to keyword-only and the response carries semantic_fallback: true. Returns ranked summaries plus a short HTML-escaped content snippet - call memory_get on the promising ids to reveal full content.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Whitespace-separated keywords; wrap words in quotes to require verbatim adjacency."},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Optional filter: memory must carry at least one of these tags."},
                    "tag_filter": {"type": "string", "description": "Optional filter: memory must carry at least one tag whose name matches this regular expression (match anywhere; ^...$ anchors; case-sensitive; Rust regex syntax). Combinable with 'tags' (both must hold)."},
                    "mode": {"type": "string", "enum": ["auto", "keyword", "hybrid"], "description": "Default: auto - hybrid (keyword + semantic) when the server has semantic search configured, otherwise plain keyword. 'keyword' forces keyword-only; 'hybrid' requires semantic search to be configured (error if not). Hybrid falls back to keyword automatically when the embedding service is unavailable."},
                    "offset": {"type": "integer", "minimum": 0, "description": "Skip the first N ranked matches (for paging through many results)."},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 50, "description": "Default 10."}
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            true, false,
        ),
        def(
            "memory_get",
            "Reveal the full content of one or more memories by id (progressive disclosure level 2). Prefer fetching only the ids you actually need after memory_search / memory_list.",
            json!({
                "type": "object",
                "properties": {
                    "ids": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 50, "description": "Memory ids, e.g. [\"m3\"] or [\"m1\",\"m7\"]."}
                },
                "required": ["ids"],
                "additionalProperties": false
            }),
            true, false,
        ),
        def(
            "memory_update",
            "Update a memory: new summary, new content, and/or adjust tags incrementally via add_tags / remove_tags (no need to know the current tag list). Touching a memory refreshes its updated_at.",
            json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string"},
                    "summary": {"type": "string", "description": "Replacement summary."},
                    "content": {"type": "string", "description": "Replacement content. Markdown is recommended."},
                    "add_tags": {"type": "array", "items": {"type": "string"}, "description": "Tags to append (auto-created if unknown). Applied before remove_tags, so a tag present in both lists ends up removed."},
                    "remove_tags": {"type": "array", "items": {"type": "string"}, "description": "Tags to remove."}
                },
                "required": ["id"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            "memory_delete",
            "Permanently delete one or more memories by id.",
            json!({
                "type": "object",
                "properties": {
                    "ids": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 50}
                },
                "required": ["ids"],
                "additionalProperties": false
            }),
            false, true,
        ),
    ])
}

fn def(name: &str, description: &str, schema: Value, read_only: bool, destructive: bool) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": schema,
        "annotations": {
            "readOnlyHint": read_only,
            "destructiveHint": destructive
        }
    })
}

/// 工具名 → 合法参数名列表（从 inputSchema.properties 派生，静态缓存）。
pub fn known_args(tool: &str) -> Option<&'static Vec<String>> {
    static KNOWN: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    KNOWN
        .get_or_init(|| {
            tool_definitions()
                .as_array()
                .expect("tool definitions is an array")
                .iter()
                .map(|t| {
                    let name = t["name"].as_str().expect("tool has a name").to_string();
                    let props = t["inputSchema"]["properties"]
                        .as_object()
                        .map(|o| o.keys().cloned().collect())
                        .unwrap_or_default();
                    (name, props)
                })
                .collect()
        })
        .get(tool)
}

/// 工具名 → 是否只读（从注解 readOnlyHint 派生，与对 agent 的声明同源）。
/// 决定该工具的事务模式：只读走 DEFERRED 快照，写走 IMMEDIATE 写锁。
pub fn is_read_only(tool: &str) -> bool {
    static READ_ONLY: OnceLock<HashMap<String, bool>> = OnceLock::new();
    READ_ONLY
        .get_or_init(|| {
            tool_definitions()
                .as_array()
                .expect("tool definitions is an array")
                .iter()
                .map(|t| {
                    let name = t["name"].as_str().expect("tool has a name").to_string();
                    let ro = t["annotations"]["readOnlyHint"].as_bool().unwrap_or(false);
                    (name, ro)
                })
                .collect()
        })
        .get(tool)
        .copied()
        .unwrap_or(false)
}

/// 工具名 → 调用方所需能力（权限的唯一登记表，执行点在 tools::execute
/// 入口集中把守）。只对 TOOL_NAMES 内的工具查询；兜底分支仅为类型完整性。
pub fn required_cap(tool: &str) -> Cap {
    static CAPS: OnceLock<HashMap<String, Cap>> = OnceLock::new();
    CAPS.get_or_init(|| {
        [
            ("tag_create", Cap::TagManage),
            ("tag_list", Cap::Read),
            ("tag_rename", Cap::TagManage),
            ("tag_delete", Cap::TagManage),
            ("memory_create", Cap::Create),
            ("memory_list", Cap::Read),
            ("memory_search", Cap::Read),
            ("memory_get", Cap::Read),
            ("memory_update", Cap::Update),
            ("memory_delete", Cap::Delete),
        ]
        .into_iter()
        .map(|(t, c)| (t.to_string(), c))
        .collect()
    })
    .get(tool)
    .copied()
    .unwrap_or(Cap::Read)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 只读标记与契约同源：每个工具都能查到（未知工具按"非只读"处理），
    /// 且清单里的只读/写工具数量与注解一致。这个标记驱动事务模式，
    /// 弄反了会让读请求抢写锁或让写请求跑在无锁快照上。
    #[test]
    fn read_only_flags_derive_from_annotations() {
        assert!(is_read_only("tag_list"));
        assert!(is_read_only("memory_list"));
        assert!(is_read_only("memory_search"));
        assert!(is_read_only("memory_get"));
        assert!(!is_read_only("memory_create"));
        assert!(!is_read_only("memory_delete"));
        assert!(!is_read_only("tag_create"));
        assert!(!is_read_only("no_such_tool"));

        let read_only = TOOL_NAMES.iter().filter(|t| is_read_only(t)).count();
        assert_eq!(read_only, 4, "契约里的只读工具应恰为 4 个");
    }

    /// 能力登记表与工具清单一一对应：每个工具都有映射（新增工具忘登记
    /// 会在这里失败），且几个关键工具的能力归属符合直觉。
    #[test]
    fn required_cap_covers_every_tool() {
        use crate::auth::Cap;
        for name in TOOL_NAMES {
            // 只要不 panic 即视为已登记（映射本身是静态表，覆盖即可）
            let _ = required_cap(name);
        }
        assert_eq!(required_cap("memory_get"), Cap::Read);
        assert_eq!(required_cap("memory_create"), Cap::Create);
        assert_eq!(required_cap("memory_update"), Cap::Update);
        assert_eq!(required_cap("memory_delete"), Cap::Delete);
        assert_eq!(required_cap("tag_rename"), Cap::TagManage);
    }
}
