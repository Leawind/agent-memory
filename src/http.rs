//! HTTP 层：一个进程同时服务三类客户端。
//!
//! - `POST /mcp`：MCP Streamable HTTP（无状态 POST/JSON 模式），供 agent 使用
//! - `/api/*`：管理后端（实现在 `crate::api`，复用工具层 handler），供管理界面使用
//! - `/`：rust-embed 嵌入的 Vue3 管理界面（SPA）
//!
//! 另有 `GET /health` 探活。无鉴权（使用者自担）：仅有的防护是 Origin 校验
//! （防浏览器 DNS rebinding，MCP 客户端与同源 fetch 不受影响）。
//! 诊断日志只写 stderr。

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

pub fn serve_http(host: &str, port: u16, db_path: &Path) -> i32 {
    if port == 0 {
        // 端口 0 会绑定到随机端口，但调用方无从得知实际端口，等于不可用
        eprintln!("agent-memory: --port 0 is not supported; choose a fixed port");
        return 2;
    }
    let addr = format!("{host}:{port}");
    // 启动前先打开一遍数据库（含迁移校验）；坏库拒绝启动，绝不带病服务。
    if let Err(e) = store::Store::open(db_path) {
        eprintln!("agent-memory: cannot open database ({e}).");
        eprintln!("agent-memory: refusing to start to protect your data.");
        return 1;
    }
    let server = match Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("agent-memory: cannot listen on {addr}: {e}");
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
    eprintln!("  MCP 端点  http://{addr}{ENDPOINT_MCP}（无鉴权，请只暴露给可信网络）");

    let db: PathBuf = db_path.to_path_buf();
    let mut handles = Vec::new();
    for _ in 0..WORKERS {
        let server = Arc::clone(&server);
        let db = db.clone();
        handles.push(std::thread::spawn(move || loop {
            match server.recv() {
                Ok(req) => handle_request(&db, req),
                Err(_) => return, // server 已关闭
            }
        }));
    }
    for h in handles {
        let _ = h.join();
    }
    0
}

fn handle_request(db_path: &Path, req: tiny_http::Request) {
    let mut req = req;
    let method = req.method().as_str().to_owned();
    let url = req.url().to_owned();
    let (path, query) = split_url(&url);

    if !origin_allowed(header_value(&req, "Origin").as_deref()) {
        log_request(&method, &path, 403, req.remote_addr());
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
                log_request(&method, &path, 415, remote);
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

    // body：除 GET/HEAD 外都读（上限内），供 /mcp 与 /api 使用
    let capped: Option<Vec<u8>> = if method == "GET" || method == "HEAD" {
        Some(Vec::new())
    } else {
        read_body_capped(&mut req)
    };
    let body_too_large = capped.is_none();
    let body = capped.unwrap_or_default();

    let (status, content_type, payload) = if body_too_large {
        (413, "application/json", err_bytes("request body too large"))
    } else {
        match route(&method, &path) {
            Route::Mcp => {
                // 与请求级事务配合的 panic 隔离：单个请求不拖垮服务器
                match catch_unwind(AssertUnwindSafe(|| process_mcp(db_path, &body))) {
                    Ok((status, payload)) => (status, "application/json", payload),
                    Err(_) => (
                        500,
                        "application/json",
                        err_bytes("internal error: handler panicked; the store was left unchanged"),
                    ),
                }
            }
            Route::MethodNotAllowed => return respond_method_not_allowed(req, &method, &path),
            Route::Health => {
                let v = json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")});
                (
                    200,
                    "application/json",
                    serde_json::to_vec(&v).unwrap_or_default(),
                )
            }
            Route::Api => {
                if path == "/api/export" && method == "GET" {
                    return respond_export(req, db_path, &method, &path);
                }
                match catch_unwind(AssertUnwindSafe(|| {
                    api::handle(db_path, &method, &path, &query, &body)
                })) {
                    Ok((status, v)) => (
                        status,
                        "application/json",
                        serde_json::to_vec(&v).unwrap_or_default(),
                    ),
                    Err(_) => (
                        500,
                        "application/json",
                        err_bytes("internal error: handler panicked; the store was left unchanged"),
                    ),
                }
            }
            Route::Static => return respond_static(req, &path, &method),
            Route::NotFound => (404, "application/json", err_bytes("not found")),
        }
    };
    let remote = req.remote_addr();
    log_request(&method, &path, status, remote);
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
fn process_mcp(db_path: &Path, body: &[u8]) -> (u16, Vec<u8>) {
    let msg: Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(e) => {
            let err = protocol::error_value(&Value::Null, -32700, &format!("parse error: {e}"));
            return (400, serde_json::to_vec(&err).unwrap_or_default());
        }
    };
    match protocol::handle_message(db_path, &msg) {
        Some(resp) => (200, serde_json::to_vec(&resp).unwrap_or_default()),
        None => (202, Vec::new()),
    }
}

// ---------------------------------------------------------------- 导出与静态资源

fn respond_export(req: tiny_http::Request, db_path: &Path, method: &str, path: &str) {
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
            log_request(method, path, 200, remote);
            let _ = req.respond(resp);
        }
        Err(e) => {
            log_request(method, path, 500, remote);
            let _ = req.respond(
                Response::from_data(err_bytes(&e))
                    .with_status_code(500)
                    .with_header(content_type_header("application/json")),
            );
        }
    }
}

fn respond_static(req: tiny_http::Request, path: &str, method: &str) {
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
                log_request(method, path, 404, remote);
                let _ = req.respond(
                    Response::from_data(err_bytes("not found"))
                        .with_status_code(404)
                        .with_header(content_type_header("application/json")),
                );
                return;
            }
        },
    };
    log_request(method, path, 200, remote);
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

fn respond_method_not_allowed(req: tiny_http::Request, method: &str, path: &str) {
    let remote = req.remote_addr();
    log_request(method, path, 405, remote);
    let allow = Header::from_bytes(b"Allow", b"POST").ok();
    let mut resp = Response::empty(405);
    if let Some(h) = allow {
        resp = resp.with_header(h);
    }
    let _ = req.respond(resp);
}

fn log_request(method: &str, path: &str, status: u16, remote: Option<&std::net::SocketAddr>) {
    let now = crate::model::now();
    let remote = remote.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
    eprintln!(
        "agent-memory: {} {} {} {} -> {}",
        util::format_utc_iso(now),
        remote,
        method,
        path,
        status
    );
}

#[cfg(test)]
mod tests {
    use super::*;

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
            "ui/dist/index.html is missing; run: cd ui && npm run build"
        );
    }

    /// 端口被占用时 serve_http 必须干净地失败（退出码 1），不能带病启动。
    #[test]
    fn serve_http_fails_cleanly_on_port_conflict() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let db = std::env::temp_dir().join(format!("agent-memory-port-{}.db", std::process::id()));
        let code = serve_http("127.0.0.1", port, &db);
        drop(listener);
        let _ = std::fs::remove_file(&db);
        assert_eq!(code, 1, "expected failure on occupied port");
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

    #[test]
    fn process_mcp_parses_and_routes_protocol_errors() {
        let path =
            std::env::temp_dir().join(format!("agent-memory-http-{}.db", std::process::id()));
        // 解析失败 → 400 + -32700
        let (status, body) = process_mcp(&path, b"not json");
        assert_eq!(status, 400);
        assert_eq!(
            serde_json::from_slice::<Value>(&body).unwrap()["error"]["code"],
            -32700
        );

        // 通知 → 202 空 body
        let (status, body) = process_mcp(
            &path,
            br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        );
        assert_eq!(status, 202);
        assert!(body.is_empty());
        let _ = std::fs::remove_file(&path);
    }
}
