//! MCP resources: the memory store exposed as `memory://` resources (custom scheme, RFC 3986).
//!
//! Two resource templates:
//! - `memory://tags/{tag}` — one tag's catalog as JSON: tag metadata plus the 100 most recently
//!   updated memory summaries. A directory-style resource: it never carries memory content.
//! - `memory://memories/{id}` — one memory's full content as Markdown (progressive disclosure
//!   level 2); the only resource that reveals a body.
//!
//! `resources/list` walks the catalog: one entry per tag plus one entry per resident convention
//! memory (the reserved `conventions` tag), paged 50 at a time with an opaque base64 offset cursor.
//!
//! Authorization uses a filtering-mask semantic, as the spec allows resource sets to vary by
//! authorization: a caller without the read capability sees an empty list, and reads answer the
//! regular "not found" invalid-params error (never a permission error that would reveal existence).

use crate::auth::{Cap, IdentityCtx};
use crate::model::{normalize_tag_name, Memory};
use crate::store::{self, ListFilter, Store, TxMode};
use crate::util;
use serde_json::{json, Value};
use std::path::Path;

/// The reserved tag whose memories surface as individual resources in `resources/list`
/// (shared with the tool layer's reserved-tag guards).
pub use crate::model::RESERVED_TAG as CONVENTIONS_TAG;

/// Entries per `resources/list` page.
const LIST_PAGE_SIZE: usize = 50;
/// Cap on the summaries embedded in one tag resource (bigger catalogs go through memory_list).
const TAG_RESOURCE_LIMIT: u64 = 100;

/// Cache hints for the catalog read paths: contents depend on the caller's authorization → private.
/// The list changes with writes (list_changed notifications invalidate it), so a short TTL suffices;
/// a single read is always best fetched fresh.
const LIST_CACHE: (i64, &str) = (30_000, "private");
const READ_CACHE: (i64, &str) = (0, "private");
const TEMPLATES_CACHE: (i64, &str) = (300_000, "public");

/// A resource request failure, surfaced as JSON-RPC -32602 with structured `data`.
#[derive(Debug)]
pub struct ResourceError {
    pub message: String,
    pub data: Value,
}

impl ResourceError {
    fn invalid(message: impl Into<String>, data: Value) -> Self {
        ResourceError {
            message: message.into(),
            data,
        }
    }

    /// A target that does not exist (masking permission gaps too): the spec mandates -32602 with
    /// the requested uri and forbids answering with empty contents.
    fn not_found(uri: &str) -> Self {
        ResourceError {
            message: format!("resource not found: {uri}"),
            data: json!({ "uri": uri }),
        }
    }

    /// Attach the requested uri to the error's data: every read error carries it, but store-layer
    /// strings arrive via `From<String>` without one.
    fn with_uri(mut self, uri: &str) -> Self {
        if self.data.get("uri").is_none() {
            self.data["uri"] = json!(uri);
        }
        self
    }
}

impl From<String> for ResourceError {
    /// Store / infrastructure errors arrive as plain strings with no kind information and classify
    /// as invalid. Existence is never inferred from error text: read paths decide it with
    /// dedicated queries and construct `not_found` explicitly.
    fn from(message: String) -> Self {
        ResourceError::invalid(message, json!({}))
    }
}

type ResourceResult = Result<Value, ResourceError>;

/// `resources/list`: every tag plus every resident convention memory, paged.
pub fn list(store_path: &Path, ctx: &IdentityCtx, cursor: Option<&str>) -> ResourceResult {
    let offset = decode_cursor(cursor)?;
    if !ctx.can(Cap::Read) {
        // Filtering mask: no read capability, no catalog (not an error — the set simply is empty).
        return Ok(paginate(Vec::new(), offset));
    }
    let entries = store::with_db_in(
        store_path,
        TxMode::ReadOnly,
        |st| -> Result<Vec<Value>, ResourceError> {
            let mut entries: Vec<Value> = st
                .tag_views()?
                .into_iter()
                .filter_map(|t| tag_entry(&t))
                .collect();
            // Resident conventions surface individually so hosts can pin and subscribe to them.
            // A big-but-safe page bound: conventions memories are few by design (resident rules),
            // and SQLite treats a negative LIMIT as unbounded, which `u64::MAX as i64` would yield.
            let (_, conventions) = st.list_memories(
                ListFilter {
                    tag: Some(CONVENTIONS_TAG),
                    ..Default::default()
                },
                "updated_at",
                false,
                0,
                1_000_000,
            )?;
            let conv: Vec<Value> = conventions.iter().filter_map(memory_entry).collect();
            entries.extend(conv);
            Ok(entries)
        },
    )?;
    Ok(paginate(entries, offset))
}

/// `resources/templates/list`: the two `memory://` templates (static, identical for every caller).
pub fn templates_list() -> Value {
    let (ttl, scope) = TEMPLATES_CACHE;
    json!({
        "resultType": "complete",
        "resourceTemplates": [
            {
                "uriTemplate": "memory://tags/{tag}",
                "name": "Tag",
                "description": "Catalog of one tag as JSON: tag metadata plus the 100 most recently updated memory summaries (no full content; use memory_get / memory_search for bodies).",
                "mimeType": "application/json",
            },
            {
                "uriTemplate": "memory://memories/{id}",
                "name": "Memory",
                "description": "Full content of one memory as Markdown (progressive disclosure level 2).",
                "mimeType": "text/markdown",
            },
        ],
        "ttlMs": ttl,
        "cacheScope": scope,
    })
}

/// `resources/read`: dispatch on the URI shape. Only `memory://memories/{id}` carries content;
/// directory reads return the tag's JSON catalog.
pub fn read(store_path: &Path, ctx: &IdentityCtx, uri: &str) -> ResourceResult {
    if !ctx.can(Cap::Read) {
        // Masking: a caller without read permission learns nothing, not even existence.
        return Err(ResourceError::not_found(uri));
    }
    match parse_uri(uri).map_err(|m| ResourceError::invalid(m, json!({ "uri": uri })))? {
        Target::Tag(name) => read_tag(store_path, uri, &name),
        Target::Memory(raw_id) => read_memory(store_path, uri, &raw_id),
    }
}

// ---------------------------------------------------------------- URI handling

enum Target {
    /// Tag name after percent-decoding.
    Tag(String),
    /// Raw memory id ("m3").
    Memory(String),
}

/// Strict `memory://` URI parsing: `memory://tags/{percent-encoded tag}` or `memory://memories/{id}`.
/// One path segment per kind; no query or fragment components.
fn parse_uri(uri: &str) -> Result<Target, String> {
    let rest = uri
        .strip_prefix("memory://")
        .ok_or("unsupported resource uri scheme (expected memory://)")?;
    if rest.contains('?') || rest.contains('#') {
        return Err("resource uri must not contain query or fragment components".into());
    }
    let (kind, value) = rest
        .split_once('/')
        .ok_or("resource uri must be memory://tags/{tag} or memory://memories/{id}")?;
    if value.is_empty() {
        return Err("resource uri path segment is empty".into());
    }
    match kind {
        "tags" => {
            let name = util::percent_decode_strict(value)
                .map_err(|_| "resource uri contains a malformed percent-encoding".to_string())?;
            let name = normalize_tag_name(&name)
                .map_err(|e| format!("invalid tag in resource uri: {e}"))?;
            Ok(Target::Tag(name))
        }
        "memories" => {
            let id = util::percent_decode_strict(value)
                .map_err(|_| "resource uri contains a malformed percent-encoding".to_string())?;
            Ok(Target::Memory(id))
        }
        other => Err(format!(
            "unknown resource kind '/{other}/' (expected tags/ or memories/)"
        )),
    }
}

/// Percent-encode a tag name into one URI path segment: RFC 3986 unreserved characters pass
/// through, every other byte becomes %XX.
fn encode_uri_segment(name: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(name.len());
    for byte in name.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => {
                let _ = write!(out, "%{byte:02X}");
            }
        }
    }
    out
}

// ---------------------------------------------------------------- Reads

fn read_tag(store_path: &Path, uri: &str, name: &str) -> ResourceResult {
    let doc = store::with_db_in(store_path, TxMode::ReadOnly, |st| -> Result<Value, ResourceError> {
        // Existence is a dedicated query, not an error-text guess: a missing tag is a structured
        // not-found, everything the store reports as a string stays a plain invalid error
        if !st.tag_exists(name)? {
            return Err(ResourceError::not_found(uri));
        }
        let view = st.tag_view(name)?;
        let (_, page) = st.list_memories(
            ListFilter {
                tag: Some(name),
                ..Default::default()
            },
            "updated_at",
            false,
            0,
            TAG_RESOURCE_LIMIT,
        )?;
        let memories: Vec<Value> = page.iter().map(Memory::summary_view).collect();
        let memory_count = view["count"].clone();
        let mut doc = json!({
            // Directory resource: metadata + summaries only, never content (progressive disclosure)
            "name": view["name"],
            "count": memory_count,
            "memories": memories,
        });
        // The sparse tag view omits description when empty; mirror that here
        if let Some(description) = view["description"].as_str() {
            doc["description"] = json!(description);
        }
        if page.len() < memory_count.as_u64().unwrap_or(0) as usize {
            doc["note"] = json!(format!(
                "showing the {TAG_RESOURCE_LIMIT} most recently updated; use memory_list with tag '{name}' to page through the rest"
            ));
        }
        Ok(doc)
    })
    .map_err(|e| e.with_uri(uri))?;
    Ok(read_result(
        uri,
        "application/json",
        &serde_json::to_string(&doc).unwrap_or_else(|_| "{}".to_string()),
        None,
    ))
}

fn read_memory(store_path: &Path, uri: &str, raw_id: &str) -> ResourceResult {
    let Some(id) = Store::parse_id(raw_id) else {
        return Err(ResourceError::invalid(
            format!("malformed memory id '{raw_id}' in resource uri: ids look like 'm123'"),
            json!({ "uri": uri }),
        ));
    };
    let (mut found, _) =
        store::with_db_in(store_path, TxMode::ReadOnly, |st| st.get_memories(&[id]))
            .map_err(ResourceError::from)
            .map_err(|e| e.with_uri(uri))?;
    let memory = found.pop().ok_or_else(|| ResourceError::not_found(uri))?;
    Ok(read_result(
        uri,
        "text/markdown",
        &memory.content,
        Some(util::format_utc_iso(memory.updated_at)),
    ))
}

/// The `resources/read` result envelope: one content block plus the private, immediately-stale
/// cache hint (a read is per-identity and cheap to re-fetch).
fn read_result(uri: &str, mime: &str, text: &str, last_modified: Option<String>) -> Value {
    let mut content = json!({
        "uri": uri,
        "mimeType": mime,
        "text": text,
    });
    if let Some(at) = last_modified {
        content["annotations"] = json!({ "lastModified": at });
    }
    let (ttl, scope) = READ_CACHE;
    json!({
        "resultType": "complete",
        "contents": [content],
        "ttlMs": ttl,
        "cacheScope": scope,
    })
}

// ---------------------------------------------------------------- List assembly

/// The canonical URI of a tag resource (used by the catalog and by the notification hooks).
pub(crate) fn tag_resource_uri(name: &str) -> String {
    format!("memory://tags/{}", encode_uri_segment(name))
}

/// The canonical URI of a memory resource (ids are already URI-safe).
pub(crate) fn memory_resource_uri(id: &str) -> String {
    format!("memory://memories/{id}")
}

fn tag_entry(view: &Value) -> Option<Value> {
    let name = view["name"].as_str()?;
    let mut entry = json!({
        "uri": tag_resource_uri(name),
        "name": name,
        "mimeType": "application/json",
    });
    if let Some(description) = view["description"].as_str() {
        if !description.is_empty() {
            entry["description"] = json!(description);
        }
    }
    Some(entry)
}

fn memory_entry(memory: &Memory) -> Option<Value> {
    Some(json!({
        "uri": memory_resource_uri(&memory.id),
        "name": memory.summary,
        "mimeType": "text/markdown",
        "annotations": {
            // Resident conventions are directed at agents (hosts may pin them)
            "audience": ["assistant"],
            "priority": 1.0,
        },
    }))
}

fn paginate(mut entries: Vec<Value>, offset: usize) -> Value {
    let total = entries.len();
    let page: Vec<Value> = if offset >= total {
        entries.clear();
        Vec::new()
    } else {
        entries.drain(..offset);
        entries.into_iter().take(LIST_PAGE_SIZE).collect()
    };
    let mut result = json!({
        "resultType": "complete",
        "resources": page,
    });
    let next_offset = offset + LIST_PAGE_SIZE;
    if next_offset < total {
        result["nextCursor"] = json!(encode_cursor(next_offset));
    }
    let (ttl, scope) = LIST_CACHE;
    result["ttlMs"] = json!(ttl);
    result["cacheScope"] = json!(scope);
    result
}

/// Pagination cursors are the base64 of a decimal offset — opaque to clients, trivially checkable
/// on the server (a tampered or stale cursor simply errors instead of mispaging).
fn encode_cursor(offset: usize) -> String {
    util::base64_encode(offset.to_string().as_bytes())
}

fn decode_cursor(cursor: Option<&str>) -> Result<usize, ResourceError> {
    let Some(raw) = cursor else {
        return Ok(0);
    };
    let decoded = util::base64_decode(raw)
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .and_then(|text| text.parse::<usize>().ok());
    decoded.ok_or_else(|| {
        ResourceError::invalid("malformed pagination cursor", json!({ "cursor": raw }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;
    use std::path::PathBuf;

    fn temp_db(tag: &str) -> PathBuf {
        crate::store::test_support::temp_db(&format!("res-{tag}"))
    }

    fn cleanup(path: &Path) {
        crate::store::test_support::cleanup(path);
    }

    fn open_ctx() -> IdentityCtx {
        IdentityCtx::open_mode()
    }

    /// Identity without the read capability (the masking case).
    fn no_read_ctx() -> IdentityCtx {
        let perms = crate::auth::Permissions::from_json(&json!({"create": true})).unwrap();
        IdentityCtx::new("writer", perms)
    }

    fn seed(path: &Path) {
        store::with_db_in(path, TxMode::Write, |st| -> Result<(), String> {
            st.tag_create("rust", "Rust language")?;
            st.tag_create("项目", "")?;
            let rust_ids = st.link_tags(&["rust".into()], true)?.ids;
            st.insert_memory(
                "Borrow checker",
                "The borrow checker forbids aliasing + mutation.",
                &rust_ids,
                10,
                20,
            )?;
            let conv_ids = st.link_tags(&[CONVENTIONS_TAG.into()], true)?.ids;
            st.insert_memory(
                "Commit rules",
                "Summary in one line; tags lowercase.",
                &conv_ids,
                30,
                40,
            )?;
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn parse_uri_accepts_and_rejects_shapes() {
        assert!(matches!(parse_uri("memory://tags/rust"), Ok(Target::Tag(n)) if n == "rust"));
        assert!(matches!(parse_uri("memory://memories/m12"), Ok(Target::Memory(i)) if i == "m12"));
        // Percent-decoding restores the tag name; UTF-8 included
        assert!(
            matches!(parse_uri("memory://tags/%E9%A1%B9%E7%9B%AE"), Ok(Target::Tag(n)) if n == "项目")
        );
        // Encoded slash stays one segment
        assert!(
            matches!(parse_uri("memory://tags/proj%2Falpha"), Ok(Target::Tag(n)) if n == "proj/alpha")
        );
        // Invalid shapes
        for bad in [
            "file:///etc/passwd",
            "memory://",
            "memory://tags",
            "memory://tags/",
            "memory://memories/",
            "memory://other/x",
            "memory://tags/rust?x=1",
            "memory://tags/rust#frag",
            "memory://tags/%ZZ",
            "memory://tags/%E9%A1", // truncated escape
        ] {
            assert!(parse_uri(bad).is_err(), "must reject: {bad}");
        }
        // A tag name that cannot normalize (empty after trim) is rejected
        assert!(parse_uri("memory://tags/%20%20").is_err());
    }

    #[test]
    fn encode_uri_segment_roundtrip() {
        for name in ["rust", "proj/alpha", "项目 记忆", "a b&c?d#e", "100%"] {
            let encoded = encode_uri_segment(name);
            let decoded = util::percent_decode_strict(&encoded).unwrap();
            assert_eq!(decoded, name, "roundtrip for {name}");
            assert!(!encoded.contains('?') && !encoded.contains('#'));
        }
        assert_eq!(encode_uri_segment("proj/alpha"), "proj%2Falpha");
    }

    #[test]
    fn read_tag_returns_catalog_without_content() {
        let path = temp_db("read-tag");
        cleanup(&path);
        seed(&path);
        let uri = "memory://tags/rust";
        let result = read(&path, &open_ctx(), uri).unwrap();
        assert_eq!(result["resultType"], "complete");
        assert_eq!(result["ttlMs"], 0);
        assert_eq!(result["cacheScope"], "private");
        let content = &result["contents"][0];
        assert_eq!(content["uri"], uri);
        assert_eq!(content["mimeType"], "application/json");
        let doc: Value = serde_json::from_str(content["text"].as_str().unwrap()).unwrap();
        assert_eq!(doc["name"], "rust");
        assert_eq!(doc["description"], "Rust language");
        assert_eq!(doc["count"], 1);
        assert_eq!(doc["memories"][0]["summary"], "Borrow checker");
        // Progressive disclosure: the directory carries no memory content
        assert!(
            !content["text"]
                .as_str()
                .unwrap()
                .contains("borrow checker forbids"),
            "tag resources must not leak content"
        );
        assert!(doc["memories"][0].get("content").is_none());
        cleanup(&path);
    }

    #[test]
    fn read_memory_returns_content_with_last_modified() {
        let path = temp_db("read-memory");
        cleanup(&path);
        seed(&path);
        let uri = "memory://memories/m1";
        let result = read(&path, &open_ctx(), uri).unwrap();
        let content = &result["contents"][0];
        assert_eq!(content["uri"], uri);
        assert_eq!(content["mimeType"], "text/markdown");
        assert_eq!(
            content["text"],
            "The borrow checker forbids aliasing + mutation."
        );
        assert_eq!(
            content["annotations"]["lastModified"],
            "1970-01-01T00:00:20Z"
        );

        // Missing target: -32602 style error carrying the uri (never empty contents)
        let err = read(&path, &open_ctx(), "memory://memories/m99").unwrap_err();
        assert!(err.message.contains("not found"), "{}", err.message);
        assert_eq!(err.data["uri"], "memory://memories/m99");

        // Malformed id
        let err = read(&path, &open_ctx(), "memory://memories/102").unwrap_err();
        assert!(err.message.contains("malformed"), "{}", err.message);
        assert_eq!(err.data["uri"], "memory://memories/102");

        // Unknown tag: classified structurally (dedicated existence query), not by error text
        let err = read(&path, &open_ctx(), "memory://tags/nope").unwrap_err();
        assert!(err.message.contains("not found"), "{}", err.message);
        assert_eq!(err.data["uri"], "memory://tags/nope");
        cleanup(&path);
    }

    #[test]
    fn list_walks_tags_and_conventions_with_pagination() {
        let path = temp_db("list");
        cleanup(&path);
        seed(&path);
        let first = list(&path, &open_ctx(), None).unwrap();
        assert_eq!(first["resultType"], "complete");
        assert_eq!(first["cacheScope"], "private");
        // link_tags auto-created the conventions tag, so: 3 tags + 1 conventions memory
        let resources = first["resources"].as_array().unwrap();
        assert_eq!(resources.len(), 4);
        assert_eq!(first.get("nextCursor"), None);
        assert!(
            resources.iter().any(|r| r["uri"] == "memory://tags/rust"),
            "{first}"
        );
        assert!(
            resources
                .iter()
                .any(|r| r["uri"] == "memory://tags/%E9%A1%B9%E7%9B%AE"),
            "tag uris are percent-encoded: {first}"
        );
        let conv = resources
            .iter()
            .find(|r| r["uri"] == "memory://memories/m2")
            .unwrap();
        assert_eq!(conv["name"], "Commit rules");
        assert_eq!(conv["annotations"]["priority"], 1.0);
        assert_eq!(conv["annotations"]["audience"], json!(["assistant"]));
        // Catalog entries never carry content
        assert!(conv.get("text").is_none());

        // Page size honored: seed more conventions memories than one page holds
        store::with_db_in(&path, TxMode::Write, |st| -> Result<(), String> {
            let ids = st.link_tags(&[CONVENTIONS_TAG.into()], true)?.ids;
            for i in 0..60 {
                st.insert_memory(&format!("conv {i}"), "body", &ids, i as u64, i as u64)?;
            }
            Ok(())
        })
        .unwrap();
        let page1 = list(&path, &open_ctx(), None).unwrap();
        let items = page1["resources"].as_array().unwrap();
        assert_eq!(items.len(), LIST_PAGE_SIZE);
        let cursor = page1["nextCursor"].as_str().unwrap().to_string();
        let page2 = list(&path, &open_ctx(), Some(&cursor)).unwrap();
        let items2 = page2["resources"].as_array().unwrap();
        assert_eq!(items2.len(), 3 + 61 - LIST_PAGE_SIZE);
        let uris1: Vec<&str> = items.iter().filter_map(|r| r["uri"].as_str()).collect();
        let uris2: Vec<&str> = items2.iter().filter_map(|r| r["uri"].as_str()).collect();
        assert!(
            uris1.iter().all(|u| !uris2.contains(u)),
            "pages must not overlap"
        );

        // Invalid cursor
        let err = list(&path, &open_ctx(), Some("!!!")).unwrap_err();
        assert!(err.message.contains("cursor"), "{}", err.message);
        cleanup(&path);
    }

    #[test]
    fn templates_are_static_and_public() {
        let t = templates_list();
        assert_eq!(t["resultType"], "complete");
        assert_eq!(t["cacheScope"], "public");
        assert_eq!(t["ttlMs"], 300_000);
        let templates = t["resourceTemplates"].as_array().unwrap();
        assert_eq!(templates[0]["uriTemplate"], "memory://tags/{tag}",);
        assert_eq!(templates[1]["uriTemplate"], "memory://memories/{id}");
    }

    #[test]
    fn no_read_capability_masks_catalog_and_reads() {
        let path = temp_db("mask");
        cleanup(&path);
        seed(&path);
        let writer = no_read_ctx();
        // List: empty set instead of an error (resource sets may vary by authorization)
        let empty = list(&path, &writer, None).unwrap();
        assert_eq!(empty["resources"].as_array().unwrap().len(), 0);
        assert_eq!(empty.get("nextCursor"), None);
        // Read: the regular not-found error, no existence leak
        let err = read(&path, &writer, "memory://tags/rust").unwrap_err();
        assert!(err.message.contains("not found"), "{}", err.message);
        cleanup(&path);
    }
}
