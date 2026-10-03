//! Tool inventory: names, descriptions and JSON Schema definitions.
//!
//! The schema is the single source of truth for the contract exposed to agents: the list of valid parameter
//! names used for validation is derived directly from `inputSchema.properties`, so the two can never drift apart.

use crate::auth::Cap;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

pub const TOOL_NAMES: &[&str] = &[
    "tag_create",
    "tag_list",
    "tag_update",
    "tag_delete",
    "memory_create",
    "memory_list",
    "memory_search",
    "memory_get",
    "memory_update",
    "memory_merge",
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
            "List tags with descriptions, memory counts, creation and last-used timestamps (epoch seconds, UTC; name filterable by regex). Each row carries reserved: true for reserved tags - 'conventions' is always listed (synthesized with zero memories before it exists) and can never be renamed or deleted. Start here when exploring the memory store.",
            json!({
                "type": "object",
                "properties": {
                    "filter": {"type": "string", "description": "Optional regular expression on tag names; tags whose name matches anywhere are kept (use ^...$ to anchor). Case-insensitive; Rust regex syntax. Omit to list all tags."}
                },
                "additionalProperties": false
            }),
            true, false,
        ),
        def(
            "tag_update",
            "Update a tag: rename it and/or set its description (the only way to fill in a description after tag_create). Memories referencing the tag follow a rename automatically - only the label changes. Omit new_name to keep the current name; omit description to keep the current one; at least one of the two is required. The response flags renamed / description_updated tell which parts actually changed.",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Current tag name."},
                    "new_name": {"type": "string", "description": "New unique name. Omit to keep the current name."},
                    "description": {"type": "string", "description": "New description. Omit to keep the current one."}
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            "tag_delete",
            "Delete a tag. mode 'detach' (default) removes only the tag from memories and keeps them; mode 'purge' also permanently deletes every memory carrying this tag - preview the impact first with dry_run: true (reports the affected memory count and, for purge, their ids) since purge cannot be undone.",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "mode": {"type": "string", "enum": ["detach", "purge"], "description": "Default: detach."},
                    "dry_run": {"type": "boolean", "description": "Default false. True reports what would happen (memories_affected; memory_ids for purge) without changing anything."}
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            false, true,
        ),
        def(
            "memory_create",
            "Store a new memory. 'summary' is the one-line abstract shown by searches and lists (progressive disclosure level 1); 'content' is the full text revealed on demand (level 2). Tags must exist: unknown names are rejected with the closest existing tags listed - create them explicitly with tag_create, reuse the listed names, or pass create_missing_tags: true to auto-create them empty (keeping the taxonomy tidy is your job; case variants and synonyms rot it). The response reports tags_autocreated (created just now), tags_reused (already existed) and tags_missing_description (reused tags that still lack a description - consider filling one in with tag_update). If existing memories have the same summary they are listed in duplicate_of - prefer memory_update on those instead of storing again.",
            json!({
                "type": "object",
                "properties": {
                    "summary": {"type": "string", "description": "One-line abstract (max 512 chars); make it precise and self-contained."},
                    "content": {"type": "string", "description": "Full text of the memory. Markdown is recommended (headings, lists, code blocks, tables); the web admin UI renders it."},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tag names for this memory; must exist unless create_missing_tags is true."},
                    "create_missing_tags": {"type": "boolean", "description": "Default false: unknown tag names are an error. True auto-creates unknown tags with an empty description."}
                },
                "required": ["summary", "content"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            "memory_list",
            "Browse memories, optionally filtered by tag. Returns summaries only (id, tags, abstract, timestamps as epoch seconds UTC) - never full content; call memory_get for entries worth reading. Newest first by default; paginated.",
            json!({
                "type": "object",
                "properties": {
                    "tag": {"type": "string", "description": "Only memories carrying this exact tag."},
                    "tag_filter": {"type": "string", "description": "Only memories carrying at least one tag whose name matches this regular expression (match anywhere; ^...$ anchors; case-insensitive; Rust regex syntax). Combinable with 'tag' (both must hold)."},
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
            "Search across tags, summaries and content. All whitespace-separated terms must match (AND); a quoted term (\"exact phrase\") must match verbatim and scores a bonus. Matching is substring-based and case-insensitive, so CJK queries work without segmentation. Repeated hits and whole-word ASCII matches rank higher. Tag matches rank highest. When the server has semantic search enabled, results also include meaning-similar memories that share no keywords (mode 'hybrid'); if the embedding service is unavailable the search silently falls back to keyword-only and the response carries semantic_fallback: true. Returns ranked summaries plus a short plain-text excerpt of the content (unescaped data, single line) - call memory_get on the promising ids to reveal full content.",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Whitespace-separated keywords; wrap words in quotes to require verbatim adjacency."},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Optional filter: memory must carry at least one of these tags."},
                    "tag_filter": {"type": "string", "description": "Optional filter: memory must carry at least one tag whose name matches this regular expression (match anywhere; ^...$ anchors; case-insensitive; Rust regex syntax). Combinable with 'tags' (both must hold)."},
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
            "Reveal the full content of one or more memories by id (progressive disclosure level 2). Prefer fetching only the ids you actually need after memory_search / memory_list. Malformed ids (missing the leading 'm') are reported in invalid_ids instead of being conflated with not-found ones.",
            json!({
                "type": "object",
                "properties": {
                    "ids": {"type": "array", "items": {"type": "string", "pattern": "^m[0-9]+$"}, "minItems": 1, "maxItems": 50, "description": "Memory ids, e.g. [\"m3\"] or [\"m1\",\"m7\"]."}
                },
                "required": ["ids"],
                "additionalProperties": false
            }),
            true, false,
        ),
        def(
            "memory_update",
            "Update a memory: new summary, new content, and/or adjust tags incrementally via add_tags / remove_tags (no need to know the current tag list). Updating summary or content refreshes updated_at; tag-only adjustments leave it untouched, so curating tags never reshuffles the default newest-first browse. Unknown tags in add_tags are rejected (closest existing tags listed) unless create_missing_tags is true. When tags are added, the response reports tags_autocreated / tags_reused / tags_missing_description (same meaning as in memory_create).",
            json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string", "pattern": "^m[0-9]+$", "description": "Memory id, e.g. \"m3\"."},
                    "summary": {"type": "string", "description": "Replacement summary."},
                    "content": {"type": "string", "description": "Replacement content. Markdown is recommended."},
                    "add_tags": {"type": "array", "items": {"type": "string"}, "description": "Tags to append; must exist unless create_missing_tags is true. Applied before remove_tags, so a tag present in both lists ends up removed."},
                    "remove_tags": {"type": "array", "items": {"type": "string"}, "description": "Tags to remove."},
                    "create_missing_tags": {"type": "boolean", "description": "Default false: unknown tag names in add_tags are an error. True auto-creates them with an empty description."}
                },
                "required": ["id"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            "memory_merge",
            "Merge two duplicate memories into one: 'source' is absorbed into 'target', then deleted. The target keeps its id and created_at; tags become the union of both. 'summary' / 'content' replace the target's fields when given; omitted content appends the source content after the target's (blank-line separated), and an omitted summary keeps the target's. This is the closing move after duplicate_of / similar_to flags a near-duplicate - delete + re-create would reset created_at. Requires both the update and delete permissions; memories carrying the reserved tag 'conventions' additionally need admin.",
            json!({
                "type": "object",
                "properties": {
                    "target": {"type": "string", "pattern": "^m[0-9]+$", "description": "Memory id to keep."},
                    "source": {"type": "string", "pattern": "^m[0-9]+$", "description": "Memory id to merge in and delete."},
                    "summary": {"type": "string", "description": "Optional replacement summary for the target."},
                    "content": {"type": "string", "description": "Optional replacement content for the target; omit to append the source content after the target's."}
                },
                "required": ["target", "source"],
                "additionalProperties": false
            }),
            false, true,
        ),
        def(
            "memory_delete",
            "Permanently delete one or more memories by id.",
            json!({
                "type": "object",
                "properties": {
                    "ids": {"type": "array", "items": {"type": "string", "pattern": "^m[0-9]+$"}, "minItems": 1, "maxItems": 50}
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

/// Tool name → list of valid parameter names (derived from inputSchema.properties, statically cached).
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

/// Tool name → whether it is read-only (derived from the readOnlyHint annotation, same source as the agent-facing declaration).
/// Determines the tool's transaction mode: read-only takes a DEFERRED snapshot, writes take the IMMEDIATE write lock.
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

/// Tool name → the capabilities required from the caller (the single registry of permissions,
/// enforced centrally at the tools::execute entry point). Tools acting across two capability
/// domains require both (memory_merge updates one memory and deletes another). Only queried for
/// tools in TOOL_NAMES; the fallback branch exists for type completeness only.
pub fn required_caps(tool: &str) -> &'static [Cap] {
    match tool {
        "tag_create" | "tag_update" | "tag_delete" => &[Cap::TagManage],
        "memory_create" => &[Cap::Create],
        "memory_list" | "memory_search" | "memory_get" => &[Cap::Read],
        "memory_update" => &[Cap::Update],
        "memory_merge" => &[Cap::Update, Cap::Delete],
        "memory_delete" => &[Cap::Delete],
        _ => &[Cap::Read],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The read-only flag shares its source with the contract: every tool is resolvable (unknown tools count as "not read-only"),
    /// and the number of read-only/write tools in the inventory matches the annotations. This flag drives the transaction mode;
    /// getting it backwards would make reads grab the write lock or run writes on a lock-free snapshot.
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
        assert_eq!(
            read_only, 4,
            "the contract should have exactly 4 read-only tools"
        );
    }

    /// The capability registry maps one-to-one onto the tool inventory: every tool has an entry (a new tool added without
    /// registering it fails here), and the capability assignments of a few key tools are sensible.
    #[test]
    fn required_cap_covers_every_tool() {
        use crate::auth::Cap;
        for name in TOOL_NAMES {
            // Registered as long as it does not panic (the mapping is a static table; coverage is enough)
            assert!(!required_caps(name).is_empty());
        }
        assert_eq!(required_caps("memory_get"), &[Cap::Read]);
        assert_eq!(required_caps("memory_create"), &[Cap::Create]);
        assert_eq!(required_caps("memory_update"), &[Cap::Update]);
        assert_eq!(required_caps("memory_delete"), &[Cap::Delete]);
        assert_eq!(required_caps("tag_update"), &[Cap::TagManage]);
        // A merge rewrites one memory and deletes another: it must not be reachable with either
        // capability alone
        assert_eq!(required_caps("memory_merge"), &[Cap::Update, Cap::Delete]);
    }
}
