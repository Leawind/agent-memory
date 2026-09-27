//! token 鉴权全流程：开放模式 → 创建管理员身份 → 显式打开鉴权开关 →
//! 401/403/能力边界 → token reset 兜底 → 关闭开关回到开放模式。

use serde_json::json;

use crate::common::{cleanup, json_body, request, run_cli, temp_db, try_request, HttpProc};

#[test]
fn token_auth_end_to_end() {
    let db = temp_db("auth");
    cleanup(&db);
    // 开放模式启动（个人部署零配置形态）
    let server = HttpProc::start(&db, "auth");
    let port = server.port;

    // 开放模式：免 token 可用，whoami 报告 open
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

    // 守卫：无 admin 身份时开启鉴权被拒绝（否则开启后无人能再访问管理面）
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

    // 只有非 admin 身份同样拒绝（admin 缺失时 token reset 也无从恢复）
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

    // 开放模式下创建身份不改变鉴权状态（开关才是唯一事实源）
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

    // 显式打开鉴权开关（open 上下文具备 admin 能力）→ 鉴权即刻生效
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

    // 其余身份由管理员凭 token 创建。
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

    // 鉴权立即生效：无 token / 错 token / 非 Bearer 方案 → 401
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

    // 静态 UI 与 /health 始终免鉴权
    let (status, _, _) = request(port, "GET", "/", None);
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "GET", "/health", None);
    assert_eq!(status, 200);

    // 只读身份：读 OK；写/管理 403；MCP 写以 isError 回显权限错误
    let viewer_auth = format!("Bearer {viewer_token}");
    let viewer_headers = [("Authorization", viewer_auth.as_str())];
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
    let (status, body, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(
            &serde_json::to_string(&json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/call",
                "params": {"name": "memory_create", "arguments": {"summary": "s", "content": "c"}}
            }))
            .unwrap(),
        ),
        &viewer_headers,
    )
    .unwrap();
    assert_eq!(status, 200);
    let resp = json_body(&body);
    assert_eq!(resp["result"]["isError"], true);
    assert!(
        resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("permission"),
        "permission error text: {resp}"
    );

    // 管理员身份：全通
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
    // admin、lone-viewer（守卫用例所建）、viewer 共三个
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
    // initialize 的 instructions 携带自定义文案与身份行
    let (status, body, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(
            &serde_json::to_string(&json!({
                "jsonrpc": "2.0", "id": 3, "method": "initialize",
                "params": {"protocolVersion": "2025-06-18"}
            }))
            .unwrap(),
        ),
        &admin_headers,
    )
    .unwrap();
    assert_eq!(status, 200);
    let init_resp = json_body(&body);
    let instructions = init_resp["result"]["instructions"].as_str().unwrap();
    assert!(instructions.contains("team rules"), "{instructions}");
    assert!(
        instructions.contains("Caller identity: admin"),
        "{instructions}"
    );
    // whoami 报告 token 模式与身份名
    let (status, body, _) = try_request(port, "GET", "/api/whoami", None, &admin_headers).unwrap();
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["mode"], "token");
    assert_eq!(json_body(&body)["name"], "admin");

    // token reset 兜底：CLI 重置 admin，旧 token 立即失效
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

    // 关闭鉴权开关（而非删除身份）→ 回到开放模式；身份仍在库里
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

/// 匿名身份：鉴权开启后，无 token 请求按 settings 的 anonymous_permissions
/// 解析为匿名身份（能力可配置）；未设置/全无能力 → 401；带无效 token 不回退
/// 匿名；能力不足 → 403（与其他身份同一套语义）。
#[test]
fn anonymous_permissions_end_to_end() {
    let db = temp_db("anon-auth");
    cleanup(&db);
    let server = HttpProc::start(&db, "anon-auth");
    let port = server.port;

    // 准备：admin 身份 + 打开鉴权（此后匿名配置只能由 admin 修改）
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

    // 基线：未配置匿名能力集 → 无 token 一律 401（原有行为）
    let (status, _, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 401);
    let (status, _, _) = request(port, "GET", "/api/memories", None);
    assert_eq!(status, 401);

    // 配置匿名 = 只读
    let (status, _, _) = try_request(
        port,
        "PUT",
        "/api/settings",
        Some(r#"{"anonymous_permissions": {"read": true}}"#),
        &admin_headers,
    )
    .unwrap();
    assert_eq!(status, 200);

    // 无 token whoami → 匿名身份摘要
    let (status, body, _) = request(port, "GET", "/api/whoami", None);
    assert_eq!(status, 200);
    let who = json_body(&body);
    assert_eq!(who["mode"], "anonymous");
    assert_eq!(who["name"], "anonymous");
    assert_eq!(who["permissions"]["read"], true);
    assert_eq!(who["permissions"]["create"], false);

    // 只读放行；写/管理 → 403（错误信息带匿名身份名）
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

    // MCP 同语义：initialize 自述匿名；list 放行；create 以 isError 回显权限错误
    let (status, body, _) = request(
        port,
        "POST",
        "/mcp",
        Some(
            &serde_json::to_string(&json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {"protocolVersion": "2025-06-18"}
            }))
            .unwrap(),
        ),
    );
    assert_eq!(status, 200);
    let init_resp = json_body(&body);
    let instructions = init_resp["result"]["instructions"].as_str().unwrap();
    assert!(
        instructions.contains("Access mode: anonymous"),
        "{instructions}"
    );
    let (status, body, _) = request(
        port,
        "POST",
        "/mcp",
        Some(
            &serde_json::to_string(&json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/call",
                "params": {"name": "memory_list", "arguments": {}}
            }))
            .unwrap(),
        ),
    );
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["result"]["isError"], json!(null));
    let (status, body, _) = request(
        port,
        "POST",
        "/mcp",
        Some(
            &serde_json::to_string(&json!({
                "jsonrpc": "2.0", "id": 3, "method": "tools/call",
                "params": {"name": "memory_create", "arguments": {"summary": "s", "content": "c"}}
            }))
            .unwrap(),
        ),
    );
    assert_eq!(status, 200);
    let resp = json_body(&body);
    assert_eq!(resp["result"]["isError"], true);
    assert!(
        resp["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("permission"),
        "{resp}"
    );

    // 带无效 token 是认证失败，不回退匿名
    let (status, _, _) = try_request(
        port,
        "GET",
        "/api/memories",
        None,
        &[("Authorization", "Bearer wrong-token")],
    )
    .unwrap();
    assert_eq!(status, 401);

    // 全无能力 = 匿名被整体拒绝（与未设置同义）
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

    // null 清除 → 回到基线 401；admin 自身不受影响
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
