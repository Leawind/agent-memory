//! Semantic search end-to-end: hybrid recall, the fallback promise when the service is down,
//! and the write-fallback + backfill loop. The embedding service is a single-threaded mock
//! (deterministic one-hot vectors), keeping the tests fully controllable.

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;

use crate::common::{cleanup, encodeURIComponent, json_body, request, run_cli, temp_db, HttpProc};

/// Mock vector dimensions: the embedding is a one-hot vector indexed by the text's byte sum modulo 16.
/// Same bucket = cosine 1 (semantically close), different bucket = 0 (not recalled), keeping tests fully controllable.
const MOCK_DIM: usize = 16;

fn mock_bucket(text: &str) -> usize {
    text.bytes().map(|b| b as usize).sum::<usize>() % MOCK_DIM
}

/// Same assembly as embed::embed_memory_text (summary + "\n\n" + content).
fn mock_memory_text(summary: &str, content: &str) -> String {
    format!("{}\n\n{}", summary.trim(), content.trim())
}

/// Single-threaded mock embedding service: OpenAI-compatible /embeddings, deterministic one-hot vectors.
fn spawn_mock_embedding() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("mock bind");
    let port = listener.local_addr().expect("mock addr").port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            // Read until the header-terminating delimiter
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

/// Service available: hybrid recalls memories with zero keyword hits that are semantically in the same bucket;
/// service unavailable: falls back to keywords and flags semantic_fallback.
#[test]
fn semantic_search_hybrid_and_fallback() {
    let db = temp_db("semantic");
    cleanup(&db);
    let mock_port = spawn_mock_embedding();
    let server = HttpProc::start(&db, "semantic");
    let port = server.port;

    // Explicit hybrid request while unconfigured -> 400 (configuration is required explicitly, no silent pass-through)
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

    // Configure (pointing at the mock)
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

    // Test-connection endpoint
    let (status, body, _) = request(port, "POST", "/api/embeddings/test", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["ok"], true);
    assert_eq!(json_body(&body)["dim"], MOCK_DIM);

    // Two memories: m1 hits both words ("zigzag" + "marker"), m2 has zero keyword hits but shares a bucket with the query
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

    // auto (configured) -> hybrid: both recalled (m1 via keywords, m2 via the vector path only).
    // Vectors land via the background backfill worker triggered by each write, so poll briefly
    // for the vector-only hit instead of assuming synchronous embedding.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let out = loop {
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
        if out["results"].as_array().is_some_and(|a| a.len() == 2) {
            break out;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "vector-only hit was never recalled: {out}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
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

    // keyword mode forces pure keywords: only m1
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

    // Service down (pointing at a closed port): both auto and hybrid fall back to keywords without interrupting the search
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

    // Invalid mode -> 400
    let (status, _, _) = request(port, "GET", "/api/memories?query=zz&mode=quantum", None);
    assert_eq!(status, 400);

    drop(server);
    cleanup(&db);
}

/// Write-fallback + backfill loop: when the service is down, creation succeeds and the memory is visible
/// (vectors left pending for backfill); once the service recovers, backfill fills them in and semantic
/// recall takes effect; doctor/stats reflect the whole journey.
#[test]
fn embedding_write_fallback_then_backfill() {
    let db = temp_db("backfill");
    cleanup(&db);
    let server = HttpProc::start(&db, "backfill");
    let port = server.port;

    // Point at a closed port (connection refused immediately, does not slow the test)
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

    // The write must succeed (fallback promise), the memory immediately searchable via keywords
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

    // stats reflects pending; doctor lists it as a to-do
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

    // CLI backfill fails while the service is down (exit code 1) but does not corrupt data
    let out = run_cli(&["embed-backfill", "--db", &db.display().to_string()]);
    assert!(!out.status.success(), "服务不可用时补跑应报错退出");

    // Service recovered: the backfill endpoint fills in (?batch controls items per batch)
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

    // Run again: zero work pass-through
    let (status, body, _) = request(port, "POST", "/api/embeddings/backfill", None);
    assert_eq!(status, 200);
    let out = json_body(&body);
    assert_eq!(out["processed"], 0);
    assert_eq!(out["remaining"], 0);

    // stats back to zero, doctor clean
    let (_, body, _) = request(port, "GET", "/api/stats", None);
    assert_eq!(json_body(&body)["embedding"]["pending"], 0);
    let (_, body, _) = request(port, "GET", "/api/doctor", None);
    assert_eq!(json_body(&body)["ok"], true);

    // Find a query sharing a bucket with the memory text to verify semantic recall works after backfill
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

    // Unconfigured db: the backfill endpoint reports configured=false (the UI guides configuration)
    let db2 = temp_db("backfill-unconfigured");
    cleanup(&db2);
    let server2 = HttpProc::start(&db2, "backfill-unconfigured");
    let (status, body, _) = request(server2.port, "POST", "/api/embeddings/backfill", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["configured"], false);
    // CLI exits with code 1 on an unconfigured db
    let out = run_cli(&["embed-backfill", "--db", &db2.display().to_string()]);
    assert!(!out.status.success());
    drop(server2);
    cleanup(&db2);

    drop(server);
    cleanup(&db);
}
