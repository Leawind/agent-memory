//! Read-side default filter: an identity bound to a scoped tag sees that tag's holder plus the
//! memories carrying no tag in its scope, and the response names the filter it ran under. An
//! explicit tag_expr replaces the default outright; reading by id is not filtered.

use serde_json::{json, Value};

use crate::common::{
    cleanup, json_body, mcp_post, request, temp_db, tool_data, try_request, HttpProc,
};

/// The rendered tool result text (the single data channel of a successful tools/call).
fn text(resp: &Value) -> String {
    resp["result"]["content"][0]["text"]
        .as_str()
        .expect("tool result carries a text content block")
        .to_string()
}

fn tool(port: u16, token: Option<&str>, name: &str, arguments: Value) -> Value {
    let auth = token.map(|t| format!("Bearer {t}"));
    let headers: Vec<(&str, &str)> = auth
        .as_deref()
        .map(|a| ("Authorization", a))
        .into_iter()
        .collect();
    let (status, resp) = mcp_post(
        port,
        json!(1),
        "tools/call",
        json!({ "name": name, "arguments": arguments }),
        &headers,
    );
    assert_eq!(status, 200, "{name}: {resp}");
    resp
}

/// PUT an identity with an admin token: body is the `tag` value (a JSON literal).
fn set_binding(port: u16, admin: &str, name: &str, tag: &str) -> (u16, String) {
    let auth = format!("Bearer {admin}");
    let path = format!("/api/identities/{name}");
    let payload = format!(r#"{{"permissions": {{"read": true}}, "tag": {tag}}}"#);
    let (status, body, _) = try_request(
        port,
        "PUT",
        &path,
        Some(&payload),
        &[("Authorization", auth.as_str())],
    )
    .expect("http request succeeds");
    (status, String::from_utf8_lossy(&body).into_owned())
}

#[test]
fn a_bound_identity_reads_its_own_value_plus_shared() {
    let db = temp_db("scope-filter");
    cleanup(&db);
    let server = HttpProc::start(&db, "scope-filter");
    let port = server.port;

    // Three memories: one about one machine, one about another, one stating no value at all
    let mut ids = Vec::new();
    for (summary, system) in [
        ("pc1 上的构建配置", Some("pc1sw0")),
        ("lp2 上的构建配置", Some("lp2sw0")),
        ("构建配置的通用约定", None),
    ] {
        let mut arguments = json!({
            "summary": summary,
            "content": "c",
            "create_missing_tags": true,
        });
        if let Some(value) = system {
            arguments["attributes"] = json!({ "system": value });
        }
        let created = tool(port, None, "memory_create", arguments);
        ids.push(tool_data(&created)["id"].as_str().unwrap().to_string());
    }

    // Open mode trusts everyone, so the operator sets up identities and turns auth on
    let (status, body, _) = request(
        port,
        "POST",
        "/api/identities",
        Some(r#"{"name": "operator", "permissions": {"read": true, "admin": true}}"#),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    let admin = json_body(&body)["token"].as_str().unwrap().to_string();
    let admin_auth = format!("Bearer {admin}");
    let (status, body, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": true}"#),
        &[("Authorization", admin_auth.as_str())],
    )
    .expect("http request succeeds");
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));

    // A bare tag cannot serve as a binding: "picked no value" needs a scope to be absent from
    let (status, body, _) = try_request(
        port,
        "POST",
        "/api/identities",
        Some(r#"{"name": "box", "permissions": {"read": true}, "tag": "rust"}"#),
        &[("Authorization", admin_auth.as_str())],
    )
    .expect("http request succeeds");
    assert_eq!(status, 400, "{}", String::from_utf8_lossy(&body));

    let (status, body, _) = try_request(
        port,
        "POST",
        "/api/identities",
        Some(r#"{"name": "box", "permissions": {"read": true}, "tag": "system::pc1sw0"}"#),
        &[("Authorization", admin_auth.as_str())],
    )
    .expect("http request succeeds");
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    let created = json_body(&body);
    assert_eq!(created["tag"], "system::pc1sw0");
    let boxed = created["token"].as_str().unwrap().to_string();

    // The default view: the caller's own value plus everything that says nothing about a machine
    let listed = text(&tool(port, Some(&boxed), "memory_list", json!({})));
    assert!(
        listed.starts_with("total: 2 | offset: 0\nfilter: !system::*|system::pc1sw0\n"),
        "default filter must be stated: {listed}"
    );
    assert!(listed.contains("pc1 上的构建配置"), "{listed}");
    assert!(listed.contains("构建配置的通用约定"), "{listed}");
    assert!(!listed.contains("lp2 上的构建配置"), "{listed}");

    // Search runs under the same default
    let searched = text(&tool(
        port,
        Some(&boxed),
        "memory_search",
        json!({"query": "构建配置"}),
    ));
    assert!(
        searched.contains("filter: !system::*|system::pc1sw0"),
        "{searched}"
    );
    assert!(!searched.contains("lp2 上的构建配置"), "{searched}");

    // An explicit expression replaces the default, so the other machine stays reachable
    let other = text(&tool(
        port,
        Some(&boxed),
        "memory_list",
        json!({"tag_expr": "system::lp2sw0"}),
    ));
    assert!(!other.contains("filter:"), "{other}");
    assert!(other.contains("lp2 上的构建配置"), "{other}");
    assert!(!other.contains("构建配置的通用约定"), "{other}");

    // Reading by id is an explicit act: the other machine's memory is still there
    let read = tool_data(&tool(
        port,
        Some(&boxed),
        "memory_get",
        json!({"ids": [ids[1]]}),
    ));
    assert_eq!(read["memories"][0]["summary"], "lp2 上的构建配置");

    // An unbound caller keeps the whole store: the filter follows the binding, not the person
    let admin_view = text(&tool(port, Some(&admin), "memory_list", json!({})));
    assert!(!admin_view.contains("filter:"), "{admin_view}");
    assert!(admin_view.contains("lp2 上的构建配置"), "{admin_view}");

    // Clearing the binding restores the unfiltered view for that identity
    let (status, body) = set_binding(port, &admin, "box", "null");
    assert_eq!(status, 200, "{body}");
    let cleared = text(&tool(port, Some(&boxed), "memory_list", json!({})));
    assert!(!cleared.contains("filter:"), "{cleared}");
    assert!(cleared.contains("lp2 上的构建配置"), "{cleared}");

    drop(server);
    cleanup(&db);
}
