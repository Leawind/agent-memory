//! 工具清单：名称、描述与 JSON Schema 定义。
//!
//! Schema 是对 agent 暴露的契约的唯一权威来源：参数校验所用的
//! 合法参数名列表直接从 `inputSchema.properties` 派生，两者不会失同步。

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
            "Create a new tag. Tags are labels used to organize memories; you own the taxonomy. Fails if the tag already exists (check with tag_list).",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Unique tag name (trimmed, 1-100 chars, case-sensitive, any language)."},
                    "description": {"type": "string", "description": "Optional: what this tag groups (max 500 chars)."}
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            "tag_list",
            "List all tags with descriptions and memory counts. Start here when exploring the memory store.",
            json!({"type": "object", "properties": {}, "additionalProperties": false}),
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
                    "content": {"type": "string", "description": "Full text of the memory."},
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
                    "sort": {"type": "string", "enum": ["updated_at", "created_at"], "description": "Default: updated_at."},
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
            "Keyword search across tags, summaries and content. All whitespace-separated terms must match (AND). Matching is substring-based and case-insensitive, so CJK queries work without segmentation. Tag matches rank highest. Returns ranked summaries plus a short content snippet - call memory_get on the promising ids to reveal full content.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Whitespace-separated keywords."},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Optional filter: memory must carry at least one of these tags."},
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
                    "content": {"type": "string", "description": "Replacement content."},
                    "add_tags": {"type": "array", "items": {"type": "string"}, "description": "Tags to append (auto-created if unknown)."},
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
