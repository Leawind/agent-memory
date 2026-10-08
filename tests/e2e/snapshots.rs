use crate::common::{cleanup, json_body, mcp_post, request, temp_db, HttpProc};
use serde_json::{json, Value};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn api(port: u16, method: &str, path: &str, value: Option<Value>) -> (u16, Value) {
    let body = value.map(|value| value.to_string());
    let (status, bytes, _) = request(port, method, path, body.as_deref());
    (status, json_body(&bytes))
}

#[test]
fn pagination_is_stable_across_feedback_and_invalidated_by_other_process_writes() {
    let path = temp_db("snapshots");
    cleanup(&path);
    let server = HttpProc::start(&path, "snapshots");
    let port = server.port;
    let memories: serde_json::Map<String, Value> = (1..=3)
        .map(|index| {
            (format!("m{index}"), json!({"summary":"alpha", "content":format!("{} PRIVATE_TAIL", "preview ".repeat(100)), "created_at":100,"updated_at":100}))
        })
        .collect();
    assert_eq!(
        api(
            port,
            "POST",
            "/api/import",
            Some(json!({"tags":{},"memories":memories}))
        )
        .0,
        200
    );
    let search_path = "/api/memories?query=alpha&mode=keyword&limit=1";
    let first = api(port, "GET", search_path, None).1;
    assert_eq!(first["results"][0]["id"], "m1");
    assert!(first["results"][0].get("content").is_none());
    let cursor = first["cursor"].as_str().unwrap();
    let second_page = format!("{search_path}&offset=1&cursor={cursor}");
    assert_eq!(
        api(
            port,
            "POST",
            "/api/memories/m3/use",
            Some(json!({"event_key":"used-in-answer"}))
        )
        .0,
        200
    );
    assert_eq!(
        api(port, "GET", search_path, None).1["results"][0]["id"],
        "m3"
    );
    assert_eq!(
        api(port, "GET", &second_page, None).1["results"][0]["id"],
        "m2"
    );
    assert_eq!(
        api(port, "PUT", "/api/memories/m1", Some(json!({"summary":""}))).0,
        400
    );
    assert_eq!(api(port, "GET", &second_page, None).0, 200);
    let (status, rpc) = mcp_post(
        port,
        json!(1),
        "tools/call",
        json!({"name":"memory_search", "arguments":{"query":"alpha", "mode":"keyword", "limit":1,"offset":1,"cursor":cursor}}),
        &[],
    );
    assert_eq!(status, 200);
    let text = rpc["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains(cursor));
    assert!(text.contains("m2"));
    assert!(!text.contains("PRIVATE_TAIL"));
    assert_eq!(
        api(
            port,
            "GET",
            &format!("/api/memories?query=other&cursor={cursor}"),
            None
        )
        .1["code"],
        "stale_search_cursor"
    );
    // Rejecting another intent must not destroy the original snapshot.
    assert_eq!(api(port, "GET", &second_page, None).0, 200);
    let writer = HttpProc::start(&path, "snapshots-other-process");
    assert_eq!(
        api(
            writer.port,
            "PUT",
            "/api/memories/m1",
            Some(json!({"summary":"alpha changed"}))
        )
        .0,
        200
    );
    let stale = api(port, "GET", &second_page, None);
    assert_eq!(stale.0, 400);
    assert_eq!(stale.1["code"], "stale_search_cursor");
    assert_eq!(api(port, "GET", search_path, None).0, 200);
    drop(writer);
    drop(server);
    cleanup(&path);
}

#[test]
fn hard_expiry_bounds_the_snapshot_lifetime() {
    let path = temp_db("snapshot-expiry");
    cleanup(&path);
    let server = HttpProc::start(&path, "snapshot-expiry");
    let port = server.port;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let expires_at = now + 3;
    let dump = json!({"tags":{},"memories":{"m1":{"summary":"alpha", "content":"body", "created_at":100,"updated_at":100,"lifecycle":{"expires_at":expires_at}}}});
    assert_eq!(api(port, "POST", "/api/import", Some(dump)).0, 200);
    let search_path = "/api/memories?query=alpha&mode=keyword";
    let first = api(port, "GET", search_path, None).1;
    assert_eq!(first["total_matches"], 1);
    assert_eq!(first["cursor_expires_at"], expires_at);
    let cursor = first["cursor"].as_str().unwrap();
    std::thread::sleep(Duration::from_secs(3));
    assert_eq!(
        api(port, "GET", &format!("{search_path}&cursor={cursor}"), None).1["code"],
        "stale_search_cursor"
    );
    assert_eq!(api(port, "GET", search_path, None).1["total_matches"], 0);
    drop(server);
    cleanup(&path);
}
