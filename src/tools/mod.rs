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

pub use defs::{tool_definitions, TOOL_NAMES};

pub const INSTRUCTIONS: &str = "Persistent long-term memory store. Each memory has: tags (a taxonomy YOU curate), a one-line summary, and full content. Progressive disclosure: memory_search / memory_list return only ids, tags and summaries; call memory_get on just the ids worth reading to reveal full content. Timestamps (created_at / updated_at / last_used_at) are epoch seconds in UTC and recorded automatically — never state creation time inside content. Write content as concise Markdown; avoid bold formatting. Save durable knowledge (decisions, facts, preferences, project context) with memory_create; write precise, self-contained summaries so future scans stay cheap; prefer memory_update over re-storing near-duplicates; keep tags tidy with the tag_* tools. The 'conventions' tag is reserved for operator-curated standing rules: those memories are the store's resident conventions (also exposed as memory:// resources) — read them before your first write and follow them. Access is permission-gated per caller identity: when a call fails with a permission error, report it to the user instead of retrying.";

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
    if name == "memory_create" || name == "memory_update" {
        crate::embed::after_write(db_path);
        if name == "memory_create" {
            crate::embed::dedup_hint(db_path, &mut out);
        }
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
        "tag_create" => tag_ops::tag_create(st, map),
        "tag_list" => tag_ops::tag_list(st, map),
        "tag_update" => tag_ops::tag_update(st, map),
        "tag_delete" => tag_ops::tag_delete(st, map),
        "memory_create" => memory_ops::memory_create(st, map),
        "memory_list" => memory_ops::memory_list(st, map),
        "memory_search" => memory_ops::memory_search(st, map),
        "memory_get" => memory_ops::memory_get(st, map),
        "memory_update" => memory_ops::memory_update(st, map),
        // The merge needs the identity for the data-dependent reserved-tag check (its arguments
        // name no tags; only the loaded memories reveal whether conventions is involved)
        "memory_merge" => memory_ops::memory_merge(st, ctx, map),
        "memory_delete" => memory_ops::memory_delete(st, map),
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
        "tag_create" => {
            if raw_tag(args, "name").as_deref() == Some(RESERVED_TAG) && !ctx.can(Cap::Admin) {
                return Err(ToolError::forbidden(format!(
                    "'{RESERVED_TAG}' is a reserved tag; creating it requires the 'admin' permission (it anchors the operator-curated conventions)"
                )));
            }
        }
        "tag_update" | "tag_delete" => {
            let mut targets = vec![raw_tag(args, "name")];
            if name == "tag_update" {
                targets.push(raw_tag(args, "new_name"));
            }
            if targets.into_iter().flatten().any(|t| t == RESERVED_TAG) {
                return Err(ToolError::invalid(format!(
                    "'{RESERVED_TAG}' is a reserved tag: it anchors the resident conventions and cannot be renamed or deleted"
                )));
            }
        }
        "memory_create" => {
            if involves_reserved(args, &["tags"]) && !ctx.can(Cap::Admin) {
                return Err(ToolError::forbidden(format!(
                    "attaching the reserved tag '{RESERVED_TAG}' to a memory requires the 'admin' permission (conventions are operator-curated)"
                )));
            }
        }
        "memory_update"
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
        assert_eq!(created["tags_reused"].as_array().unwrap().len(), 0);
        assert_eq!(
            created["tags_missing_description"]
                .as_array()
                .unwrap()
                .len(),
            0
        );

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
        assert_eq!(upd["tags_reused"], json!([]));
        assert_eq!(upd["tags_missing_description"], json!([]));

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
        assert_eq!(first["tags_missing_description"], json!([]));

        // Second call reuses everything; fresh was auto-created last time with an empty description → named hint
        let second = call(
            &path,
            "memory_create",
            json!({"summary": "b", "content": "cb", "tags": ["described", "fresh"], "create_missing_tags": true}),
        )
        .unwrap();
        assert_eq!(second["tags_autocreated"], json!([]));
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
        assert_eq!(del["missing"], json!([]));
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
        // Missing required argument
        assert!(call(&path, "memory_create", json!({"summary": "s"})).is_err());
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
        // Used together with an exact tag = AND
        let ml_and = call(
            &path,
            "memory_list",
            json!({"tag": "misc", "tag_filter": "^proj/"}),
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

    #[test]
    fn memory_list_reports_tag_state() {
        let path = temp_db("list-note");
        call(&path, "tag_create", json!({"name": "empty-tag"})).unwrap();
        // Tag does not exist
        let missing = call(&path, "memory_list", json!({"tag": "nope"})).unwrap();
        assert!(missing["note"].as_str().unwrap().contains("does not exist"));
        // Tag exists but is empty
        let empty = call(&path, "memory_list", json!({"tag": "empty-tag"})).unwrap();
        assert!(empty["note"].as_str().unwrap().contains("no memories"));

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

    /// memory_merge closes the duplicate loop: the target keeps id and created_at, tags union,
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
        let target_created = first["memory"]["created_at"].as_u64().unwrap();
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
        assert_eq!(merged["merged"], true);
        assert_eq!(merged["removed"], source.as_str());
        assert_eq!(merged["memory"]["id"], target.as_str());
        assert_eq!(
            merged["memory"]["created_at"], target_created,
            "created_at survives the merge"
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
        assert_eq!(ok["merged"], true);

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
        // Empty array when there is no duplicate
        let third = call(
            &path,
            "memory_create",
            json!({"summary": "totally different", "content": "v3"}),
        )
        .unwrap();
        assert_eq!(third["duplicate_of"].as_array().unwrap().len(), 0);
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
