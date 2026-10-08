//! Full token auth flow: open mode -> create admin identity -> explicitly flip the auth switch ->
//! 401/403/capability boundaries -> token reset fallback -> flip the switch back to open mode.

use serde_json::json;

use crate::common::{
    cleanup, json_body, mcp_post, request, run_cli, temp_db, try_request, HttpProc,
};

#[test]
fn token_auth_end_to_end() {
    let db = temp_db("auth");
    cleanup(&db);
    // Start in open mode (zero-config form of a personal deployment)
    let server = HttpProc::start(&db, "auth");
    let port = server.port;

    // Open mode: usable without a token, whoami reports open
    let (status, body, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["mode"], "open");
    let (status, _, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(r#"{"summary": "open-mode write", "content": "c"}"#),
    );
    assert_eq!(status, 200);

    // Guard: enabling auth is rejected when no admin identity exists (otherwise nothing could reach the admin surface afterwards)
    let (status, body, _) = request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": true}"#),
    );
    let guard_msg = String::from_utf8_lossy(&body);
    assert_eq!(status, 400, "{guard_msg}");
    assert!(
        guard_msg.contains("no admin identity exists"),
        "guard error must explain the precondition: {guard_msg}"
    );

    // Also rejected with only a non-admin identity (without an admin, token reset could not recover either)
    let (status, _, _) = request(
        port,
        "POST",
        "/api/identities",
        Some(r#"{"name": "lone-viewer", "permissions": {"read": true}}"#),
    );
    assert_eq!(status, 200);
    let (status, body, _) = request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": true}"#),
    );
    assert_eq!(status, 400, "{}", String::from_utf8_lossy(&body));

    // Creating an identity in open mode does not change auth state (the switch is the single source of truth)
    let (status, body, _) = request(
        port,
        "POST",
        "/api/identities",
        Some(
            r#"{"name": "admin", "permissions": {"read": true, "create": true, "update": true, "delete": true, "tag_manage": true, "admin": true}}"#,
        ),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    let admin_token = json_body(&body)["token"].as_str().unwrap().to_string();
    assert_eq!(admin_token.len(), 65);
    assert!(admin_token.starts_with("sk_"), "token = {admin_token}");
    let admin_auth = format!("Bearer {admin_token}");
    let admin_headers = [("Authorization", admin_auth.as_str())];
    let (status, body, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 200);
    assert_eq!(
        json_body(&body)["mode"],
        "open",
        "identity alone must not enable auth"
    );

    // Explicitly turn on the auth switch (open context holds admin capability) -> auth takes effect immediately
    let (status, _, _) = request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": true}"#),
    );
    assert_eq!(status, 200);
    let (status, body, _) =
        try_request(port, "GET", "/api/settings", None, &admin_headers).unwrap();
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["auth_required"], true);
    let (status, _, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 401);

    // Remaining identities are created by the admin with its token.
    let (status, body, _) = try_request(
        port,
        "POST",
        "/api/identities",
        Some(r#"{"name": "viewer", "permissions": {"read": true}}"#),
        &admin_headers,
    )
    .unwrap();
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    let viewer_token = json_body(&body)["token"].as_str().unwrap().to_string();

    // Auth takes effect immediately: no token / wrong token / non-Bearer scheme -> 401
    let (status, _, _) = request(port, "GET", "/api/memories", None);
    assert_eq!(status, 401);
    let (status, _, _) = request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#),
    );
    assert_eq!(status, 401);
    let (status, _, _) = try_request(
        port,
        "GET",
        "/api/memories",
        None,
        &[("Authorization", "Bearer wrong-token")],
    )
    .unwrap();
    assert_eq!(status, 401);
    let (status, _, _) = try_request(
        port,
        "GET",
        "/api/memories",
        None,
        &[("Authorization", "Basic dXNlcjpwYXNz")],
    )
    .unwrap();
    assert_eq!(status, 401);

    // Static UI and /health are always exempt from auth
    let (status, _, _) = request(port, "GET", "/", None);
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "GET", "/health", None);
    assert_eq!(status, 200);

    // Read-only identity: reads OK; writes/admin 403; MCP writes report the permission error via isError
    let viewer_auth = format!("Bearer {viewer_token}");
    let viewer_headers = [("Authorization", viewer_auth.as_str())];
    assert_eq!(
        try_request(port, "GET", "/api/tag-rules", None, &viewer_headers)
            .unwrap()
            .0,
        200
    );
    for (method, path) in [
        ("PUT", "/api/tag-rules"),
        ("POST", "/api/tag-rules/preview"),
    ] {
        assert_eq!(
            try_request(
                port,
                method,
                path,
                Some(r#"{"constraints": []}"#),
                &viewer_headers
            )
            .unwrap()
            .0,
            403
        );
    }
    let (status, _, _) = try_request(port, "GET", "/api/memories", None, &viewer_headers).unwrap();
    assert_eq!(status, 200);
    let (status, body, _) = try_request(
        port,
        "POST",
        "/api/memories",
        Some(r#"{"summary": "nope", "content": "c"}"#),
        &viewer_headers,
    )
    .unwrap();
    assert_eq!(status, 403, "{}", String::from_utf8_lossy(&body));
    let (status, _, _) =
        try_request(port, "GET", "/api/identities", None, &viewer_headers).unwrap();
    assert_eq!(status, 403);
    let (status, _, _) = try_request(port, "GET", "/api/export", None, &viewer_headers).unwrap();
    assert_eq!(status, 403);
    let (status, resp) = mcp_post(
        port,
        json!(2),
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "s", "content": "c"}}),
        &[("Authorization", viewer_auth.as_str())],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["isError"], true);
    assert!(
        resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("permission"),
        "permission error text: {resp}"
    );

    // Admin identity: everything passes
    let (status, _, _) = try_request(
        port,
        "POST",
        "/api/memories",
        Some(r#"{"summary": "by admin", "content": "c"}"#),
        &admin_headers,
    )
    .unwrap();
    assert_eq!(status, 200);
    let (status, body, _) =
        try_request(port, "GET", "/api/identities", None, &admin_headers).unwrap();
    assert_eq!(status, 200);
    // three in total: admin, lone-viewer (created by the guard case), viewer
    assert_eq!(json_body(&body)["identities"].as_array().unwrap().len(), 3);
    let (status, _, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"instructions": "team rules"}"#),
        &admin_headers,
    )
    .unwrap();
    assert_eq!(status, 200);
    let (status, body, _) =
        try_request(port, "GET", "/api/settings", None, &admin_headers).unwrap();
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["instructions"], "team rules");
    let (status, _, _) = try_request(port, "GET", "/api/export", None, &admin_headers).unwrap();
    assert_eq!(status, 200);
    // discover's instructions carry the custom prompt and the identity line
    let (status, discover) = mcp_post(
        port,
        json!(3),
        "server/discover",
        json!({}),
        &[("Authorization", admin_auth.as_str())],
    );
    assert_eq!(status, 200);
    let instructions = discover["result"]["instructions"].as_str().unwrap();
    assert!(instructions.contains("team rules"), "{instructions}");
    assert!(
        instructions.contains("Caller identity: admin"),
        "{instructions}"
    );
    // whoami reports token mode and identity name
    let (status, body, _) = try_request(port, "GET", "/api/whoami", None, &admin_headers).unwrap();
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["mode"], "token");
    assert_eq!(json_body(&body)["name"], "admin");

    // token reset fallback: CLI resets admin, old token becomes invalid immediately
    let out = run_cli(&["token", "reset", "--db", &db.display().to_string(), "admin"]);
    assert!(out.status.success(), "token reset failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let new_token = stdout.lines().last().unwrap().trim().to_string();
    assert_eq!(new_token.len(), 65, "reset stdout: {stdout}");
    assert!(new_token.starts_with("sk_"), "reset stdout: {stdout}");
    let (status, _, _) = try_request(
        port,
        "GET",
        "/api/memories",
        None,
        &[("Authorization", admin_auth.as_str())],
    )
    .unwrap();
    assert_eq!(status, 401, "old admin token must be revoked");
    let fresh_admin = format!("Bearer {new_token}");
    let (status, _, _) = try_request(
        port,
        "GET",
        "/api/memories",
        None,
        &[("Authorization", fresh_admin.as_str())],
    )
    .unwrap();
    assert_eq!(status, 200);

    // Turn the auth switch off (rather than deleting identities) -> back to open mode; identities remain in the db
    let (status, _, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": false}"#),
        &[("Authorization", fresh_admin.as_str())],
    )
    .unwrap();
    assert_eq!(status, 200);
    let (status, body, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["mode"], "open");
    let (status, _, _) = request(port, "GET", "/api/memories", None);
    assert_eq!(status, 200, "open mode must not require tokens");

    drop(server);
    cleanup(&db);
}

/// Anonymous identity: once auth is on, token-less requests resolve to the anonymous identity
/// per the settings' anonymous_permissions (capabilities configurable); unset or empty -> 401;
/// an invalid token does not fall back to anonymous; insufficient capability -> 403 (same semantics as other identities).
#[test]
fn anonymous_permissions_end_to_end() {
    let db = temp_db("anon-auth");
    cleanup(&db);
    let server = HttpProc::start(&db, "anon-auth");
    let port = server.port;

    // Setup: admin identity + auth on (afterwards only the admin can change anonymous config)
    let (status, body, _) = request(
        port,
        "POST",
        "/api/identities",
        Some(
            r#"{"name": "admin", "permissions": {"read": true, "create": true, "update": true, "delete": true, "tag_manage": true, "admin": true}}"#,
        ),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    let admin_token = json_body(&body)["token"].as_str().unwrap().to_string();
    let admin_auth = format!("Bearer {admin_token}");
    let admin_headers = [("Authorization", admin_auth.as_str())];
    let (status, _, _) = request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"auth_required": true}"#),
    );
    assert_eq!(status, 200);

    // Baseline: anonymous capability set not configured -> token-less requests always 401 (original behavior)
    let (status, _, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 401);
    let (status, _, _) = request(port, "GET", "/api/memories", None);
    assert_eq!(status, 401);

    // Configure anonymous = read-only
    let (status, _, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"anonymous_permissions": {"read": true}}"#),
        &admin_headers,
    )
    .unwrap();
    assert_eq!(status, 200);

    // Token-less whoami -> anonymous identity summary
    let (status, body, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 200);
    let who = json_body(&body);
    assert_eq!(who["mode"], "anonymous");
    assert_eq!(who["name"], "anonymous");
    assert_eq!(who["permissions"]["read"], true);
    assert_eq!(who["permissions"]["create"], false);

    // Reads pass; writes/admin -> 403 (error message names the anonymous identity)
    let (status, _, _) = request(port, "GET", "/api/memories", None);
    assert_eq!(status, 200);
    let (status, body, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(r#"{"summary": "anon write", "content": "c"}"#),
    );
    assert_eq!(status, 403, "{}", String::from_utf8_lossy(&body));
    assert!(
        String::from_utf8_lossy(&body).contains("anonymous"),
        "403 must name the anonymous identity"
    );
    let (status, _, _) = request(port, "GET", "/api/identities", None);
    assert_eq!(status, 403);
    let (status, _, _) = request(port, "GET", "/api/export", None);
    assert_eq!(status, 403);

    // MCP has the same semantics: discover self-reports anonymous; list passes; create reports the permission error via isError
    let (status, discover) = mcp_post(port, json!(1), "server/discover", json!({}), &[]);
    assert_eq!(status, 200);
    let instructions = discover["result"]["instructions"].as_str().unwrap();
    assert!(
        instructions.contains("Access mode: anonymous"),
        "{instructions}"
    );
    let (status, resp) = mcp_post(
        port,
        json!(2),
        "tools/call",
        json!({"name": "memory_list", "arguments": {}}),
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["isError"], json!(null));
    let (status, resp) = mcp_post(
        port,
        json!(3),
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "s", "content": "c"}}),
        &[],
    );
    assert_eq!(status, 200);
    assert_eq!(resp["result"]["isError"], true);
    assert!(
        resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("permission"),
        "{resp}"
    );

    // An invalid token is an authentication failure, no anonymous fallback
    let (status, _, _) = try_request(
        port,
        "GET",
        "/api/memories",
        None,
        &[("Authorization", "Bearer wrong-token")],
    )
    .unwrap();
    assert_eq!(status, 401);

    // Empty capability set = anonymous rejected wholesale (same as unset)
    let (status, _, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"anonymous_permissions": {"read": false}}"#),
        &admin_headers,
    )
    .unwrap();
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "GET", "/api/memories", None);
    assert_eq!(status, 401);

    // null clears the setting -> back to baseline 401; admin itself is unaffected
    let (status, _, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"anonymous_permissions": null}"#),
        &admin_headers,
    )
    .unwrap();
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 401);
    let (status, _, _) = try_request(port, "GET", "/api/memories", None, &admin_headers).unwrap();
    assert_eq!(status, 200);

    drop(server);
    cleanup(&db);
}
