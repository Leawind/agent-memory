use crate::common::{cleanup, json_body, request, temp_db, HttpProc};
use serde_json::{json, Value};

fn api(port: u16, method: &str, path: &str, value: Option<Value>) -> (u16, Value) {
    let body = value.map(|v| v.to_string());
    let (status, bytes, _) = request(port, method, path, body.as_deref());
    (status, json_body(&bytes))
}

#[test]
fn archive_expiry_filters_and_backup_keep_content_and_timestamps() {
    let path = temp_db("lifecycle");
    cleanup(&path);
    let server = HttpProc::start(&path, "lifecycle");
    let port = server.port;
    let created = api(port, "POST", "/api/memories", Some(json!({"summary": "durable fact", "content": "private full body", "tags": ["topic"], "create_missing_tags": true}))).1;
    let id = created["id"].as_str().unwrap();
    let get = || api(port, "GET", &format!("/api/memories/{id}"), None).1;
    let before = get();
    let set = |value| {
        api(
            port,
            "PUT",
            &format!("/api/memories/{id}/lifecycle"),
            Some(value),
        )
    };
    assert_eq!(
        set(json!({"kind": "context", "pinned": true, "expires_at": 0})).0,
        200
    );
    assert_eq!(get()["lifecycle"]["state"], "expired");
    assert_eq!(api(port, "GET", "/api/memories", None).1["total"], 0);
    assert_eq!(
        api(
            port,
            "GET",
            "/api/memories?query=durable&mode=keyword",
            None
        )
        .1["total_matches"],
        0
    );
    let expired = api(port, "GET", "/api/memories?state=expired", None).1;
    assert_eq!(expired["total"], 1);
    assert!(expired["memories"][0].get("content").is_none());
    assert_eq!(
        api(port, "GET", "/api/memories?query=durable&state=all", None).1["total_matches"],
        1
    );
    assert_eq!(set(json!({"archived": true})).0, 200);
    assert_eq!(get()["lifecycle"]["state"], "archived");
    assert_eq!(
        api(port, "GET", "/api/memories?state=expired", None).1["total"],
        0
    );
    assert_eq!(
        api(
            port,
            "GET",
            "/api/memories?state=archived&tag_expr=topic",
            None
        )
        .1["total"],
        1
    );
    assert_eq!(set(json!({"archived": false})).0, 200);
    assert_eq!(
        get()["lifecycle"]["state"],
        "expired",
        "restoring an archive does not erase expiry"
    );
    assert_eq!(set(json!({"expires_at": null})).0, 200);
    assert_eq!(get()["lifecycle"]["state"], "active");
    let active = get();
    assert_eq!(active["updated"], before["updated"]);
    assert_eq!(active["created"], before["created"]);
    assert_eq!(active["content"], "private full body");
    for bad in [
        json!({}),
        json!({"kind": "unknown"}),
        json!({"expires_at": -1, "pinned": false}),
        json!({"expires_at": 253402300800u64}),
        json!({"archived": "true"}),
        json!({"typo": true}),
    ] {
        assert_eq!(set(bad).0, 400);
        assert_eq!(get()["lifecycle"], active["lifecycle"]);
    }
    assert_eq!(api(port, "GET", "/api/memories?state=invalid", None).0, 400);
    assert_eq!(
        api(
            port,
            "PUT",
            "/api/memories/m999/lifecycle",
            Some(json!({"archived": true}))
        )
        .0,
        404
    );
    assert_eq!(set(json!({"archived": true})).0, 200);
    let archived = get()["lifecycle"].clone();
    let dump = api(port, "GET", "/api/export", None).1;
    let restore_path = temp_db("lifecycle-restore");
    cleanup(&restore_path);
    let restore = HttpProc::start(&restore_path, "lifecycle-restore");
    assert_eq!(api(restore.port, "POST", "/api/import", Some(dump)).0, 200);
    assert_eq!(
        api(restore.port, "GET", "/api/memories", None).1["total"],
        0
    );
    let listing = api(restore.port, "GET", "/api/memories?state=all", None).1;
    let restored_id = listing["memories"][0]["id"].as_str().unwrap();
    assert_eq!(
        api(
            restore.port,
            "GET",
            &format!("/api/memories/{restored_id}"),
            None
        )
        .1["lifecycle"],
        archived
    );
    drop(restore);
    cleanup(&restore_path);
    drop(server);
    cleanup(&path);
}
