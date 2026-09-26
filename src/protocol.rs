//! MCP 协议层：JSON-RPC 2.0 消息解析与响应（HTTP 传输专用）。
//!
//! 实现为 MCP Streamable HTTP 的无状态模式：initialize / ping / tools/list / tools/call，
//! 通知（无 id）不产生响应。工具执行错误以 isError 结果返回，
//! 协议级错误（未知方法、未知工具、解析失败）以 JSON-RPC error 返回。

use crate::auth::IdentityCtx;
use crate::store;
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
/// `ctx` 是 HTTP 层已解析的调用方身份（开放模式 = 全能力）。
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
    // params 缺失或为 null 都按空对象处理（JSON-RPC 允许省略 params）。
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

/// 生效的 initialize 提示词：`instructions` 非空时覆盖内置默认，`conventions`
/// 非空时追加为第二段（提交规范等运营者约定）。读取失败或值损坏一律回退，
/// 绝不阻塞 initialize。
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

    /// 进程内执行一条 JSON-RPC 消息（开放模式身份 = 全能力）。
    fn roundtrip(store_path: &Path, negotiated: Option<&str>, line: &str) -> Option<Value> {
        let msg: Value = serde_json::from_str(line).unwrap();
        handle_message(store_path, &IdentityCtx::open_mode(), negotiated, &msg)
    }

    // token 明文只在创建返回值出现一次：闭包间传递用 thread_local 中转
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

        // 写入自定义提示词（基础覆盖 + 附加规范）+ 一个身份，然后用该身份的
        // token 上下文 initialize
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
        // 附加规范紧跟基础提示词之后（中间恰好一个空行），身份行在最后
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

        // 仅写附加规范：基础回退内置默认，附加段仍追加
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

        // 未自定义时回退内置默认；开放模式带 open-mode 身份行
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
