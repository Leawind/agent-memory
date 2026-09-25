//! MCP 协议层：JSON-RPC 2.0 消息解析与响应（HTTP 传输专用）。
//!
//! 实现为 MCP Streamable HTTP 的无状态模式：initialize / ping / tools/list / tools/call，
//! 通知（无 id）不产生响应。工具执行错误以 isError 结果返回，
//! 协议级错误（未知方法、未知工具、解析失败）以 JSON-RPC error 返回。

use crate::tools;
use serde_json::{json, Value};
use std::path::Path;

const LATEST_PROTOCOL: &str = "2025-06-18";
const KNOWN_PROTOCOLS: [&str; 3] = ["2024-11-05", "2025-03-26", "2025-06-18"];

/// structuredContent 自 2025-06-18 才进入规范；更早的协商版本不附带，
/// 避免旧客户端把双份数据都注入上下文。请求头缺失或无法识别时按最新
/// 版本处理（2025-06-18 起客户端必须在后续请求携带 MCP-Protocol-Version）。
fn supports_structured_content(negotiated: Option<&str>) -> bool {
    !matches!(negotiated, Some("2024-11-05" | "2025-03-26"))
}

pub fn error_value(id: &Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn ok_value(id: &Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

/// 处理一条入站消息；需要回应时返回响应。支持批量消息（数组）。
/// `negotiated` 是客户端经 MCP-Protocol-Version 头声明的协议版本。
pub fn handle_message(store_path: &Path, negotiated: Option<&str>, msg: &Value) -> Option<Value> {
    if let Some(arr) = msg.as_array() {
        let mut responses = Vec::new();
        for m in arr {
            if m.is_object() {
                if let Some(r) = handle_single(store_path, negotiated, m) {
                    responses.push(r);
                }
            } else {
                // 批量成员不是对象：按 JSON-RPC 规范逐个回无效请求（id 无从得知，用 null）。
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
        handle_single(store_path, negotiated, msg)
    }
}

fn handle_single(store_path: &Path, negotiated: Option<&str>, msg: &Value) -> Option<Value> {
    let has_id = msg.get("id").is_some();
    let id = msg.get("id").cloned().unwrap_or(Value::Null);
    // params 缺失或为 null 都按空对象处理（JSON-RPC 允许省略 params）。
    let params = match msg.get("params") {
        Some(v) if !v.is_null() => v.clone(),
        _ => json!({}),
    };

    let method = msg.get("method").and_then(Value::as_str);
    match method {
        Some("initialize") if has_id => Some(ok_value(&id, initialize_result(&params))),
        Some("ping") if has_id => Some(ok_value(&id, json!({}))),
        Some("tools/list") if has_id => {
            Some(ok_value(&id, json!({"tools": tools::tool_definitions()})))
        }
        Some("tools/call") if has_id => Some(tools_call(store_path, negotiated, &id, &params)),
        Some(m) if m.starts_with("notifications/") => None,
        // 未知方法（字符串）：协议级错误
        Some(m) if has_id => Some(error_value(&id, -32601, &format!("method '{m}' not found"))),
        // method 缺失或不是字符串但带了 id：无效请求；无 id 则无从应答，只能丢弃
        None if has_id => Some(error_value(
            &id,
            -32600,
            "invalid request: 'method' is missing or not a string",
        )),
        _ => None,
    }
}

fn initialize_result(params: &Value) -> Value {
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
        "instructions": tools::INSTRUCTIONS,
    })
}

fn tools_call(store_path: &Path, negotiated: Option<&str>, id: &Value, params: &Value) -> Value {
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

    match tools::execute_with_db(store_path, name, &args) {
        Ok(v) => {
            // 文本承载与 structuredContent 相同的数据；紧凑序列化——pretty 的
            // 缩进空白每次工具调用都要由客户端的模型上下文买单
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

    /// 进程内执行一条 JSON-RPC 消息。
    fn roundtrip(store_path: &Path, negotiated: Option<&str>, line: &str) -> Option<Value> {
        let msg: Value = serde_json::from_str(line).unwrap();
        handle_message(store_path, negotiated, &msg)
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
    fn notifications_are_silently_ignored() {
        let store = temp_db("notify");
        assert!(roundtrip(
            &store,
            None,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#
        )
        .is_none());
        // 未知的 notifications/* 同样静默（MCP 客户端会发各种通知，如 cancelled）
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

        // 带 id 但缺 method：必须回 -32600，否则客户端会挂起等待
        let invalid = roundtrip(&store, None, r#"{"jsonrpc":"2.0","id":3}"#).unwrap();
        assert_eq!(invalid["error"]["code"], -32600);

        // method 不是字符串：同样 -32600
        let bad_type = roundtrip(&store, None, r#"{"jsonrpc":"2.0","id":4,"method":42}"#).unwrap();
        assert_eq!(bad_type["error"]["code"], -32600);

        // 无 id 也无 method：无从应答，静默
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
        // ping 的结果 + 非对象成员的 -32600；通知不产生响应
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["result"], json!({}));
        assert_eq!(arr[0]["id"], 1);
        assert_eq!(arr[1]["error"]["code"], -32600);

        // 只有通知的批量：不返回任何内容（规范禁止空响应数组）
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
        // 文本内容与结构化内容承载同一数据，且为紧凑 JSON（无换行缩进）
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

        // arguments 缺省视为空对象 → 缺必填参数的 isError，而不是协议错误
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

        // 2025-06-18：带 structuredContent
        let latest = roundtrip(&store, Some("2025-06-18"), call).unwrap();
        assert!(latest["result"]["structuredContent"].is_object());

        // 旧版本（2025-03-26 / 2024-11-05）：只回文本，避免双份注入上下文
        for ver in ["2025-03-26", "2024-11-05"] {
            let old = roundtrip(&store, Some(ver), call).unwrap();
            assert!(
                old["result"].get("structuredContent").is_none(),
                "{ver} must not carry structuredContent"
            );
            assert!(old["result"]["content"][0]["text"].is_string());
        }

        // 头缺失：按最新版本处理
        let unspecified = roundtrip(&store, None, call).unwrap();
        assert!(unspecified["result"]["structuredContent"].is_object());
        cleanup(&store);
    }
}
