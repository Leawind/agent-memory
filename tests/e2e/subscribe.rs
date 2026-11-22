//! End-to-end subscription flow: `subscriptions/listen` opens an SSE stream, the acknowledgment
//! arrives first, writes from other connections deliver change notifications, and closing the
//! stream cancels the subscription.

use serde_json::{json, Value};
use std::time::Duration;

use crate::common::{
    cleanup, mcp_post, request, rpc_body, temp_db, try_request, HttpProc, SseStream,
    MCP_PROTOCOL_VERSION,
};

const EVENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Read events until one with the expected method arrives (skipping keep-alive comments and
/// unrelated notifications), then parse its data as JSON-RPC.
fn next_notification(stream: &mut SseStream, method: &str) -> Value {
    let deadline = std::time::Instant::now() + EVENT_TIMEOUT;
    loop {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {method}"
        );
        let Some(ev) = stream.next_event(Duration::from_secs(2)) else {
            continue;
        };
        assert_eq!(ev.event, "message", "unexpected SSE event type");
        let message: Value = serde_json::from_str(&ev.data)
            .unwrap_or_else(|e| panic!("SSE data is not JSON ({e}): {}", ev.data));
        if message["method"] == method {
            return message;
        }
    }
}

#[test]
fn subscriptions_end_to_end() {
    let db = temp_db("subscribe");
    cleanup(&db);
    let server = HttpProc::start(&db, "subscribe");
    let port = server.port;

    // Seed one tag so the catalog is non-trivial
    let (_, resp) = mcp_post(
        port,
        json!(1),
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "seed", "content": "c", "tags": ["seed-tag"], "create_missing_tags": true}}),
        &[],
    );
    assert_eq!(resp["result"]["isError"], json!(null), "{resp}");

    // Open a subscription: list changes + updates for the watched tag resource
    let watched_uri = "memory://tags/watched";
    let listen_body = rpc_body(
        json!("listen-1"),
        "subscriptions/listen",
        json!({"notifications": {
            "toolsListChanged": true,
            "resourcesListChanged": true,
            "resourceSubscriptions": [watched_uri],
        }}),
    );
    let mut stream = SseStream::open(
        port,
        &listen_body,
        &[
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "subscriptions/listen"),
        ],
    )
    .expect("SSE stream opens");

    // First message: the acknowledgment, echoing the subscription id and the honored subset
    let ack = next_notification(&mut stream, "notifications/subscriptions/acknowledged");
    assert_eq!(
        ack["params"]["_meta"]["io.modelcontextprotocol/subscriptionId"],
        "listen-1"
    );
    assert_eq!(ack["params"]["notifications"]["resourcesListChanged"], true);
    assert_eq!(
        ack["params"]["notifications"]["resourceSubscriptions"],
        json!([watched_uri])
    );
    assert!(
        ack["params"]["notifications"]
            .get("toolsListChanged")
            .is_none(),
        "tools never change at runtime, so the type is not acknowledged"
    );

    // A write from ANOTHER connection: creating the watched tag → list_changed
    let (_, resp) = mcp_post(
        port,
        json!(2),
        "tools/call",
        json!({"name": "tag_create", "arguments": {"name": "watched", "description": "watch me"}}),
        &[],
    );
    assert_eq!(resp["result"]["isError"], json!(null), "{resp}");
    let list_changed = next_notification(&mut stream, "notifications/resources/list_changed");
    assert_eq!(
        list_changed["params"]["_meta"]["io.modelcontextprotocol/subscriptionId"], "listen-1",
        "every notification carries its subscription id"
    );

    // A memory in the watched tag → the tag resource's read result changes → updated(uri)
    let (_, resp) = mcp_post(
        port,
        json!(3),
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "note", "content": "body", "tags": ["watched"], "create_missing_tags": true}}),
        &[],
    );
    assert_eq!(resp["result"]["isError"], json!(null), "{resp}");
    let updated = next_notification(&mut stream, "notifications/resources/updated");
    assert_eq!(updated["params"]["uri"], watched_uri);

    // The REST face notifies the same way (write hooks are shared)
    let (status, _, _) = request(port, "POST", "/api/tags", Some(r#"{"name": "rest-tag"}"#));
    assert_eq!(status, 200);
    next_notification(&mut stream, "notifications/resources/list_changed");

    // Closing the stream cancels the subscription: after the disconnect the server keeps serving,
    // and a fresh subscription still works (the registry entry was cleaned up).
    drop(stream);
    std::thread::sleep(Duration::from_millis(200));
    let (status, pong) = mcp_post(port, json!(9), "ping", json!({}), &[]);
    assert_eq!(status, 200);
    assert_eq!(pong["result"]["resultType"], "complete");

    let listen_body = rpc_body(
        json!("listen-2"),
        "subscriptions/listen",
        json!({"notifications": {"resourcesListChanged": true}}),
    );
    let mut stream2 = SseStream::open(
        port,
        &listen_body,
        &[
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "subscriptions/listen"),
        ],
    )
    .expect("second subscription opens");
    let ack = next_notification(&mut stream2, "notifications/subscriptions/acknowledged");
    assert_eq!(
        ack["params"]["_meta"]["io.modelcontextprotocol/subscriptionId"],
        "listen-2"
    );
    let (_, resp) = mcp_post(
        port,
        json!(10),
        "tools/call",
        json!({"name": "tag_create", "arguments": {"name": "after-reconnect"}}),
        &[],
    );
    assert_eq!(resp["result"]["isError"], json!(null), "{resp}");
    next_notification(&mut stream2, "notifications/resources/list_changed");
    drop(stream2);

    drop(server);
    cleanup(&db);
}

/// A subscriber without the read capability receives an empty acknowledgment (mask semantics),
/// and the listen request itself needs no capability.
#[test]
fn subscription_without_read_capability_gets_nothing() {
    let db = temp_db("subscribe-mask");
    cleanup(&db);
    let server = HttpProc::start(&db, "subscribe-mask");
    let port = server.port;

    // Setup: admin + auth on + anonymous = create-only
    let (status, body, _) = request(
        port,
        "POST",
        "/api/identities",
        Some(
            r#"{"name": "admin", "permissions": {"read": true, "create": true, "update": true, "delete": true, "tag_manage": true, "admin": true}}"#,
        ),
    );
    assert_eq!(status, 200);
    let admin = format!("Bearer {}", json_body_of(&body)["token"].as_str().unwrap());
    let (status, _, _) = request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": true}"#),
    );
    assert_eq!(status, 200);
    let (status, _, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"anonymous_permissions": {"create": true}}"#),
        &[("Authorization", admin.as_str())],
    )
    .unwrap();
    assert_eq!(status, 200);

    // Anonymous listener: acknowledged with an empty filter (cannot see the catalog at all)
    let listen_body = rpc_body(
        json!(1),
        "subscriptions/listen",
        json!({"notifications": {
            "resourcesListChanged": true,
            "resourceSubscriptions": ["memory://tags/whatever"],
        }}),
    );
    let mut stream = SseStream::open(
        port,
        &listen_body,
        &[
            ("MCP-Protocol-Version", MCP_PROTOCOL_VERSION),
            ("Mcp-Method", "subscriptions/listen"),
        ],
    )
    .expect("SSE stream opens");
    let ack = next_notification(&mut stream, "notifications/subscriptions/acknowledged");
    assert_eq!(ack["params"]["notifications"], json!({}));

    // A write happens (by the admin) — no notification arrives
    let (status, resp) = mcp_post(
        port,
        json!(2),
        "tools/call",
        json!({"name": "tag_create", "arguments": {"name": "invisible"}}),
        &[("Authorization", admin.as_str())],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["isError"], json!(null), "{resp}");
    // Within a generous window nothing but keep-alive comments may arrive
    match stream.next_event(Duration::from_millis(1500)) {
        None => {} // timed out on the read: no event, exactly right
        Some(ev) => {
            assert!(
                ev.data.is_empty() || ev.event.is_empty(),
                "no notification may reach a capability-less subscriber, got: {ev:?}"
            );
        }
    }

    // Malformed listen filters are rejected with -32602 on a regular (non-stream) response
    let (status, err) = mcp_post(
        port,
        json!(3),
        "subscriptions/listen",
        json!({"notifications": {"unknownFilter": true}}),
        &[],
    );
    assert_eq!(status, 400);
    assert_eq!(err["error"]["code"], -32602);

    drop(server);
    cleanup(&db);
}

fn json_body_of(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("valid JSON body")
}
