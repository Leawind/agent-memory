//! MCP protocol layer: JSON-RPC 2.0 over Streamable HTTP, modern protocol revision 2026-07-28.
//!
//! The modern protocol is fully stateless: there is no initialize handshake, no version negotiation
//! and no batch messages. Every request carries its protocol version and client capabilities in
//! `params._meta`; the transport mirrors selected body fields into HTTP headers (MCP-Protocol-Version /
//! Mcp-Method / Mcp-Name), which are validated against the body here — the body is the source of truth.
//! `server/discover` replaces initialize as the discovery entry point. Legacy clients are rejected with
//! recognizable modern errors so dual-era clients can identify the server era and fail fast.
//!
//! Tool execution errors come back as isError results, while protocol-level errors come back as JSON-RPC
//! errors. An unknown RPC method answers HTTP 404 + -32601 (distinguishing it from a missing endpoint);
//! every other JSON-RPC error rides on HTTP 200. Notifications have no response (HTTP 202).

use crate::auth::IdentityCtx;
use crate::store;
use crate::tools;
use serde_json::{json, Value};
use std::path::Path;

/// The only protocol revision this server speaks (modern protocol: per-request `_meta`, no handshake).
pub const PROTOCOL_VERSION: &str = "2026-07-28";

const META_PROTOCOL_VERSION: &str = "io.modelcontextprotocol/protocolVersion";
const META_CLIENT_CAPABILITIES: &str = "io.modelcontextprotocol/clientCapabilities";

/// Caching hints for `tools/list` (`ttlMs`, `cacheScope`): the tool surface is static (tools never
/// change at runtime), so the list is identical for every caller and cacheable for a long time.
const TOOLS_LIST_CACHE: (i64, &str) = (3_600_000, "public");

/// Mirrored request headers collected by the transport ("request metadata" in the Streamable HTTP spec),
/// validated against the request body at the protocol layer.
pub struct TransportHeaders {
    pub protocol_version: Option<String>,
    pub method: Option<String>,
    pub name: Option<String>,
}

/// What the transport must do with one inbound message.
pub enum Outcome {
    /// A JSON-RPC response plus the HTTP status it must ride on: 200 for results and tool-level
    /// errors, 400 for malformed requests, 404 for unknown methods.
    Reply { status: u16, value: Value },
    /// A notification: accepted with no response body (HTTP 202).
    Accepted,
}

pub fn error_value(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn error_with_data(id: &Value, code: i64, message: &str, data: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message, "data": data}})
}

fn ok_value(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn reply(status: u16, value: Value) -> Outcome {
    Outcome::Reply { status, value }
}

/// Handle one inbound message (the body of POST /mcp). `ctx` is the caller identity already resolved
/// by the HTTP layer (open mode = full capabilities). Returns what the transport should send back.
pub fn handle(
    store_path: &Path,
    ctx: &IdentityCtx,
    headers: &TransportHeaders,
    msg: &Value,
) -> Outcome {
    // The modern protocol dropped batch messages: exactly one JSON-RPC message per POST.
    if msg.is_array() {
        return reply(
            400,
            error_value(
                &Value::Null,
                -32600,
                "invalid request: batch messages are not supported; send exactly one JSON-RPC message per POST",
            ),
        );
    }
    let Some(obj) = msg.as_object() else {
        return reply(
            400,
            error_value(
                &Value::Null,
                -32600,
                "invalid request: the body must be a single JSON-RPC message",
            ),
        );
    };

    let id = match obj.get("id") {
        // No id = notification: accept with no response (202). Header requirements for notification
        // POSTs are not defined by the spec, so none are enforced.
        None => {
            return match obj.get("method").and_then(Value::as_str) {
                Some(_) => Outcome::Accepted,
                None => reply(
                    400,
                    error_value(
                        &Value::Null,
                        -32600,
                        "invalid request: 'method' is missing or not a string",
                    ),
                ),
            };
        }
        Some(v) if v.is_string() || v.is_number() => v.clone(),
        // Unlike base JSON-RPC, the modern protocol forbids a null id; other types are equally malformed.
        Some(_) => {
            return reply(
                400,
                error_value(
                    &Value::Null,
                    -32600,
                    "invalid request: 'id' must be a string or a number",
                ),
            );
        }
    };

    let Some(method) = obj.get("method").and_then(Value::as_str) else {
        return reply(
            400,
            error_value(
                &id,
                -32600,
                "invalid request: 'method' is missing or not a string",
            ),
        );
    };

    // Legacy clients get the definitive modern error naming the supported versions (the spec asks
    // servers to name them in any error returned to an initialize request).
    if method == "initialize" {
        return reply(400, unsupported_version_error(&id, None));
    }

    // Missing or null params are both treated as an empty object (JSON-RPC allows omitting params).
    let params = match obj.get("params") {
        Some(v) if !v.is_null() => v.clone(),
        _ => json!({}),
    };

    // Required per-request _meta fields (the stateless replacement for the initialize handshake).
    let meta = params.get("_meta").and_then(Value::as_object);
    let Some(meta) = meta else {
        return reply(
            400,
            error_value(
                &id,
                -32602,
                "invalid params: params._meta is required (io.modelcontextprotocol/protocolVersion and io.modelcontextprotocol/clientCapabilities)",
            ),
        );
    };
    let Some(pv) = meta.get(META_PROTOCOL_VERSION).and_then(Value::as_str) else {
        return reply(
            400,
            error_value(
                &id,
                -32602,
                "invalid params: _meta.io.modelcontextprotocol/protocolVersion must be a string",
            ),
        );
    };
    if !meta
        .get(META_CLIENT_CAPABILITIES)
        .is_some_and(Value::is_object)
    {
        return reply(
            400,
            error_value(
                &id,
                -32602,
                "invalid params: _meta.io.modelcontextprotocol/clientCapabilities must be an object",
            ),
        );
    }

    // Header mirroring: MCP-Protocol-Version is required and must agree with the body (the body wins
    // on disagreement — the mismatch itself is the client bug).
    let Some(header_pv) = headers.protocol_version.as_deref() else {
        return reply(
            400,
            error_value(
                &id,
                -32020,
                "Header mismatch: the MCP-Protocol-Version header is required",
            ),
        );
    };
    if header_pv != pv {
        return reply(
            400,
            error_value(
                &id,
                -32020,
                &format!("Header mismatch: MCP-Protocol-Version header value '{header_pv}' does not match body value '{pv}'"),
            ),
        );
    }
    if pv != PROTOCOL_VERSION {
        return reply(400, unsupported_version_error(&id, Some(pv)));
    }

    let Some(header_method) = headers.method.as_deref() else {
        return reply(
            400,
            error_value(
                &id,
                -32020,
                "Header mismatch: the Mcp-Method header is required",
            ),
        );
    };
    if header_method != method {
        return reply(
            400,
            error_value(
                &id,
                -32020,
                &format!("Header mismatch: Mcp-Method header value '{header_method}' does not match body value '{method}'"),
            ),
        );
    }
    // Mcp-Name mirrors params.name (tools/call) or params.uri (resources/read). Values outside the
    // header-safe ASCII set are carried as =?base64?...?= sentinels and decoded before comparing.
    let mirrored_name = match method {
        "tools/call" => params.get("name").and_then(Value::as_str),
        "resources/read" => params.get("uri").and_then(Value::as_str),
        _ => None,
    };
    if let Some(expected) = mirrored_name {
        let actual = match headers.name.as_deref().map(decode_header_value) {
            Some(Some(v)) => v,
            Some(None) => {
                return reply(
                    400,
                    error_value(
                        &id,
                        -32020,
                        "Header mismatch: the Mcp-Name header value is malformed",
                    ),
                );
            }
            None => {
                return reply(
                    400,
                    error_value(
                        &id,
                        -32020,
                        &format!("Header mismatch: the Mcp-Name header is required for {method}"),
                    ),
                );
            }
        };
        if actual != expected {
            return reply(
                400,
                error_value(
                    &id,
                    -32020,
                    &format!("Header mismatch: Mcp-Name header value '{actual}' does not match body value '{expected}'"),
                ),
            );
        }
    }

    match method {
        "server/discover" => reply(200, ok_value(&id, discover(store_path, ctx))),
        "ping" => reply(200, ok_value(&id, json!({ "resultType": "complete" }))),
        "tools/list" => {
            let (ttl, scope) = TOOLS_LIST_CACHE;
            reply(
                200,
                ok_value(
                    &id,
                    json!({
                        "resultType": "complete",
                        "tools": tools::tool_definitions(),
                        "ttlMs": ttl,
                        "cacheScope": scope,
                    }),
                ),
            )
        }
        "tools/call" => reply(200, tools_call(store_path, ctx, &id, &params)),
        // Unknown method: HTTP 404 with the JSON-RPC body still naming -32601, so caches and
        // gateways can treat it as a missing endpoint.
        _ => reply(
            404,
            error_value(&id, -32601, &format!("method '{method}' not found")),
        ),
    }
}

/// The `server/discover` result: supported versions, capabilities, identity and the effective
/// instructions (a non-empty `instructions` setting overrides the built-in default; read failures
/// always fall back, never blocking discovery). Identity-dependent → private and immediately stale.
fn discover(store_path: &Path, ctx: &IdentityCtx) -> Value {
    let base = effective_instructions(store_path);
    let instructions = format!("{base}\n\n{}", ctx.describe_line());
    json!({
        "resultType": "complete",
        "supportedVersions": [PROTOCOL_VERSION],
        "capabilities": {
            "tools": {"listChanged": false},
            "resources": {"listChanged": true, "subscribe": true},
        },
        "_meta": {
            "io.modelcontextprotocol/serverInfo": {
                "name": "agent-memory",
                "title": "Agent Memory",
                "version": env!("CARGO_PKG_VERSION"),
            }
        },
        "instructions": instructions,
        "ttlMs": 0,
        "cacheScope": "private",
    })
}

/// The effective discovery prompt: a non-empty `instructions` setting overrides the built-in default.
fn effective_instructions(store_path: &Path) -> String {
    let base = store::with_db_in(store_path, store::TxMode::ReadOnly, |st| {
        st.settings_get("instructions")
    })
    .ok()
    .flatten();
    match base {
        Some(s) if !s.trim().is_empty() => s,
        _ => tools::INSTRUCTIONS.to_string(),
    }
}

fn tools_call(store_path: &Path, ctx: &IdentityCtx, id: &Value, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    if !tools::TOOL_NAMES.contains(&name) {
        return error_value(id, -32602, &format!("unknown tool '{name}'"));
    }
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    if !args.is_object() {
        return error_value(id, -32602, "tools/call arguments must be an object");
    }

    match tools::execute_with_db(store_path, ctx, name, &args) {
        Ok(v) => {
            // The text carries the same data as structuredContent; serialized compactly — a pretty-printed
            // layout's indentation whitespace would be paid for out of the client model's context on every tool call
            let text = serde_json::to_string(&v).unwrap_or_else(|_| "{}".to_string());
            ok_value(
                id,
                json!({
                    "content": [{"type": "text", "text": text}],
                    "structuredContent": v,
                    "resultType": "complete",
                }),
            )
        }
        Err(e) => ok_value(
            id,
            json!({
                "content": [{"type": "text", "text": e.message()}],
                "isError": true,
                "resultType": "complete",
            }),
        ),
    }
}

fn unsupported_version_error(id: &Value, requested: Option<&str>) -> Value {
    error_with_data(
        id,
        -32022,
        "Unsupported protocol version",
        json!({
            "supported": [PROTOCOL_VERSION],
            "requested": requested,
        }),
    )
}

/// Decode a mirrored header value: `=?base64?...?=` (lowercase sentinels, exactly as the spec writes
/// them) carries Base64 of the UTF-8 value; anything else is the plain value. `None` = malformed.
fn decode_header_value(raw: &str) -> Option<String> {
    let Some(rest) = raw.strip_prefix("=?base64?") else {
        return Some(raw.to_string());
    };
    let encoded = rest.strip_suffix("?=")?;
    base64_decode(encoded).and_then(|bytes| String::from_utf8(bytes).ok())
}

/// Standard-alphabet Base64 decoder (std-only, mirroring the project's minimal-dependency stance;
/// the only use is decoding `=?base64?...?=` header sentinels).
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes = input.as_bytes();
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks_exact(4) {
        let pad = chunk.iter().filter(|&&c| c == b'=').count();
        if pad > 2 || chunk[..4 - pad].contains(&b'=') {
            return None; // padding only allowed as the 1-2 final characters
        }
        let mut acc: u32 = 0;
        for &c in &chunk[..4 - pad] {
            acc = (acc << 6) | val(c)?;
        }
        acc <<= 6 * pad as u32;
        let decoded = [(acc >> 16) as u8, (acc >> 8) as u8, acc as u8];
        out.extend_from_slice(&decoded[..3 - pad]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    fn temp_db(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agent-memory-proto-{}-{}.db",
            std::process::id(),
            tag
        ))
    }

    fn cleanup(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(PathBuf::from(format!("{}{}", path.display(), suffix)));
        }
    }

    /// Headers a conforming modern client sends: mirrors derived from the message itself.
    fn conforming_headers(method: &str, params: &Value) -> TransportHeaders {
        let pv = params
            .pointer("/_meta/io.modelcontextprotocol/protocolVersion")
            .and_then(Value::as_str)
            .unwrap_or(PROTOCOL_VERSION)
            .to_string();
        let name = match method {
            "tools/call" => params.get("name").and_then(Value::as_str),
            "resources/read" => params.get("uri").and_then(Value::as_str),
            _ => None,
        };
        TransportHeaders {
            protocol_version: Some(pv),
            method: Some(method.to_string()),
            name: name.map(str::to_string),
        }
    }

    /// Execute one JSON-RPC message in-process as a conforming modern client would (injecting the
    /// required `_meta` and the mirrored headers; open-mode identity = full capabilities).
    fn roundtrip(store_path: &Path, line: &str) -> (u16, Value) {
        let mut msg: Value = serde_json::from_str(line).unwrap();
        let method = msg["method"].as_str().unwrap_or_default().to_string();
        if msg.get("params").is_none_or(|p| !p.is_object()) {
            msg["params"] = json!({});
        }
        if msg["params"].get("_meta").is_none() {
            msg["params"]["_meta"] = json!({
                "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
                "io.modelcontextprotocol/clientCapabilities": {},
            });
        }
        let headers = conforming_headers(&method, msg.get("params").unwrap_or(&Value::Null));
        match handle(store_path, &IdentityCtx::open_mode(), &headers, &msg) {
            Outcome::Reply { status, value } => (status, value),
            Outcome::Accepted => panic!("expected a reply for: {line}"),
        }
    }

    fn request_of(line: &str) -> Value {
        serde_json::from_str(line).unwrap()
    }

    /// Handle a pre-built message with explicit headers.
    fn handle_with(store_path: &Path, headers: &TransportHeaders, msg: &Value) -> Outcome {
        handle(store_path, &IdentityCtx::open_mode(), headers, msg)
    }

    fn modern_meta() -> Value {
        json!({
            "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
            "io.modelcontextprotocol/clientCapabilities": {},
        })
    }

    // Token plaintext appears only once, in the create return value: a thread_local relays it between closures
    thread_local! {
        static ALICE_TOKEN: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    }

    #[test]
    fn discover_shape_and_identity_line() {
        let store = temp_db("discover");
        let (status, resp) = roundtrip(
            &store,
            r#"{"jsonrpc":"2.0","id":"d1","method":"server/discover","params":{}}"#,
        );
        assert_eq!(status, 200);
        assert_eq!(resp["result"]["resultType"], "complete");
        assert_eq!(resp["result"]["supportedVersions"], json!(["2026-07-28"]));
        assert_eq!(
            resp["result"]["capabilities"]["tools"],
            json!({"listChanged": false})
        );
        assert_eq!(
            resp["result"]["capabilities"]["resources"],
            json!({"listChanged": true, "subscribe": true})
        );
        assert_eq!(
            resp["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            "agent-memory"
        );
        // Identity-dependent → private and immediately stale
        assert_eq!(resp["result"]["ttlMs"], 0);
        assert_eq!(resp["result"]["cacheScope"], "private");

        // The instructions end with the open-mode identity line
        let instructions = resp["result"]["instructions"].as_str().unwrap();
        assert!(
            instructions.starts_with(tools::INSTRUCTIONS),
            "default base prompt first: {instructions}"
        );
        assert!(
            instructions.ends_with(&IdentityCtx::open_mode().describe_line()),
            "identity line last: {instructions}"
        );
        cleanup(&store);
    }

    #[test]
    fn discover_carries_custom_instructions_and_identity_line() {
        let store = temp_db("discover-instructions");

        // A custom prompt overrides the built-in default; the identity line follows last
        store::with_db_in(&store, store::TxMode::Write, |st| -> Result<(), String> {
            st.settings_put("instructions", "这是团队共享记忆库，提交前先检索。")?;
            let (token, _) = st.identity_create("alice", &crate::auth::Permissions::all())?;
            ALICE_TOKEN.with(|cell| *cell.borrow_mut() = Some(token));
            Ok(())
        })
        .unwrap();
        let ctx = store::with_db_in(
            &store,
            store::TxMode::ReadOnly,
            |st| -> Result<crate::auth::IdentityCtx, String> {
                let token = ALICE_TOKEN.with(|cell| cell.take()).unwrap();
                Ok(st.identity_ctx_by_token(&token).unwrap().unwrap())
            },
        )
        .unwrap();
        let msg = request_of(&format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{{"_meta":{}}}}}"#,
            modern_meta()
        ));
        let headers =
            conforming_headers("server/discover", msg.get("params").unwrap_or(&Value::Null));
        let resp = match handle(&store, &ctx, &headers, &msg) {
            Outcome::Reply { value, .. } => value,
            Outcome::Accepted => panic!("expected a reply"),
        };
        let instructions = resp["result"]["instructions"].as_str().unwrap();
        assert_eq!(
            instructions,
            "这是团队共享记忆库，提交前先检索。\n\nCaller identity: alice; permissions: read, create, update, delete, tag_manage, admin."
        );
        cleanup(&store);
    }

    #[test]
    fn initialize_is_rejected_with_supported_versions() {
        let store = temp_db("initialize");
        let (status, resp) = roundtrip(
            &store,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        );
        assert_eq!(status, 400);
        assert_eq!(resp["error"]["code"], -32022);
        assert_eq!(resp["error"]["data"]["supported"], json!(["2026-07-28"]));
        cleanup(&store);
    }

    #[test]
    fn notifications_are_silently_ignored() {
        let store = temp_db("notify");
        for line in [
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            // Unknown notifications/* are silenced the same way (clients send all kinds, e.g. cancelled)
            r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{}}"#,
            // A method-named message without an id is a notification by JSON-RPC shape
            r#"{"jsonrpc":"2.0","method":"ping"}"#,
        ] {
            let msg = request_of(line);
            let headers = TransportHeaders {
                protocol_version: Some(PROTOCOL_VERSION.into()),
                method: Some("notifications/x".into()),
                name: None,
            };
            assert!(
                matches!(handle_with(&store, &headers, &msg), Outcome::Accepted),
                "expected 202 for: {line}"
            );
        }
        // A body without id or method has nothing to accept: 400
        let (status, resp) = roundtrip(&store, r#"{"jsonrpc":"2.0"}"#);
        assert_eq!(status, 400);
        assert_eq!(resp["error"]["code"], -32600);
        cleanup(&store);
    }

    #[test]
    fn meta_validation_matrix() {
        let store = temp_db("meta");

        // Missing _meta entirely → -32602
        let msg = request_of(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#);
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: Some("ping".into()),
            name: None,
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32602);

        // Missing protocolVersion → -32602
        let msg = request_of(
            r#"{"jsonrpc":"2.0","id":2,"method":"ping","params":{"_meta":{"io.modelcontextprotocol/clientCapabilities":{}}}}"#,
        );
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: Some("ping".into()),
            name: None,
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32602);

        // Missing clientCapabilities → -32602
        let msg = request_of(
            r#"{"jsonrpc":"2.0","id":3,"method":"ping","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}"#,
        );
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32602);

        // An unsupported (but present and header-matching) version → -32022 with data
        let msg = request_of(
            r#"{"jsonrpc":"2.0","id":4,"method":"ping","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"1900-01-01","io.modelcontextprotocol/clientCapabilities":{}}}}"#,
        );
        let headers = TransportHeaders {
            protocol_version: Some("1900-01-01".into()),
            method: Some("ping".into()),
            name: None,
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32022);
        assert_eq!(value["error"]["data"]["supported"], json!(["2026-07-28"]));
        assert_eq!(value["error"]["data"]["requested"], "1900-01-01");
        cleanup(&store);
    }

    #[test]
    fn header_mismatch_matrix() {
        let store = temp_db("headers");
        let msg = request_of(&format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"ping","params":{{"_meta":{}}}}}"#,
            modern_meta()
        ));

        // Missing MCP-Protocol-Version header → -32020
        let headers = TransportHeaders {
            protocol_version: None,
            method: Some("ping".into()),
            name: None,
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32020);

        // Header disagrees with the body → -32020
        let headers = TransportHeaders {
            protocol_version: Some("2025-06-18".into()),
            method: Some("ping".into()),
            name: None,
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32020);

        // Missing Mcp-Method header → -32020
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: None,
            name: None,
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32020);

        // Mcp-Method disagrees with the body → -32020
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: Some("tools/list".into()),
            name: None,
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32020);

        // tools/call without the Mcp-Name header → -32020; with a mismatching one → -32020
        let call = request_of(&format!(
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"memory_list","arguments":{{}},"_meta":{}}}}}"#,
            modern_meta()
        ));
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: Some("tools/call".into()),
            name: None,
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &call) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32020);

        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: Some("tools/call".into()),
            name: Some("memory_delete".into()),
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &call) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32020);
        assert!(value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("memory_delete"));
        cleanup(&store);
    }

    #[test]
    fn mcp_name_accepts_base64_sentinel_and_plain() {
        let store = temp_db("sentinel");
        // A non-ASCII tool name is not possible (tool names are ASCII), so exercise the sentinel
        // through resources/read-style comparison logic via tools/call with an ASCII name encoded.
        let call = format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"memory_list","arguments":{{}},"_meta":{}}}}}"#,
            modern_meta()
        );
        let msg = request_of(&call);
        // Plain value passes
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: Some("tools/call".into()),
            name: Some("memory_list".into()),
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 200);
        assert_eq!(value["result"]["resultType"], "complete");

        // Base64 sentinel of the same value passes ("memory_list" → bWVtb3J5X2xpc3Q=)
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: Some("tools/call".into()),
            name: Some("=?base64?bWVtb3J5X2xpc3Q=?=".into()),
        };
        let Outcome::Reply { status, .. } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 200);

        // A malformed sentinel → -32020
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: Some("tools/call".into()),
            name: Some("=?base64?!!!?=".into()),
        };
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32020);
        cleanup(&store);
    }

    #[test]
    fn base64_decode_roundtrip_and_strictness() {
        assert_eq!(base64_decode("bWVtb3J5X2xpc3Q=").unwrap(), b"memory_list");
        assert!(base64_decode("").is_none());
        assert_eq!(base64_decode("AAAA").unwrap(), vec![0, 0, 0]);
        assert_eq!(base64_decode("AAA=").unwrap(), vec![0, 0]);
        assert_eq!(base64_decode("AA==").unwrap(), vec![0]);
        // Padding misplaced / bad characters / bad length
        assert!(base64_decode("=AAA").is_none());
        assert!(base64_decode("A===").is_none());
        assert!(base64_decode("AAA").is_none());
        assert!(base64_decode("A@A=").is_none());
    }

    #[test]
    fn ping_unknown_method_and_invalid_request() {
        let store = temp_db("methods");
        let (status, pong) = roundtrip(
            &store,
            &format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"ping","params":{{"_meta":{}}}}}"#,
                modern_meta()
            ),
        );
        assert_eq!(status, 200);
        assert_eq!(pong["result"]["resultType"], "complete");

        // Unknown method → 404 + -32601 (the HTTP status distinguishes it from a wrong endpoint)
        let (status, unknown) = roundtrip(
            &store,
            &format!(
                r#"{{"jsonrpc":"2.0","id":2,"method":"bogus","params":{{"_meta":{}}}}}"#,
                modern_meta()
            ),
        );
        assert_eq!(status, 404);
        assert_eq!(unknown["error"]["code"], -32601);

        // Id present but method missing: -32600, otherwise the client hangs waiting
        let (status, invalid) = roundtrip(
            &store,
            &format!(
                r#"{{"jsonrpc":"2.0","id":3,"params":{{"_meta":{}}}}}"#,
                modern_meta()
            ),
        );
        assert_eq!(status, 400);
        assert_eq!(invalid["error"]["code"], -32600);

        // Non-string method: also -32600
        let (status, bad_type) = roundtrip(
            &store,
            &format!(
                r#"{{"jsonrpc":"2.0","id":4,"method":42,"params":{{"_meta":{}}}}}"#,
                modern_meta()
            ),
        );
        assert_eq!(status, 400);
        assert_eq!(bad_type["error"]["code"], -32600);

        // A null id is malformed in the modern protocol (notifications omit the id entirely)
        let (status, resp) = roundtrip(&store, r#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#);
        assert_eq!(status, 400);
        assert_eq!(resp["error"]["code"], -32600);
        cleanup(&store);
    }

    #[test]
    fn batch_bodies_are_rejected() {
        let store = temp_db("batch");
        let headers = TransportHeaders {
            protocol_version: Some(PROTOCOL_VERSION.into()),
            method: None,
            name: None,
        };
        let batch = request_of(
            r#"[{"jsonrpc":"2.0","id":1,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"}]"#,
        );
        let Outcome::Reply { status, value } = handle_with(&store, &headers, &batch) else {
            panic!("expected a reply");
        };
        assert_eq!(status, 400);
        assert_eq!(value["error"]["code"], -32600);

        // Non-object bodies likewise
        for body in [r#"42"#, r#""str""#] {
            let msg = request_of(body);
            let Outcome::Reply { status, value } = handle_with(&store, &headers, &msg) else {
                panic!("expected a reply");
            };
            assert_eq!(status, 400);
            assert_eq!(value["error"]["code"], -32600);
        }
        cleanup(&store);
    }

    #[test]
    fn tools_call_paths_through_protocol() {
        let store = temp_db("tools");
        let (status, created) = roundtrip(
            &store,
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory_create","arguments":{"summary":"s","content":"c"}}}"#,
        );
        assert_eq!(status, 200);
        assert_eq!(created["result"]["structuredContent"]["memory"]["id"], "m1");
        assert_eq!(created["result"]["resultType"], "complete");
        // Text content and structured content carry the same data, as compact JSON (no newlines or indentation)
        let text = created["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("\"id\""));
        assert!(
            !text.contains('\n'),
            "tool result text must be compact JSON"
        );

        let (_, unknown_tool) = roundtrip(
            &store,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"nope"}}"#,
        );
        assert_eq!(unknown_tool["error"]["code"], -32602);

        let (_, bad_args) = roundtrip(
            &store,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"memory_create","arguments":{"summary":"only"}}}"#,
        );
        assert_eq!(bad_args["result"]["isError"], true);
        assert_eq!(bad_args["result"]["resultType"], "complete");

        // Missing arguments count as an empty object → an isError for missing required parameters, not a protocol error
        let (_, no_args) = roundtrip(
            &store,
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"memory_get"}}"#,
        );
        assert_eq!(no_args["result"]["isError"], true);
        cleanup(&store);
    }

    #[test]
    fn tools_list_carries_static_cache_hints() {
        let store = temp_db("tools-list");
        let (_, listing) = roundtrip(&store, r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#);
        assert_eq!(listing["result"]["resultType"], "complete");
        assert_eq!(listing["result"]["tools"].as_array().unwrap().len(), 10);
        assert_eq!(listing["result"]["ttlMs"], 3_600_000);
        assert_eq!(listing["result"]["cacheScope"], "public");
        cleanup(&store);
    }
}
