//! 端到端集成测试：真实启动服务器二进制，通过 stdin/stdout 说 MCP 协议。

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};

static SEQ: AtomicU32 = AtomicU32::new(0);

struct Server {
    child: Child,
    stdin: Option<ChildStdin>,
    reader: BufReader<ChildStdout>,
}

impl Server {
    fn start(data: &Path) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
            .env("AGENT_MEMORY_PATH", data)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn agent-memory");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        Server {
            child,
            stdin: Some(stdin),
            reader: BufReader::new(stdout),
        }
    }

    fn send_raw(&mut self, line: &str) {
        let stdin = self.stdin.as_mut().expect("stdin still open");
        stdin.write_all(line.as_bytes()).unwrap();
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();
    }

    fn rpc(&mut self, id: u64, method: &str, params: Value) -> Value {
        let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        self.send_raw(&serde_json::to_string(&msg).unwrap());
        self.read_response(id)
    }

    fn notify(&mut self, method: &str) {
        let msg = json!({"jsonrpc": "2.0", "method": method});
        self.send_raw(&serde_json::to_string(&msg).unwrap());
    }

    fn read_response(&mut self, id: u64) -> Value {
        loop {
            let mut line = String::new();
            let n = self.reader.read_line(&mut line).expect("read line");
            assert!(n > 0, "server closed stdout before answering id {}", id);
            let v: Value = serde_json::from_str(line.trim()).expect("server sent valid JSON");
            if v.get("id").and_then(Value::as_u64) == Some(id) {
                return v;
            }
        }
    }

    /// 读取下一条响应，不管它的 id 是什么（用于 id 非法/缺失的用例）。
    fn read_next(&mut self) -> Value {
        let mut line = String::new();
        let n = self.reader.read_line(&mut line).expect("read line");
        assert!(n > 0, "server closed stdout");
        serde_json::from_str(line.trim()).expect("server sent valid JSON")
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn temp_file(tag: &str) -> PathBuf {
    let n = SEQ.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "agent-memory-e2e-{}-{}-{}.json",
        std::process::id(),
        tag,
        n
    ))
}

fn cleanup(path: &Path) {
    for p in [
        path.to_path_buf(),
        PathBuf::from(format!("{}.bak", path.display())),
        PathBuf::from(format!("{}.lock", path.display())),
        PathBuf::from(format!("{}.tmp", path.display())),
    ] {
        let _ = std::fs::remove_file(p);
    }
}

fn initialize(s: &mut Server, id: u64) -> Value {
    s.rpc(
        id,
        "initialize",
        json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "e2e-test", "version": "0.0.1"}}),
    )
}

#[test]
fn full_agent_workflow() {
    let data = temp_file("workflow");
    cleanup(&data);
    let mut s = Server::start(&data);

    let init = initialize(&mut s, 1);
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert!(init["result"]["capabilities"]["tools"].is_object());
    assert_eq!(init["result"]["serverInfo"]["name"], "agent-memory");
    assert!(init["result"]["instructions"].is_string());
    s.notify("notifications/initialized");

    let list = s.rpc(2, "tools/list", json!({}));
    let tools = list["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 10);
    for t in tools {
        assert!(
            t["inputSchema"].is_object(),
            "tool {} missing inputSchema",
            t["name"]
        );
    }

    // agent 典型流程：搜索 → 摘要 → 按需取全文
    let created = s.rpc(
        3,
        "tools/call",
        json!({"name": "memory_create", "arguments": {
            "summary": "项目使用 Rust 实现 agent 记忆系统",
            "content": "仓库位于 D:\\Workspace\\agent-memory，用 serde_json 做持久化，搜索无需分词。",
            "tags": ["项目", "rust"]
        }}),
    );
    assert!(created.get("error").is_none(), "create failed: {}", created);
    assert!(created["result"].get("isError").is_none());
    let mem_id = created["result"]["structuredContent"]["memory"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        created["result"]["structuredContent"]["tags_autocreated"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let searched = s.rpc(
        4,
        "tools/call",
        json!({"name": "memory_search", "arguments": {"query": "记忆系统"}}),
    );
    let results = searched["result"]["structuredContent"]["results"]
        .as_array()
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["id"], mem_id.as_str());
    assert!(
        results[0].get("content").is_none(),
        "search must not leak full content"
    );

    let fetched = s.rpc(
        5,
        "tools/call",
        json!({"name": "memory_get", "arguments": {"ids": [mem_id]}}),
    );
    let full = &fetched["result"]["structuredContent"]["memories"][0];
    assert!(full["content"].as_str().unwrap().contains("serde_json"));

    // 标签增删查改
    s.rpc(
        6,
        "tools/call",
        json!({"name": "tag_create", "arguments": {"name": "infra", "description": "基础设施"}}),
    );
    s.rpc(
        7,
        "tools/call",
        json!({"name": "memory_update", "arguments": {"id": mem_id, "add_tags": ["infra"]}}),
    );
    let renamed = s.rpc(
        8,
        "tools/call",
        json!({"name": "tag_rename", "arguments": {"old_name": "infra", "new_name": "基建"}}),
    );
    assert_eq!(
        renamed["result"]["structuredContent"]["memories_updated"],
        1
    );

    let tl = s.rpc(
        9,
        "tools/call",
        json!({"name": "tag_list", "arguments": {}}),
    );
    let tags = tl["result"]["structuredContent"]["tags"]
        .as_array()
        .unwrap();
    assert!(tags
        .iter()
        .any(|t| t["name"] == "基建" && t["memory_count"] == 1));

    // 数据在进程重启后仍在（验证落盘）
    drop(s);
    let mut s2 = Server::start(&data);
    let init2 = initialize(&mut s2, 1);
    assert_eq!(init2["result"]["protocolVersion"], "2025-06-18");
    let again = s2.rpc(
        2,
        "tools/call",
        json!({"name": "memory_search", "arguments": {"query": "rust"}}),
    );
    assert_eq!(again["result"]["structuredContent"]["total_matches"], 1);

    // 清理：删除记忆后 tag purge
    s2.rpc(
        3,
        "tools/call",
        json!({"name": "memory_delete", "arguments": {"ids": [mem_id]}}),
    );
    let purged = s2.rpc(
        4,
        "tools/call",
        json!({"name": "tag_delete", "arguments": {"name": "项目", "mode": "purge"}}),
    );
    assert_eq!(
        purged["result"]["structuredContent"]["memories_deleted"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    drop(s2);
    cleanup(&data);
}

#[test]
fn protocol_error_handling() {
    let data = temp_file("errors");
    cleanup(&data);
    let mut s = Server::start(&data);
    initialize(&mut s, 1);

    // 未知工具 → JSON-RPC 错误
    let r = s.rpc(
        2,
        "tools/call",
        json!({"name": "no_such_tool", "arguments": {}}),
    );
    assert_eq!(r["error"]["code"], -32602);

    // 未知方法 → -32601
    let r = s.rpc(3, "bogus/method", json!({}));
    assert_eq!(r["error"]["code"], -32601);

    // 工具参数错误 → isError 结果
    let r = s.rpc(
        4,
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "only summary"}}),
    );
    assert_eq!(r["result"]["isError"], true);
    assert!(r["result"]["content"][0]["text"].is_string());

    // 旧版协议号按客户端请求的回显
    let legacy = s.rpc(5, "initialize", json!({"protocolVersion": "2024-11-05"}));
    assert_eq!(legacy["result"]["protocolVersion"], "2024-11-05");

    // 未知协议号 → 服务器最新版
    let unknown = s.rpc(6, "initialize", json!({"protocolVersion": "1999-01-01"}));
    assert_eq!(unknown["result"]["protocolVersion"], "2025-06-18");

    // ping
    let pong = s.rpc(7, "ping", json!({}));
    assert_eq!(pong["result"], json!({}));

    // 发送垃圾数据后服务器仍能继续服务
    s.send_raw("this is not json at all");
    let after = s.rpc(8, "tools/list", json!({}));
    assert_eq!(after["result"]["tools"].as_array().unwrap().len(), 10);

    // 缺 method 但带 id → 无效请求 -32600（而不是无响应导致客户端挂起）
    s.send_raw(r#"{"jsonrpc":"2.0","id":20}"#);
    let invalid = s.read_response(20);
    assert_eq!(invalid["error"]["code"], -32600);

    // params 为 null 的合法请求按空参数照常处理
    s.send_raw(r#"{"jsonrpc":"2.0","id":21,"method":"tools/list","params":null}"#);
    let null_params = s.read_response(21);
    assert_eq!(null_params["result"]["tools"].as_array().unwrap().len(), 10);

    // memory_create 的重复摘要提示走完整协议链路
    s.rpc(
        22,
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "dup check", "content": "c1"}}),
    );
    let second = s.rpc(
        23,
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "Dup Check", "content": "c2"}}),
    );
    let dups = second["result"]["structuredContent"]["duplicate_of"]
        .as_array()
        .unwrap();
    assert_eq!(dups.len(), 1);
    assert_eq!(dups[0], "m1");

    // 对抗性输入样本（模糊探针的代表用例）：非法 id 类型、params 数组、深嵌套、标量整行
    s.send_raw(r#"{"jsonrpc":"2.0","id":{"o":1},"method":"ping"}"#);
    let obj_id = s.read_next();
    assert_eq!(obj_id["result"], json!({}));
    assert_eq!(obj_id["id"]["o"], 1);

    s.send_raw(r#"{"jsonrpc":"2.0","id":30,"method":"ping","params":[1,2]}"#);
    let arr_params = s.read_response(30);
    assert_eq!(arr_params["result"], json!({}));

    // 深嵌套超过 serde_json 递归限制 → 解析错误而不是崩溃
    let mut deep = String::from(r#"{"jsonrpc":"2.0","id":31,"method":"ping","params":{"d":"#);
    deep.push_str(&"[".repeat(200));
    deep.push_str(&"]".repeat(200));
    deep.push_str("}}");
    s.send_raw(&deep);
    let deep_r = s.read_next();
    assert_eq!(deep_r["error"]["code"], -32700);

    // 无 id 无 method 的标量整行：静默丢弃
    s.send_raw("42");
    s.send_raw("null");

    // 风暴后服务器仍然完全可用
    let after = s.rpc(32, "tools/list", json!({}));
    assert_eq!(after["result"]["tools"].as_array().unwrap().len(), 10);

    drop(s);
    cleanup(&data);
}

#[test]
fn graceful_shutdown_on_eof() {
    let data = temp_file("eof");
    cleanup(&data);
    let mut s = Server::start(&data);
    initialize(&mut s, 1);
    s.rpc(
        2,
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "s", "content": "c"}}),
    );
    // 关闭 stdin 触发 EOF，服务器应正常退出（exit code 0）
    drop(s.stdin.take().expect("stdin still open"));
    let status = s.child.wait().expect("wait for exit");
    assert!(status.success(), "expected exit code 0, got {:?}", status);
    drop(s);
    cleanup(&data);
}

#[test]
fn two_processes_do_not_lose_updates() {
    let data = temp_file("concurrent");
    cleanup(&data);

    // 两个服务器进程共享同一数据文件（模拟用户同时开多个 agent 会话）
    let mut a = Server::start(&data);
    let mut b = Server::start(&data);
    initialize(&mut a, 1);
    initialize(&mut b, 1);

    // A 写入一条；B 的下一次请求会在锁内重读磁盘，必须能看到 A 的写入
    a.rpc(
        2,
        "tools/call",
        json!({"name": "memory_create", "arguments": {
            "summary": "written by process A", "content": "content A", "tags": ["proc-a"]
        }}),
    );
    b.rpc(
        2,
        "tools/call",
        json!({"name": "memory_create", "arguments": {
            "summary": "written by process B", "content": "content B", "tags": ["proc-b"]
        }}),
    );

    // B 修改 A 创建的标签体系：重命名 A 的标签，A 端数据不丢
    let renamed = b.rpc(
        3,
        "tools/call",
        json!({"name": "tag_rename", "arguments": {"old_name": "proc-a", "new_name": "from-a"}}),
    );
    assert_eq!(
        renamed["result"]["structuredContent"]["memories_updated"],
        1
    );

    // 第三个全新进程验证：A、B 的记忆都还在，标签改名已生效
    drop(a);
    drop(b);
    let mut c = Server::start(&data);
    initialize(&mut c, 1);
    let search = c.rpc(
        2,
        "tools/call",
        json!({"name": "memory_search", "arguments": {"query": "written by process"}}),
    );
    assert_eq!(search["result"]["structuredContent"]["total_matches"], 2);
    let listing = c.rpc(
        3,
        "tools/call",
        json!({"name": "memory_list", "arguments": {"tag": "from-a"}}),
    );
    let items = listing["result"]["structuredContent"]["memories"]
        .as_array()
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["summary"], "written by process A");

    drop(c);
    cleanup(&data);
}

#[test]
fn unknown_argument_returns_clear_error() {
    let data = temp_file("unknown-arg");
    cleanup(&data);
    let mut s = Server::start(&data);
    initialize(&mut s, 1);
    let r = s.rpc(
        2,
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summray": "typo", "content": "c"}}),
    );
    let text = r["result"]["content"][0]["text"].as_str().unwrap();
    assert_eq!(r["result"]["isError"], true);
    assert!(
        text.contains("'summray'") && text.contains("summary"),
        "got: {}",
        text
    );
    drop(s);
    cleanup(&data);
}

#[test]
fn concurrent_processes_serialize_on_the_lock() {
    let data = temp_file("race");
    cleanup(&data);

    let mut a = Server::start(&data);
    initialize(&mut a, 1);

    // 第二个进程在独立线程里跑，与 A 同时完成首写：两个请求会竞争同一把
    // 文件锁，写入由锁串行化，两条都必须存活。线程把 Server 带回来继续用。
    let data_for_b = data.clone();
    let b_thread = std::thread::spawn(move || {
        let mut b = Server::start(&data_for_b);
        initialize(&mut b, 1);
        let first = b.rpc(
            1,
            "tools/call",
            json!({"name": "memory_create", "arguments": {"summary": "race: written by B", "content": "B", "tags": ["race"]}}),
        );
        (b, first)
    });
    let a_first = a.rpc(
        1,
        "tools/call",
        json!({"name": "memory_create", "arguments": {"summary": "race: written by A", "content": "A", "tags": ["race"]}}),
    );
    let (mut b, b_first) = b_thread.join().expect("writer B thread");
    assert!(
        a_first.get("error").is_none(),
        "A's concurrent write failed: {}",
        a_first
    );
    assert!(
        b_first.get("error").is_none(),
        "B's concurrent write failed: {}",
        b_first
    );

    // 交替写入：每次请求都要在锁内看到对方的写入，10 次后总数必须恰好 12
    for i in 0..5 {
        a.rpc(
            10 + i,
            "tools/call",
            json!({"name": "memory_create", "arguments": {"summary": format!("race: A interleaved {}", i), "content": "A", "tags": ["race"]}}),
        );
        b.rpc(
            10 + i,
            "tools/call",
            json!({"name": "memory_create", "arguments": {"summary": format!("race: B interleaved {}", i), "content": "B", "tags": ["race"]}}),
        );
    }

    let search = a.rpc(
        99,
        "tools/call",
        json!({"name": "memory_search", "arguments": {"query": "race", "limit": 50}}),
    );
    assert_eq!(
        search["result"]["structuredContent"]["total_matches"], 12,
        "lost updates detected under concurrent writers"
    );

    drop(a);
    drop(b);
    cleanup(&data);
}
