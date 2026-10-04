//! Full flow of the agent's MCP endpoint (POST /mcp, modern protocol 2026-07-28, stateless).
//!
//! Covers the discovery flow, per-request `_meta` and mirrored-header validation, the result
//! envelope (resultType / caching hints), status-code mapping (404 for unknown methods, 202 for
//! notifications, 400 for batch bodies) and the tool surface with progressive disclosure.

use serde_json::{json, Value};

use crate::common::{
    cleanup, json_body, mcp_post, request, rpc_body, temp_db, try_request, HttpProc,
    MCP_PROTOCOL_VERSION,
};

/// POST to /mcp and parse the response body (Null when there is no body, e.g. 202).
fn send(port: u16, body: &str, headers: &[(&str, &str)]) -> (u16, Value) {
    let (status, bytes, _) = try_request(port, "POST", "/mcp", Some(body), headers).unwrap();
    let parsed = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, parsed)
}

/// Send a JSON-RPC request as a conforming modern client (proper `_meta` + mirrored headers);
/// asserts HTTP 200 and returns the parsed response.
fn mcp_rpc(port: u16, id: Value, method: &str, params: Value) -> Value {
    let (status, resp) = mcp_post(port, id, method, params, &[]);
    assert_eq!(status, 200, "unexpected status for {method}: {resp}");
    resp
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

    // server/discover replaces initialize: supported versions, capabilities, identity line
    let discover = mcp_rpc(port, json!(1), "server/discover", json!({}));
    let result = &discover["result"];
    assert_eq!(result["resultType"], "complete");
    assert_eq!(result["supportedVersions"], json!(["2026-07-28"]));
    assert_eq!(result["capabilities"]["resources"]["listChanged"], true);
    assert_eq!(result["capabilities"]["resources"]["subscribe"], true);
    assert_eq!(
        result["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
        "agent-memory"
    );
    // Identity-dependent → private and immediately stale
    assert_eq!(result["ttlMs"], 0);
    assert_eq!(result["cacheScope"], "private");
    let instructions = result["instructions"].as_str().unwrap();
    assert!(
        instructions.contains("Access mode: open"),
        "discover carries the caller identity line: {instructions}"
    );

    // Notification: no response body -> 202
    let (status, body) = send(
        port,
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        &[],
    );
    assert_eq!(status, 202);
    assert_eq!(body, Value::Null);

    // tools/list carries static caching hints (the tool surface never changes at runtime)
    let tools = mcp_rpc(port, json!(2), "tools/list", json!({}));
    assert_eq!(tools["result"]["resultType"], "complete");
    assert_eq!(tools["result"]["ttlMs"], 3_600_000);
    assert_eq!(tools["result"]["cacheScope"], "public");
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 12);

    // Full agent flow: create -> search (progressive disclosure, no content leak) -> fetch full text
    let created = mcp_rpc(
        port,
        json!(3),
        "tools/call",
        json!({
            "name": "memory_create", "arguments": {
                "summary": "项目使用 Rust 实现 agent 记忆系统",
                "content": "仓库位于 D:\\Workspace，SQLite 做持久化，搜索无需分词。",
                "tags": ["项目", "rust"], "create_missing_tags": true
            }
        }),
    );
    assert!(created.get("error").is_none(), "create failed: {created}");
    assert_eq!(created["result"]["resultType"], "complete");
    let mem_id = created["result"]["structuredContent"]["memory"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let searched = mcp_rpc(
        port,
        json!(4),
        "tools/call",
        json!({"name": "memory_search", "arguments": {"query": "记忆系统"}}),
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
        json!(5),
        "tools/call",
        json!({"name": "memory_search", "arguments": {"query": "绝对不存在的词", "offset": 10}}),
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

    let fetched = mcp_rpc(
        port,
        json!(6),
        "tools/call",
        json!({"name": "memory_get", "arguments": {"ids": [mem_id]}}),
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
        json!(7),
        "tools/call",
        json!({
            "name": "memory_update", "arguments": {
                "id": mem_id,
                "summary": "项目使用混合检索的记忆系统",
                "content": "正文已更新：SQLite 做持久化，关键词 + 语义混合排序。"
            }
        }),
    );
    assert!(updated.get("error").is_none(), "update failed: {updated}");
    assert_eq!(updated["result"]["structuredContent"]["updated"], true);

    // memory_edit: replace a span inside the content without restating it; the response stays
    // sparse (no content echo) and the change is visible through memory_get
    let edited = mcp_rpc(
        port,
        json!(70),
        "tools/call",
        json!({
            "name": "memory_edit", "arguments": {
                "id": mem_id,
                "old_string": "SQLite 做持久化",
                "new_string": "SQLite 做持久化（WAL 模式）"
            }
        }),
    );
    assert!(edited.get("error").is_none(), "edit failed: {edited}");
    let edit_sc = &edited["result"]["structuredContent"];
    assert_eq!(edit_sc["replaced"], 1);
    assert!(
        edit_sc.get("content").is_none(),
        "edit must not echo the full content"
    );
    let refetched = mcp_rpc(
        port,
        json!(71),
        "tools/call",
        json!({"name": "memory_get", "arguments": {"ids": [mem_id]}}),
    );
    assert!(
        refetched["result"]["structuredContent"]["memories"][0]["content"]
            .as_str()
            .unwrap()
            .contains("WAL 模式"),
        "edited span must be visible through memory_get: {refetched}"
    );

    // Error paths: unknown tool -> -32602 on HTTP 200; unknown method -> 404 + -32601; bad params -> isError
    let unknown_tool = mcp_rpc(port, json!(8), "tools/call", json!({"name": "nope"}));
    assert_eq!(unknown_tool["error"]["code"], -32602);
    let (status, unknown_method) = send(
        port,
        &rpc_body(json!(9), "bogus", json!({})),
        &[
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "bogus"),
        ],
    );
    assert_eq!(status, 404, "unknown methods answer 404");
    assert_eq!(unknown_method["error"]["code"], -32601);
    let bad_args = mcp_rpc(
        port,
        json!(10),
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "only summary"}}),
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
        Some(&rpc_body(json!(11), "ping", json!({}))),
        &[("Content-Type", "text/plain")],
    )
    .unwrap();
    assert_eq!(status, 415);

    // Batch over HTTP: the modern protocol requires exactly one message per POST -> 400 + -32600
    let (status, batch_resp) = send(
        port,
        r#"[{"jsonrpc":"2.0","id":20,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"}]"#,
        &[],
    );
    assert_eq!(status, 400);
    assert_eq!(batch_resp["error"]["code"], -32600);

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
        Some(&rpc_body(json!(12), "ping", json!({}))),
        &[
            ("Origin", "http://evil.example"),
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "ping"),
        ],
    )
    .unwrap();
    assert_eq!(status, 403);
    let (status, _, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(&rpc_body(json!(13), "ping", json!({}))),
        &[
            ("Origin", "http://127.0.0.1:3000"),
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "ping"),
        ],
    )
    .unwrap();
    assert_eq!(status, 200);

    // Still usable after the storm
    let pong = mcp_rpc(port, json!(14), "ping", json!({}));
    assert_eq!(pong["result"]["resultType"], "complete");

    drop(server);
    cleanup(&db);
}

/// Per-request `_meta` and mirrored-header validation: the stateless request contract.
#[test]
fn modern_protocol_request_validation() {
    let db = temp_db("mcp-validation");
    cleanup(&db);
    let server = HttpProc::start(&db, "mcp-validation");
    let port = server.port;
    let ping_body = rpc_body(json!(1), "ping", json!({}));
    let ping_headers: Vec<(&str, &str)> = vec![
        ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
        ("Mcp-Method", "ping"),
    ];

    // Baseline: a conforming request passes
    let (status, resp) = send(port, &ping_body, &ping_headers);
    assert_eq!(status, 200, "{resp}");
    assert_eq!(resp["result"]["resultType"], "complete");

    // Missing _meta (legacy-style request) -> 400 + -32602
    let (status, resp) = send(
        port,
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#,
        &ping_headers,
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32602);

    // Missing clientCapabilities -> -32602
    let (status, resp) = send(
        port,
        r#"{"jsonrpc":"2.0","id":3,"method":"ping","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28"}}}"#,
        &ping_headers,
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32602);

    // Unsupported protocol version (header agrees with body) -> 400 + -32022 with data
    let body = r#"{"jsonrpc":"2.0","id":4,"method":"ping","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"1999-01-01","io.modelcontextprotocol/clientCapabilities":{}}}}"#;
    let (status, resp) = send(
        port,
        body,
        &[
            ("MCP-Protocol-Version", "1999-01-01"),
            ("Mcp-Method", "ping"),
        ],
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32022);
    assert_eq!(resp["error"]["data"]["supported"], json!(["2026-07-28"]));
    assert_eq!(resp["error"]["data"]["requested"], "1999-01-01");

    // Legacy initialize gets the same modern error (naming the supported versions)
    let (status, resp) = send(
        port,
        r#"{"jsonrpc":"2.0","id":5,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        &[],
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32022);
    assert_eq!(resp["error"]["data"]["supported"], json!(["2026-07-28"]));

    // Missing MCP-Protocol-Version header -> 400 + -32020
    let (status, resp) = send(port, &ping_body, &[("Mcp-Method", "ping")]);
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32020);

    // Header disagrees with the body -> -32020
    let (status, resp) = send(
        port,
        &ping_body,
        &[
            ("MCP-Protocol-Version", "2025-06-18"),
            ("Mcp-Method", "ping"),
        ],
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32020);

    // Missing Mcp-Method header -> -32020
    let (status, resp) = send(
        port,
        &ping_body,
        &[("MCP-Protocol-Version", MCP_PROTOCOL_VERSION)],
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32020);

    // tools/call without Mcp-Name -> -32020; with a mismatching name -> -32020
    let call_body = rpc_body(json!(6), "tools/call", json!({"name": "memory_list"}));
    let (status, resp) = send(
        port,
        &call_body,
        &[
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "tools/call"),
        ],
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32020);
    let (status, resp) = send(
        port,
        &call_body,
        &[
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "tools/call"),
            ("Mcp-Name", "memory_delete"),
        ],
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32020);
    // With the matching name the call goes through
    let (status, resp) = send(
        port,
        &call_body,
        &[
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "tools/call"),
            ("Mcp-Name", "memory_list"),
        ],
    );
    assert_eq!(status, 200, "{resp}");
    assert_eq!(resp["result"]["structuredContent"]["total"], 0);

    // A null id is malformed in the modern protocol (notifications omit the id entirely)
    let (status, resp) = send(
        port,
        r#"{"jsonrpc":"2.0","id":null,"method":"ping","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}"#,
        &ping_headers,
    );
    assert_eq!(status, 400);
    assert_eq!(resp["error"]["code"], -32600);

    drop(server);
    cleanup(&db);
}
