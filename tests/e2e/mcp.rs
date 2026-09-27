//! agent 的 MCP 端点（POST /mcp，Streamable HTTP 无状态模式）全流程。

use serde_json::{json, Value};

use crate::common::{cleanup, json_body, request, temp_db, try_request, HttpProc};

/// 对 /mcp 发一条 JSON-RPC 请求，断言 HTTP 200 并返回解析后的响应。
fn mcp_rpc(port: u16, body: Value) -> Value {
    let (status, bytes, _) = request(
        port,
        "POST",
        "/mcp",
        Some(&serde_json::to_string(&body).unwrap()),
    );
    assert_eq!(status, 200, "unexpected status for rpc: {body}");
    json_body(&bytes)
}

#[test]
fn mcp_endpoint_end_to_end() {
    let db = temp_db("mcp");
    cleanup(&db);
    let server = HttpProc::start(&db, "mcp");
    let port = server.port;

    // 探活
    let (status, body, _) = request(port, "GET", "/health", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["status"], "ok");

    // initialize：协议版本回显
    let init = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "e2e", "version": "0.0.1"}
        }}),
    );
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["serverInfo"]["name"], "agent-memory");

    // 通知：无响应体 → 202
    let (status, body, _) = request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
    );
    assert_eq!(status, 202);
    assert!(body.is_empty());

    // 完整 agent 流程：创建 → 搜索（渐进式披露，不泄露正文）→ 取全文
    let created = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
            "name": "memory_create", "arguments": {
                "summary": "项目使用 Rust 实现 agent 记忆系统",
                "content": "仓库位于 D:\\Workspace，SQLite 做持久化，搜索无需分词。",
                "tags": ["项目", "rust"]
            }
        }}),
    );
    assert!(created.get("error").is_none(), "create failed: {created}");
    let mem_id = created["result"]["structuredContent"]["memory"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let searched = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
            "name": "memory_search", "arguments": {"query": "记忆系统"}
        }}),
    );
    let results = searched["result"]["structuredContent"]["results"]
        .as_array()
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["id"], mem_id.as_str());
    assert!(
        results[0].get("content").is_none(),
        "search must not leak content"
    );
    // 响应瘦身：不回显 query；hint 只在首页；零结果才带 note
    let sc = &searched["result"]["structuredContent"];
    assert!(sc.get("query").is_none(), "query echo must be dropped");
    assert!(sc.get("hint").is_some(), "first page carries the hint");
    assert!(sc.get("note").is_none(), "non-empty results carry no note");
    let empty = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 30, "method": "tools/call", "params": {
            "name": "memory_search", "arguments": {"query": "绝对不存在的词", "offset": 10}
        }}),
    );
    let empty_sc = &empty["result"]["structuredContent"];
    assert_eq!(empty_sc["total_matches"], 0);
    assert!(empty_sc.get("hint").is_none(), "later pages omit the hint");
    assert!(
        empty_sc.get("note").is_some(),
        "zero results guide the caller"
    );

    // 文本通道为紧凑 JSON（缩进空白会白占模型上下文）
    let text = searched["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        !text.contains('\n'),
        "tool result text must be compact JSON"
    );

    // MCP-Protocol-Version 头声明旧版本时不带 structuredContent（避免双份注入）
    let (status, body, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(
            &serde_json::to_string(&json!({
                "jsonrpc": "2.0", "id": 31, "method": "tools/call",
                "params": {"name": "memory_search", "arguments": {"query": "记忆系统"}}
            }))
            .unwrap(),
        ),
        &[("MCP-Protocol-Version", "2025-03-26")],
    )
    .unwrap();
    assert_eq!(status, 200);
    let old = json_body(&body);
    assert!(
        old["result"].get("structuredContent").is_none(),
        "pre-2025-06-18 clients must not receive structuredContent"
    );
    assert!(old["result"]["content"][0]["text"].is_string());

    let fetched = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {
            "name": "memory_get", "arguments": {"ids": [mem_id]}
        }}),
    );
    assert!(
        fetched["result"]["structuredContent"]["memories"][0]["content"]
            .as_str()
            .unwrap()
            .contains("SQLite")
    );

    // 更新正文与摘要（回归：字段更新路径曾在嵌入表缺失时整条更新失败）
    let updated = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 11, "method": "tools/call", "params": {
            "name": "memory_update", "arguments": {
                "id": mem_id,
                "summary": "项目使用混合检索的记忆系统",
                "content": "正文已更新：SQLite 做持久化，关键词 + 语义混合排序。"
            }
        }}),
    );
    assert!(updated.get("error").is_none(), "update failed: {updated}");
    assert_eq!(updated["result"]["structuredContent"]["updated"], true);
    // 仅改字段的更新不携带标签字段
    assert!(updated["result"]["structuredContent"]
        .get("tags_autocreated")
        .is_none());

    // 错误路径：未知工具 → -32602；未知方法 → -32601；参数错误 → isError
    let unknown_tool = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "nope"}}),
    );
    assert_eq!(unknown_tool["error"]["code"], -32602);
    let unknown_method = mcp_rpc(port, json!({"jsonrpc": "2.0", "id": 6, "method": "bogus"}));
    assert_eq!(unknown_method["error"]["code"], -32601);
    let bad_args = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {
            "name": "memory_create", "arguments": {"summary": "only summary"}
        }}),
    );
    assert_eq!(bad_args["result"]["isError"], true);

    // 非 JSON body → 400 + -32700
    let (status, body, _) = request(port, "POST", "/mcp", Some("not json"));
    assert_eq!(status, 400);
    assert_eq!(json_body(&body)["error"]["code"], -32700);

    // Content-Type 非 JSON → 415（MCP 规范要求）
    let (status, _, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#),
        &[("Content-Type", "text/plain")],
    )
    .unwrap();
    assert_eq!(status, 415);

    // 批量消息经 HTTP：ping 的响应 + 通知不产生响应 → 单元素数组
    let (status, body, _) = request(
        port,
        "POST",
        "/mcp",
        Some(
            r#"[{"jsonrpc":"2.0","id":20,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"}]"#,
        ),
    );
    assert_eq!(status, 200);
    let batch_resp = json_body(&body);
    let batch = batch_resp.as_array().unwrap();
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0]["id"], 20);
    assert_eq!(batch[0]["result"], json!({}));

    // GET /mcp → 405；未知路径：GET 走 SPA 回退，非 GET 404
    let (status, _, _) = request(port, "GET", "/mcp", None);
    assert_eq!(status, 405);
    let (status, _, _) = request(port, "GET", "/nope", None);
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "PUT", "/nope", Some("{}"));
    assert_eq!(status, 404);

    // Origin 防护：非本机来源 403；本机来源放行
    let (status, _, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","id":8,"method":"ping"}"#),
        &[("Origin", "http://evil.example")],
    )
    .unwrap();
    assert_eq!(status, 403);
    let (status, _, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","id":9,"method":"ping"}"#),
        &[("Origin", "http://127.0.0.1:3000")],
    )
    .unwrap();
    assert_eq!(status, 200);

    // 风暴后仍可用
    let pong = mcp_rpc(port, json!({"jsonrpc": "2.0", "id": 10, "method": "ping"}));
    assert_eq!(pong["result"], json!({}));

    drop(server);
    cleanup(&db);
}
