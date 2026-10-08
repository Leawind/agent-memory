//! Legacy-protocol client flows (2025-06-18): initialize handshake, no per-request `_meta`, no
//! mirrored headers — exactly what an official SDK 1.x client sends. Proves the shared handler
//! surface is reachable from the legacy era, in open mode and under token auth.

use serde_json::{json, Value};

use crate::common::{cleanup, json_body, temp_db, tool_data, try_request, HttpProc};

/// POST one JSON-RPC message as a legacy client: no `_meta`, no mirrored headers.
fn legacy_post(port: u16, body: &str, extra_headers: &[(&str, &str)]) -> (u16, Value) {
    let (status, bytes, _) = try_request(port, "POST", "/mcp", Some(body), extra_headers).unwrap();
    let parsed = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, parsed)
}

fn rpc(id: Value, method: &str, params: Value) -> String {
    serde_json::to_string(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
        .unwrap()
}

#[test]
fn legacy_client_full_flow() {
    let db = temp_db("legacy");
    cleanup(&db);
    let server = HttpProc::start(&db, "legacy");
    let port = server.port;

    // Handshake: the requested legacy version is echoed, capabilities promise no notification channel
    let (status, resp) = legacy_post(
        port,
        &rpc(
            json!(1),
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "legacy-e2e", "version": "0.0.0"},
            }),
        ),
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(
        resp["result"]["capabilities"]["resources"],
        json!({"subscribe": false, "listChanged": false})
    );
    assert_eq!(
        resp["result"]["capabilities"]["tools"]["listChanged"],
        false
    );
    assert_eq!(resp["result"]["serverInfo"]["name"], "agent-memory");
    assert!(resp["result"]["instructions"].is_string());

    // The initialized notification has no id: 202, empty body
    let (status, bytes, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
        &[],
    )
    .unwrap();
    assert_eq!(status, 202);
    assert!(bytes.is_empty());

    // A client requesting an older revision gets the latest legacy version to decide on
    let (_, resp) = legacy_post(
        port,
        &rpc(
            json!(2),
            "initialize",
            json!({"protocolVersion": "2024-11-05"}),
        ),
        &[],
    );
    assert_eq!(resp["result"]["protocolVersion"], "2025-06-18");

    // The full tool surface works with no envelope at all
    let (status, resp) = legacy_post(port, &rpc(json!(3), "tools/list", json!({})), &[]);
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["tools"].as_array().unwrap().len(), 13);

    let (status, resp) = legacy_post(
        port,
        &rpc(
            json!(4),
            "tools/call",
            json!({
                "name": "memory_create",
                "arguments": {"summary": "legacy client write", "content": "written by the 2025-06-18 flow"},
            }),
        ),
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["isError"], Value::Null);
    let id = tool_data(&resp)["id"].clone();

    let (status, resp) = legacy_post(
        port,
        &rpc(
            json!(5),
            "tools/call",
            json!({"name": "memory_get", "arguments": {"ids": [id]}}),
        ),
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(
        tool_data(&resp)["memories"][0]["content"],
        "written by the 2025-06-18 flow"
    );

    // Resources work the same way on the legacy path
    let (status, resp) = legacy_post(
        port,
        &rpc(
            json!(6),
            "resources/read",
            json!({"uri": "memory://memories/m1"}),
        ),
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["contents"][0]["mimeType"], "text/markdown");

    // Protocol errors keep their era-independent semantics
    let (status, resp) = legacy_post(port, &rpc(json!(7), "no/such/method", json!({})), &[]);
    assert_eq!(status, 404);
    assert_eq!(resp["error"]["code"], -32601);

    let (status, resp) = legacy_post(
        port,
        &rpc(
            json!(8),
            "tools/call",
            json!({"name": "memory_get", "arguments": {}}),
        ),
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["isError"], true);

    // Non-JSON bodies are rejected before any era logic applies
    let (status, _, _) = try_request(port, "POST", "/mcp", Some("not json"), &[]).unwrap();
    assert_eq!(status, 400);

    cleanup(&db);
}

#[test]
fn legacy_client_under_token_auth() {
    let db = temp_db("legacy-auth");
    cleanup(&db);
    let server = HttpProc::start(&db, "legacy-auth");
    let port = server.port;

    // Enable token auth: create an admin identity, then flip the switch (the guarded order)
    let (status, body, _) = try_request(
        port,
        "POST",
        "/api/identities",
        Some(r#"{"name": "admin", "permissions": {"read": true, "create": true, "update": true, "delete": true, "tag_manage": true, "admin": true}}"#),
        &[],
    )
    .unwrap();
    assert_eq!(status, 200);
    let token = json_body(&body)["token"].as_str().unwrap().to_string();
    let (status, _, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": true}"#),
        &[],
    )
    .unwrap();
    assert_eq!(status, 200);

    let bearer = format!("Bearer {token}");
    let auth = [("Authorization", bearer.as_str())];

    // Handshake with the token: same legacy answer as in open mode
    let (status, resp) = legacy_post(
        port,
        &rpc(
            json!(1),
            "initialize",
            json!({"protocolVersion": "2025-06-18"}),
        ),
        &auth,
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["protocolVersion"], "2025-06-18");
    // The identity line reflects the token identity, same as the modern discover prompt
    assert!(
        resp["result"]["instructions"]
            .as_str()
            .unwrap()
            .contains("Caller identity: admin"),
        "instructions carry the caller identity"
    );

    // A legacy write through the authenticated envelope
    let (status, resp) = legacy_post(
        port,
        &rpc(
            json!(2),
            "tools/call",
            json!({
                "name": "memory_create",
                "arguments": {"summary": "legacy auth write", "content": "c"},
            }),
        ),
        &auth,
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["isError"], Value::Null);

    // Without the token the legacy client is rejected like any other (401, no WWW-Authenticate)
    let (status, _, head) = try_request(
        port,
        "POST",
        "/mcp",
        Some(&rpc(json!(3), "ping", json!({}))),
        &[],
    )
    .unwrap();
    assert_eq!(status, 401);
    assert!(!head.to_ascii_lowercase().contains("www-authenticate"));

    cleanup(&db);
}
