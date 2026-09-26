//! 测试基建：临时数据库、服务器进程管理、手写最小 HTTP 客户端与 CLI 运行器。

use serde_json::Value;
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

static SEQ: AtomicU32 = AtomicU32::new(0);

pub(crate) fn temp_db(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "agent-memory-e2e-{}-{}-{}.db",
        std::process::id(),
        tag,
        n
    ))
}

pub(crate) fn cleanup(path: &Path) {
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(PathBuf::from(format!("{}{}", path.display(), suffix)));
    }
}

fn free_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
    l.local_addr().expect("local addr").port()
}

pub(crate) struct HttpProc {
    child: std::process::Child,
    pub(crate) port: u16,
}

/// cargo test 默认多线程并行：端口分配到子进程绑定之间存在竞争窗口，
/// 用全局锁把"取空闲端口 → 子进程绑定成功"串行化。
static START_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

impl HttpProc {
    pub(crate) fn start(db: &Path, tag: &str) -> HttpProc {
        Self::start_with(db, tag, &[])
    }

    pub(crate) fn start_with(db: &Path, tag: &str, extra_args: &[&str]) -> HttpProc {
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
            .args(extra_args)
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
pub(crate) fn try_request(
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

pub(crate) fn request(
    port: u16,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> (u16, Vec<u8>, String) {
    try_request(port, method, path, body, &[]).expect("http request succeeds")
}

pub(crate) fn json_body(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("valid JSON body")
}

pub(crate) fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_agent-memory"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("run agent-memory CLI")
}

#[allow(non_snake_case)] // 与 JS encodeURIComponent 同名，便于对照
pub(crate) fn encodeURIComponent(s: &str) -> String {
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
