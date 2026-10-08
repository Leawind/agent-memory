//! Full flow of the agent's MCP endpoint (POST /mcp, modern protocol 2026-07-28, stateless).
//!
//! Covers the discovery flow, per-request `_meta` and mirrored-header validation (and their
//! legacy-era counterparts, where the envelope is not required), the result envelope
//! (resultType / caching hints), status-code mapping (404 for unknown methods, 202 for
//! notifications, 400 for batch bodies) and the tool surface with progressive disclosure.

use serde_json::{json, Value};

use crate::common::{
    cleanup, json_body, mcp_post, request, rpc_body, temp_db, tool_data, try_request, HttpProc,
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

/// The raw text block of a successful tools/call (the single data channel). For the three list
/// tools this is the line format; for everything else compact JSON.
fn result_text(resp: &Value) -> &str {
    resp["result"]["content"][0]["text"].as_str().unwrap()
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

    // server/discover: supported versions (both eras), capabilities, identity line
    let discover = mcp_rpc(port, json!(1), "server/discover", json!({}));
    let result = &discover["result"];
    assert_eq!(result["resultType"], "complete");
    assert_eq!(
        result["supportedVersions"],
        json!(["2026-07-28", "2025-06-18"])
    );
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
    assert_eq!(tools["result"]["tools"].as_array().unwrap().len(), 15);

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
    let mem_id = tool_data(&created)["id"].as_str().unwrap().to_string();

    let searched = mcp_rpc(
        port,
        json!(4),
        "tools/call",
        json!({"name": "memory_search", "arguments": {"query": "记忆系统"}}),
    );
    // List tools render the token-frugal line format: header, then flag lines, then one row per
    // hit with an indented snippet continuation. The grammar leaves no column for content — the
    // progressive-disclosure boundary is structural, so pinning the shape pins the boundary.
    let text = result_text(&searched);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines[0], "total_matches: 1 | offset: 0 | returned: 1 | mode: keyword",
        "search header: {text}"
    );
    assert_eq!(
        lines[2], "semantic: disabled",
        "unconfigured semantic search is stated, not inferred: {text}"
    );
    assert!(
        lines[3].starts_with("hint: "),
        "first page carries the hint: {text}"
    );
    assert!(lines[1].starts_with("cursor: s_"));
    assert_eq!(lines.len(), 6, "one row plus its snippet only: {text}");
    // Row grammar: <id> [<tags>] <updated> <score> <summary>
    let row = lines[4];
    let rest = row
        .strip_prefix(mem_id.as_str())
        .unwrap_or_else(|| panic!("row must start with the memory id {mem_id}: {row}"));
    let close = rest
        .find(']')
        .unwrap_or_else(|| panic!("row must carry a tags column: {row}"));
    let tags = &rest[..close];
    assert!(
        tags.contains("项目") && tags.contains("rust"),
        "tags column: {row}"
    );
    let after = &rest[close + 2..]; // skip "] "
    assert_eq!(
        &after[4..5],
        "-",
        "updated column is the compact timestamp: {row}"
    );
    let (score, summary) = after[17..].split_once(' ').unwrap();
    assert!(score.parse::<u64>().is_ok(), "score column: {row}");
    assert_eq!(
        summary, "项目使用 Rust 实现 agent 记忆系统",
        "summary column: {row}"
    );
    assert!(
        lines[5].starts_with("  > "),
        "snippet rides its own indented line: {text}"
    );
    let empty = mcp_rpc(
        port,
        json!(5),
        "tools/call",
        json!({"name": "memory_search", "arguments": {"query": "绝对不存在的词", "offset": 10}}),
    );
    let empty_text = result_text(&empty);
    assert!(
        empty_text
            .starts_with("total_matches: 0 | offset: 10 | returned: 0 | mode: keyword\ncursor: s_")
            && empty_text.contains("\nsemantic: disabled\nnote: "),
        "empty search: {empty_text}"
    );
    assert!(
        !empty_text.contains("\nhint: "),
        "later pages omit the hint"
    );

    // Single data channel: no structuredContent echo beside the text block (clients that surface
    // every content block would ingest the data twice). List tools are multi-line; every other
    // tool stays compact JSON.
    assert!(
        searched["result"].get("structuredContent").is_none(),
        "no structuredContent beside the text channel"
    );
    assert!(
        !result_text(&created).contains('\n'),
        "non-list tool results stay compact JSON"
    );

    let fetched = mcp_rpc(
        port,
        json!(6),
        "tools/call",
        json!({"name": "memory_get", "arguments": {"ids": [mem_id]}}),
    );
    assert!(tool_data(&fetched)["memories"][0]["content"]
        .as_str()
        .unwrap()
        .contains("SQLite"));

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
    assert_eq!(tool_data(&updated)["updated"], true);

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
    let edit_sc = tool_data(&edited);
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
        tool_data(&refetched)["memories"][0]["content"]
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
        json!({"name": "memory_create", "arguments": {"content": "body without summary"}}),
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

    // Missing _meta: served as a legacy client (that era has no per-request metadata), even
    // when modern mirrored headers ride along — headers are advisory on the legacy path
    let (status, resp) = send(
        port,
        r#"{"jsonrpc":"2.0","id":2,"method":"ping"}"#,
        &ping_headers,
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["resultType"], "complete");

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
    assert_eq!(
        resp["error"]["data"]["supported"],
        json!(["2026-07-28", "2025-06-18"])
    );
    assert_eq!(resp["error"]["data"]["requested"], "1999-01-01");

    // Legacy initialize gets the legacy handshake answer (echo + no notification promises)
    let (status, resp) = send(
        port,
        r#"{"jsonrpc":"2.0","id":5,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#,
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(
        resp["result"]["capabilities"]["resources"],
        json!({"subscribe": false, "listChanged": false})
    );

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
    let list_text = resp["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        list_text.starts_with("total: 0 | offset: 0\n"),
        "empty list renders as a header-only line format: {list_text}"
    );

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
