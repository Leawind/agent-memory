//! MCP protocol layer: JSON-RPC 2.0 message parsing and responses (specific to the HTTP transport).
//!
//! Implemented as MCP Streamable HTTP's stateless mode: initialize / ping / tools/list / tools/call;
//! notifications (no id) produce no response. Tool execution errors come back as isError results,
//! while protocol-level errors (unknown method, unknown tool, parse failure) come back as JSON-RPC errors.

use crate::auth::IdentityCtx;
use crate::store;
use crate::tools;
use serde_json::{json, Value};
use std::path::Path;

const LATEST_PROTOCOL: &str = "2025-06-18";
const KNOWN_PROTOCOLS: [&str; 3] = ["2024-11-05", "2025-03-26", "2025-06-18"];

/// structuredContent only entered the spec in 2025-06-18; earlier negotiated versions do not include it,
/// so old clients never inject the same data twice into context. A missing or unrecognized request header is treated as the latest
/// version (from 2025-06-18 on, clients must carry MCP-Protocol-Version in subsequent requests).
fn supports_structured_content(negotiated: Option<&str>) -> bool {
    !matches!(negotiated, Some("2024-11-05" | "2025-03-26"))
}

pub fn error_value(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn ok_value(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

/// Handle one inbound message; returns a response when one is warranted. Batch messages (arrays) are supported.
/// `negotiated` is the protocol version the client declared via the MCP-Protocol-Version header.
/// `ctx` is the caller identity already resolved by the HTTP layer (open mode = full capabilities).
pub fn handle_message(
    store_path: &Path,
    ctx: &IdentityCtx,
    negotiated: Option<&str>,
    msg: &Value,
) -> Option<Value> {
    if let Some(arr) = msg.as_array() {
        let mut responses = Vec::new();
        for m in arr {
            if m.is_object() {
                if let Some(r) = handle_single(store_path, ctx, negotiated, m) {
                    responses.push(r);
                }
            } else {
                // A batch member that is not an object: answer each one with an invalid request per the JSON-RPC spec (the id is unknowable; use null).
                responses.push(error_value(
                    &Value::Null,
                    -32600,
                    "invalid request: batch member is not an object",
                ));
            }
        }
        if responses.is_empty() {
            None
        } else {
            Some(Value::Array(responses))
        }
    } else {
        handle_single(store_path, ctx, negotiated, msg)
    }
}

fn handle_single(
    store_path: &Path,
    ctx: &IdentityCtx,
    negotiated: Option<&str>,
    msg: &Value,
) -> Option<Value> {
    let has_id = msg.get("id").is_some();
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    // Missing or null params are both treated as an empty object (JSON-RPC allows omitting params).
    let params = match msg.get("params") {
        Some(v) if !v.is_null() => v.clone(),
        _ => json!({}),
    };

    let method = msg.get("method").and_then(Value::as_str);
    match method {
        Some("initialize") if has_id => {
            let base = effective_instructions(store_path);
            let instructions = format!("{}\n\n{}", base, ctx.describe_line());
            Some(ok_value(&id, initialize_result(&params, instructions)))
        }
        Some("ping") if has_id => Some(ok_value(&id, json!({}))),
        Some("tools/list") if has_id => {
            Some(ok_value(&id, json!({"tools": tools::tool_definitions()})))
        }
        Some("tools/call") if has_id => Some(tools_call(store_path, ctx, negotiated, &id, &params)),
        Some(m) if m.starts_with("notifications/") => None,
        // Unknown method (a string): protocol-level error
        Some(m) if has_id => Some(error_value(&id, -32601, &format!("method '{m}' not found"))),
        // Missing/non-string method with an id: invalid request; without an id there is nothing to answer, so it is dropped
        None if has_id => Some(error_value(
            &id,
            -32600,
            "invalid request: 'method' is missing or not a string",
        )),
        _ => None,
    }
}

/// The effective initialize prompt: a non-empty `instructions` overrides the built-in default, and a non-empty
/// `conventions` is appended as a second section (operator conventions such as submission rules). Read failures or corrupt values always fall back,
/// never blocking initialize.
fn effective_instructions(store_path: &Path) -> String {
    let (base, extra) = store::with_db_in(
        store_path,
        store::TxMode::ReadOnly,
        |st| -> Result<(Option<String>, Option<String>), String> {
            Ok((
                st.settings_get("instructions")?,
                st.settings_get("conventions")?,
            ))
        },
    )
    .ok()
    .unwrap_or((None, None));
    let base = match base {
        Some(s) if !s.trim().is_empty() => s,
        _ => tools::INSTRUCTIONS.to_string(),
    };
    match extra {
        Some(e) if !e.trim().is_empty() => format!("{base}\n\n{e}"),
        _ => base,
    }
}

fn initialize_result(params: &Value, instructions: String) -> Value {
    let requested = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or("");
    let version = if KNOWN_PROTOCOLS.contains(&requested) {
        requested
    } else {
        LATEST_PROTOCOL
    };
    json!({
        "protocolVersion": version,
        "capabilities": {"tools": {"listChanged": false}},
        "serverInfo": {
            "name": "agent-memory",
            "title": "Agent Memory",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "instructions": instructions,
    })
}

fn tools_call(
    store_path: &Path,
    ctx: &IdentityCtx,
    negotiated: Option<&str>,
    id: &Value,
    params: &Value,
) -> Value {
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
            let mut result = json!({ "content": [{"type": "text", "text": text}] });
            if supports_structured_content(negotiated) {
                result["structuredContent"] = v;
            }
            ok_value(id, result)
        }
        Err(e) => ok_value(
            id,
            json!({
                "content": [{"type": "text", "text": e.message()}],
                "isError": true,
            }),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    /// Execute one JSON-RPC message in-process (open-mode identity = full capabilities).
    fn roundtrip(store_path: &Path, negotiated: Option<&str>, line: &str) -> Option<Value> {
        let msg: Value = serde_json::from_str(line).unwrap();
        handle_message(store_path, &IdentityCtx::open_mode(), negotiated, &msg)
    }

    // Token plaintext appears only once, in the create return value: a thread_local relays it between closures
    thread_local! {
        static ALICE_TOKEN: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    }

    #[test]
    fn initialize_echoes_known_and_falls_back_to_latest() {
        let store = temp_db("init");
        let known = roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05"}}"#,
        )
        .unwrap();
        assert_eq!(known["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(known["result"]["serverInfo"]["name"], "agent-memory");
        assert!(known["result"]["instructions"].is_string());

        let unknown = roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","id":2,"method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#,
        )
        .unwrap();
        assert_eq!(unknown["result"]["protocolVersion"], LATEST_PROTOCOL);
        cleanup(&store);
    }

    #[test]
    fn initialize_carries_custom_instructions_and_identity_line() {
        let store = temp_db("instructions");

        // Write a custom prompt (base override + additional conventions) plus one identity, then initialize
        // with that identity's token context
        store::with_db_in(&store, store::TxMode::Write, |st| -> Result<(), String> {
            st.settings_put("instructions", "这是团队共享记忆库，提交前先检索。")?;
            st.settings_put("conventions", "提交规范：摘要一行，标签用小写。")?;
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
        let resp = handle_message(
            &store,
            &ctx,
            None,
            &serde_json::from_str::<Value>(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            )
            .unwrap(),
        )
        .unwrap();
        let instructions = resp["result"]["instructions"].as_str().unwrap();
        assert!(
            instructions.contains("团队共享记忆库"),
            "got: {instructions}"
        );
        // The additional conventions follow right after the base prompt (exactly one blank line between), with the identity line last
        assert!(
            instructions.contains(
                                "这是团队共享记忆库，提交前先检索。\n\n提交规范：摘要一行，标签用小写。\n\nCaller identity: alice"
            ),
            "sections must compose in order: {instructions}"
        );
        assert!(
            instructions.contains("admin"),
            "permissions listed: {instructions}"
        );

        // Conventions-only write: the base falls back to the built-in default, the additional section still appended
        let extra_only = temp_db("instructions-extra");
        store::with_db_in(
            &extra_only,
            store::TxMode::Write,
            |st| -> Result<(), String> {
                st.settings_put("conventions", "extra rules")?;
                Ok(())
            },
        )
        .unwrap();
        let resp = handle_message(
            &extra_only,
            &IdentityCtx::open_mode(),
            None,
            &serde_json::from_str::<Value>(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            )
            .unwrap(),
        )
        .unwrap();
        let instructions = resp["result"]["instructions"].as_str().unwrap();
        assert!(
            instructions.starts_with(tools::INSTRUCTIONS)
                && instructions.contains("\n\nextra rules\n\n"),
            "default base + extra: {instructions}"
        );

        // Falls back to the built-in default when not customized; open mode carries the open-mode identity line
        let fresh = temp_db("instructions-default");
        let resp = handle_message(
            &fresh,
            &IdentityCtx::open_mode(),
            None,
            &serde_json::from_str::<Value>(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
            )
            .unwrap(),
        )
        .unwrap();
        let instructions = resp["result"]["instructions"].as_str().unwrap();
        assert_eq!(
            instructions,
            tools::INSTRUCTIONS.to_string() + "\n\n" + &IdentityCtx::open_mode().describe_line()
        );
        cleanup(&store);
        cleanup(&extra_only);
        cleanup(&fresh);
    }

    #[test]
    fn notifications_are_silently_ignored() {
        let store = temp_db("notify");
        assert!(roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
        )
        .is_none());
        // Unknown notifications/* are silenced the same way (MCP clients send all kinds of notifications, e.g. cancelled)
        assert!(roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{}}"#
        )
        .is_none());
        cleanup(&store);
    }

    #[test]
    fn ping_unknown_method_and_invalid_request() {
        let store = temp_db("methods");
        let pong = roundtrip(&store, None, r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#).unwrap();
        assert_eq!(pong["result"], json!({}));

        let unknown = roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","id":2,"method":"resources/list"}"#,
        )
        .unwrap();
        assert_eq!(unknown["error"]["code"], -32601);

        // Id present but method missing: must answer -32600, otherwise the client hangs waiting
        let invalid = roundtrip(&store, None, r#"{"jsonrpc":"2.0","id":3}"#).unwrap();
        assert_eq!(invalid["error"]["code"], -32600);

        // Non-string method: also -32600
        let bad_type = roundtrip(&store, None, r#"{"jsonrpc":"2.0","id":4,"method":42}"#).unwrap();
        assert_eq!(bad_type["error"]["code"], -32600);

        // Neither id nor method: nothing to answer, silenced
        assert!(roundtrip(&store, None, r#"{"jsonrpc":"2.0"}"#).is_none());
        cleanup(&store);
    }

    #[test]
    fn batch_responses_and_non_object_members() {
        let store = temp_db("batch");
        let batch = roundtrip(
            &store,
            None,
            r#"[{"jsonrpc":"2.0","id":1,"method":"ping"}, 42, {"jsonrpc":"2.0","method":"notifications/initialized"}]"#,
        )
        .unwrap();
        let arr = batch.as_array().unwrap();
        // ping's result + -32600 for the non-object member; notifications produce no response
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["result"], json!({}));
        assert_eq!(arr[0]["id"], 1);
        assert_eq!(arr[1]["error"]["code"], -32600);

        // A batch of notifications only: return nothing (the spec forbids an empty response array)
        assert!(roundtrip(
            &store,
            None,
            r#"[{"jsonrpc":"2.0","method":"notifications/initialized"}]"#
        )
        .is_none());
        cleanup(&store);
    }

    #[test]
    fn tools_call_paths_through_protocol() {
        let store = temp_db("tools");
        let created = roundtrip(
            &store,
            Some("2025-06-18"),
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory_create","arguments":{"summary":"s","content":"c"}}}"#,
        )
        .unwrap();
        assert_eq!(created["result"]["structuredContent"]["memory"]["id"], "m1");
        // Text content and structured content carry the same data, as compact JSON (no newlines or indentation)
        let text = created["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("\"id\""));
        assert!(
            !text.contains('\n'),
            "tool result text must be compact JSON"
        );

        let unknown_tool = roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"nope"}}"#,
        )
        .unwrap();
        assert_eq!(unknown_tool["error"]["code"], -32602);

        let bad_args = roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"memory_create","arguments":{"summary":"only"}}}"#,
        )
        .unwrap();
        assert_eq!(bad_args["result"]["isError"], true);

        // Missing arguments count as an empty object → an isError for missing required parameters, not a protocol error
        let no_args = roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"memory_get"}}"#,
        )
        .unwrap();
        assert_eq!(no_args["result"]["isError"], true);
        cleanup(&store);
    }

    #[test]
    fn structured_content_follows_negotiated_version() {
        let store = temp_db("sc");
        let call = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory_create","arguments":{"summary":"s","content":"c"}}}"#;

        // 2025-06-18: includes structuredContent
        let latest = roundtrip(&store, Some("2025-06-18"), call).unwrap();
        assert!(latest["result"]["structuredContent"].is_object());

        // Older versions (2025-03-26 / 2024-11-05): text only, avoiding double injection into context
        for ver in ["2025-03-26", "2024-11-05"] {
            let old = roundtrip(&store, Some(ver), call).unwrap();
            assert!(
                old["result"].get("structuredContent").is_none(),
                "{ver} must not carry structuredContent"
            );
            assert!(old["result"]["content"][0]["text"].is_string());
        }

        // Missing header: treated as the latest version
        let unspecified = roundtrip(&store, None, call).unwrap();
        assert!(unspecified["result"]["structuredContent"].is_object());
        cleanup(&store);
    }
}
