//! Full flow of the admin backend /api/* and the statically served admin UI.

use serde_json::Value;

use crate::common::{cleanup, encodeURIComponent, json_body, request, temp_db, HttpProc};

#[test]
fn rest_api_end_to_end() {
    let db = temp_db("rest");
    cleanup(&db);
    let server = HttpProc::start(&db, "rest");
    let port = server.port;

    // Tags: create (including non-ASCII path escaping) -> list -> exact duplicate name error -> rename
    let (status, body, _) = request(
        port,
        "POST",
        "/api/tags",
        Some(r#"{"name": "项目", "description": "与项目相关的长期事实"}"#),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    let (status, _, _) = request(port, "POST", "/api/tags", Some(r#"{"name": "rust"}"#));
    assert_eq!(status, 200);
    let (status, body, _) = request(port, "POST", "/api/tags", Some(r#"{"name": "rust"}"#));
    assert_eq!(status, 400);
    assert!(json_body(&body)["error"]
        .as_str()
        .unwrap()
        .contains("already exists"));

    let (status, body, _) = request(port, "GET", "/api/tags", None);
    assert_eq!(status, 200);
    // 2 created + the reserved tag, which tag_list always surfaces (synthesized placeholder)
    let listed = json_body(&body);
    assert_eq!(listed["tags"].as_array().unwrap().len(), 3);
    assert_eq!(listed["total_tags"], 3);
    let conv = listed["tags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "conventions")
        .expect("reserved tag listed");
    assert_eq!(conv["reserved"], true);
    assert_eq!(conv["memory_count"], 0);

    // Tag regex filter: match / no match / invalid regex -> 400
    let (status, body, _) = request(port, "GET", "/api/tags?filter=%5E%E9%A1%B9", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["tags"].as_array().unwrap().len(), 1);
    let (status, body, _) = request(port, "GET", "/api/tags?filter=zzz", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["tags"].as_array().unwrap().len(), 0);
    let (status, body, _) = request(port, "GET", "/api/tags?filter=%28", None);
    assert_eq!(status, 400);
    assert!(json_body(&body)["error"]
        .as_str()
        .unwrap()
        .contains("not a valid regular expression"));

    let (status, body, _) = request(
        port,
        "PUT",
        &format!("/api/tags/{}", encodeURIComponent("项目")),
        Some(r#"{"new_name": "项目A", "description": "改名后的描述"}"#),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    assert_eq!(json_body(&body)["name"], "项目A");
    assert_eq!(json_body(&body)["renamed"], true);
    assert_eq!(json_body(&body)["description_updated"], true);

    // Memories: create -> list/filter -> fetch full text -> update -> search -> delete
    let (status, body, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(
            r#"{"summary": "用户偏好深色主题", "content": "2026-09 确认", "tags": ["项目A", "偏好"], "create_missing_tags": true}"#,
        ),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    let mem_id = json_body(&body)["memory"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, body, _) = request(port, "GET", "/api/memories?limit=10", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total"], 1);

    let (status, body, _) = request(
        port,
        "GET",
        &format!("/api/memories?tag={}&limit=10", encodeURIComponent("偏好")),
        None,
    );
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total"], 1);

    // Memory tag regex filter: match / ANDed with an exact tag yields empty
    let (status, body, _) = request(
        port,
        "GET",
        &format!(
            "/api/memories?tag_filter={}&limit=10",
            encodeURIComponent("^项目")
        ),
        None,
    );
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total"], 1);
    let (status, body, _) = request(
        port,
        "GET",
        &format!(
            "/api/memories?tag={}&tag_filter={}&limit=10",
            encodeURIComponent("偏好"),
            encodeURIComponent("^外观")
        ),
        None,
    );
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total"], 0);

    let (status, body, _) = request(port, "GET", &format!("/api/memories/{mem_id}"), None);
    assert_eq!(status, 200);
    assert!(json_body(&body)["content"]
        .as_str()
        .unwrap()
        .contains("2026"));

    let (status, _, _) = request(
        port,
        "PUT",
        &format!("/api/memories/{mem_id}"),
        Some(
            r#"{"summary": "用户偏好浅色主题", "remove_tags": ["偏好"], "add_tags": ["外观"], "create_missing_tags": true}"#,
        ),
    );
    assert_eq!(status, 200);

    let (status, body, _) = request(
        port,
        "GET",
        &format!("/api/memories?query={}", encodeURIComponent("浅色")),
        None,
    );
    assert_eq!(status, 200);
    let searched = json_body(&body);
    let results = searched["results"].as_array().unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        results[0].get("content").is_none(),
        "search must not leak content"
    );

    // Nonexistent memory -> 404
    let (status, _, _) = request(port, "GET", "/api/memories/m999", None);
    assert_eq!(status, 404);

    // stats / doctor
    let (status, body, _) = request(port, "GET", "/api/stats", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["memories"], 1);
    let (status, body, _) = request(port, "GET", "/api/doctor", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["ok"], true);

    // export: downloaded as an attachment
    let (status, body, ctype) = request(port, "GET", "/api/export", None);
    assert_eq!(status, 200);
    assert!(ctype.contains("application/json"));
    let dump: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dump["memories"].as_object().unwrap().len(), 1);

    // Delete memory -> subsequent fetch 404
    let (status, _, _) = request(port, "DELETE", &format!("/api/memories/{mem_id}"), None);
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "GET", &format!("/api/memories/{mem_id}"), None);
    assert_eq!(status, 404);

    // purging a tag also deletes its memories
    let (status, _, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(
            r#"{"summary": "临时记忆", "content": "x", "tags": ["临时"], "create_missing_tags": true}"#,
        ),
    );
    assert_eq!(status, 200);
    let (status, body, _) = request(
        port,
        "DELETE",
        &format!("/api/tags/{}?mode=purge", encodeURIComponent("临时")),
        None,
    );
    assert_eq!(status, 200);
    assert_eq!(
        json_body(&body)["memories_deleted"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let (_, body, _) = request(port, "GET", "/api/stats", None);
    assert_eq!(json_body(&body)["memories"], 0);

    drop(server);
    cleanup(&db);
}

#[test]
fn static_ui_is_served() {
    let db = temp_db("ui");
    cleanup(&db);
    let server = HttpProc::start(&db, "ui");
    let port = server.port;

    // Root path returns HTML (embedded assets; a placeholder page before build, the Vue app after)
    let (status, body, ctype) = request(port, "GET", "/", None);
    assert_eq!(status, 200);
    assert!(ctype.contains("text/html"), "content-type: {ctype}");
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("<html") || html.contains("<!doctype"),
        "not html"
    );

    // SPA fallback: unknown non-API paths also serve index
    let (status, _, ctype2) = request(port, "GET", "/some/spa/route", None);
    assert_eq!(status, 200);
    assert!(ctype2.contains("text/html"));

    // /health is JSON rather than the SPA fallback
    let (status, body, ctype3) = request(port, "GET", "/health", None);
    assert_eq!(status, 200);
    assert!(ctype3.contains("application/json"));
    assert_eq!(json_body(&body)["status"], "ok");

    drop(server);
    cleanup(&db);
}
