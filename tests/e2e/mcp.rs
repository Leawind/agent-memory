//! Full flow of the agent's MCP endpoint (POST /mcp, stateless Streamable HTTP mode).

use serde_json::{json, Value};

use crate::common::{cleanup, json_body, request, temp_db, try_request, HttpProc};

/// Send a JSON-RPC request to /mcp, assert HTTP 200, and return the parsed response.
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

    // Liveness check
    let (status, body, _) = request(port, "GET", "/health", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["status"], "ok");

    // initialize: protocol version echoed back
    let init = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "e2e", "version": "0.0.1"}
        }}),
    );
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["serverInfo"]["name"], "agent-memory");

    // Notification: no response body -> 202
    let (status, body, _) = request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
    );
    assert_eq!(status, 202);
    assert!(body.is_empty());

    // Full agent flow: create -> search (progressive disclosure, no content leak) -> fetch full text
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
    // Response slimming: query is not echoed; hint only on the first page; note only when there are no results
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

    // Text channel is compact JSON (indentation whitespace would waste model context)
    let text = searched["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        !text.contains('\n'),
        "tool result text must be compact JSON"
    );

    // No structuredContent when the MCP-Protocol-Version header declares an older version (avoids injecting it twice)
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

    // Update content and summary (regression: the field-update path once failed entirely when the embeddings table was absent)
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
    // A field-only update carries no tag fields
    assert!(updated["result"]["structuredContent"]
        .get("tags_autocreated")
        .is_none());

    // Error paths: unknown tool -> -32602; unknown method -> -32601; bad params -> isError
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

    // Non-JSON body -> 400 + -32700
    let (status, body, _) = request(port, "POST", "/mcp", Some("not json"));
    assert_eq!(status, 400);
    assert_eq!(json_body(&body)["error"]["code"], -32700);

    // Non-JSON Content-Type -> 415 (required by the MCP spec)
    let (status, _, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#),
        &[("Content-Type", "text/plain")],
    )
    .unwrap();
    assert_eq!(status, 415);

    // Batch over HTTP: the ping response + notification producing no response -> single-element array
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

    // GET /mcp -> 405; unknown paths: GET falls back to the SPA, non-GET 404
    let (status, _, _) = request(port, "GET", "/mcp", None);
    assert_eq!(status, 405);
    let (status, _, _) = request(port, "GET", "/nope", None);
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "PUT", "/nope", Some("{}"));
    assert_eq!(status, 404);

    // Origin guard: non-local origins 403; local origins pass
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

    // Still usable after the storm
    let pong = mcp_rpc(port, json!({"jsonrpc": "2.0", "id": 10, "method": "ping"}));
    assert_eq!(pong["result"], json!({}));

    drop(server);
    cleanup(&db);
}
