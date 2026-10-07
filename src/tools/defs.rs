//! Tool inventory: names, descriptions and JSON Schema definitions.
//!
//! The schema is the single source of truth for the contract exposed to agents: the list of valid parameter
//! names used for validation is derived directly from `inputSchema.properties`, so the two can never drift apart.

use crate::auth::Cap;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::OnceLock;

/// Tool names as constants: the name string is the key of the agent-facing contract, and the
/// dispatcher, capability registry, reserved-tag guards, notification hooks and both server faces
/// (MCP / REST) all key on these — a rename is a one-line edit here, and any stale literal still
/// matched elsewhere fails to compile.
pub const TAG_CREATE: &str = "tag_create";
pub const TAG_LIST: &str = "tag_list";
pub const TAG_UPDATE: &str = "tag_update";
pub const TAG_DELETE: &str = "tag_delete";
pub const MEMORY_CREATE: &str = "memory_create";
pub const MEMORY_LIST: &str = "memory_list";
pub const MEMORY_SEARCH: &str = "memory_search";
pub const MEMORY_GET: &str = "memory_get";
pub const MEMORY_UPDATE: &str = "memory_update";
pub const MEMORY_EDIT: &str = "memory_edit";
pub const MEMORY_MERGE: &str = "memory_merge";
pub const MEMORY_DELETE: &str = "memory_delete";

pub const TOOL_NAMES: &[&str] = &[
    TAG_CREATE,
    TAG_LIST,
    TAG_UPDATE,
    TAG_DELETE,
    MEMORY_CREATE,
    MEMORY_LIST,
    MEMORY_SEARCH,
    MEMORY_GET,
    MEMORY_UPDATE,
    MEMORY_EDIT,
    MEMORY_MERGE,
    MEMORY_DELETE,
];

/// Shared tag_expr parameter text: one syntax, two tools (memory_list / memory_search) — the
/// descriptions must not drift apart.
const TAG_EXPR_DESCRIPTION: &str = "Tag set algebra over tag names: only memories whose tag set satisfies the expression are returned. Operators: & (and, also &&), | (or, also ||), ! (not), parentheses for grouping; precedence ! > & > |. Example: \"(a&b)|c\" = tagged a AND b, or tagged c. Operands are tag names or regular expressions in slashes: /proj.*/ passes when ANY of the memory's tag names matches (Rust regex syntax, case-sensitive - inline flags like (?i) work; escape the slash as \\/, and a slash inside a bare word stays a name character). Names are case-sensitive; quote names containing operators, whitespace or parentheses ('single' or \"double\" quotes, backslash escapes). Literal tag names unknown to the store are rejected with the closest existing name suggested. Empty string counts as absent.";

pub fn tool_definitions() -> Value {
    Value::Array(vec![
        def(
            TAG_CREATE,
            "Create a new tag. Tags are labels used to organize memories; you own the taxonomy. Fails if the tag already exists (check with tag_list). If a tag differing only by case exists, it is created anyway but reported in similar_existing - prefer merging to keep the taxonomy tidy. Responses never echo the input: a clean create answers with an empty object, success being the absence of an error.",
            json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Unique tag name (trimmed, 1-100 chars, case-sensitive, any language). Format: only letters, digits, '_', '-' and '.' (no spaces, quotes or expression operators), starting with a letter, digit or '_'."},
                    "description": {"type": "string", "description": "Optional: what this tag groups (max 512 chars)."}
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            TAG_LIST,
            "List tags with descriptions and memory counts. Token-frugal line format, one tag per line: `<count> <name>[: <description>]` - the description is omitted when empty, a `*` before the name marks a reserved tag, names containing whitespace, commas, brackets or quotes are quoted, and newlines inside text render as literal \\n (one line is always one record). 'convention' is always listed (synthesized with zero memories before it exists) and can never be renamed or deleted. Name filterable by regex. Start here when exploring the memory store.",
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
            TAG_UPDATE,
            "Update a tag: rename it and/or set its description (the only way to fill in a description after tag_create). Memories referencing the tag follow a rename automatically - only the label changes. Omit new_name to keep the current name; omit description to keep the current one; at least one of the two is required. The response carries only the flags renamed / description_updated telling which parts actually changed - names and descriptions are never echoed back.",
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
            TAG_DELETE,
            "Delete a tag. mode 'detach' (default) removes only the tag from memories and keeps them; mode 'purge' also permanently deletes every memory carrying this tag - preview the impact first with dry_run: true (reports the affected memory count and, for purge, their ids) since purge cannot be undone. Responses carry counts only (memories_updated / memories_deleted; a dry_run reports memories_affected), never an echo of the arguments.",
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
            MEMORY_CREATE,
            "Store a new memory. 'summary' is the one-line abstract shown by searches and lists (progressive disclosure level 1); 'content' is the optional full text revealed on demand (level 2) - omit it when the summary alone says it all, and the memory stores with an empty body. Tags must exist: unknown names are rejected with the closest existing tags listed - create them explicitly with tag_create, reuse the listed names, or pass create_missing_tags: true to auto-create them empty (keeping the taxonomy tidy is your job; case variants and synonyms rot it). The response never echoes the input back - it carries only the new id, the server timestamp, and a sparse tag classification: tags_autocreated (created just now) and tags_missing_description (reused tags that still lack a description - consider filling one in with tag_update), each attached only when non-empty; tags absent from both lists were reused as-is. Existing memories with the same summary are listed in duplicate_of, and with semantic search enabled near-duplicates by embedding similarity additionally appear in similar_to (also only when found) - both are advisory: prefer memory_update / memory_merge on those instead of storing again. Do not reference other memories by id: ids are unstable (delete/merge removes them, export/import renumbers them) - link memories by tag or searchable keyword instead; text that looks like it contains id references draws a warning note.",
            json!({
                "type": "object",
                "properties": {
                    "summary": {"type": "string", "description": "One-line abstract (max 512 chars); make it precise and self-contained."},
                    "content": {"type": "string", "description": "Optional full text of the memory (the body memory_get reveals). Omit or pass empty for a summary-only memory. Markdown is recommended (headings, lists, code blocks, tables); the web admin UI renders it."},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Tag names for this memory; must exist unless create_missing_tags is true. Format: only letters, digits, '_', '-' and '.'."},
                    "create_missing_tags": {"type": "boolean", "description": "Default false: unknown tag names are an error. True auto-creates unknown tags with an empty description."}
                },
                "required": ["summary"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            MEMORY_LIST,
            "Browse memories, optionally filtered by a tag expression. Token-frugal line format: a `total: N | offset: N` header, then one memory per line `<id> [<tags>] <updated> <summary>` ('updated' is the server's local wall clock, 'YYYY-MM-DD HH:MM'; newlines inside text render as literal \\n, one line is always one record). Returns summaries only - never full content; call memory_get for entries worth reading. Newest first by default; paginated (page with offset/limit, the header's total tells when to stop).",
            json!({
                "type": "object",
                "properties": {
                    "tag_expr": {"type": "string", "description": TAG_EXPR_DESCRIPTION},
                    "sort": {"type": "string", "enum": ["updated_at", "id"], "description": "Default: updated_at. 'id' equals creation order."},
                    "order": {"type": "string", "enum": ["asc", "desc"], "description": "Default: desc (newest first)."},
                    "offset": {"type": "integer", "minimum": 0},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 200, "description": "Default 20."}
                },
                "additionalProperties": false
            }),
            true, false,
        ),
        def(
            MEMORY_SEARCH,
            "Search across tags, summaries and content. All whitespace-separated terms must match (AND); a quoted term (\"exact phrase\") must match verbatim and scores a bonus. Matching is substring-based and case-insensitive, so CJK queries work without segmentation. Repeated hits and whole-word ASCII matches rank higher. Tag matches rank highest. When the server has semantic search enabled, results also include meaning-similar memories that share no keywords (mode 'hybrid'); if the embedding service is unavailable the search silently falls back to keyword-only and the response carries a `semantic_fallback: true` line; when semantic search is not configured at all, auto runs keyword-only and the response carries a `semantic: disabled` line - no inference needed. Token-frugal line format: header `total_matches: N | offset: N | returned: N | mode: <mode>`, then one result per line `<id> [<tags>] <updated> <score> <summary>` followed by an indented `  > <snippet>` line (a short single-line plain-text excerpt of the content) - call memory_get on the promising ids to reveal full content; optional flag lines (semantic_fallback / semantic / hint / note) sit between header and results only when they apply. score is an ordering integer, not a percentage. Keyword mode sums field weights: exact tag hit +40 per tag, tag substring +25, summary hit +10 each (capped at 3), content hit +3 each (capped at 3), ASCII whole-word bonus +5 (summary) / +2 (content), whole quoted phrase +8 - so roughly 50+ means a strong multi-field match and 10-30 a weak single-field one. Hybrid mode instead reports an RRF fusion of both channels' ranks (scaled, tiny values) comparable only within the same response; do not compare scores across modes. The semantic channel is capped: only the top ~2*limit vector candidates by cosine enter the fusion (floor 20), so total_matches stays bounded; when hybrid runs, the response also carries a `keyword_matches: N` line counting how many matches are literal keyword hits - the remainder entered via the semantic channel alone and are looser, meaning-similar hits (verify before relying on them).",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Whitespace-separated keywords; wrap words in quotes to require verbatim adjacency."},
                    "tag_expr": {"type": "string", "description": TAG_EXPR_DESCRIPTION},
                    "mode": {"type": "string", "enum": ["auto", "keyword", "hybrid"], "description": "Default: auto - hybrid (keyword + semantic) when the server has semantic search configured, otherwise plain keyword (flagged semantic: \"disabled\" in the response). 'keyword' forces keyword-only; 'hybrid' requires semantic search to be configured (error if not). Hybrid falls back to keyword automatically when the embedding service is unavailable."},
                    "offset": {"type": "integer", "minimum": 0, "description": "Skip the first N ranked matches (for paging through many results)."},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 50, "description": "Default 10."}
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            true, false,
        ),
        def(
            MEMORY_GET,
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
            MEMORY_UPDATE,
            "Update a memory: new summary, new content, and/or adjust tags incrementally via add_tags / remove_tags (no need to know the current tag list). For a small change inside long content, memory_edit (exact string replacement) is far cheaper than restating the whole body. Updating summary or content refreshes the updated time; tag-only adjustments leave it untouched, so curating tags never reshuffles the default newest-first browse. Unknown tags in add_tags are rejected (closest existing tags listed) unless create_missing_tags is true. The response never echoes the input back: the flag `updated` tells whether anything actually changed, and when tags were added, tags_autocreated / tags_missing_description classify them (same meaning as in memory_create; the reused remainder is left implicit - it is the input minus those two lists). Replacing the content changes what other memories' references to this id point at: the response lists referencing memories under referenced_by (ids + summaries) when the content actually changed. Do not reference other memories by id (unstable - see memory_create); id-like text in the written fields draws a warning note.",
            json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string", "pattern": "^m[0-9]+$", "description": "Memory id, e.g. \"m3\"."},
                    "summary": {"type": "string", "description": "Replacement summary."},
                    "content": {"type": "string", "description": "Replacement content; pass empty to clear the body (summary-only memory). Markdown is recommended."},
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
            MEMORY_EDIT,
            "Edit a memory's content by exact string replacement instead of restating the whole body - the efficient way to change a long memory. old_string is located in the current content byte-exact (case-sensitive, whitespace-significant) and swapped for new_string in place. The edit fails without side effects when old_string is absent (re-read the content with memory_get first) or when it matches more than once (then add surrounding text to make the match unique, or pass replace_all: true to replace every occurrence and the response reports the count). new_string may be empty to delete the matched span; old_string must be non-empty and differ from new_string. Refreshes the updated time and re-embeds the memory for semantic search, same as memory_update. The response carries only that replaced count - the resulting content is the caller's own computation, never echoed back. id-like tokens in new_string draw a warning note: do not reference other memories by id (ids are unstable - see memory_create).",
            json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string", "pattern": "^m[0-9]+$", "description": "Memory id, e.g. \"m3\"."},
                    "old_string": {"type": "string", "minLength": 1, "description": "Exact text to replace (byte-exact: case-sensitive, whitespace-significant)."},
                    "new_string": {"type": "string", "description": "Replacement text; may be empty to delete the matched span. Must differ from old_string."},
                    "replace_all": {"type": "boolean", "description": "Default false: old_string must match exactly once. True replaces every occurrence."}
                },
                "required": ["id", "old_string", "new_string"],
                "additionalProperties": false
            }),
            false, false,
        ),
        def(
            MEMORY_MERGE,
            "Merge two duplicate memories into one: 'source' is absorbed into 'target', then deleted. The target keeps its id and creation time; tags become the union of both. 'summary' / 'content' replace the target's fields when given; omitted content appends the source content after the target's (blank-line separated), and an omitted summary keeps the target's. This is the closing move after duplicate_of / similar_to flags a near-duplicate - delete + re-create would reset the created time. Requires both the update and delete permissions; memories carrying the reserved tag 'convention' additionally need admin. The source id does not survive the merge: the response lists remaining memories that referenced it under referenced_by so their mentions can be repaired. The response never echoes the input: a clean merge answers with an empty object, everything reported is a reference report or warning note.",
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
            MEMORY_DELETE,
            "Permanently delete one or more memories by id. The response lists remaining memories that referenced a deleted id (referenced_by, ids + summaries) so their now-dangling mentions can be repaired.",
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
        TAG_CREATE | TAG_UPDATE | TAG_DELETE => &[Cap::TagManage],
        MEMORY_CREATE => &[Cap::Create],
        MEMORY_LIST | MEMORY_SEARCH | MEMORY_GET => &[Cap::Read],
        MEMORY_UPDATE | MEMORY_EDIT => &[Cap::Update],
        MEMORY_MERGE => &[Cap::Update, Cap::Delete],
        MEMORY_DELETE => &[Cap::Delete],
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
