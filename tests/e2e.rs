//! 端到端集成测试：真实启动 HTTP 服务器二进制与 CLI 子命令。
//!
//! - agent 的 MCP 端点（POST /mcp，Streamable HTTP 无状态模式）
//! - 管理后端 /api/*（供内嵌管理界面使用）
//! - 静态托管的管理界面（/）
//! - 多进程共享同一数据库的并发正确性
//! - CLI 运维子命令（stats / doctor / export）
//!
//! HTTP 客户端用 std::net 手写最小实现，不引入 dev 依赖。

use serde_json::{json, Value};
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

static SEQ: AtomicU32 = AtomicU32::new(0);

fn temp_db(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "agent-memory-e2e-{}-{}-{}.db",
        std::process::id(),
        tag,
        n
    ))
}

fn cleanup(path: &Path) {
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(PathBuf::from(format!("{}{}", path.display(), suffix)));
    }
}

fn free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    l.local_addr().expect("local addr").port()
}

struct HttpProc {
    child: std::process::Child,
    port: u16,
}

/// cargo test 默认多线程并行：端口分配到子进程绑定之间存在竞争窗口，
/// 用全局锁把"取空闲端口 → 子进程绑定成功"串行化。
static START_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

impl HttpProc {
    fn start(db: &Path, tag: &str) -> HttpProc {
        let _guard = START_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let port = free_port();
        let child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
            .args([
                "serve",
                "--host",
                "127.0.0.1",
                "--port",
                &port.to_string(),
                "--db",
            ])
            .arg(db)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap_or_else(|e| panic!("spawn server {tag}: {e}"));
        let mut proc = HttpProc { child, port };
        proc.wait_ready(tag);
        proc
    }

    /// 轮询 /health 直到就绪；子进程提前退出视为启动失败。
    fn wait_ready(&mut self, tag: &str) {
        for _ in 0..100 {
            if try_request(self.port, "GET", "/health", None, &[]).is_ok() {
                return;
            }
            if let Ok(Some(status)) = self.child.try_wait() {
                panic!("server {tag} exited early with {status:?}");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("server {tag} did not become ready in time");
    }
}

impl Drop for HttpProc {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 发一条 HTTP/1.1 请求并读完整响应（Connection: close）。
fn try_request(
    port: u16,
    method: &str,
    path: &str,
    body: Option<&str>,
    extra_headers: &[(&str, &str)],
) -> Result<(u16, Vec<u8>, String), std::io::Error> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))?;
    let payload = body.unwrap_or("");
    let mut req =
        format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
    for (k, v) in extra_headers {
        write!(req, "{k}: {v}\r\n").unwrap();
    }
    if body.is_some() {
        req.push_str("Content-Type: application/json\r\n");
        write!(req, "Content-Length: {}\r\n", payload.len()).unwrap();
    }
    req.push_str("\r\n");
    stream.write_all(req.as_bytes())?;
    if !payload.is_empty() {
        stream.write_all(payload.as_bytes())?;
    }
    stream.flush()?;

    let mut raw = Vec::new();
    stream.read_to_end(&mut raw)?;
    let sep = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("HTTP response has header/body separator");
    let head = String::from_utf8_lossy(&raw[..sep]).to_string();
    let status: u16 = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .expect("HTTP status line");
    let content_type = head
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("content-type:"))
        .unwrap_or("")
        .to_string();
    Ok((status, raw[sep + 4..].to_vec(), content_type))
}

fn request(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, Vec<u8>, String) {
    try_request(port, method, path, body, &[]).expect("http request succeeds")
}

fn json_body(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("valid JSON body")
}

/// 对 /mcp 发一条 JSON-RPC 请求，断言 HTTP 200 并返回解析后的响应。
fn mcp_rpc(port: u16, body: Value) -> Value {
    let (status, bytes, _) = request(
        port,
        "POST",
        "/mcp",
        Some(&serde_json::to_string(&body).unwrap()),
    );
    assert_eq!(status, 200, "unexpected status for rpc: {body}");
    json_body(&bytes)
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-memory"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("run agent-memory CLI")
}

#[allow(non_snake_case)] // 与 JS encodeURIComponent 同名，便于对照
fn encodeURIComponent(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => write!(out, "%{b:02X}").unwrap(),
        }
    }
    out
}

// ---------------------------------------------------------------------------

#[test]
fn mcp_endpoint_end_to_end() {
    let db = temp_db("mcp");
    cleanup(&db);
    let server = HttpProc::start(&db, "mcp");
    let port = server.port;

    // 探活
    let (status, body, _) = request(port, "GET", "/health", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["status"], "ok");

    // initialize：协议版本回显
    let init = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "e2e", "version": "0.0.1"}
        }}),
    );
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["serverInfo"]["name"], "agent-memory");

    // 通知：无响应体 → 202
    let (status, body, _) = request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#),
    );
    assert_eq!(status, 202);
    assert!(body.is_empty());

    // 完整 agent 流程：创建 → 搜索（渐进式披露，不泄露正文）→ 取全文
    let created = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
            "name": "memory_create", "arguments": {
                "summary": "项目使用 Rust 实现 agent 记忆系统",
                "content": "仓库位于 D:\\Workspace，SQLite 做持久化，搜索无需分词。",
                "tags": ["项目", "rust"]
            }
        }}),
    );
    assert!(created.get("error").is_none(), "create failed: {created}");
    let mem_id = created["result"]["structuredContent"]["memory"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let searched = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
            "name": "memory_search", "arguments": {"query": "记忆系统"}
        }}),
    );
    let results = searched["result"]["structuredContent"]["results"]
        .as_array()
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["id"], mem_id.as_str());
    assert!(
        results[0].get("content").is_none(),
        "search must not leak content"
    );

    let fetched = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": {
            "name": "memory_get", "arguments": {"ids": [mem_id]}
        }}),
    );
    assert!(
        fetched["result"]["structuredContent"]["memories"][0]["content"]
            .as_str()
            .unwrap()
            .contains("SQLite")
    );

    // 错误路径：未知工具 → -32602；未知方法 → -32601；参数错误 → isError
    let unknown_tool = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {"name": "nope"}}),
    );
    assert_eq!(unknown_tool["error"]["code"], -32602);
    let unknown_method = mcp_rpc(port, json!({"jsonrpc": "2.0", "id": 6, "method": "bogus"}));
    assert_eq!(unknown_method["error"]["code"], -32601);
    let bad_args = mcp_rpc(
        port,
        json!({"jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {
            "name": "memory_create", "arguments": {"summary": "only summary"}
        }}),
    );
    assert_eq!(bad_args["result"]["isError"], true);

    // 非 JSON body → 400 + -32700
    let (status, body, _) = request(port, "POST", "/mcp", Some("not json"));
    assert_eq!(status, 400);
    assert_eq!(json_body(&body)["error"]["code"], -32700);

    // Content-Type 非 JSON → 415（MCP 规范要求）
    let (status, _, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#),
        &[("Content-Type", "text/plain")],
    )
    .unwrap();
    assert_eq!(status, 415);

    // 批量消息经 HTTP：ping 的响应 + 通知不产生响应 → 单元素数组
    let (status, body, _) = request(
        port,
        "POST",
        "/mcp",
        Some(
            r#"[{"jsonrpc":"2.0","id":20,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"}]"#,
        ),
    );
    assert_eq!(status, 200);
    let batch_resp = json_body(&body);
    let batch = batch_resp.as_array().unwrap();
    assert_eq!(batch.len(), 1);
    assert_eq!(batch[0]["id"], 20);
    assert_eq!(batch[0]["result"], json!({}));

    // GET /mcp → 405；未知路径：GET 走 SPA 回退，非 GET 404
    let (status, _, _) = request(port, "GET", "/mcp", None);
    assert_eq!(status, 405);
    let (status, _, _) = request(port, "GET", "/nope", None);
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "PUT", "/nope", Some("{}"));
    assert_eq!(status, 404);

    // Origin 防护：非本机来源 403；本机来源放行
    let (status, _, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","id":8,"method":"ping"}"#),
        &[("Origin", "http://evil.example")],
    )
    .unwrap();
    assert_eq!(status, 403);
    let (status, _, _) = try_request(
        port,
        "POST",
        "/mcp",
        Some(r#"{"jsonrpc":"2.0","id":9,"method":"ping"}"#),
        &[("Origin", "http://127.0.0.1:3000")],
    )
    .unwrap();
    assert_eq!(status, 200);

    // 风暴后仍可用
    let pong = mcp_rpc(port, json!({"jsonrpc": "2.0", "id": 10, "method": "ping"}));
    assert_eq!(pong["result"], json!({}));

    drop(server);
    cleanup(&db);
}

#[test]
fn rest_api_end_to_end() {
    let db = temp_db("rest");
    cleanup(&db);
    let server = HttpProc::start(&db, "rest");
    let port = server.port;

    // 标签：创建（含中文路径转义）→ 列表 → 精确重名报错 → 改名
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
    assert_eq!(json_body(&body)["tags"].as_array().unwrap().len(), 2);

    let (status, body, _) = request(
        port,
        "PUT",
        &format!("/api/tags/{}", encodeURIComponent("项目")),
        Some(r#"{"new_name": "项目A", "description": "改名后的描述"}"#),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
    assert_eq!(json_body(&body)["new_name"], "项目A");

    // 记忆：创建 → 列表/过滤 → 取全文 → 更新 → 搜索 → 删除
    let (status, body, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(
            r#"{"summary": "用户偏好深色主题", "content": "2026-09 确认", "tags": ["项目A", "偏好"]}"#,
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
        Some(r#"{"summary": "用户偏好浅色主题", "remove_tags": ["偏好"], "add_tags": ["外观"]}"#),
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

    // 不存在的记忆 → 404
    let (status, _, _) = request(port, "GET", "/api/memories/m999", None);
    assert_eq!(status, 404);

    // stats / doctor
    let (status, body, _) = request(port, "GET", "/api/stats", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["memories"], 1);
    let (status, body, _) = request(port, "GET", "/api/doctor", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["ok"], true);

    // export：作为附件下载
    let (status, body, ctype) = request(port, "GET", "/api/export", None);
    assert_eq!(status, 200);
    assert!(ctype.contains("application/json"));
    let dump: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(dump["total_memories"], 1);

    // 删除记忆 → 再查 404
    let (status, _, _) = request(port, "DELETE", &format!("/api/memories/{mem_id}"), None);
    assert_eq!(status, 200);
    let (status, _, _) = request(port, "GET", &format!("/api/memories/{mem_id}"), None);
    assert_eq!(status, 404);

    // purge 标签连带删记忆
    let (status, _, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(r#"{"summary": "临时记忆", "content": "x", "tags": ["临时"]}"#),
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

    // 根路径返回 HTML（内嵌资产；构建前为占位页，构建后为 Vue 应用）
    let (status, body, ctype) = request(port, "GET", "/", None);
    assert_eq!(status, 200);
    assert!(ctype.contains("text/html"), "content-type: {ctype}");
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("<html") || html.contains("<!doctype"),
        "not html"
    );

    // SPA 回退：未知的非 API 路径也回 index
    let (status, _, ctype2) = request(port, "GET", "/some/spa/route", None);
    assert_eq!(status, 200);
    assert!(ctype2.contains("text/html"));

    // /health 是 JSON 而不是 SPA 回退
    let (status, body, ctype3) = request(port, "GET", "/health", None);
    assert_eq!(status, 200);
    assert!(ctype3.contains("application/json"));
    assert_eq!(json_body(&body)["status"], "ok");

    drop(server);
    cleanup(&db);
}

#[test]
fn two_server_processes_share_one_db() {
    let db = temp_db("share");
    cleanup(&db);
    let a = HttpProc::start(&db, "share-a");
    let b = HttpProc::start(&db, "share-b");

    // A 写入，B 立即可见（WAL 读写并发）
    let (status, _, _) = request(
        a.port,
        "POST",
        "/api/memories",
        Some(r#"{"summary": "written by A", "content": "a", "tags": ["share"]}"#),
    );
    assert_eq!(status, 200);
    let (status, body, _) = request(b.port, "GET", "/api/memories?query=written", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total_matches"], 1);

    // B 改名标签，A 端数据同步
    let (status, _, _) = request(
        b.port,
        "PUT",
        &format!("/api/tags/{}", encodeURIComponent("share")),
        Some(r#"{"new_name": "shared-tag"}"#),
    );
    assert_eq!(status, 200);
    let (status, body, _) = request(a.port, "GET", "/api/memories?tag=shared-tag", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total"], 1);

    // 4 线程并发经 REST 写入（两个进程各自承接一半），零丢失；
    // 同时一个读者线程持续搜索——只读事务（DEFERRED 快照）不与写者互斥
    let ports = [a.port, b.port, a.port, b.port];
    let reader_port = a.port;
    let reader = std::thread::spawn(move || {
        for _ in 0..12 {
            let (status, body, _) = request(
                reader_port,
                "GET",
                "/api/memories?query=parallel%20writer&limit=50",
                None,
            );
            assert_eq!(status, 200, "concurrent reader failed");
            let total = json_body(&body)["total_matches"].as_u64().unwrap();
            assert!(total <= 12, "reader saw impossible count {total}");
        }
    });
    let mut handles = vec![reader];
    for (w, port) in ports.into_iter().enumerate() {
        handles.push(std::thread::spawn(move || {
            for i in 0..3 {
                let body = json!({
                    "summary": format!("parallel writer {} item {}", w, i),
                    "content": "race"
                });
                let (status, _, _) = request(
                    port,
                    "POST",
                    "/api/memories",
                    Some(&serde_json::to_string(&body).unwrap()),
                );
                assert_eq!(status, 200, "writer {w} item {i} failed");
            }
        }));
    }
    for h in handles {
        h.join().expect("thread");
    }
    let (status, body, _) = request(
        a.port,
        "GET",
        "/api/memories?query=parallel%20writer&limit=50",
        None,
    );
    assert_eq!(status, 200);
    assert_eq!(
        json_body(&body)["total_matches"],
        12,
        "lost updates under concurrent writers"
    );

    drop(a);
    drop(b);
    cleanup(&db);
}

#[test]
fn cli_subcommands_work() {
    let db = temp_db("cli");
    cleanup(&db);
    let db_s = db.display().to_string();

    // 准备数据：起一个临时 server 经 REST 写入
    {
        let server = HttpProc::start(&db, "cli-writer");
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(r#"{"summary": "cli test memory", "content": "body", "tags": ["cli"]}"#),
        );
        assert_eq!(status, 200);
        drop(server);
    }

    // stats
    let out = run_cli(&["stats", "--db", &db_s]);
    assert!(out.status.success(), "stats failed: {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("memories: 1"), "stats output: {stdout}");
    assert!(stdout.contains("tags: 1"), "stats output: {stdout}");

    // doctor：干净库 → 退出码 0
    let out = run_cli(&["doctor", "--db", &db_s]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("no issues found"));

    // export：导出成功 → 再次导出拒绝覆盖
    let export_path = temp_db("export-out");
    let out = run_cli(&["export", export_path.to_str().unwrap(), "--db", &db_s]);
    assert!(out.status.success(), "export failed");
    assert!(export_path.exists());
    let dump: Value =
        serde_json::from_str(&std::fs::read_to_string(&export_path).unwrap()).unwrap();
    assert_eq!(dump["total_memories"], 1);
    assert_eq!(dump["memories"][0]["content"], "body");

    let out = run_cli(&["export", export_path.to_str().unwrap(), "--db", &db_s]);
    assert!(!out.status.success(), "export must refuse to overwrite");
    assert!(String::from_utf8_lossy(&out.stderr).contains("already exists"));
    let _ = std::fs::remove_file(&export_path);

    cleanup(&db);
}

#[test]
fn export_import_roundtrip_via_cli() {
    let src = temp_db("roundtrip-src");
    let dst = temp_db("roundtrip-dst");
    cleanup(&src);
    cleanup(&dst);
    let src_s = src.display().to_string();
    let dst_s = dst.display().to_string();

    // 源库准备数据
    {
        let server = HttpProc::start(&src, "roundtrip-src");
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(r#"{"summary": "roundtrip 记忆甲", "content": "内容甲", "tags": ["tag甲"]}"#),
        );
        assert_eq!(status, 200);
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(r#"{"summary": "roundtrip 记忆乙", "content": "内容乙", "tags": ["tag乙"]}"#),
        );
        assert_eq!(status, 200);
        drop(server);
    }

    // CLI 导出 → CLI 导入到全新库
    let export_path = temp_db("rt-export");
    let out = run_cli(&["export", export_path.to_str().unwrap(), "--db", &src_s]);
    assert!(out.status.success(), "export failed");

    let out = run_cli(&["import", export_path.to_str().unwrap(), "--db", &dst_s]);
    assert!(
        out.status.success(),
        "import failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("imported 2 memories"));

    // 导入的库数据完整可用（内容 + 标签 + 时间戳保留）
    let server = HttpProc::start(&dst, "roundtrip-verify");
    let (status, body, _) = request(
        server.port,
        "GET",
        "/api/memories?query=%E5%86%85%E5%AE%B9%E7%94%B2",
        None,
    );
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total_matches"], 1);
    let (status, body, _) = request(server.port, "GET", "/api/stats", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["memories"], 2);
    assert_eq!(json_body(&body)["tags"], 2);
    drop(server);

    // 目标库非空 → 再次导入被拒绝
    let out = run_cli(&["import", export_path.to_str().unwrap(), "--db", &dst_s]);
    assert!(!out.status.success(), "import into non-empty db must fail");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("not empty"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // REST 导入端点（UI 的导入走这条路径）：恢复到全新库并验证
    let rest_db = temp_db("roundtrip-rest");
    cleanup(&rest_db);
    {
        let server = HttpProc::start(&rest_db, "roundtrip-rest");
        let dump_text = std::fs::read_to_string(&export_path).unwrap();
        let (status, _, _) = try_request(
            server.port,
            "POST",
            "/api/import",
            Some(dump_text.trim()),
            &[],
        )
        .unwrap();
        assert_eq!(status, 200);
        let (status, body, _) = request(server.port, "GET", "/api/stats", None);
        assert_eq!(status, 200);
        assert_eq!(json_body(&body)["memories"], 2);
        // 非空库再次导入 → 400
        let (status, _, _) = try_request(
            server.port,
            "POST",
            "/api/import",
            Some(dump_text.trim()),
            &[],
        )
        .unwrap();
        assert_eq!(status, 400);
        drop(server);
        cleanup(&rest_db);
    }

    let _ = std::fs::remove_file(&export_path);
    cleanup(&src);
    cleanup(&dst);
}
