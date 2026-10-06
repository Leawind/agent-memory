//! MCP tool layer: the entry point for argument parsing, tool definitions and business handling.
//!
//! Module layout:
//! - `defs`    tool inventory and JSON Schema (the agent-facing contract, the single source of truth for argument validation)
//! - `params`  argument parsing and validation helpers
//! - `tag_ops` tag create/read/update/delete
//! - `memory_ops` memory CRUD, browsing and search
//!
//! Data-access convention: `execute_with_db` runs "open → execute → commit" inside a single SQLite transaction;
//! on panic or error the whole transaction rolls back, and multi-process concurrency is guaranteed by SQLite WAL + busy_timeout.

mod defs;
mod memory_ops;
mod params;
mod tag_ops;

use crate::auth::{Cap, IdentityCtx};
use crate::model::RESERVED_TAG;
use crate::store::{self, Store};
use serde_json::{Map, Value};
use std::path::Path;

pub use defs::{
    tool_definitions, MEMORY_CREATE, MEMORY_DELETE, MEMORY_EDIT, MEMORY_GET, MEMORY_LIST,
    MEMORY_MERGE, MEMORY_SEARCH, MEMORY_UPDATE, TAG_CREATE, TAG_DELETE, TAG_LIST, TAG_UPDATE,
    TOOL_NAMES,
};

pub const INSTRUCTIONS: &str = "Persistent long-term memory store. Each memory has: tags (a taxonomy YOU curate), a one-line summary, and full content. Progressive disclosure: memory_search / memory_list return only ids, tags and summaries; call memory_get on just the ids worth reading to reveal full content. Timestamps are rendered as the server's local wall clock ('YYYY-MM-DD HH:MM') and recorded automatically — summaries carry only 'updated'; never state creation time inside content. Write content as concise Markdown; avoid bold formatting. Save durable knowledge (decisions, facts, preferences, project context) with memory_create; write precise, self-contained summaries so future scans stay cheap; prefer memory_update over re-storing near-duplicates; to change a small part of a memory's content, prefer memory_edit (exact string replacement) over restating the whole body; never reference other memories by id — ids are unstable (delete/merge removes them, export/import renumbers them), so link memories by tag or searchable keyword instead; keep tags tidy with the tag_* tools. The 'conventions' tag is reserved for operator-curated standing rules: those memories are the store's resident conventions (also exposed as memory:// resources) — read them before your first write and follow them. Access is permission-gated per caller identity: when a call fails with a permission error, report it to the user instead of retrying.";

/// Tool-layer errors: classified by kind, never by text; the REST layer maps kinds to HTTP status codes
/// (NotFound → 404, Invalid → 400, Forbidden → 403), while the MCP layer always echoes the message as an
/// isError result.
#[derive(Debug, Clone)]
pub enum ToolError {
    /// Resource does not exist (REST → 404)
    NotFound(String),
    /// Argument validation failure or business conflict (REST → 400)
    Invalid(String),
    /// The caller identity lacks a required capability (REST → 403; consistent with the MCP spec's
    /// "403 = insufficient permissions" semantics)
    Forbidden(String),
}

impl ToolError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        ToolError::NotFound(msg.into())
    }

    pub fn invalid(msg: impl Into<String>) -> Self {
        ToolError::Invalid(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        ToolError::Forbidden(msg.into())
    }

    /// Readable message for agents / the management UI.
    pub fn message(&self) -> &str {
        match self {
            ToolError::NotFound(m) | ToolError::Invalid(m) | ToolError::Forbidden(m) => m,
        }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl From<String> for ToolError {
    /// String errors from the data/params layers default to Invalid;
    /// handlers construct the "not found" kind explicitly via NotFound.
    fn from(message: String) -> Self {
        ToolError::Invalid(message)
    }
}

/// Completes "open database → execute → commit" inside a single transaction.
/// The transaction mode is decided by the tool contract: tools whose readOnlyHint is true take a DEFERRED snapshot,
/// the rest take the IMMEDIATE write lock (see store::TxMode).
/// Permissions are checked centrally at the `execute` entry point (capability requirements in `defs::required_cap`).
pub fn execute_with_db(
    db_path: &Path,
    ctx: &IdentityCtx,
    name: &str,
    args: &Value,
) -> Result<Value, ToolError> {
    let mode = if defs::is_read_only(name) {
        store::TxMode::ReadOnly
    } else {
        store::TxMode::Write
    };
    // The notification diff needs the state before the write (outside any transaction).
    let pre = crate::notify::capture(db_path, name, args);
    let mut out = store::with_db_in(db_path, mode, |st| execute(st, ctx, name, args))?;
    // Embedding runs after the transaction commits (network calls never enter transactions): when the embedding
    // service is unavailable the tool degrades silently, results are unaffected, and vectors are left for backfill.
    // A successful create then gets a semantic near-duplicate scan attached to its result (advisory, fallback-safe).
    if crate::embed::needs_backfill(name) {
        // The dedup hint must see the new memory's vector deterministically, so for a create it
        // embeds that one text synchronously (bounded by the query timeout); everything else
        // drains on a background thread and never delays the response.
        if name == defs::MEMORY_CREATE {
            crate::embed::dedup_hint(db_path, &mut out);
        }
        crate::embed::after_write(db_path);
    }
    crate::notify::after_write(name, args, &out, &pre);
    Ok(out)
}

pub fn execute(
    st: &Store,
    ctx: &IdentityCtx,
    name: &str,
    args: &Value,
) -> Result<Value, ToolError> {
    let map = args
        .as_object()
        .ok_or_else(|| ToolError::invalid("arguments must be a JSON object"))?;
    check_known_args(name, map)?;
    if TOOL_NAMES.contains(&name) {
        ctx.require_all(defs::required_caps(name))?;
        reserved_tag_guard(ctx, name, map)?;
    }

    match name {
        defs::TAG_CREATE => tag_ops::tag_create(st, map),
        defs::TAG_LIST => tag_ops::tag_list(st, map),
        defs::TAG_UPDATE => tag_ops::tag_update(st, map),
        defs::TAG_DELETE => tag_ops::tag_delete(st, map),
        defs::MEMORY_CREATE => memory_ops::memory_create(st, map),
        defs::MEMORY_LIST => memory_ops::memory_list(st, map),
        defs::MEMORY_SEARCH => memory_ops::memory_search(st, map),
        defs::MEMORY_GET => memory_ops::memory_get(st, map),
        defs::MEMORY_UPDATE => memory_ops::memory_update(st, map),
        defs::MEMORY_EDIT => memory_ops::memory_edit(st, map),
        // The merge needs the identity for the data-dependent reserved-tag check (its arguments
        // name no tags; only the loaded memories reveal whether conventions is involved)
        defs::MEMORY_MERGE => memory_ops::memory_merge(st, ctx, map),
        defs::MEMORY_DELETE => memory_ops::memory_delete(st, map),
        _ => Err(ToolError::invalid(format!("unknown tool '{name}'"))),
    }
}

/// Reserved-tag guards for 'conventions' (the resident conventions, also surfaced as resources).
/// Checked here at the entry with the identity context at hand, so the MCP face and the REST face
/// (which reuses tools::execute) enforce identical rules:
/// - the tag itself can never be renamed or deleted (hard rule, admins included);
/// - creating it and attaching it to / detaching it from memories requires the admin capability,
///   keeping the operator-curated conventions out of agents' reach.
fn reserved_tag_guard(
    ctx: &IdentityCtx,
    name: &str,
    args: &Map<String, Value>,
) -> Result<(), ToolError> {
    fn raw_tag(args: &Map<String, Value>, key: &str) -> Option<String> {
        args.get(key)
            .and_then(Value::as_str)
            .map(|s| s.trim().to_string())
    }
    fn involves_reserved(args: &Map<String, Value>, keys: &[&str]) -> bool {
        keys.iter()
            .filter_map(|k| args.get(*k).and_then(Value::as_array))
            .flatten()
            .filter_map(Value::as_str)
            .any(|t| t.trim() == RESERVED_TAG)
    }
    match name {
        defs::TAG_CREATE => {
            if raw_tag(args, "name").as_deref() == Some(RESERVED_TAG) && !ctx.can(Cap::Admin) {
                return Err(ToolError::forbidden(format!(
                    "'{RESERVED_TAG}' is a reserved tag; creating it requires the 'admin' permission (it anchors the operator-curated conventions)"
                )));
            }
        }
        defs::TAG_UPDATE | defs::TAG_DELETE => {
            let mut targets = vec![raw_tag(args, "name")];
            if name == defs::TAG_UPDATE {
                targets.push(raw_tag(args, "new_name"));
            }
            if targets.into_iter().flatten().any(|t| t == RESERVED_TAG) {
                return Err(ToolError::invalid(format!(
                    "'{RESERVED_TAG}' is a reserved tag: it anchors the resident conventions and cannot be renamed or deleted"
                )));
            }
        }
        defs::MEMORY_CREATE => {
            if involves_reserved(args, &["tags"]) && !ctx.can(Cap::Admin) {
                return Err(ToolError::forbidden(format!(
                    "attaching the reserved tag '{RESERVED_TAG}' to a memory requires the 'admin' permission (conventions are operator-curated)"
                )));
            }
        }
        defs::MEMORY_UPDATE
            if involves_reserved(args, &["add_tags", "remove_tags"]) && !ctx.can(Cap::Admin) =>
        {
            return Err(ToolError::forbidden(format!(
                "attaching or detaching the reserved tag '{RESERVED_TAG}' requires the 'admin' permission (conventions are operator-curated)"
            )));
        }
        _ => {}
    }
    Ok(())
}

/// Reject unknown arguments: valid parameter names are derived from inputSchema.properties, inherently in sync with the agent-facing contract.
/// This surfaces caller typos early; otherwise arguments would be silently ignored, causing behavior that is much harder to debug.
fn check_known_args(name: &str, args: &Map<String, Value>) -> Result<(), String> {
    if !TOOL_NAMES.contains(&name) {
        return Ok(()); // the unknown-tool error message is produced by the match's fallback branch below
    }
    let known = defs::known_args(name).expect("every listed tool has a definition");
    for key in args.keys() {
        if !known.iter().any(|k| k == key) {
            let valid = if known.is_empty() {
                "(none)".to_string()
            } else {
                known.join(", ")
            };
            return Err(format!(
                "unknown argument '{key}' for tool '{name}'; valid arguments: {valid}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEQ: AtomicU32 = AtomicU32::new(0);

    fn temp_db(tag: &str) -> PathBuf {
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "agent-memory-tools-{}-{}-{}.db",
            std::process::id(),
            tag,
            n
        ))
    }

    fn call(path: &Path, name: &str, args: Value) -> Result<Value, ToolError> {
        execute_with_db(path, &crate::auth::IdentityCtx::open_mode(), name, &args)
    }

    /// Call with a given capability set (for permission boundary tests).
    fn call_as(
        path: &Path,
        caps: &[crate::auth::Cap],
        name: &str,
        args: Value,
    ) -> Result<Value, ToolError> {
        let mut obj = serde_json::Map::new();
        for c in caps {
            obj.insert(c.as_str().to_string(), json!(true));
        }
        let perms = crate::auth::Permissions::from_json(&Value::Object(obj)).unwrap();
        execute_with_db(
            path,
            &crate::auth::IdentityCtx::new("tester", perms),
            name,
            &args,
        )
    }

    fn cleanup(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(PathBuf::from(format!("{}{}", path.display(), suffix)));
        }
    }

    #[test]
    fn full_memory_lifecycle() {
        let path = temp_db("lifecycle");
        let created = call(
            &path,
            "memory_create",
            json!({
                "summary": "Rust borrow checker notes",
                "content": "The borrow checker forbids simultaneous aliasing and mutation.",
                "tags": ["rust", "notes"],
                "create_missing_tags": true
            }),
        )
        .unwrap();
        let id = created["memory"]["id"].as_str().unwrap().to_string();
        assert_eq!(created["tags_autocreated"].as_array().unwrap().len(), 2);
        // Sparse response: empty classifications are omitted outright
        assert!(created.get("tags_reused").is_none());
        assert!(created.get("tags_missing_description").is_none());
        assert!(created.get("duplicate_of").is_none());

        // Search hits, with no content leaked (first layer of progressive disclosure)
        let found = call(&path, "memory_search", json!({"query": "borrow"})).unwrap();
        let results = found["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["id"], id.as_str());
        assert!(results[0].get("content").is_none());
        assert!(results[0]["snippet"].as_str().unwrap().contains("borrow"));

        // Listing carries only titles
        let listing = call(&path, "memory_list", json!({})).unwrap();
        let items = listing["memories"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].get("content").is_none());

        // Fetch full text (second layer)
        let got = call(&path, "memory_get", json!({"ids": [id]})).unwrap();
        assert_eq!(
            got["memories"][0]["content"],
            "The borrow checker forbids simultaneous aliasing and mutation."
        );

        // Update: incrementally add/remove tags (rust/notes were both left by auto-creation with empty descriptions)
        let upd = call(
            &path,
            "memory_update",
            json!({"id": id, "add_tags": ["study"], "remove_tags": ["notes"], "create_missing_tags": true}),
        )
        .unwrap();
        assert_eq!(upd["memory"]["tags"].as_array().unwrap().len(), 2);
        // The three-way classification only describes tags passed to add_tags this time (rust is not in the list, so not classified)
        assert_eq!(upd["tags_autocreated"], json!(["study"]));
        assert!(upd.get("tags_reused").is_none());
        assert!(upd.get("tags_missing_description").is_none());

        // Update content and title (regression: the field-update path once failed the whole update when the embeddings table was missing)
        let upd_fields = call(
            &path,
            "memory_update",
            json!({"id": id, "summary": "Borrow checker 101", "content": "Updated body."}),
        )
        .unwrap();
        assert_eq!(upd_fields["updated"], true);
        assert!(
            upd_fields.get("tags_autocreated").is_none(),
            "field-only update carries no tag fields"
        );

        // Delete
        let del = call(&path, "memory_delete", json!({"ids": [id]})).unwrap();
        assert_eq!(del["deleted"].as_array().unwrap().len(), 1);
        let after = call(&path, "memory_list", json!({})).unwrap();
        assert_eq!(after["total"], 0);

        cleanup(&path);
    }

    /// memory_create's three-way tag-link classification: created / reused / reused-but-missing-description.
    #[test]
    fn create_reports_tag_mount_classification() {
        let path = temp_db("tag-mount");
        call(
            &path,
            "tag_create",
            json!({"name": "described", "description": "has one"}),
        )
        .unwrap();
        let first = call(
            &path,
            "memory_create",
            json!({"summary": "a", "content": "ca", "tags": ["described", "fresh"], "create_missing_tags": true}),
        )
        .unwrap();
        assert_eq!(first["tags_autocreated"], json!(["fresh"]));
        assert_eq!(first["tags_reused"], json!(["described"]));
        assert!(first.get("tags_missing_description").is_none());

        // Second call reuses everything; fresh was auto-created last time with an empty description → named hint
        let second = call(
            &path,
            "memory_create",
            json!({"summary": "b", "content": "cb", "tags": ["described", "fresh"], "create_missing_tags": true}),
        )
        .unwrap();
        assert!(second.get("tags_autocreated").is_none());
        assert_eq!(second["tags_reused"], json!(["described", "fresh"]));
        assert_eq!(second["tags_missing_description"], json!(["fresh"]));

        cleanup(&path);
    }

    /// Bare numeric ids are reported separately from "not found": get/delete put them in invalid_ids,
    /// update reports a format error as 400; delete never reports something it did not delete as deleted.
    #[test]
    fn invalid_ids_are_reported_separately_from_missing() {
        let path = temp_db("invalid-ids");
        call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": "c"}),
        )
        .unwrap();

        let got = call(&path, "memory_get", json!({"ids": ["102", "m999"]})).unwrap();
        assert_eq!(got["invalid_ids"], json!(["102"]));
        assert_eq!(got["missing"], json!(["m999"]));
        let note = got["note"].as_str().unwrap();
        assert!(
            note.contains("malformed") && note.contains("not found"),
            "both hints coexist: {note}"
        );

        let del = call(&path, "memory_delete", json!({"ids": ["102"]})).unwrap();
        assert_eq!(del["deleted"], json!([]));
        // Sparse: nothing missing, no missing key
        assert!(del.get("missing").is_none());
        assert_eq!(del["invalid_ids"], json!(["102"]));

        let err = call(&path, "memory_update", json!({"id": "102", "summary": "x"})).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("malformed") && msg.contains("m123"),
            "got: {msg}"
        );

        cleanup(&path);
    }

    #[test]
    fn tag_management_flows() {
        let path = temp_db("tags");
        call(
            &path,
            "tag_create",
            json!({"name": "rust", "description": "Rust language"}),
        )
        .unwrap();
        assert!(call(&path, "tag_create", json!({"name": "rust"})).is_err());

        call(
            &path,
            "memory_create",
            json!({
                "summary": "s", "content": "c", "tags": ["rust"]
            }),
        )
        .unwrap();

        // Update: rename + description change in one go; memory references follow the new name
        let upd = call(
            &path,
            "tag_update",
            json!({"name": "rust", "new_name": "lang", "description": "Programming languages"}),
        )
        .unwrap();
        assert_eq!(upd["renamed"], true);
        assert_eq!(upd["description_updated"], true);
        assert_eq!(upd["name"], "lang");
        let tl = call(&path, "tag_list", json!({})).unwrap();
        let tags = tl["tags"].as_array().unwrap();
        let lang = tags
            .iter()
            .find(|t| t["name"] == "lang")
            .expect("renamed tag listed");
        assert_eq!(lang["count"], 1);
        assert_eq!(lang["description"], "Programming languages");

        // detach only removes the tag
        call(
            &path,
            "tag_delete",
            json!({"name": "lang", "mode": "detach"}),
        )
        .unwrap();
        let after = call(&path, "memory_list", json!({})).unwrap();
        assert_eq!(after["total"], 1);
        assert_eq!(after["memories"][0]["tags"].as_array().unwrap().len(), 0);

        cleanup(&path);
    }

    #[test]
    fn purge_deletes_memories() {
        let path = temp_db("purge");
        call(
            &path,
            "memory_create",
            json!({"summary": "a", "content": "ca", "tags": ["x"], "create_missing_tags": true}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "b", "content": "cb", "tags": ["x", "keep"], "create_missing_tags": true}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "c", "content": "cc", "tags": ["keep"], "create_missing_tags": true}),
        )
        .unwrap();

        let r = call(&path, "tag_delete", json!({"name": "x", "mode": "purge"})).unwrap();
        assert_eq!(r["memories_deleted"].as_array().unwrap().len(), 2);
        let after = call(&path, "memory_list", json!({})).unwrap();
        assert_eq!(after["total"], 1);

        cleanup(&path);
    }

    /// dry_run prices a deletion without performing it: counts for both modes, exact ids for
    /// purge, and provably no side effect.
    #[test]
    fn tag_delete_dry_run_previews_without_touching_anything() {
        let path = temp_db("dry-run");
        call(
            &path,
            "memory_create",
            json!({"summary": "a", "content": "ca", "tags": ["x"], "create_missing_tags": true}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "b", "content": "cb", "tags": ["x", "keep"], "create_missing_tags": true}),
        )
        .unwrap();

        let preview = call(
            &path,
            "tag_delete",
            json!({"name": "x", "mode": "purge", "dry_run": true}),
        )
        .unwrap();
        assert_eq!(preview["dry_run"], true);
        assert_eq!(preview["memories_affected"], 2);
        let ids = preview["memory_ids"].as_array().unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.iter().all(|i| i.as_str().unwrap().starts_with('m')));

        // Nothing happened: both memories and the tag are still there
        assert_eq!(call(&path, "memory_list", json!({})).unwrap()["total"], 2);
        let tl = call(&path, "tag_list", json!({})).unwrap();
        assert!(tl["tags"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["name"] == "x" && t.get("reserved").is_none()));

        // detach preview counts, too, but carries no ids
        let preview = call(
            &path,
            "tag_delete",
            json!({"name": "x", "mode": "detach", "dry_run": true}),
        )
        .unwrap();
        assert_eq!(preview["memories_affected"], 2);
        assert!(preview["memory_ids"].is_null());

        cleanup(&path);
    }

    #[test]
    fn validation_and_missing_errors() {
        let path = temp_db("errors");
        // Missing required argument (content is optional; summary is not)
        assert!(call(&path, "memory_create", json!({"content": "c"})).is_err());
        // Empty title
        assert!(call(
            &path,
            "memory_create",
            json!({"summary": "   ", "content": "c"})
        )
        .is_err());
        // Nonexistent memory
        assert!(call(&path, "memory_update", json!({"id": "m99", "summary": "x"})).is_err());
        // Nonexistent tag
        assert!(call(&path, "tag_delete", json!({"name": "nope"})).is_err());
        // memory_get returns missing ids as missing rather than erroring, with a guiding hint attached
        let got = call(&path, "memory_get", json!({"ids": ["m99"]})).unwrap();
        assert_eq!(got["missing"].as_array().unwrap().len(), 1);
        assert!(got["note"].as_str().unwrap().contains("memory_search"));

        cleanup(&path);
    }

    #[test]
    fn unknown_arguments_are_rejected() {
        let path = temp_db("unknown-args");
        // A misspelled parameter name errors immediately and lists valid parameters, instead of being silently ignored
        let err = call(
            &path,
            "memory_create",
            json!({"summray": "typo", "content": "c"}),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("'summray'") && err.to_string().contains("summary"),
            "got: {err}"
        );
        // Unknown parameter names error (tag_list's valid parameter is filter; a typo is still rejected)
        let err = call(&path, "tag_list", json!({"flter": "x"})).unwrap_err();
        assert!(err.to_string().contains("'flter'"), "got: {err}");
        // Valid parameters are unaffected
        call(
            &path,
            "memory_create",
            json!({"summary": "ok", "content": "c"}),
        )
        .unwrap();

        cleanup(&path);
    }

    /// Tag regex filtering: tag_list's filter, memory_list / memory_search's
    /// tag_filter (invalid regex errors, AND-ed with exact filtering, zero-hit note).
    #[test]
    fn regex_filters_on_tags_and_memories() {
        let path = temp_db("regex-filter");
        call(&path, "tag_create", json!({"name": "proj/alpha"})).unwrap();
        call(&path, "tag_create", json!({"name": "misc"})).unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "alpha note", "content": "c", "tags": ["proj/alpha"]}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "misc note", "content": "c", "tags": ["misc"]}),
        )
        .unwrap();

        // tag_list filter: unanchored substring matching, ^...$ anchors the full name
        let tl = call(&path, "tag_list", json!({"filter": "^proj/"})).unwrap();
        assert_eq!(tl["tags"][0]["name"], "proj/alpha");
        // Case-insensitive, mirroring content search's case folding: case variants in the
        // taxonomy must not split the filters either
        let up = call(&path, "tag_list", json!({"filter": "^PROJ/"})).unwrap();
        assert_eq!(up["tags"].as_array().unwrap().len(), 1);
        let ml = call(&path, "memory_list", json!({"tag_filter": "MISC"})).unwrap();
        assert_eq!(ml["total"], 1);
        let none = call(&path, "tag_list", json!({"filter": "zzz"})).unwrap();
        assert_eq!(none["tags"].as_array().unwrap().len(), 0);
        // Invalid regex → Invalid (400 kind)
        let err = call(&path, "tag_list", json!({"filter": "("})).unwrap_err();
        assert!(
            err.to_string().contains("not a valid regular expression"),
            "got: {err}"
        );

        // memory_list tag_filter: the total counts only matching memories
        let ml = call(&path, "memory_list", json!({"tag_filter": "^proj/"})).unwrap();
        assert_eq!(ml["total"], 1);
        assert_eq!(ml["memories"][0]["summary"], "alpha note");
        // Regex matched no tags → note hint
        let ml_empty = call(&path, "memory_list", json!({"tag_filter": "zzz"})).unwrap();
        assert_eq!(ml_empty["total"], 0);
        assert!(
            ml_empty["note"]
                .as_str()
                .unwrap()
                .contains("matched no tags"),
            "got: {ml_empty}"
        );
        // Used together with a tag expression = AND
        let ml_and = call(
            &path,
            "memory_list",
            json!({"tag_filter": "^proj/", "tag_expr": "misc"}),
        )
        .unwrap();
        assert_eq!(ml_and["total"], 0);

        // memory_search tag_filter
        let ms = call(
            &path,
            "memory_search",
            json!({"query": "note", "tag_filter": "^proj/"}),
        )
        .unwrap();
        assert_eq!(ms["total_matches"], 1);
        assert_eq!(ms["results"][0]["summary"], "alpha note");

        cleanup(&path);
    }

    #[test]
    fn search_offset_paginates() {
        let path = temp_db("search-offset");
        for i in 0..4 {
            call(
                &path,
                "memory_create",
                json!({"summary": format!("item {}", i), "content": "shared keyword", "tags": []}),
            )
            .unwrap();
        }
        let first = call(
            &path,
            "memory_search",
            json!({"query": "shared", "limit": 2}),
        )
        .unwrap();
        assert_eq!(first["total_matches"], 4);
        assert_eq!(first["returned"], 2);
        let page2 = call(
            &path,
            "memory_search",
            json!({"query": "shared", "limit": 2, "offset": 2}),
        )
        .unwrap();
        assert_eq!(page2["returned"], 2);
        let ids1: Vec<&str> = first["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        let ids2: Vec<&str> = page2["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert!(
            ids1.iter().all(|id| !ids2.contains(id)),
            "pages must not overlap"
        );

        cleanup(&path);
    }

    /// Semantic-state signaling on search responses: with embedding unconfigured, auto runs
    /// keyword-only and must say so explicitly (semantic: "disabled") instead of leaving the
    /// caller to infer it from mode + tool description. Explicit keyword stays unflagged (the
    /// caller chose it), and unconfigured hybrid is a parameter error, not a silent downgrade.
    #[test]
    fn search_flags_semantic_disabled_under_auto() {
        let path = temp_db("semantic-disabled");
        call(
            &path,
            "memory_create",
            json!({"summary": "note", "content": "searchable body"}),
        )
        .unwrap();

        let auto = call(&path, "memory_search", json!({"query": "searchable"})).unwrap();
        assert_eq!(auto["mode"], "keyword");
        assert_eq!(auto["semantic"], "disabled");
        assert!(auto.get("semantic_fallback").is_none());

        let keyword = call(
            &path,
            "memory_search",
            json!({"query": "searchable", "mode": "keyword"}),
        )
        .unwrap();
        assert_eq!(keyword["mode"], "keyword");
        assert!(keyword.get("semantic").is_none());

        let hybrid = call(
            &path,
            "memory_search",
            json!({"query": "searchable", "mode": "hybrid"}),
        );
        assert!(hybrid.is_err(), "unconfigured hybrid must be a hard error");

        cleanup(&path);
    }

    /// Cross-memory id references: freshly written text containing id-shaped tokens draws a
    /// warning note (non-blocking, word-bounded so fragments stay unnoticed); delete / update /
    /// merge report the memories whose mentions now dangle or drift, ids + summaries only.
    #[test]
    fn id_references_warn_on_write_and_report_inbound() {
        let path = temp_db("id-refs");

        // create: id-shaped text warns without blocking
        let warned = call(
            &path,
            "memory_create",
            json!({"summary": "alpha", "content": "see m2 for details"}),
        )
        .unwrap();
        assert_eq!(warned["memory"]["id"], "m1");
        let note = warned["note"].as_str().unwrap();
        assert!(
            note.contains("m2") && note.contains("unstable"),
            "got: {note}"
        );

        // Word-boundary discipline: fragments and case variants are not id references
        let quiet = call(
            &path,
            "memory_create",
            json!({"summary": "beta", "content": "cm3 connector and M4 bolt stay unnoticed"}),
        )
        .unwrap();
        assert!(quiet.get("note").is_none(), "got: {quiet:?}");

        call(
            &path,
            "memory_create",
            json!({"summary": "gamma", "content": "the referenced body"}),
        )
        .unwrap();

        // edit: id-like tokens in new_string warn too
        let edited = call(
            &path,
            "memory_edit",
            json!({
                "id": "m3",
                "old_string": "the referenced body",
                "new_string": "the referenced body (see m9)"
            }),
        )
        .unwrap();
        assert_eq!(edited["replaced"], 1);
        assert!(
            edited["note"].as_str().unwrap().contains("m9"),
            "got: {edited}"
        );

        // update replacing the content: inbound references are reported (content drift), summary-level only
        let updated = call(
            &path,
            "memory_update",
            json!({"id": "m2", "content": "rewritten body"}),
        )
        .unwrap();
        assert_eq!(updated["updated"], true);
        let refs = updated["referenced_by"].as_array().unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0]["id"], "m1");
        assert_eq!(refs[0]["to"], "m2");
        assert_eq!(refs[0]["summary"], "alpha");
        assert!(
            refs[0].get("content").is_none(),
            "inbound report must not leak content"
        );

        // Tag-only update: nothing drifted, no inbound report
        let tag_only = call(
            &path,
            "memory_update",
            json!({"id": "m2", "add_tags": ["solo"], "create_missing_tags": true}),
        )
        .unwrap();
        assert!(tag_only.get("referenced_by").is_none(), "got: {tag_only:?}");

        // delete: surviving memories mentioning the deleted id are reported
        let deleted = call(&path, "memory_delete", json!({"ids": ["m2"]})).unwrap();
        assert_eq!(deleted["deleted"], json!(["m2"]));
        let refs = deleted["referenced_by"].as_array().unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0]["id"], "m1");
        assert_eq!(refs[0]["to"], "m2");

        // merge: references to the absorbed id are reported — the target included, its merged
        // content may carry the mention
        call(
            &path,
            "memory_create",
            json!({"summary": "delta", "content": "filler"}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "epsilon", "content": "see m4"}),
        )
        .unwrap();
        let merged = call(
            &path,
            "memory_merge",
            json!({"target": "m5", "source": "m4"}),
        )
        .unwrap();
        assert_eq!(merged["removed"], "m4");
        let refs = merged["referenced_by"].as_array().unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0]["id"], "m5");
        assert_eq!(refs[0]["to"], "m4");
        assert!(
            merged["note"].as_str().unwrap().contains("absorbed id m4"),
            "got: {merged}"
        );

        cleanup(&path);
    }

    /// Tag set algebra (tag_expr): list and search narrow to memories whose tag set satisfies the
    /// expression; empty string counts as absent; syntax errors and unknown tags (with a
    /// did-you-mean hint) are Invalid; the expression ANDs with the other tag filters.
    #[test]
    fn tag_expr_filters_list_and_search() {
        let path = temp_db("tag-expr");
        for name in ["rust", "web", "notes"] {
            call(&path, "tag_create", json!({"name": name})).unwrap();
        }
        // m1: rust · m2: web · m3: rust+web · m4: notes
        call(
            &path,
            "memory_create",
            json!({"summary": "one", "content": "c", "tags": ["rust"]}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "two", "content": "c", "tags": ["web"]}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "three", "content": "c", "tags": ["rust", "web"]}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "four", "content": "c", "tags": ["notes"]}),
        )
        .unwrap();

        let ids = |out: &Value| -> Vec<String> {
            let mut got: Vec<String> = out["memories"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| m["id"].as_str().unwrap().to_string())
                .collect();
            got.sort();
            got
        };

        // The user-facing example: both rust and web, or notes → m3, m4
        let out = call(
            &path,
            "memory_list",
            json!({"tag_expr": "(rust&web)|notes"}),
        )
        .unwrap();
        assert_eq!(out["total"], 2);
        assert_eq!(ids(&out), vec!["m3".to_string(), "m4".to_string()]);
        // Negation is the complement within each memory's tag set → m2, m4
        let out = call(&path, "memory_list", json!({"tag_expr": "!rust"})).unwrap();
        assert_eq!(out["total"], 2);
        assert_eq!(ids(&out), vec!["m2".to_string(), "m4".to_string()]);
        // ANDs with the regex filter: rust ∩ web → m3 only
        let out = call(
            &path,
            "memory_list",
            json!({"tag_filter": "^rust$", "tag_expr": "web"}),
        )
        .unwrap();
        assert_eq!(out["total"], 1);
        assert_eq!(ids(&out), vec!["m3".to_string()]);
        // Empty string counts as absent
        let out = call(&path, "memory_list", json!({"tag_expr": "   "})).unwrap();
        assert_eq!(out["total"], 4);
        // Quoted names carry operators and whitespace (the tag exists, so no error)
        call(&path, "tag_create", json!({"name": "rust & life"})).unwrap();
        let out = call(&path, "memory_list", json!({"tag_expr": "'rust & life'"})).unwrap();
        assert_eq!(out["total"], 0);

        // Syntax errors surface as Invalid with position
        let err = call(&path, "memory_list", json!({"tag_expr": "(rust|web"})).unwrap_err();
        assert!(err.to_string().contains("missing ')'"), "got: {err}");
        // Unknown tag rejected with the did-you-mean hint (case-insensitive lookup)
        let err = call(&path, "memory_list", json!({"tag_expr": "Rust"})).unwrap_err();
        assert!(
            err.to_string().contains("did you mean 'rust'"),
            "got: {err}"
        );

        // search narrows its ranked candidates by the expression too: c hits all four bodies,
        // rust & !web keeps m1 only
        let out = call(
            &path,
            "memory_search",
            json!({"query": "c", "tag_expr": "rust&!web"}),
        )
        .unwrap();
        assert_eq!(out["total_matches"], 1);
        assert_eq!(out["results"][0]["id"], "m1");

        cleanup(&path);
    }

    /// Summary-only memories: content is optional on create (whitespace counts as none), memory_get
    /// reveals an empty body, search hits the summary with an empty snippet, update can add and
    /// clear the body, and edit on a bodyless memory fails without side effects.
    #[test]
    fn summary_only_memories() {
        let path = temp_db("summary-only");
        call(
            &path,
            "memory_create",
            json!({"summary": "只此一句摘要", "tags": ["t"], "create_missing_tags": true}),
        )
        .unwrap();
        let got = call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap();
        assert_eq!(got["memories"][0]["content"], "");
        // Search hits the summary; empty content yields an empty snippet (progressive disclosure holds)
        let found = call(&path, "memory_search", json!({"query": "一句"})).unwrap();
        assert_eq!(found["total_matches"], 1);
        assert_eq!(found["results"][0]["snippet"], "");
        // Explicit whitespace content also counts as no body
        call(
            &path,
            "memory_create",
            json!({"summary": "空白正文归空", "content": "   "}),
        )
        .unwrap();
        let got = call(&path, "memory_get", json!({"ids": ["m2"]})).unwrap();
        assert_eq!(got["memories"][0]["content"], "");

        // update adds a body, then clears it again (summary-only round trip)
        call(
            &path,
            "memory_update",
            json!({"id": "m1", "content": "现在有正文了"}),
        )
        .unwrap();
        let got = call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap();
        assert_eq!(got["memories"][0]["content"], "现在有正文了");
        call(
            &path,
            "memory_update",
            json!({"id": "m1", "content": "   "}),
        )
        .unwrap();
        let got = call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap();
        assert_eq!(got["memories"][0]["content"], "");

        // edit on a bodyless memory fails without side effects and points at memory_update
        let err = call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "x", "new_string": "y"}),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("old_string not found"),
            "got: {err}"
        );

        cleanup(&path);
    }

    /// The single-tag and tag-array filter parameters were folded into tag_expr (a leaf / an OR
    /// chain express both); the schema-derived unknown-parameter guard must now reject them so
    /// there is exactly one way to filter by tags.
    #[test]
    fn retired_tag_filter_params_are_rejected() {
        let path = temp_db("retired-params");
        let err = call(&path, "memory_list", json!({"tag": "rust"})).unwrap_err();
        assert!(err.to_string().contains("'tag'"), "got: {err}");
        let err = call(
            &path,
            "memory_search",
            json!({"query": "x", "tags": ["rust"]}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("'tags'"), "got: {err}");

        cleanup(&path);
    }

    #[test]
    fn tag_create_warns_on_case_collision() {
        let path = temp_db("case-collision");
        call(&path, "tag_create", json!({"name": "rust"})).unwrap();
        let second = call(&path, "tag_create", json!({"name": "Rust"})).unwrap();
        assert_eq!(second["similar_existing"], "rust");
        assert!(second["note"].as_str().unwrap().contains("tag_update"));
        // Non-blocking: both tags created successfully (plus the synthesized reserved row)
        let tl = call(&path, "tag_list", json!({})).unwrap();
        assert_eq!(tl["tags"].as_array().unwrap().len(), 3);

        cleanup(&path);
    }

    #[test]
    fn tag_update_supports_case_only_rename() {
        let path = temp_db("case-rename");
        call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": "c", "tags": ["rust"], "create_missing_tags": true}),
        )
        .unwrap();

        // Case-only renames are the designated fix path for the similar_existing hint and must work
        let r = call(
            &path,
            "tag_update",
            json!({"name": "rust", "new_name": "Rust"}),
        )
        .unwrap();
        assert_eq!(r["renamed"], true);
        let got = call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap();
        assert_eq!(got["memories"][0]["tags"][0], "Rust");

        // Description-only fill-in: without a rename, renamed=false and description_updated=true
        let d = call(
            &path,
            "tag_update",
            json!({"name": "Rust", "description": "the language"}),
        )
        .unwrap();
        assert_eq!(d["renamed"], false);
        assert_eq!(d["description_updated"], true);
        assert_eq!(d["tag"]["description"], "the language");

        // After a rename, creating any other case-spelled variant: the hint must trigger (exactly the anti-fragmentation scenario)
        let back = call(&path, "tag_create", json!({"name": "rust"})).unwrap();
        assert_eq!(back["similar_existing"], "Rust");
        let hint = call(&path, "tag_create", json!({"name": "RUST"})).unwrap();
        assert_eq!(hint["similar_existing"], "Rust");
        // While exact duplicates still error out
        assert!(call(&path, "tag_create", json!({"name": "Rust"})).is_err());

        cleanup(&path);
    }

    #[test]
    fn every_listed_tool_has_a_dispatch_arm() {
        // Sync guard: TOOL_NAMES and execute's match branches must correspond one-to-one.
        // If either side is missed (a tool added without updating the inventory, or an inventory entry without an implementation), this test fails.
        let path = temp_db("dispatch");
        for name in TOOL_NAMES {
            let r = execute_with_db(
                &path,
                &crate::auth::IdentityCtx::open_mode(),
                name,
                &json!({}),
            );
            if let Err(e) = r {
                assert!(
                    !e.to_string().contains("unknown tool"),
                    "tool '{name}' is listed but has no dispatch arm: {e}"
                );
            }
        }
        assert!(execute_with_db(
            &path,
            &crate::auth::IdentityCtx::open_mode(),
            "nonexistent",
            &json!({})
        )
        .unwrap_err()
        .to_string()
        .contains("unknown tool"));
        cleanup(&path);
    }

    /// Permissions are enforced centrally at the execute entry point: insufficient capability → Forbidden, and the request leaves no side effects.
    #[test]
    fn missing_capability_is_forbidden_before_execution() {
        use crate::auth::Cap;
        let path = temp_db("caps");

        // Read-only identity: read tools work, write tools are always Forbidden
        let read_caps = [Cap::Read];
        assert!(call_as(&path, &read_caps, "memory_list", json!({})).is_ok());
        let err = call_as(
            &path,
            &read_caps,
            "memory_create",
            json!({"summary": "s", "content": "c"}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)), "got: {err:?}");
        assert!(err.to_string().contains("'create'"), "got: {err}");
        assert!(call_as(&path, &read_caps, "tag_create", json!({"name": "t"})).is_err());
        // Forbidden must happen before any side effect: the database should have no new tag
        // (one row: the always-listed reserved placeholder, created by nobody)
        assert_eq!(
            call(&path, "tag_list", json!({})).unwrap()["tags"]
                .as_array()
                .unwrap()
                .len(),
            1
        );

        // create without delete: can write, cannot delete
        call_as(
            &path,
            &[Cap::Create],
            "memory_create",
            json!({"summary": "s", "content": "c"}),
        )
        .unwrap();
        let err = call_as(
            &path,
            &[Cap::Create],
            "memory_delete",
            json!({"ids": ["m1"]}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)));

        // Per-tool capability mapping: read tools must also be Forbidden to a "no capabilities at all" identity
        for name in TOOL_NAMES {
            let r = call_as(&path, &[], name, json!({}));
            let Err(e) = r else {
                continue; // allowed only for paths where empty arguments are valid and read-only (none currently exist)
            };
            assert!(
                matches!(e, ToolError::Forbidden(_)),
                "tool '{name}' with zero permissions must be Forbidden, got: {e}"
            );
        }

        cleanup(&path);
    }

    /// Reserved-tag rules for 'conventions': the tag itself can never be renamed or deleted
    /// (admins included); creating it and attaching/detaching it on memories requires admin.
    #[test]
    fn reserved_conventions_tag_is_guarded() {
        use crate::model::RESERVED_TAG;
        let path = temp_db("reserved-tag");

        // An agent without admin cannot create the reserved tag or attach it to memories
        let writer = [
            Cap::Read,
            Cap::Create,
            Cap::Update,
            Cap::Delete,
            Cap::TagManage,
        ];
        let err = call_as(&path, &writer, "tag_create", json!({"name": RESERVED_TAG})).unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)), "got: {err:?}");
        let err = call_as(
            &path,
            &writer,
            "memory_create",
            json!({"summary": "s", "content": "c", "tags": [RESERVED_TAG, "ok"]}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)), "got: {err:?}");
        // The attach message says "attaching", not "changing" — the caller referenced the tag on a
        // new memory, and a change wording would misdescribe the attempted action
        assert!(
            err.to_string().starts_with("attaching the reserved tag"),
            "got: {err}"
        );
        // Forbidden happens before any side effect: no tag, no memory (the one row is the
        // always-listed reserved placeholder, not a side effect)
        assert_eq!(
            call(&path, "tag_list", json!({})).unwrap()["tags"]
                .as_array()
                .unwrap()
                .len(),
            1
        );

        // Renaming/deleting the reserved tag is refused outright — even to a full admin
        let admin_caps = Cap::ALL;
        call_as(
            &path,
            &admin_caps,
            "tag_create",
            json!({"name": RESERVED_TAG}),
        )
        .unwrap();
        for tool in ["tag_update", "tag_delete"] {
            let mut args = json!({"name": RESERVED_TAG});
            if tool == "tag_update" {
                args["new_name"] = json!("free");
            }
            let err = call_as(&path, &admin_caps, tool, args).unwrap_err();
            assert!(matches!(err, ToolError::Invalid(_)), "got: {err:?}");
            assert!(err.to_string().contains("reserved"), "got: {err}");
        }
        // Renaming another tag ONTO the reserved name is refused as well
        call_as(&path, &admin_caps, "tag_create", json!({"name": "other"})).unwrap();
        let err = call_as(
            &path,
            &admin_caps,
            "tag_update",
            json!({"name": "other", "new_name": RESERVED_TAG}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Invalid(_)), "got: {err:?}");

        // Detaching via memory_update (remove_tags) also needs admin
        let created = call_as(
            &path,
            &admin_caps,
            "memory_create",
            json!({"summary": "rule", "content": "c", "tags": [RESERVED_TAG]}),
        )
        .unwrap();
        let id = created["memory"]["id"].as_str().unwrap().to_string();
        let err = call_as(
            &path,
            &writer,
            "memory_update",
            json!({"id": id, "remove_tags": [RESERVED_TAG]}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)), "got: {err:?}");
        // Whitespace around the name does not slip past the guard
        let err = call_as(
            &path,
            &writer,
            "memory_update",
            json!({"id": id, "add_tags": [format!("  {RESERVED_TAG} ")]}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)), "got: {err:?}");

        // Admin CAN write conventions memories and manage other tags freely
        let ok = call_as(
            &path,
            &admin_caps,
            "memory_update",
            json!({"id": id, "add_tags": ["notes"], "remove_tags": [RESERVED_TAG], "create_missing_tags": true}),
        )
        .unwrap();
        assert_eq!(ok["updated"], true);
        assert_eq!(ok["memory"]["tags"], json!(["notes"]), "admin detach works");

        cleanup(&path);
    }

    /// The reserved tag is discoverable through tag_list: flagged on its real row, and synthesized
    /// (zero memories, builtin description) before anyone created it — discoverability must not
    /// depend on having first crashed into a guard error.
    #[test]
    fn tag_list_always_surfaces_the_reserved_tag() {
        use crate::model::RESERVED_TAG;
        let path = temp_db("reserved-listed");

        let listed = call(&path, "tag_list", json!({})).unwrap();
        let rows = listed["tags"].as_array().unwrap();
        assert_eq!(rows.len(), 1, "only the synthesized reserved row");
        assert_eq!(rows[0]["name"], RESERVED_TAG);
        assert_eq!(rows[0]["reserved"], true);
        assert_eq!(rows[0]["count"], 0);
        assert!(
            rows[0]["description"]
                .as_str()
                .unwrap()
                .contains("Reserved"),
            "synthesized row carries the builtin description"
        );

        // The regex filter applies to the synthesized row like any other name
        let filtered = call(&path, "tag_list", json!({"filter": "^conv"})).unwrap();
        assert_eq!(filtered["tags"].as_array().unwrap().len(), 1);
        let filtered = call(&path, "tag_list", json!({"filter": "^nomatch"})).unwrap();
        assert_eq!(filtered["tags"].as_array().unwrap().len(), 0);

        // Once created, the real row replaces the placeholder (admin creates conventions + one other)
        let admin_caps = Cap::ALL;
        call_as(
            &path,
            &admin_caps,
            "tag_create",
            json!({"name": RESERVED_TAG, "description": "house rules"}),
        )
        .unwrap();
        call_as(&path, &admin_caps, "tag_create", json!({"name": "rust"})).unwrap();
        let listed = call(&path, "tag_list", json!({})).unwrap();
        let rows = listed["tags"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        let conv = rows
            .iter()
            .find(|t| t["name"] == RESERVED_TAG)
            .expect("conventions listed");
        assert_eq!(conv["reserved"], true);
        assert_eq!(conv["description"], "house rules", "real description wins");
        let other = rows.iter().find(|t| t["name"] == "rust").unwrap();
        // Sparse reserved flag: present (true) only on the reserved tag
        assert!(other.get("reserved").is_none());

        cleanup(&path);
    }

    /// Default-off auto-creation: unknown tags are rejected with the closest existing tags listed,
    /// so agents cannot fragment the taxonomy by casual tagging (case variants, plurals). The
    /// opt-in flag keeps the auto-create path available when it really is wanted.
    #[test]
    fn unknown_tags_rejected_unless_creation_is_explicit() {
        let path = temp_db("unknown-tags");
        call(&path, "tag_create", json!({"name": "meta"})).unwrap();

        // Unknown tag without the flag: rejected, the existing case variant is suggested
        let err = call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": "c", "tags": ["META"]}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Invalid(_)), "got: {err:?}");
        let msg = err.to_string();
        assert!(msg.contains("unknown tags: 'META'"), "got: {msg}");
        assert!(msg.contains("did you mean 'meta'"), "got: {msg}");
        assert!(
            msg.contains("tag_create") && msg.contains("create_missing_tags"),
            "got: {msg}"
        );
        // Nothing was created by the failed call
        assert_eq!(
            call(&path, "tag_list", json!({})).unwrap()["tags"]
                .as_array()
                .unwrap()
                .len(),
            2
        );

        // The same call with the flag opts into auto-creation
        let ok = call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": "c", "tags": ["META"], "create_missing_tags": true}),
        )
        .unwrap();
        assert_eq!(ok["tags_autocreated"], json!(["META"]));

        // memory_update's add_tags path follows the same rule
        let err = call(
            &path,
            "memory_update",
            json!({"id": "m1", "add_tags": ["totally-new"]}),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("unknown tags: 'totally-new'"),
            "got: {err}"
        );
        let ok = call(
            &path,
            "memory_update",
            json!({"id": "m1", "add_tags": ["totally-new"], "create_missing_tags": true}),
        )
        .unwrap();
        assert_eq!(ok["tags_autocreated"], json!(["totally-new"]));

        // Existing tags never need the flag
        call(
            &path,
            "memory_create",
            json!({"summary": "s2", "content": "c2", "tags": ["meta"]}),
        )
        .unwrap();

        cleanup(&path);
    }

    /// memory_merge closes the duplicate loop: the target keeps id and creation time, tags union,
    /// content appends (or is replaced), and the source disappears.
    #[test]
    fn memory_merge_absorbs_source_into_target() {
        let path = temp_db("merge");
        let first = call(
            &path,
            "memory_create",
            json!({"summary": "Deploy runbook", "content": "step one", "tags": ["ops"], "create_missing_tags": true}),
        )
        .unwrap();
        let target = first["memory"]["id"].as_str().unwrap().to_string();
        // The create echo is a summary view (no content, no created field): the creation time is
        // read back through the full view, which is exactly where the contract exposes it
        let target_created = call(&path, "memory_get", json!({"ids": [target]})).unwrap()
            ["memories"][0]["created"]
            .as_str()
            .unwrap()
            .to_string();
        let second = call(
            &path,
            "memory_create",
            json!({"summary": "deploy runbook v2", "content": "step two", "tags": ["ops", "release"], "create_missing_tags": true}),
        )
        .unwrap();
        let source = second["memory"]["id"].as_str().unwrap().to_string();

        let merged = call(
            &path,
            "memory_merge",
            json!({"target": target, "source": source}),
        )
        .unwrap();
        assert_eq!(merged["removed"], source.as_str());
        assert_eq!(merged["memory"]["id"], target.as_str());
        let after = call(&path, "memory_get", json!({"ids": [target]})).unwrap();
        assert_eq!(
            after["memories"][0]["created"], target_created,
            "creation time survives the merge"
        );
        assert_eq!(merged["content_appended"], true);
        let tags = merged["memory"]["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 2, "tags union: {merged}");

        // Content was appended, the source is gone
        let got = call(&path, "memory_get", json!({"ids": [target]})).unwrap();
        let content = got["memories"][0]["content"].as_str().unwrap();
        assert_eq!(content, "step one\n\nstep two");
        let gone = call(&path, "memory_get", json!({"ids": [source]})).unwrap();
        assert_eq!(gone["missing"].as_array().unwrap().len(), 1);

        // Explicit summary/content replace instead of append
        let third = call(
            &path,
            "memory_create",
            json!({"summary": "another duplicate", "content": "more"}),
        )
        .unwrap();
        let source2 = third["memory"]["id"].as_str().unwrap().to_string();
        let merged = call(
            &path,
            "memory_merge",
            json!({"target": target, "source": source2, "summary": "Deploy runbook (unified)", "content": "unified body"}),
        )
        .unwrap();
        assert_eq!(merged["content_appended"], false);
        assert_eq!(merged["memory"]["summary"], "Deploy runbook (unified)");
        let got = call(&path, "memory_get", json!({"ids": [target]})).unwrap();
        assert_eq!(got["memories"][0]["content"], "unified body");

        // Misuse: self-merge, malformed ids, missing ids
        let err = call(
            &path,
            "memory_merge",
            json!({"target": target, "source": target}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("into itself"), "got: {err}");
        let err = call(
            &path,
            "memory_merge",
            json!({"target": target, "source": "5"}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("malformed"), "got: {err}");
        let err = call(
            &path,
            "memory_merge",
            json!({"target": target, "source": "m999"}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::NotFound(_)), "got: {err:?}");

        cleanup(&path);
    }

    /// Merge spans two capability domains: update alone (or delete alone) must not reach it, and
    /// merging anything that carries the reserved tag needs admin on top.
    #[test]
    fn memory_merge_permission_boundaries() {
        use crate::model::RESERVED_TAG;
        let path = temp_db("merge-caps");
        let both = [Cap::Update, Cap::Delete];
        let mk = |summary: &str, tags: Value| {
            call(&path, "memory_create", json!({"summary": summary, "content": "c", "tags": tags, "create_missing_tags": true}))
                .unwrap()["memory"]["id"]
                .as_str()
                .unwrap()
                .to_string()
        };
        let a = mk("a", json!(["ops"]));
        let b = mk("b", json!(["ops"]));

        // Update-only cannot merge (it would smuggle a delete)
        let err = call_as(
            &path,
            &[Cap::Update],
            "memory_merge",
            json!({"target": a, "source": b}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)), "got: {err:?}");
        assert!(
            err.to_string().contains("'delete'"),
            "names the missing permission: {err}"
        );
        // Delete-only cannot either (it would smuggle an update)
        let err = call_as(
            &path,
            &[Cap::Delete],
            "memory_merge",
            json!({"target": a, "source": b}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("'update'"), "got: {err}");

        // Conventions memories need admin even with update + delete
        let admin = Cap::ALL;
        let conv = mk("rule", json!([RESERVED_TAG]));
        let other = mk("plain", json!([]));
        let err = call_as(
            &path,
            &both,
            "memory_merge",
            json!({"target": conv, "source": other}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)), "got: {err:?}");
        assert!(err.to_string().contains("reserved"), "got: {err}");
        let ok = call_as(
            &path,
            &admin,
            "memory_merge",
            json!({"target": conv, "source": other}),
        )
        .unwrap();
        assert_eq!(ok["removed"], other.as_str());

        cleanup(&path);
    }

    /// memory_edit replaces an exact span inside the content: single matches swap in place, the
    /// response stays sparse (no content echo — progressive disclosure), and an empty new_string
    /// deletes the span.
    #[test]
    fn memory_edit_replaces_exact_spans() {
        let path = temp_db("memory-edit");
        call(
            &path,
            "memory_create",
            json!({
                "summary": "Deploy runbook",
                "content": "step one: build\nstep two: test\nstep three: release",
                "tags": ["ops"],
                "create_missing_tags": true
            }),
        )
        .unwrap();

        let r = call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "step two: test", "new_string": "step two: full test suite"}),
        )
        .unwrap();
        assert_eq!(r["replaced"], 1);
        assert!(
            r.get("content").is_none(),
            "edit response must not echo the full content"
        );
        let got = call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap();
        assert_eq!(
            got["memories"][0]["content"],
            "step one: build\nstep two: full test suite\nstep three: release"
        );

        // Empty new_string = delete the span (byte-exact, newline included)
        call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "step one: build\n", "new_string": ""}),
        )
        .unwrap();
        let got = call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap();
        assert_eq!(
            got["memories"][0]["content"],
            "step two: full test suite\nstep three: release"
        );

        cleanup(&path);
    }

    /// Ambiguity discipline: 0 hits and multi-hits both fail without touching anything; the
    /// multi-hit error reports the count and offers replace_all, which then replaces every
    /// occurrence.
    #[test]
    fn memory_edit_ambiguity_is_rejected_until_resolved() {
        let path = temp_db("memory-edit-ambiguity");
        let original = "todo: a\ndone: b\ntodo: c";
        call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": original}),
        )
        .unwrap();

        // 0 hits (byte-exact, so a case change already misses), with a re-read hint
        let err = call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "TODO: a", "new_string": "x"}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Invalid(_)), "got: {err:?}");
        let msg = err.to_string();
        assert!(
            msg.contains("not found") && msg.contains("memory_get"),
            "got: {msg}"
        );

        // 2 hits without replace_all: rejected with the count, memory untouched
        let err = call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "todo:", "new_string": "x"}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("2 times"), "got: {err}");
        assert_eq!(
            call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap()["memories"][0]["content"],
            original
        );

        // replace_all replaces every occurrence and reports how many
        let r = call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "todo:", "new_string": "note:", "replace_all": true}),
        )
        .unwrap();
        assert_eq!(r["replaced"], 2);
        assert_eq!(
            call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap()["memories"][0]["content"],
            "note: a\ndone: b\nnote: c"
        );

        cleanup(&path);
    }

    /// No-op edits are rejected outright (identical strings, empty old_string), as are malformed
    /// ids and unknown memories — the same id discipline as memory_update.
    #[test]
    fn memory_edit_rejects_no_ops_and_bad_ids() {
        let path = temp_db("memory-edit-noop");
        call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": "hello world"}),
        )
        .unwrap();

        let err = call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "hello", "new_string": "hello"}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("identical"), "got: {err}");
        let err = call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "", "new_string": "x"}),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("old_string must not be empty"),
            "got: {err}"
        );
        let err = call(
            &path,
            "memory_edit",
            json!({"id": "12", "old_string": "hello", "new_string": "x"}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("malformed"), "got: {err}");
        let err = call(
            &path,
            "memory_edit",
            json!({"id": "m99", "old_string": "hello", "new_string": "x"}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::NotFound(_)), "got: {err:?}");

        cleanup(&path);
    }

    /// The content cap applies to the post-edit result: growing past MAX_CONTENT_CHARS fails the
    /// whole edit and the memory keeps its previous body.
    #[test]
    fn memory_edit_respects_the_content_cap() {
        let path = temp_db("memory-edit-cap");
        let base = "x".repeat(crate::model::MAX_CONTENT_CHARS - 1);
        call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": base}),
        )
        .unwrap();

        // replace_all sidesteps the uniqueness check so the size rejection is what fires
        let err = call(
            &path,
            "memory_edit",
            json!({"id": "m1", "old_string": "x", "new_string": "xxx", "replace_all": true}),
        )
        .unwrap_err();
        assert!(err.to_string().contains("too long"), "got: {err}");
        assert_eq!(
            call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap()["memories"][0]["content"],
            base,
            "the failed edit must not have touched anything"
        );

        cleanup(&path);
    }

    /// memory_edit sits in the update capability domain: a zero-permission identity is Forbidden
    /// before execution, an update-only identity can edit.
    #[test]
    fn memory_edit_permission_boundaries() {
        use crate::auth::Cap;
        let path = temp_db("memory-edit-caps");
        call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": "hello world"}),
        )
        .unwrap();

        let err = call_as(
            &path,
            &[],
            "memory_edit",
            json!({"id": "m1", "old_string": "hello", "new_string": "bye"}),
        )
        .unwrap_err();
        assert!(matches!(err, ToolError::Forbidden(_)), "got: {err:?}");
        assert!(err.to_string().contains("'update'"), "got: {err}");

        let ok = call_as(
            &path,
            &[Cap::Update],
            "memory_edit",
            json!({"id": "m1", "old_string": "hello", "new_string": "bye"}),
        )
        .unwrap();
        assert_eq!(ok["replaced"], 1);

        cleanup(&path);
    }

    /// Editing a conventions memory's body follows memory_update's rule: content rewrites need only
    /// the update permission (the reserved-tag guard covers attaching/detaching, not editing).
    #[test]
    fn memory_edit_on_conventions_content_needs_no_admin() {
        use crate::auth::Cap;
        use crate::model::RESERVED_TAG;
        let path = temp_db("memory-edit-conventions");
        let created = call_as(
            &path,
            &Cap::ALL,
            "memory_create",
            json!({"summary": "rule", "content": "always lint", "tags": [RESERVED_TAG], "create_missing_tags": true}),
        )
        .unwrap();
        let id = created["memory"]["id"].as_str().unwrap().to_string();

        let ok = call_as(
            &path,
            &[Cap::Update],
            "memory_edit",
            json!({"id": id, "old_string": "always lint", "new_string": "always lint before commit"}),
        )
        .unwrap();
        assert_eq!(ok["replaced"], 1);

        cleanup(&path);
    }

    #[test]
    fn duplicate_summary_is_flagged() {
        let path = temp_db("dup");
        call(
            &path,
            "memory_create",
            json!({"summary": "Rust notes", "content": "v1", "tags": []}),
        )
        .unwrap();
        // Same title after normalization (case-insensitive) → hint about the existing entry
        let second = call(
            &path,
            "memory_create",
            json!({"summary": "rust NOTES", "content": "v2", "tags": []}),
        )
        .unwrap();
        let dups = second["duplicate_of"].as_array().unwrap();
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0], "m1");
        // Sparse response: no duplicate → no duplicate_of key at all
        let third = call(
            &path,
            "memory_create",
            json!({"summary": "totally different", "content": "v3"}),
        )
        .unwrap();
        assert!(third.get("duplicate_of").is_none());
        cleanup(&path);
    }

    #[test]
    fn persistence_across_calls() {
        let path = temp_db("persist");
        let c = call(
            &path,
            "memory_create",
            json!({"summary": "first", "content": "one", "tags": []}),
        )
        .unwrap();
        let id = c["memory"]["id"].as_str().unwrap().to_string();
        // Every call reopens the database; the id counter must carry on
        let c2 = call(
            &path,
            "memory_create",
            json!({"summary": "second", "content": "two", "tags": []}),
        )
        .unwrap();
        assert_ne!(c2["memory"]["id"].as_str().unwrap(), id);
        cleanup(&path);
    }
}
