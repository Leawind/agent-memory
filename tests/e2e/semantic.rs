//! 语义搜索端到端：hybrid 召回、服务不可用的回退承诺、写入回退 + 补跑闭环。
//! embedding 服务用单线程 mock（确定性单热向量），测试完全可控。

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;

use crate::common::{cleanup, encodeURIComponent, json_body, request, run_cli, temp_db, HttpProc};

/// mock 向量维度：embedding = 按文本字节和对 16 取模的单热向量。
/// 同桶 = 余弦 1（语义相近），异桶 = 0（不召回），测试完全可控。
const MOCK_DIM: usize = 16;

fn mock_bucket(text: &str) -> usize {
    text.bytes().map(|b| b as usize).sum::<usize>() % MOCK_DIM
}

/// 与 embed::embed_memory_text 相同的拼装（summary + "\n\n" + content）。
fn mock_memory_text(summary: &str, content: &str) -> String {
    format!("{}\n\n{}", summary.trim(), content.trim())
}

/// 单线程 mock embedding 服务：OpenAI 兼容 /embeddings，确定性单热向量。
fn spawn_mock_embedding() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("mock bind");
    let port = listener.local_addr().expect("mock addr").port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            // 读到头部结束分隔符
            let mut buf = Vec::new();
            let mut byte = [0u8; 1];
            while let Ok(1) = stream.read(&mut byte) {
                buf.push(byte[0]);
                if buf.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            let head = String::from_utf8_lossy(&buf);
            let len: usize = head
                .lines()
                .find_map(|l| {
                    l.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|v| v.trim().parse().ok())
                })
                .unwrap_or(0);
            let mut body = vec![0u8; len];
            if len > 0 {
                let _ = stream.read_exact(&mut body);
            }
            let parsed: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
            let inputs = parsed["input"].as_array().cloned().unwrap_or_default();
            let data: Vec<Value> = inputs
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let hot = t.as_str().map(mock_bucket).unwrap_or(0);
                    let vec: Vec<f32> = (0..MOCK_DIM)
                        .map(|d| if d == hot { 1.0 } else { 0.0 })
                        .collect();
                    json!({ "index": i, "embedding": vec })
                })
                .collect();
            let payload = serde_json::to_string(&json!({ "data": data })).unwrap();
            let http = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                payload.len()
            );
            let _ = stream.write_all(http.as_bytes());
        }
    });
    port
}

fn put_settings(port: u16, body: Value) {
    let (status, resp, _) = request(
        port,
        "PUT",
        "/api/settings",
        Some(&serde_json::to_string(&body).unwrap()),
    );
    assert_eq!(
        status,
        200,
        "settings put failed: {}",
        String::from_utf8_lossy(&resp)
    );
}

/// 服务可用：hybrid 召回关键词零命中但语义同桶的记忆；
/// 服务不可用：回退关键词并打 semantic_fallback 标记。
#[test]
fn semantic_search_hybrid_and_fallback() {
    let db = temp_db("semantic");
    cleanup(&db);
    let mock_port = spawn_mock_embedding();
    let server = HttpProc::start(&db, "semantic");
    let port = server.port;

    // 未配置时 hybrid 显式请求 → 400（配置是显式要求而非静默直通）
    let (status, body, _) = request(
        port,
        "GET",
        "/api/memories?query=zigzag%20marker&mode=hybrid",
        None,
    );
    assert_eq!(status, 400);
    assert!(
        json_body(&body)["error"]
            .as_str()
            .unwrap()
            .contains("hybrid"),
        "{body:?}"
    );

    // 配置（指向 mock）
    put_settings(
        port,
        json!({
            "embedding_enabled": true,
            "embedding_base_url": format!("http://127.0.0.1:{mock_port}/v1"),
            "embedding_model": "mock-embed",
            "embedding_api_key": "sk-test"
        }),
    );
    let (status, body, _) = request(port, "GET", "/api/settings", None);
    assert_eq!(status, 200);
    let settings = json_body(&body);
    assert_eq!(settings["embedding_enabled"], true);
    assert_eq!(settings["embedding_model"], "mock-embed");
    assert_eq!(settings["embedding_api_key"], "sk-test");

    // 测试连接端点
    let (status, body, _) = request(port, "POST", "/api/embeddings/test", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["ok"], true);
    assert_eq!(json_body(&body)["dim"], MOCK_DIM);

    // 两条记忆：m1 双词都命中（"zigzag" + "marker"），m2 关键词零命中但与查询同桶
    let query = "zigzag marker";
    let bucket_q = mock_bucket(query);
    let m1_summary = "kw marker";
    let m1_content = "zigzag target here";
    assert_ne!(
        mock_bucket(&mock_memory_text(m1_summary, m1_content)),
        bucket_q,
        "m1 不得与查询同桶，否则失去对照意义"
    );
    let m2_content = (0..10000)
        .map(|i| format!("probe filler {i}"))
        .find(|c| mock_bucket(&mock_memory_text("alt wording", c)) == bucket_q)
        .expect("必须能找到与查询同桶的填充文本");
    for (summary, content) in [
        (m1_summary, m1_content),
        ("alt wording", m2_content.as_str()),
    ] {
        let (status, resp, _) = request(
            port,
            "POST",
            "/api/memories",
            Some(
                &serde_json::to_string(&json!({ "summary": summary, "content": content })).unwrap(),
            ),
        );
        assert_eq!(status, 200, "{}", String::from_utf8_lossy(&resp));
    }

    // auto（已配置）→ hybrid：两条都被召回（m1 走关键词，m2 仅向量路）
    let (status, body, _) = request(
        port,
        "GET",
        &format!("/api/memories?query={}", encodeURIComponent(query)),
        None,
    );
    assert_eq!(status, 200);
    let out = json_body(&body);
    assert_eq!(out["mode"], "hybrid");
    assert!(out.get("semantic_fallback").is_none());
    let results = out["results"].as_array().unwrap();
    assert_eq!(results.len(), 2, "向量独有命中必须被召回: {out}");
    let summaries: Vec<&str> = results
        .iter()
        .map(|r| r["summary"].as_str().unwrap())
        .collect();
    assert!(summaries.contains(&m1_summary), "{summaries:?}");
    assert!(
        summaries.contains(&"alt wording"),
        "向量独有命中缺失: {summaries:?}"
    );
    assert!(
        results.iter().all(|r| r.get("content").is_none()),
        "语义召回同样不得泄露正文"
    );

    // keyword 模式强制纯关键词：只有 m1
    let (status, body, _) = request(
        port,
        "GET",
        &format!(
            "/api/memories?query={}&mode=keyword",
            encodeURIComponent(query)
        ),
        None,
    );
    assert_eq!(status, 200);
    let out = json_body(&body);
    assert_eq!(out["mode"], "keyword");
    assert_eq!(out["results"].as_array().unwrap().len(), 1);

    // 服务不可用（指向已关闭端口）：auto 与 hybrid 都回退关键词，不打断搜索
    let dead = {
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    put_settings(
        port,
        json!({ "embedding_base_url": format!("http://127.0.0.1:{dead}/v1") }),
    );
    for mode in ["", "&mode=hybrid"] {
        let (status, body, _) = request(
            port,
            "GET",
            &format!("/api/memories?query={}{mode}", encodeURIComponent(query)),
            None,
        );
        assert_eq!(status, 200, "回退必须是成功路径: mode={mode:?}");
        let out = json_body(&body);
        assert_eq!(out["mode"], "keyword", "mode={mode:?}");
        assert_eq!(out["semantic_fallback"], true, "mode={mode:?}");
        assert_eq!(out["results"].as_array().unwrap().len(), 1, "mode={mode:?}");
    }

    // 非法 mode → 400
    let (status, _, _) = request(port, "GET", "/api/memories?query=zz&mode=quantum", None);
    assert_eq!(status, 400);

    drop(server);
    cleanup(&db);
}

/// 写入回退 + 补跑闭环：服务不可用时创建成功且记忆可见（向量留待补跑），
/// 服务恢复后 backfill 补齐，语义召回随即生效；doctor/stats 反映全程。
#[test]
fn embedding_write_fallback_then_backfill() {
    let db = temp_db("backfill");
    cleanup(&db);
    let server = HttpProc::start(&db, "backfill");
    let port = server.port;

    // 指向一个已关闭的端口（连接立刻被拒，不会拖慢测试）
    let dead = {
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    put_settings(
        port,
        json!({
            "embedding_enabled": true,
            "embedding_base_url": format!("http://127.0.0.1:{dead}/v1"),
            "embedding_model": "mock-embed"
        }),
    );

    // 写入必须成功（回退承诺），记忆立即可搜（关键词）
    let (status, resp, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(r#"{"summary": "recall probe", "content": "semantic target body"}"#),
    );
    assert_eq!(
        status,
        200,
        "服务不可用不得阻塞写入: {}",
        String::from_utf8_lossy(&resp)
    );

    // stats 反映 pending；doctor 把它列为待办
    let (status, body, _) = request(port, "GET", "/api/stats", None);
    assert_eq!(status, 200);
    let stats = json_body(&body);
    assert_eq!(stats["embedding"]["enabled"], true);
    assert_eq!(stats["embedding"]["pending"], 1);
    let (status, body, _) = request(port, "GET", "/api/doctor", None);
    assert_eq!(status, 200);
    let issues = json_body(&body)["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        issues.contains("embeddings"),
        "doctor 应报告待补跑: {issues}"
    );

    // CLI 补跑在服务不可用时失败（退出码 1），但不损坏数据
    let out = run_cli(&["embed-backfill", "--db", &db.display().to_string()]);
    assert!(!out.status.success(), "服务不可用时补跑应报错退出");

    // 服务恢复：backfill 端点补齐（?batch 控制每批条数）
    let mock_port = spawn_mock_embedding();
    put_settings(
        port,
        json!({ "embedding_base_url": format!("http://127.0.0.1:{mock_port}/v1") }),
    );
    let (status, body, _) = request(port, "POST", "/api/embeddings/backfill?batch=8", None);
    assert_eq!(status, 200);
    let out = json_body(&body);
    assert_eq!(out["configured"], true);
    assert_eq!(out["processed"], 1);
    assert_eq!(out["remaining"], 0);

    // 再跑一次：零工作量直通
    let (status, body, _) = request(port, "POST", "/api/embeddings/backfill", None);
    assert_eq!(status, 200);
    let out = json_body(&body);
    assert_eq!(out["processed"], 0);
    assert_eq!(out["remaining"], 0);

    // stats 归零，doctor 干净
    let (_, body, _) = request(port, "GET", "/api/stats", None);
    assert_eq!(json_body(&body)["embedding"]["pending"], 0);
    let (_, body, _) = request(port, "GET", "/api/doctor", None);
    assert_eq!(json_body(&body)["ok"], true);

    // 找一条与记忆文本同桶的查询词，验证补跑后语义召回生效
    let target = mock_bucket("recall probe\n\nsemantic target body");
    let query = (0..10000)
        .map(|i| format!("q{i}"))
        .find(|q| mock_bucket(q) == target)
        .expect("必须能找到同桶查询词");
    let (status, body, _) = request(
        port,
        "GET",
        &format!(
            "/api/memories?query={}&mode=hybrid",
            encodeURIComponent(&query)
        ),
        None,
    );
    assert_eq!(status, 200);
    let out = json_body(&body);
    assert_eq!(out["mode"], "hybrid");
    assert_eq!(
        out["results"].as_array().unwrap().len(),
        1,
        "补跑后应可语义召回: {out}"
    );

    // 未配置的库：backfill 端点报告 configured=false（UI 引导配置）
    let db2 = temp_db("backfill-unconfigured");
    cleanup(&db2);
    let server2 = HttpProc::start(&db2, "backfill-unconfigured");
    let (status, body, _) = request(server2.port, "POST", "/api/embeddings/backfill", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["configured"], false);
    // CLI 在未配置库上退出码 1
    let out = run_cli(&["embed-backfill", "--db", &db2.display().to_string()]);
    assert!(!out.status.success());
    drop(server2);
    cleanup(&db2);

    drop(server);
    cleanup(&db);
}
