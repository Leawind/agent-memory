//! HTTP 层：一个进程同时服务三类客户端。
//!
//! - `POST /mcp`：MCP Streamable HTTP（无状态 POST/JSON 模式），供 agent 使用
//! - `/api/*`：管理后端（实现在 `crate::api`，复用工具层 handler），供管理界面使用
//! - `/`：rust-embed 嵌入的 Vue3 管理界面（SPA）
//!
//! 另有 `GET /health` 探活。鉴权在传输层完成：请求携带
//! `Authorization: Bearer <token>`，与 MCP 规范的载体一致；鉴权开关未开启
//! 时为无鉴权开放模式（个人本地部署零配置）。仅有的其他防护是 Origin 校验
//! （防浏览器 DNS rebinding）。诊断日志只写 stderr。

use crate::auth::{Cap, IdentityCtx};
use crate::{api, protocol, store, util};
use rust_embed::RustEmbed;
use serde_json::{json, Value};
use std::io::Read as _;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tiny_http::{Header, Response, Server};

const ENDPOINT_MCP: &str = "/mcp";
const PREFIX_API: &str = "/api";
/// 单请求 body 上限：远超正文 20 万字符的合法需求，只为防滥用。
const MAX_BODY: usize = 8 * 1024 * 1024;
/// 并发 worker 数：写由 SQLite 串行化，多 worker 只为避免读请求排队。
const WORKERS: usize = 4;

#[derive(RustEmbed)]
#[folder = "ui/dist"]
struct UiAssets;

pub fn serve_http(host: &str, port: u16, db_path: &Path, verbose: bool) -> i32 {
    if port == 0 {
        // 端口 0 会绑定到随机端口，但调用方无从得知实际端口，等于不可用
        eprintln!("--port 0 is not supported; choose a fixed port");
        return 2;
    }
    let addr = format!("{host}:{port}");
    // 启动前先打开一遍数据库（含迁移校验）；坏库拒绝启动，绝不带病服务。
    if let Err(e) = store::Store::open(db_path) {
        eprintln!("cannot open database ({e}).");
        eprintln!("refusing to start to protect your data.");
        return 1;
    }
    let open_mode =
        match store::with_db_in(db_path, store::TxMode::ReadOnly, |st| st.auth_required()) {
            Ok(false) => true,
            Ok(true) => false,
            Err(e) => {
                eprintln!("cannot read auth switch ({e}).");
                return 1;
            }
        };
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot listen on {addr}: {e}");
            return 1;
        }
    };
    let server = Arc::new(server);
    eprintln!(
        "agent-memory v{}: server listening on http://{}",
        env!("CARGO_PKG_VERSION"),
        addr
    );
    eprintln!("  管理界面  http://{addr}/");
    eprintln!("  MCP 端点  http://{addr}{ENDPOINT_MCP}");
    if open_mode {
        eprintln!("  鉴权      开放模式（鉴权开关未开启，所有请求放行；请只暴露给可信网络）");
        eprintln!("            启用 token 鉴权：在管理界面「管理」页创建管理员身份后打开开关");
    } else {
        eprintln!("  鉴权      token 模式（/mcp 与 /api 须携带 Authorization: Bearer <token>）");
    }
    eprintln!(
        "  日志      {}",
        if verbose {
            "详细（全部请求，含耗时 / 请求者身份 / MCP 调用摘要）"
        } else {
            "仅错误请求与启动信息（--verbose 查看全部请求）"
        }
    );

    let db: PathBuf = db_path.to_path_buf();
    let mut handles = Vec::new();
    for _ in 0..WORKERS {
        let server = Arc::clone(&server);
        let db = db.clone();
        handles.push(std::thread::spawn(move || loop {
            match server.recv() {
                Ok(req) => handle_request(&db, req, verbose),
                Err(_) => return, // server 已关闭
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    0
}

fn handle_request(db_path: &Path, req: tiny_http::Request, verbose: bool) {
    let started = std::time::Instant::now();
    let mut req = req;
    let method = req.method().as_str().to_owned();
    let url = req.url().to_owned();
    let (path, query) = split_url(&url);

    if !origin_allowed(header_value(&req, "Origin").as_deref()) {
        log_request(
            verbose,
            &method,
            &path,
            403,
            req.remote_addr(),
            None,
            None,
            started.elapsed(),
        );
        respond_raw(
            req,
            403,
            "application/json",
            err_bytes("origin not allowed"),
        );
        return;
    }

    // MCP 规范：POST /mcp 的 Content-Type 不是 application/json 时必须回 415
    // （无 Content-Type 时宽容处理——部分客户端省略该头）
    if route(&method, &path) == Route::Mcp {
        if let Some(ct) = header_value(&req, "Content-Type") {
            if !ct.to_ascii_lowercase().contains("json") {
                let remote = req.remote_addr();
                log_request(
                    verbose,
                    &method,
                    &path,
                    415,
                    remote,
                    None,
                    None,
                    started.elapsed(),
                );
                respond_raw(
                    req,
                    415,
                    "application/json",
                    err_bytes("unsupported content type; use application/json"),
                );
                return;
            }
        }
    }

    // 鉴权：/mcp 与 /api 必须携带有效 Bearer token（鉴权开关开启时）。
    // 静态 UI 与 /health 免鉴权（页面本身不含数据，数据全走已鉴权的 /api）。
    let ctx = if matches!(route(&method, &path), Route::Mcp | Route::Api) {
        match resolve_identity(db_path, header_value(&req, "Authorization").as_deref()) {
            Ok(ctx) => ctx,
            Err(fail) => {
                if let AuthFail::Storage(reason) = &fail {
                    eprintln!("auth lookup failed, failing closed: {reason}");
                }
                log_request(
                    verbose,
                    &method,
                    &path,
                    401,
                    req.remote_addr(),
                    None,
                    None,
                    started.elapsed(),
                );
                respond_raw(
                    req,
                    401,
                    "application/json",
                    err_bytes("unauthorized: configure the 'Authorization: Bearer <token>' header with a valid access token"),
                );
                return;
            }
        }
    } else {
        IdentityCtx::open_mode()
    };

    // body：除 GET/HEAD 外都读（上限内），供 /mcp 与 /api 使用
    let capped: Option<Vec<u8>> = if method == "GET" || method == "HEAD" {
        Some(Vec::new())
    } else {
        read_body_capped(&mut req)
    };
    let body_too_large = capped.is_none();
    let body = capped.unwrap_or_default();

    let (status, content_type, payload, mcp_detail) = if body_too_large {
        (
            413,
            "application/json",
            err_bytes("request body too large"),
            None,
        )
    } else {
        match route(&method, &path) {
            Route::Mcp => {
                // 与请求级事务配合的 panic 隔离：单个请求不拖垮服务器
                let negotiated = header_value(&req, "MCP-Protocol-Version");
                match catch_unwind(AssertUnwindSafe(|| {
                    process_mcp(db_path, &ctx, &body, negotiated.as_deref())
                })) {
                    Ok((status, payload, detail)) => (status, "application/json", payload, detail),
                    Err(_) => (
                        500,
                        "application/json",
                        err_bytes("internal error: handler panicked; the store was left unchanged"),
                        None,
                    ),
                }
            }
            Route::MethodNotAllowed => {
                return respond_method_not_allowed(req, &method, &path, verbose, started)
            }
            Route::Health => {
                let v = json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")});
                (
                    200,
                    "application/json",
                    serde_json::to_vec(&v).unwrap_or_default(),
                    None,
                )
            }
            Route::Api => {
                if path == "/api/export" && method == "GET" {
                    // 导出是全库明文备份：与 doctor/import 同级，要求 admin 能力
                    if let Err(e) = ctx.require(Cap::Admin) {
                        log_request(
                            verbose,
                            &method,
                            &path,
                            403,
                            req.remote_addr(),
                            Some(&ctx.name),
                            None,
                            started.elapsed(),
                        );
                        respond_raw(req, 403, "application/json", err_bytes(e.message()));
                        return;
                    }
                    return respond_export(
                        req, db_path, &method, &path, verbose, &ctx.name, started,
                    );
                }
                match catch_unwind(AssertUnwindSafe(|| {
                    api::handle(db_path, &ctx, &method, &path, &query, &body)
                })) {
                    Ok((status, v)) => (
                        status,
                        "application/json",
                        serde_json::to_vec(&v).unwrap_or_default(),
                        None,
                    ),
                    Err(_) => (
                        500,
                        "application/json",
                        err_bytes("internal error: handler panicked; the store was left unchanged"),
                        None,
                    ),
                }
            }
            Route::Static => return respond_static(req, &path, &method, verbose, started),
            Route::NotFound => (404, "application/json", err_bytes("not found"), None),
        }
    };
    let remote = req.remote_addr();
    log_request(
        verbose,
        &method,
        &path,
        status,
        remote,
        Some(&ctx.name),
        mcp_detail.as_deref(),
        started.elapsed(),
    );
    respond_raw(req, status, content_type, payload);
}

// ---------------------------------------------------------------- 路由

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Mcp,
    MethodNotAllowed,
    Health,
    Api,
    Static,
    NotFound,
}

fn route(method: &str, path: &str) -> Route {
    if path == ENDPOINT_MCP {
        return if method == "POST" {
            Route::Mcp
        } else {
            Route::MethodNotAllowed
        };
    }
    if path == "/health" {
        return if method == "GET" {
            Route::Health
        } else {
            Route::NotFound
        };
    }
    if path == PREFIX_API || path.starts_with("/api/") {
        return Route::Api;
    }
    if method == "GET" || method == "HEAD" {
        return Route::Static;
    }
    Route::NotFound
}

// ---------------------------------------------------------------- MCP 端点

/// 把一条 JSON-RPC 消息交给协议层处理，返回 HTTP 状态码与响应体。
/// 通知（无 id）无响应体 → 202 Accepted；其余 → 200。
/// `negotiated` 来自 MCP-Protocol-Version 请求头（客户端声明协商版本）。
/// 第三个返回值是给 verbose 日志的调用摘要（方法名 + tools/call 的目标工具）。
fn process_mcp(
    db_path: &Path,
    ctx: &IdentityCtx,
    body: &[u8],
    negotiated: Option<&str>,
) -> (u16, Vec<u8>, Option<String>) {
    let msg: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => {
            let err = protocol::error_value(&Value::Null, -32700, &format!("parse error: {e}"));
            return (400, serde_json::to_vec(&err).unwrap_or_default(), None);
        }
    };
    let detail = mcp_summary(&msg);
    match protocol::handle_message(db_path, ctx, negotiated, &msg) {
        Some(resp) => (200, serde_json::to_vec(&resp).unwrap_or_default(), detail),
        None => (202, Vec::new(), detail),
    }
}

/// MCP 请求摘要：批量消息（数组）或畸形结构返回 None，不进日志。
fn mcp_summary(msg: &Value) -> Option<String> {
    let method = msg.get("method")?.as_str()?;
    Some(match method {
        "tools/call" => format!(
            "tools/call {}",
            msg.pointer("/params/name")
                .and_then(Value::as_str)
                .unwrap_or("?")
        ),
        other => other.to_string(),
    })
}

// ---------------------------------------------------------------- 导出与静态资源

fn respond_export(
    req: tiny_http::Request,
    db_path: &Path,
    method: &str,
    path: &str,
    verbose: bool,
    identity: &str,
    started: std::time::Instant,
) {
    let remote = req.remote_addr();
    match api::export_bytes(db_path) {
        Ok(bytes) => {
            let disposition = Header::from_bytes(
                b"Content-Disposition",
                b"attachment; filename=\"agent-memory-export.json\"",
            )
            .ok();
            let mut resp = Response::from_data(bytes)
                .with_status_code(200)
                .with_header(content_type_header("application/json"));
            if let Some(h) = disposition {
                resp = resp.with_header(h);
            }
            log_request(
                verbose,
                method,
                path,
                200,
                remote,
                Some(identity),
                None,
                started.elapsed(),
            );
            let _ = req.respond(resp);
        }
        Err(e) => {
            log_request(
                verbose,
                method,
                path,
                500,
                remote,
                Some(identity),
                None,
                started.elapsed(),
            );
            let _ = req.respond(
                Response::from_data(err_bytes(&e))
                    .with_status_code(500)
                    .with_header(content_type_header("application/json")),
            );
        }
    }
}

fn respond_static(
    req: tiny_http::Request,
    path: &str,
    method: &str,
    verbose: bool,
    started: std::time::Instant,
) {
    let remote = req.remote_addr();
    let lookup = path.trim_start_matches('/');
    let lookup = if lookup.is_empty() {
        "index.html"
    } else {
        lookup
    };
    // 精确命中用其自身扩展名决定 MIME；SPA 回退永远是 index.html
    let (served, mime) = match UiAssets::get(lookup) {
        Some(f) => (f, mime_of(lookup)),
        None => match UiAssets::get("index.html") {
            Some(f) => (f, mime_of("index.html")),
            None => {
                log_request(
                    verbose,
                    method,
                    path,
                    404,
                    remote,
                    None,
                    None,
                    started.elapsed(),
                );
                let _ = req.respond(
                    Response::from_data(err_bytes("not found"))
                        .with_status_code(404)
                        .with_header(content_type_header("application/json")),
                );
                return;
            }
        },
    };
    log_request(
        verbose,
        method,
        path,
        200,
        remote,
        None,
        None,
        started.elapsed(),
    );
    let resp = Response::from_data(served.data.to_vec())
        .with_status_code(200)
        .with_header(content_type_header(mime));
    let _ = req.respond(resp);
}

fn mime_of(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

// ---------------------------------------------------------------- 工具函数

fn split_url(url: &str) -> (String, String) {
    match url.split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (url.to_string(), String::new()),
    }
}

/// Origin 校验：带 Origin 头时只放行指向本机的来源（浏览器 DNS rebinding 防护）。
/// 没有 Origin 的请求（MCP 客户端、同源 fetch、curl）一律放行。
fn origin_allowed(origin: Option<&str>) -> bool {
    let Some(o) = origin else {
        return true;
    };
    if o.is_empty() {
        return true;
    }
    let Some(rest) = o
        .strip_prefix("http://")
        .or_else(|| o.strip_prefix("https://"))
    else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let authority = authority.trim_end_matches('.');
    // IPv6 字面量形如 [::1]:8899，先取方括号内主机，避免与端口号混淆
    let host = if let Some(inner) = authority.strip_prefix('[') {
        inner.split(']').next().unwrap_or(inner)
    } else {
        authority
            .rsplit_once(':')
            .map(|(h, _)| h)
            .unwrap_or(authority)
    };
    matches!(
        host.to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1"
    )
}

fn header_value(req: &tiny_http::Request, name: &str) -> Option<String> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_owned())
}

/// Bearer 解析：`Authorization: Bearer <token>`，方案名大小写不敏感
/// （RFC 7235 允许），token 本身大小写敏感。
fn parse_bearer(header: &str) -> Option<String> {
    let (scheme, rest) = header.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = rest.trim();
    (!token.is_empty()).then(|| token.to_string())
}

/// 鉴权失败原因：Denied 是正常拒绝（无效/缺失 token），Storage 是身份
/// 存储不可读——两者对外统一 401（不泄露失败细节），后者额外记日志。
enum AuthFail {
    Denied,
    Storage(String),
}

/// 解析请求身份。鉴权开关（settings 的 auth_required）关闭 → 开放模式
/// （全能力）；开启则必须携带有效 token。查库失败按"拒绝"处理（fail-closed）。
fn resolve_identity(db_path: &Path, auth_header: Option<&str>) -> Result<IdentityCtx, AuthFail> {
    let token = auth_header.and_then(parse_bearer);
    // 内层 Result 把"正常拒绝（Denied）"与存储错误分开：存储错误在事务层
    // 以 String 传递，这里再包成 Storage（响应统一 401，日志区分记因）。
    let outcome: Result<Result<IdentityCtx, AuthFail>, String> =
        store::with_db_in(db_path, store::TxMode::ReadOnly, |st| {
            if !st.auth_required()? {
                return Ok(Ok(IdentityCtx::open_mode()));
            }
            let Some(token) = token.clone() else {
                return Ok(Err(AuthFail::Denied));
            };
            Ok(st.identity_ctx_by_token(&token)?.ok_or(AuthFail::Denied))
        });
    match outcome {
        Ok(inner) => inner,
        Err(e) => Err(AuthFail::Storage(e)),
    }
}

fn read_body_capped(req: &mut tiny_http::Request) -> Option<Vec<u8>> {
    let mut body = Vec::new();
    let ok = req
        .as_reader()
        .take((MAX_BODY + 1) as u64)
        .read_to_end(&mut body)
        .is_ok();
    if ok && body.len() <= MAX_BODY {
        Some(body)
    } else {
        None
    }
}

fn content_type_header(value: &str) -> Header {
    Header::from_bytes(b"Content-Type", value.as_bytes())
        .expect("static content-type header is valid")
}

fn err_bytes(message: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({ "error": message })).unwrap_or_default()
}

fn respond_raw(req: tiny_http::Request, status: u16, content_type: &str, body: Vec<u8>) {
    let resp = if body.is_empty() {
        Response::from_data(Vec::new()).with_status_code(status)
    } else {
        Response::from_data(body)
            .with_status_code(status)
            .with_header(content_type_header(content_type))
    };
    let _ = req.respond(resp);
}

fn respond_method_not_allowed(
    req: tiny_http::Request,
    method: &str,
    path: &str,
    verbose: bool,
    started: std::time::Instant,
) {
    let remote = req.remote_addr();
    log_request(
        verbose,
        method,
        path,
        405,
        remote,
        None,
        None,
        started.elapsed(),
    );
    let allow = Header::from_bytes(b"Allow", b"POST").ok();
    let mut resp = Response::empty(405);
    if let Some(h) = allow {
        resp = resp.with_header(h);
    }
    let _ = req.respond(resp);
}

/// 默认只记 >=400 的请求与启动/异常事件；--verbose 时全部记录。
/// 查询串从不入日志（搜索关键词属用户内容）。
fn should_log(verbose: bool, status: u16) -> bool {
    verbose || status >= 400
}

#[allow(clippy::too_many_arguments)]
fn log_request(
    verbose: bool,
    method: &str,
    path: &str,
    status: u16,
    remote: Option<&std::net::SocketAddr>,
    identity: Option<&str>,
    detail: Option<&str>,
    elapsed: std::time::Duration,
) {
    if !should_log(verbose, status) {
        return;
    }
    let remote = remote.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
    let mut line = format!(
        "{} {} [{}] {} {}",
        util::format_utc_iso(crate::model::now()),
        remote,
        identity.unwrap_or("-"),
        method,
        path
    );
    if let Some(d) = detail {
        line.push(' ');
        line.push_str(d);
    }
    eprintln!("{line} -> {} ({}ms)", status, elapsed.as_millis());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_log_defaults_to_errors_only() {
        // 默认模式：成功与重定向静默，4xx/5xx 必现
        assert!(!should_log(false, 200));
        assert!(!should_log(false, 202));
        assert!(should_log(false, 400));
        assert!(should_log(false, 401));
        assert!(should_log(false, 500));
        // verbose 全量
        for s in [200u16, 202, 404, 500] {
            assert!(should_log(true, s));
        }
    }

    #[test]
    fn mcp_summary_extracts_tool_name() {
        let call = serde_json::json!({
            "jsonrpc": "2.0", "id": "m1", "method": "tools/call",
            "params": {"name": "add_memory", "arguments": {}}
        });
        assert_eq!(mcp_summary(&call).as_deref(), Some("tools/call add_memory"));
        let init = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize"});
        assert_eq!(mcp_summary(&init).as_deref(), Some("initialize"));
        // 批量消息（数组）与畸形结构不产生摘要
        assert_eq!(mcp_summary(&serde_json::json!([])), None);
        assert_eq!(mcp_summary(&serde_json::json!({"id": 1})), None);
        // tools/call 缺工具名时占位而非 None
        let broken = serde_json::json!({"method": "tools/call"});
        assert_eq!(mcp_summary(&broken).as_deref(), Some("tools/call ?"));
    }

    #[test]
    fn route_matches_known_endpoints() {
        assert!(matches!(route("POST", "/mcp"), Route::Mcp));
        assert!(matches!(route("GET", "/mcp"), Route::MethodNotAllowed));
        assert!(matches!(route("DELETE", "/mcp"), Route::MethodNotAllowed));
        assert!(matches!(route("GET", "/health"), Route::Health));
        assert!(matches!(route("GET", "/api/stats"), Route::Api));
        assert!(matches!(route("POST", "/api/tags"), Route::Api));
        assert!(matches!(route("GET", "/"), Route::Static));
        assert!(matches!(route("GET", "/assets/app.js"), Route::Static));
        assert!(matches!(route("PUT", "/elsewhere"), Route::NotFound));
    }

    /// ui/dist 必须带着 index.html（rust-embed 嵌入的 SPA 入口）。
    /// 目录被清空/未提交时，此测试会在 cargo test 阶段就暴露。
    #[test]
    fn embedded_ui_has_index_entry() {
        assert!(
            UiAssets::get("index.html").is_some(),
            "ui/dist/index.html is missing; run: pnpm build (repo root)"
        );
    }

    /// 端口被占用时 serve_http 必须干净地失败（退出码 1），不能带病启动。
    #[test]
    fn serve_http_fails_cleanly_on_port_conflict() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let db = std::env::temp_dir().join(format!("agent-memory-port-{}.db", std::process::id()));
        let code = serve_http("127.0.0.1", port, &db, false);
        drop(listener);
        let _ = std::fs::remove_file(&db);
        assert_eq!(code, 1, "expected failure on occupied port");
    }

    #[test]
    fn bearer_parsing_tolerates_case_and_rejects_garbage() {
        assert_eq!(parse_bearer("Bearer abc"), Some("abc".to_string()));
        assert_eq!(parse_bearer("bearer abc"), Some("abc".to_string()));
        assert_eq!(parse_bearer("BEARER abc"), Some("abc".to_string()));
        // 多空格：token 前的空白被吞掉
        assert_eq!(parse_bearer("Bearer   abc  "), Some("abc".to_string()));
        assert_eq!(parse_bearer("Basic abc"), None);
        assert_eq!(parse_bearer("Bearer"), None);
        assert_eq!(parse_bearer("Bearer "), None);
        assert_eq!(parse_bearer(""), None);
    }

    #[test]
    fn process_mcp_parses_and_routes_protocol_errors() {
        let path =
            std::env::temp_dir().join(format!("agent-memory-http-{}.db", std::process::id()));
        let ctx = crate::auth::IdentityCtx::open_mode();
        // 解析失败 → 400 + -32700，无调用摘要
        let (status, body, detail) = process_mcp(&path, &ctx, b"not json", None);
        assert_eq!(status, 400);
        assert_eq!(
            serde_json::from_slice::<Value>(&body).unwrap()["error"]["code"],
            -32700
        );
        assert_eq!(detail, None);

        // 通知 → 202 空 body，摘要为方法名
        let (status, body, detail) = process_mcp(
            &path,
            &ctx,
            br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
            None,
        );
        assert_eq!(status, 202);
        assert!(body.is_empty());
        assert_eq!(detail.as_deref(), Some("notifications/initialized"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn origin_check_admits_only_local_origins_or_absent() {
        assert!(origin_allowed(None));
        assert!(origin_allowed(Some("")));
        assert!(origin_allowed(Some("http://localhost")));
        assert!(origin_allowed(Some("http://localhost:8899")));
        assert!(origin_allowed(Some("https://127.0.0.1")));
        assert!(origin_allowed(Some("http://[::1]:8899")));
        assert!(origin_allowed(Some("http://[::1]")));
        assert!(origin_allowed(Some("http://LOCALHOST:3000")));
        assert!(!origin_allowed(Some("http://evil.example")));
        assert!(!origin_allowed(Some("http://localhost.evil.com")));
        assert!(!origin_allowed(Some("http://127.0.0.1.evil.com")));
        assert!(!origin_allowed(Some("ftp://localhost")));
        assert!(!origin_allowed(Some("null")));
    }
}
