//! Test infrastructure: temporary databases, server process management, a hand-written minimal HTTP client, and a CLI runner.

use serde_json::{json, Value};
use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

/// The only protocol revision the modern server speaks.
pub(crate) const MCP_PROTOCOL_VERSION: &str = "2026-07-28";

/// Build a modern-protocol JSON-RPC request body: injects the required `_meta` fields into params
/// (the stateless replacement for the initialize handshake).
pub(crate) fn rpc_body(id: Value, method: &str, params: Value) -> String {
    let mut params = if params.is_object() {
        params
    } else {
        json!({})
    };
    params["_meta"] = json!({
        "io.modelcontextprotocol/protocolVersion": MCP_PROTOCOL_VERSION,
        "io.modelcontextprotocol/clientCapabilities": {},
    });
    serde_json::to_string(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
        .expect("serialize rpc body")
}

/// The mirrored request headers a conforming modern client must send; Mcp-Name only for
/// tools/call (params.name) and resources/read (params.uri).
pub(crate) fn mcp_headers(method: &str, params: &Value) -> Vec<(String, String)> {
    let mut out = vec![
        (
            "MCP-Protocol-Version".to_string(),
            MCP_PROTOCOL_VERSION.to_string(),
        ),
        ("Mcp-Method".to_string(), method.to_string()),
    ];
    let name = match method {
        "tools/call" => params.get("name").and_then(Value::as_str),
        "resources/read" => params.get("uri").and_then(Value::as_str),
        _ => None,
    };
    if let Some(n) = name {
        out.push(("Mcp-Name".to_string(), n.to_string()));
    }
    out
}

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

/// cargo test runs multi-threaded by default: there is a race window between port allocation
/// and the child process binding it; serialize "pick a free port -> child binds successfully" with a global lock.
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

    /// Poll /health until ready; an early child exit counts as a startup failure.
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

/// Send an HTTP/1.1 request and read the full response (Connection: close).
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

/// Send a JSON-RPC request as a conforming modern client, with optional extra headers (e.g.
/// Authorization). Returns (status, parsed body or Null when the body is empty, e.g. 202).
pub(crate) fn mcp_post(
    port: u16,
    id: Value,
    method: &str,
    params: Value,
    extra: &[(&str, &str)],
) -> (u16, Value) {
    let body = rpc_body(id, method, params.clone());
    let mut headers = mcp_headers(method, &params);
    headers.extend(extra.iter().map(|(k, v)| (k.to_string(), v.to_string())));
    let refs: Vec<(&str, &str)> = headers
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let (status, bytes, _) = try_request(port, "POST", "/mcp", Some(&body), &refs).unwrap();
    let parsed = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, parsed)
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

#[allow(non_snake_case)] // same name as JS encodeURIComponent for easy cross-referencing
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
