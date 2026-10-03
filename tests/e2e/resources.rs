//! End-to-end flow of the `memory://` resource surface: catalog, templates, reads, progressive
//! disclosure and the permission mask.

use serde_json::{json, Value};

use crate::common::{cleanup, mcp_post, temp_db, HttpProc};

/// Create a memory through the tool surface and return its id.
fn create_memory(port: u16, summary: &str, content: &str, tags: &[&str]) -> String {
    let (_, resp) = mcp_post(
        port,
        json!(1),
        "tools/call",
        json!({"name": "memory_create", "arguments": {
            "summary": summary, "content": content, "tags": tags, "create_missing_tags": true
        }}),
        &[],
    );
    resp["result"]["structuredContent"]["memory"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn resources_end_to_end() {
    let db = temp_db("resources");
    cleanup(&db);
    let server = HttpProc::start(&db, "resources");
    let port = server.port;

    // Seed: two plain tags plus one resident conventions memory
    let plain_id = create_memory(
        port,
        "Borrow checker notes",
        "The borrow checker forbids aliasing plus mutation.",
        &["rust", "notes"],
    );
    let conv_id = create_memory(
        port,
        "Commit rules",
        "Summary in one line; tags lowercase.",
        &["conventions"],
    );

    // resources/templates/list: the two memory:// templates, publicly cacheable
    let (status, templates) = mcp_post(port, json!(2), "resources/templates/list", json!({}), &[]);
    assert_eq!(status, 200);
    assert_eq!(templates["result"]["resultType"], "complete");
    assert_eq!(templates["result"]["cacheScope"], "public");
    let list_of_templates = templates["result"]["resourceTemplates"].as_array().unwrap();
    assert_eq!(list_of_templates.len(), 2);
    assert_eq!(list_of_templates[0]["uriTemplate"], "memory://tags/{tag}");
    assert_eq!(
        list_of_templates[1]["uriTemplate"],
        "memory://memories/{id}"
    );

    // resources/list: one entry per tag + one per conventions memory; no content anywhere
    let (status, listing) = mcp_post(port, json!(3), "resources/list", json!({}), &[]);
    assert_eq!(status, 200);
    assert_eq!(listing["result"]["resultType"], "complete");
    assert_eq!(listing["result"]["ttlMs"], 30_000);
    assert_eq!(listing["result"]["cacheScope"], "private");
    let resources = listing["result"]["resources"].as_array().unwrap();
    // 3 tags (conventions auto-created) + 1 conventions memory
    assert_eq!(resources.len(), 4);
    let rust = resources
        .iter()
        .find(|r| r["uri"] == "memory://tags/rust")
        .expect("tag resource listed");
    assert_eq!(rust["name"], "rust");
    assert!(
        rust.get("text").is_none(),
        "catalog entries carry no content"
    );
    let conv = resources
        .iter()
        .find(|r| r["uri"] == format!("memory://memories/{conv_id}"))
        .expect("conventions memory listed");
    assert_eq!(conv["name"], "Commit rules");
    assert_eq!(conv["annotations"]["priority"], 1.0);
    assert_eq!(conv["annotations"]["audience"], json!(["assistant"]));

    // resources/read on a tag: JSON catalog with summaries but never content
    let (status, tag_read) = mcp_post(
        port,
        json!(4),
        "resources/read",
        json!({"uri": "memory://tags/rust"}),
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(tag_read["result"]["ttlMs"], 0);
    let content = &tag_read["result"]["contents"][0];
    assert_eq!(content["uri"], "memory://tags/rust");
    assert_eq!(content["mimeType"], "application/json");
    let doc: Value = serde_json::from_str(content["text"].as_str().unwrap()).unwrap();
    assert_eq!(doc["name"], "rust");
    assert_eq!(doc["memory_count"], 1);
    assert_eq!(doc["memories"][0]["id"], plain_id.as_str());
    assert!(
        !content["text"]
            .as_str()
            .unwrap()
            .contains("borrow checker forbids"),
        "tag resources must not leak memory content"
    );

    // resources/read on a memory: the full body as Markdown (progressive disclosure level 2)
    let uri = format!("memory://memories/{plain_id}");
    let (status, mem_read) = mcp_post(port, json!(5), "resources/read", json!({"uri": uri}), &[]);
    assert_eq!(status, 200);
    let content = &mem_read["result"]["contents"][0];
    assert_eq!(content["mimeType"], "text/markdown");
    assert_eq!(
        content["text"],
        "The borrow checker forbids aliasing plus mutation."
    );
    assert!(
        content["annotations"]["lastModified"]
            .as_str()
            .unwrap()
            .ends_with('Z'),
        "lastModified is ISO 8601: {content}"
    );

    // Missing and malformed targets: -32602 with data.uri, never empty contents
    let (status, err) = mcp_post(
        port,
        json!(6),
        "resources/read",
        json!({"uri": "memory://memories/m999"}),
        &[],
    );
    assert_eq!(status, 400);
    assert_eq!(err["error"]["code"], -32602);
    assert_eq!(err["error"]["data"]["uri"], "memory://memories/m999");
    let (status, err) = mcp_post(
        port,
        json!(7),
        "resources/read",
        json!({"uri": "memory://nope/x"}),
        &[],
    );
    assert_eq!(status, 400);
    assert_eq!(err["error"]["code"], -32602);

    // Unknown method resources/subscribe (removed in the modern protocol) → 404 + -32601
    let (status, err) = mcp_post(port, json!(8), "resources/subscribe", json!({}), &[]);
    assert_eq!(status, 404);
    assert_eq!(err["error"]["code"], -32601);

    drop(server);
    cleanup(&db);
}

/// Permission mask: a read-capability-less caller gets an empty catalog and not-found reads.
#[test]
fn resources_masked_without_read_capability() {
    let db = temp_db("resources-mask");
    cleanup(&db);
    let server = HttpProc::start(&db, "resources-mask");
    let port = server.port;

    // Setup: admin identity + auth on + anonymous = create-only
    let (status, body, _) = crate::common::request(
        port,
        "POST",
        "/api/identities",
        Some(
            r#"{"name": "admin", "permissions": {"read": true, "create": true, "update": true, "delete": true, "tag_manage": true, "admin": true}}"#,
        ),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    let admin_token = crate::common::json_body(&body)["token"]
        .as_str()
        .unwrap()
        .to_string();
    let admin = format!("Bearer {admin_token}");
    let (status, _, _) = crate::common::request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": true}"#),
    );
    assert_eq!(status, 200, "open mode holds admin capabilities");
    let (status, _, _) = crate::common::try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"anonymous_permissions": {"create": true}}"#),
        &[("Authorization", admin.as_str())],
    )
    .unwrap();
    assert_eq!(status, 200);

    // Seed through the admin token so data exists but is invisible to the anonymous caller
    create_memory_with(port, "secret note", "top secret body", &["rust"], &admin);

    // Anonymous (create-only): list is empty, reads answer not-found without leaking existence
    let (status, listing) = mcp_post(port, json!(1), "resources/list", json!({}), &[]);
    assert_eq!(status, 200);
    assert_eq!(listing["result"]["resources"].as_array().unwrap().len(), 0);
    let (status, err) = mcp_post(
        port,
        json!(2),
        "resources/read",
        json!({"uri": "memory://tags/rust"}),
        &[],
    );
    assert_eq!(status, 400);
    assert_eq!(err["error"]["code"], -32602);
    assert!(err["error"]["message"]
        .as_str()
        .unwrap()
        .contains("not found"));

    // With the read capability the same caller-visible store shows the resources
    let (status, listing) = mcp_post(
        port,
        json!(3),
        "resources/list",
        json!({}),
        &[("Authorization", admin.as_str())],
    );
    assert_eq!(status, 200);
    assert!(
        listing["result"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["uri"] == "memory://tags/rust"),
        "{listing}"
    );

    drop(server);
    cleanup(&db);
}

fn create_memory_with(port: u16, summary: &str, content: &str, tags: &[&str], auth: &str) {
    let (status, resp) = mcp_post(
        port,
        json!(1),
        "tools/call",
        json!({"name": "memory_create", "arguments": {
            "summary": summary, "content": content, "tags": tags, "create_missing_tags": true
        }}),
        &[("Authorization", auth)],
    );
    assert_eq!(status, 200, "{resp}");
    assert_eq!(resp["result"]["isError"], json!(null), "{resp}");
}
