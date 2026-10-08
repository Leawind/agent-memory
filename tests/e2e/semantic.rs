//! Semantic search end-to-end: hybrid recall, the fallback promise when the service is down,
//! and the write-fallback + backfill loop. The embedding service is a single-threaded mock
//! (deterministic one-hot vectors), keeping the tests fully controllable.

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

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

/// Read one HTTP request off the stream and return its JSON body (mock-service helper).
fn read_json_body(stream: &mut std::net::TcpStream) -> Value {
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
    serde_json::from_slice(&body).unwrap_or(json!({}))
}

/// Write one HTTP 200 JSON response (mock-service helper).
fn respond_json(stream: &mut std::net::TcpStream, payload: &Value) {
    let body = serde_json::to_string(payload).unwrap();
    let http = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(http.as_bytes());
}

/// Single-threaded mock embedding service: OpenAI-compatible /embeddings, deterministic one-hot vectors.
/// Returns the port plus a handle on every input text ever received (for asserting what the
/// server actually sent — instruction prefixes among them).
fn spawn_mock_embedding() -> (u16, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("mock bind");
    let port = listener.local_addr().expect("mock addr").port();
    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let handle = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let parsed = read_json_body(&mut stream);
            let inputs = parsed["input"].as_array().cloned().unwrap_or_default();
            seen.lock()
                .unwrap()
                .extend(inputs.iter().filter_map(|t| t.as_str().map(str::to_string)));
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
            respond_json(&mut stream, &json!({ "data": data }));
        }
    });
    (port, handle)
}

/// Single-threaded mock reranker: Cohere-style /rerank that scores document i with `i` —
/// descending sort therefore REVERSES the incoming (fused) order, making the reorder visible.
/// Records every (query, documents) pair it saw.
type SeenRerank = Arc<Mutex<Vec<(String, Vec<String>)>>>;

fn spawn_mock_reranker() -> (u16, SeenRerank) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("mock reranker bind");
    let port = listener.local_addr().expect("mock addr").port();
    let seen: SeenRerank = Arc::new(Mutex::new(Vec::new()));
    let handle = seen.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let parsed = read_json_body(&mut stream);
            let query = parsed["query"].as_str().unwrap_or_default().to_string();
            let docs: Vec<String> = parsed["documents"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
                .collect();
            seen.lock().unwrap().push((query.clone(), docs.clone()));
            let results: Vec<Value> = docs
                .iter()
                .enumerate()
                .map(|(i, _)| json!({ "index": i, "relevance_score": i as f64 }))
                .collect();
            respond_json(&mut stream, &json!({ "results": results }));
        }
    });
    (port, handle)
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

#[test]
fn semantic_tag_filter_applies_before_recall_limit() {
    let db = temp_db("semantic-filter-cap");
    cleanup(&db);
    let server = HttpProc::start(&db, "semantic-filter-cap");
    let port = server.port;
    let query = "needle-query";
    let query_bucket = mock_bucket(query);

    let mut selected_id = Value::Null;
    for i in 0..25 {
        let summary = format!("candidate {i}");
        let content = (0..100)
            .map(|n| format!("alternate wording {n}"))
            .find(|body| mock_bucket(&mock_memory_text(&summary, body)) == query_bucket)
            .unwrap();
        let selected = i == 24;
        let (status, body, _) = request(
            port,
            "POST",
            "/api/memories",
            Some(
                &json!({
                    "summary": summary,
                    "content": content,
                    "tags": if selected { vec!["selected"] } else { Vec::new() },
                    "create_missing_tags": true
                })
                .to_string(),
            ),
        );
        assert_eq!(status, 200);
        if selected {
            selected_id = json_body(&body)["id"].clone();
        }
    }

    let (mock_port, _) = spawn_mock_embedding();
    put_settings(
        port,
        json!({"search_limits": {"semantic_candidates": 20, "rerank_candidates": 20}, "embedding_models": [{
            "id": "mock-embed", "model": "mock-embed",
            "base_url": format!("http://127.0.0.1:{mock_port}/v1")
        }]}),
    );
    let mut remaining = 25;
    for _ in 0..25 {
        let (status, body, _) = request(port, "POST", "/api/embeddings/backfill", None);
        assert_eq!(status, 200, "{}", String::from_utf8_lossy(&body));
        remaining = json_body(&body)["remaining"].as_u64().unwrap();
        if remaining == 0 {
            break;
        }
    }
    assert_eq!(remaining, 0);

    let mut first_id = Value::Null;
    for limit in [1, 5, 50] {
        let (status, body, _) = request(
            port,
            "GET",
            &format!("/api/memories?query={query}&limit={limit}"),
            None,
        );
        assert_eq!(status, 200);
        let result = json_body(&body);
        assert_eq!(result["total_matches"], 20);
        if first_id.is_null() {
            first_id = result["results"][0]["id"].clone();
        }
        assert_eq!(result["results"][0]["id"], first_id);
    }

    for expression in ["selected", "selected&!convention", "/^selected$/"] {
        let (status, body, _) = request(
            port,
            "GET",
            &format!(
                "/api/memories?query={query}&tag_expr={}&limit=1&mode=hybrid",
                encodeURIComponent(expression)
            ),
            None,
        );
        assert_eq!(status, 200);
        let result = json_body(&body);
        assert_eq!(result["total_matches"], 1, "{result}");
        assert_eq!(result["results"][0]["id"], selected_id);
        assert!(result["results"][0].get("content").is_none());
    }

    drop(server);
    cleanup(&db);
}

#[test]
fn candidate_budgets_are_configurable_and_page_independent() {
    let db = temp_db("search-budgets");
    cleanup(&db);
    let server = HttpProc::start(&db, "search-budgets");
    let port = server.port;
    let (_, body, _) = request(port, "GET", "/api/settings", None);
    assert_eq!(
        json_body(&body)["search_limits"],
        json!({"semantic_candidates": 100, "rerank_candidates": 50})
    );
    let (status, _, _) = request(port, "PUT", "/api/settings", Some(&json!({
        "instructions": "must not persist", "search_limits": {"semantic_candidates": 2, "rerank_candidates": 3}
    }).to_string()));
    assert_eq!(status, 400);
    let (_, body, _) = request(port, "GET", "/api/settings", None);
    assert!(json_body(&body)["instructions"].is_null());

    for summary in ["alpha first", "alpha second", "alpha third"] {
        let (status, _, _) = request(
            port,
            "POST",
            "/api/memories",
            Some(&json!({"summary": summary}).to_string()),
        );
        assert_eq!(status, 200);
    }
    let (mock_port, seen) = spawn_mock_reranker();
    put_settings(
        port,
        json!({
            "rerank_models": [{"id": "mock", "model": "mock", "base_url": format!("http://127.0.0.1:{mock_port}/v1")}],
            "search_limits": {"semantic_candidates": 2, "rerank_candidates": 2}
        }),
    );
    for (limit, offset, expected) in [
        (1, 0, "alpha second"),
        (1, 1, "alpha first"),
        (3, 0, "alpha second"),
    ] {
        let (status, body, _) = request(
            port,
            "GET",
            &format!("/api/memories?query=alpha&limit={limit}&offset={offset}"),
            None,
        );
        assert_eq!(status, 200);
        let result = json_body(&body);
        assert_eq!(result["total_matches"], 3);
        assert_eq!(result["results"][0]["summary"], expected);
        assert!(result["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item.get("content").is_none()));
    }
    assert!(seen.lock().unwrap().iter().all(|(_, docs)| docs.len() == 2));
    put_settings(
        port,
        json!({"search_limits": {"semantic_candidates": 3, "rerank_candidates": 3}}),
    );
    let (_, body, _) = request(port, "GET", "/api/memories?query=alpha", None);
    assert_eq!(json_body(&body)["results"][0]["summary"], "alpha third");

    drop(server);
    cleanup(&db);
}

/// Service available: hybrid recalls memories with zero keyword hits that are semantically in the same bucket;
/// service unavailable: falls back to keywords and flags semantic_fallback.
#[test]
fn semantic_search_hybrid_and_fallback() {
    let db = temp_db("semantic");
    cleanup(&db);
    let (mock_port, _seen) = spawn_mock_embedding();
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

    // Configure (pointing at the mock; one enabled candidate)
    put_settings(
        port,
        json!({
            "embedding_models": [{
                "base_url": format!("http://127.0.0.1:{mock_port}/v1"),
                "id": "mock-embed", "model": "mock-embed",
                "api_key": "sk-test"
            }]
        }),
    );
    let (status, body, _) = request(port, "GET", "/api/settings", None);
    assert_eq!(status, 200);
    let settings = json_body(&body);
    let models = settings["embedding_models"].as_array().unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0]["model"], "mock-embed");
    assert_eq!(models[0]["api_key"], "sk-test");
    assert_eq!(models[0]["enabled"], true);

    // Test-connection endpoint: per-candidate health in one response
    let (status, body, _) = request(port, "POST", "/api/embeddings/test", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["ok"], true);
    assert_eq!(json_body(&body)["results"][0]["ok"], true);
    assert_eq!(json_body(&body)["results"][0]["dim"], MOCK_DIM);

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
        json!({ "embedding_models": [{
            "base_url": format!("http://127.0.0.1:{dead}/v1"),
            "id": "mock-embed", "name": "mock-embed", "model": "mock-embed"
        }] }),
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
            "embedding_models": [{
                "base_url": format!("http://127.0.0.1:{dead}/v1"),
                "id": "mock-embed", "name": "mock-embed", "model": "mock-embed"
            }]
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
    // The doctor response is a named checklist; flatten the issue lines across checks
    let issues = json_body(&body)["checks"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| {
            c["issues"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap_or_default().to_string())
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        issues.contains("embeddings"),
        "doctor 应报告待补跑: {issues}"
    );

    // CLI backfill fails while the service is down (exit code 1) but does not corrupt data
    let out = run_cli(&["embed-backfill", "--db", &db.display().to_string()]);
    assert!(!out.status.success(), "服务不可用时补跑应报错退出");

    // Service recovered: the backfill endpoint fills in ("batch" controls items per batch)
    let (mock_port, _seen) = spawn_mock_embedding();
    put_settings(
        port,
        json!({ "embedding_models": [{
            "base_url": format!("http://127.0.0.1:{mock_port}/v1"),
            "id": "mock-embed", "name": "mock-embed", "model": "mock-embed"
        }] }),
    );
    let (status, body, _) = request(
        port,
        "POST",
        "/api/embeddings/backfill",
        Some(r#"{"batch": 8}"#),
    );
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

/// Instruction prefixes (E5-style models): the query side and the passage side each get their
/// configured prefix verbatim, and changing a prefix re-keys every stored vector — they go
/// pending for backfill exactly like a model switch, instead of silently staying in use.
#[test]
fn embedding_prefixes_apply_and_rekey_vectors() {
    let db = temp_db("semantic-prefix");
    cleanup(&db);
    let (mock_port, seen) = spawn_mock_embedding();
    let server = HttpProc::start(&db, "semantic-prefix");
    let port = server.port;

    put_settings(
        port,
        json!({
            "embedding_models": [{
                "base_url": format!("http://127.0.0.1:{mock_port}/v1"),
                "id": "mock-embed", "name": "mock-embed", "model": "mock-embed",
                "query_prefix": "q>> ",
                "passage_prefix": "p>> "
            }]
        }),
    );
    // Settings roundtrip: prefixes are echoed verbatim
    let (_, body, _) = request(port, "GET", "/api/settings", None);
    let settings = json_body(&body);
    assert_eq!(settings["embedding_models"][0]["query_prefix"], "q>> ");
    assert_eq!(settings["embedding_models"][0]["passage_prefix"], "p>> ");

    // Creating a memory leaves its vector pending (the REST write face has no synchronous
    // embedding); the write hook's background worker (or an explicit backfill) drains it:
    // the passage prefix must go out verbatim
    let (status, resp, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(
            &serde_json::to_string(&json!({ "summary": "prefix probe", "content": "body text" }))
                .unwrap(),
        ),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&resp));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if seen
            .lock()
            .unwrap()
            .iter()
            .any(|t| t == "p>> prefix probe\n\nbody text")
        {
            break;
        }
        // Nudge the queue in case the background worker is not running (fallback, not the norm)
        let _ = request(port, "POST", "/api/embeddings/backfill", None);
        assert!(
            std::time::Instant::now() < deadline,
            "passage embedding never went out: {:?}",
            seen.lock().unwrap()
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Searching embeds the query: the query prefix went out, distinct from the passage side
    let (status, body, _) = request(
        port,
        "GET",
        &format!("/api/memories?query={}", encodeURIComponent("prefix probe")),
        None,
    );
    assert_eq!(status, 200);
    let out = json_body(&body);
    assert_eq!(out["mode"], "hybrid");
    let seen_now = seen.lock().unwrap().clone();
    assert!(
        seen_now.iter().any(|t| t == "q>> prefix probe"),
        "query text must carry the query prefix verbatim: {seen_now:?}"
    );

    // The connection test goes through the query side too
    let before = seen.lock().unwrap().len();
    let (status, body, _) = request(port, "POST", "/api/embeddings/test", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["results"][0]["ok"], true);
    let seen_now = seen.lock().unwrap().clone();
    assert!(seen_now[before..]
        .iter()
        .any(|t| t == "q>> connection test 连接测试"));

    // Changing a prefix re-keys the vectors: coverage drops to all-pending under the new key
    // (same behavior as a model switch), while the displayed model name stays the raw model
    put_settings(
        port,
        json!({ "embedding_models": [{
            "base_url": format!("http://127.0.0.1:{mock_port}/v1"),
            "id": "mock-embed", "name": "mock-embed", "model": "mock-embed",
            "query_prefix": "q>> ",
            "passage_prefix": "p2>> "
        }] }),
    );
    let (_, body, _) = request(port, "GET", "/api/stats", None);
    let emb = &json_body(&body)["embedding"];
    assert_eq!(emb["model"], "mock-embed");
    assert_eq!(
        emb["embedded"], 0,
        "old-key vectors must not count for the new key"
    );
    assert_eq!(
        emb["pending"], 1,
        "the memory must go pending for re-embedding"
    );

    drop(server);
    cleanup(&db);
}

/// Several embedding candidates at once: their caches coexist (per-identity keys), the caches
/// endpoint lists/backs up/deletes them per model, and a dead head candidate fails over to the
/// next one on the actual search path (which the response names via embedding_model).
#[test]
fn embedding_multi_model_caches_and_failover() {
    let db = temp_db("semantic-multi");
    cleanup(&db);
    let (mock_port, _seen) = spawn_mock_embedding();
    let server = HttpProc::start(&db, "semantic-multi");
    let port = server.port;

    // Two candidates sharing one mock (it serves any model name): mock-a is the priority head
    put_settings(
        port,
        json!({
            "embedding_models": [
                {"base_url": format!("http://127.0.0.1:{mock_port}/v1"), "id": "mock-a", "name": "mock-a", "model": "mock-a"},
                {"base_url": format!("http://127.0.0.1:{mock_port}/v1"), "id": "mock-b", "name": "mock-b", "model": "mock-b"}
            ]
        }),
    );
    let (status, resp, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(r#"{"summary": "cache probe", "content": "multi model body"}"#),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&resp));

    // The head candidate's cache is drained by the write hook's background worker (or an
    // explicit nudge); the second candidate's cache stays empty but listed
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let caches = loop {
        let _ = request(port, "POST", "/api/embeddings/backfill", None);
        let caches = get_caches(port);
        if find_cache(&caches, "mock-a")["embedded"] == 1 {
            break caches;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "head candidate was never backfilled: {caches:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    assert_eq!(caches.len(), 2, "{caches:?}");
    let a = find_cache(&caches, "mock-a");
    let b = find_cache(&caches, "mock-b");
    assert_eq!(a["pending"], 0);
    assert_eq!(a["configured"], true);
    assert_eq!(
        b["embedded"], 0,
        "the second candidate is never auto-backfilled"
    );
    assert_eq!(b["pending"], 1);
    assert_eq!(b["configured"], true);

    // Per-model backfill: only the targeted identity's cache fills in
    let b_key = b["key"].as_str().unwrap();
    let (status, body, _) = request(
        port,
        "POST",
        "/api/embeddings/backfill",
        Some(&serde_json::to_string(&json!({ "model_key": b_key })).unwrap()),
    );
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["processed"], 1);
    let caches = get_caches(port);
    assert_eq!(find_cache(&caches, "mock-b")["embedded"], 1);
    assert_eq!(find_cache(&caches, "mock-b")["pending"], 0);

    // Backfilling an identity no enabled entry matches -> 400 (deletion is the only op left)
    let (status, body, _) = request(
        port,
        "POST",
        "/api/embeddings/backfill",
        Some(r#"{"model_key": "ghost-model"}"#),
    );
    assert_eq!(status, 400);
    assert!(
        json_body(&body)["error"]
            .as_str()
            .unwrap()
            .contains("ghost-model"),
        "{body:?}"
    );

    // Per-model deletion: only the targeted cache vanishes
    let (status, body, _) = request(
        port,
        "DELETE",
        "/api/embeddings/caches",
        Some(&serde_json::to_string(&json!({ "model_key": b_key })).unwrap()),
    );
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["deleted"], 1);
    let caches = get_caches(port);
    assert_eq!(find_cache(&caches, "mock-b")["embedded"], 0);
    assert_eq!(find_cache(&caches, "mock-a")["embedded"], 1);
    // Deleting an empty identity reports zero without error
    let (status, body, _) = request(
        port,
        "DELETE",
        "/api/embeddings/caches",
        Some(r#"{"model_key": "ghost-model"}"#),
    );
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["deleted"], 0);

    // Failover: the head candidate points at a dead port, so the actual search walks to the
    // second candidate and names it in the response
    let dead = {
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    put_settings(
        port,
        json!({
            "embedding_models": [
                {"base_url": format!("http://127.0.0.1:{dead}/v1"), "id": "dead-model", "name": "dead-model", "model": "dead-model"},
                {"base_url": format!("http://127.0.0.1:{mock_port}/v1"), "id": "mock-b", "name": "mock-b", "model": "mock-b"}
            ]
        }),
    );
    let (status, body, _) = request(
        port,
        "GET",
        &format!(
            "/api/memories?query={}&mode=hybrid",
            encodeURIComponent("cache probe")
        ),
        None,
    );
    assert_eq!(status, 200);
    let out = json_body(&body);
    assert_eq!(
        out["mode"], "hybrid",
        "failover must keep hybrid alive: {out}"
    );
    assert_eq!(out["embedding_model"], "mock-b");
    // The per-candidate connection test shows both verdicts at once
    let (status, body, _) = request(port, "POST", "/api/embeddings/test", None);
    assert_eq!(status, 200);
    let test = json_body(&body);
    assert_eq!(test["ok"], false, "one candidate is down");
    let results = test["results"].as_array().unwrap();
    assert_eq!(results[0]["model"], "dead-model");
    assert_eq!(results[0]["ok"], false);
    assert_eq!(results[1]["model"], "mock-b");
    assert_eq!(results[1]["ok"], true);

    // An explicit `entries` array probes exactly what was sent: the admin editor verifies the
    // values currently on screen, before they are enabled or saved
    let (status, body, _) = request(
        port,
        "POST",
        "/api/embeddings/test",
        Some(
            &json!({
                "entries": [
                    {
                        "enabled": false,
                        "base_url": format!("http://127.0.0.1:{mock_port}/v1"),
                        "id": "draft-model", "name": "draft-model", "model": "draft-model"
                    },
                    { "enabled": true, "base_url": "", "id": "incomplete", "name": "incomplete", "model": "incomplete" }
                ]
            })
            .to_string(),
        ),
    );
    assert_eq!(status, 200);
    let draft = json_body(&body);
    assert_eq!(
        draft["ok"], false,
        "the incomplete candidate fails: {draft}"
    );
    let results = draft["results"].as_array().unwrap();
    assert_eq!(results.len(), 2, "one verdict per request position");
    assert_eq!(results[0]["index"], 0);
    assert_eq!(results[0]["model"], "draft-model");
    assert_eq!(
        results[0]["ok"], true,
        "a disabled, unsaved candidate can still be probed: {draft}"
    );
    assert_eq!(results[0]["dim"], MOCK_DIM);
    assert_eq!(results[0]["key"], "draft-model");
    assert_eq!(results[1]["index"], 1);
    assert_eq!(results[1]["ok"], false);
    assert!(results[1]["error"]
        .as_str()
        .unwrap()
        .contains("base_url and model"));

    drop(server);
    cleanup(&db);
}

fn get_caches(port: u16) -> Vec<Value> {
    let (status, body, _) = request(port, "GET", "/api/embeddings/caches", None);
    assert_eq!(status, 200);
    json_body(&body)["caches"]
        .as_array()
        .expect("caches array")
        .clone()
}

#[test]
fn embedding_cache_ids_survive_reordering_and_renaming() {
    let db = temp_db("semantic-cache-ids");
    cleanup(&db);
    let (mock_port, seen) = spawn_mock_embedding();
    let server = HttpProc::start(&db, "semantic-cache-ids");
    let port = server.port;
    // Create before configuring models so only explicit backfills call the mock.
    let (status, _, _) = request(
        port,
        "POST",
        "/api/memories",
        Some(r#"{"summary":"identity probe","content":"body"}"#),
    );
    assert_eq!(status, 200);
    let mut a = json!({"id":"local-a", "name":"First service", "model":"shared-model", "base_url":format!("http://127.0.0.1:{mock_port}/v1")});
    let mut b = a.clone();
    b["id"] = json!("local-b");
    b["name"] = json!("Second service");
    put_settings(port, json!({"embedding_models":[a.clone(), b.clone()]}));
    for id in ["local-a", "local-b"] {
        let (status, body, _) = request(
            port,
            "POST",
            "/api/embeddings/backfill",
            Some(&json!({"model_key":id}).to_string()),
        );
        assert_eq!(status, 200);
        assert_eq!(json_body(&body)["processed"], 1);
    }
    let before = seen.lock().unwrap().len();
    a["name"] = json!("Renamed service");
    put_settings(port, json!({"embedding_models":[b.clone(), a.clone()]}));
    let caches = get_caches(port);
    for id in ["local-a", "local-b"] {
        let cache = caches.iter().find(|c| c["key"] == id).unwrap();
        assert_eq!(cache["embedded"], 1);
        assert_eq!(cache["pending"], 0);
    }
    assert_eq!(
        caches.iter().find(|c| c["key"] == "local-a").unwrap()["name"],
        "Renamed service"
    );
    assert_eq!(
        seen.lock().unwrap().len(),
        before,
        "renaming and reordering must not re-embed"
    );
    // Changing a fingerprint under the same ID invalidates only that cache.
    a["passage_prefix"] = json!("passage: ");
    put_settings(port, json!({"embedding_models":[b.clone(), a.clone()]}));
    let caches = get_caches(port);
    let cache_a = caches.iter().find(|c| c["key"] == "local-a").unwrap();
    assert_eq!(cache_a["embedded"], 0);
    assert_eq!(cache_a["pending"], 1);
    assert_eq!(
        caches.iter().find(|c| c["key"] == "local-b").unwrap()["embedded"],
        1
    );
    // A new ID starts with its own cache; the old cache stays separately deletable.
    b["id"] = json!("local-c");
    put_settings(port, json!({"embedding_models":[b, a]}));
    let caches = get_caches(port);
    assert_eq!(
        caches.iter().find(|c| c["key"] == "local-b").unwrap()["configured"],
        false
    );
    assert_eq!(
        caches.iter().find(|c| c["key"] == "local-c").unwrap()["pending"],
        1
    );
    drop(server);
    cleanup(&db);
}

fn find_cache<'a>(caches: &'a [Value], model: &str) -> &'a Value {
    caches
        .iter()
        .find(|c| c["model"] == model)
        .unwrap_or_else(|| panic!("cache for {model} missing in {caches:?}"))
}

/// The REST create face carries the same dedup hints as MCP (`similar_to` via the shared write
/// hook): a second memory in the same mock bucket (cosine 1.0 >= threshold) is flagged, and the
/// merge route goes through the same handler as the MCP tool.
#[test]
fn rest_create_reports_similar_to_and_merge_route_works() {
    let db = temp_db("semantic-rest-dedup");
    cleanup(&db);
    let (mock_port, _seen) = spawn_mock_embedding();
    let server = HttpProc::start(&db, "semantic-rest-dedup");
    let port = server.port;
    put_settings(
        port,
        json!({
            "embedding_models": [{
                "base_url": format!("http://127.0.0.1:{mock_port}/v1"),
                "id": "mock-embed", "name": "mock-embed", "model": "mock-embed"
            }]
        }),
    );

    let create = |summary: &str, content: &str| {
        let (status, resp, _) = request(
            port,
            "POST",
            "/api/memories",
            Some(
                &serde_json::to_string(&json!({ "summary": summary, "content": content })).unwrap(),
            ),
        );
        assert_eq!(status, 200, "{}", String::from_utf8_lossy(&resp));
        json_body(&resp)
    };

    // Two memories landing in the same mock bucket: cosine 1.0, over the dedup threshold.
    // The bucket must be computed over the full embedded text (summary + body), not just the
    // summary — that is what the service actually receives.
    let target = mock_bucket(&mock_memory_text("dedup probe one", "original body"));
    let twin_content = (0..10000)
        .map(|i| format!("filler {i}"))
        .find(|c| mock_bucket(&mock_memory_text("dedup probe two", c)) == target)
        .expect("必须能找到同桶正文");
    let first = create("dedup probe one", "original body");
    assert!(first
        .get("similar_to")
        .is_none_or(|v| v.as_array().is_some_and(|a| a.is_empty())));
    // The first memory's vector lands via the background worker; the dedup scan of the second
    // create reads stored embeddings, so wait for the coverage to settle before creating it
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let (_, body, _) = request(port, "GET", "/api/stats", None);
        let emb = &json_body(&body)["embedding"];
        if emb["embedded"].as_u64() == Some(1) && emb["pending"].as_u64() == Some(0) {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "first memory was never embedded: {emb}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let second = create("dedup probe two", &twin_content);
    let similar = second["similar_to"]
        .as_array()
        .expect("similar_to must be present");
    assert_eq!(similar.len(), 1);
    assert_eq!(similar[0]["id"], first["id"]);
    assert!(
        similar[0]["similarity"].as_f64().unwrap_or(0.0) >= 0.9,
        "same-bucket mock vectors are cosine 1.0"
    );

    // The merge route shares the MCP handler: source absorbed, target keeps its id
    let target_id = first["id"].as_str().unwrap();
    let source_id = second["id"].as_str().unwrap();
    let (status, resp, _) = request(
        port,
        "POST",
        "/api/memories/merge",
        Some(&serde_json::to_string(&json!({ "target": target_id, "source": source_id })).unwrap()),
    );
    assert_eq!(status, 200, "{}", String::from_utf8_lossy(&resp));
    let (status, _, _) = request(port, "GET", &format!("/api/memories/{source_id}"), None);
    assert_eq!(status, 404, "the source must be gone");
    let (status, body, _) = request(port, "GET", &format!("/api/memories/{target_id}"), None);
    assert_eq!(status, 200);
    let merged = json_body(&body);
    assert!(
        merged["content"].as_str().unwrap().contains(&twin_content),
        "the target content must carry the absorbed body"
    );

    drop(server);
    cleanup(&db);
}

/// The rerank stage: a configured cross-encoder re-orders the candidate pool (the response
/// names it via reranked_by and scores switch to the reranker scale), explicit keyword mode
/// keeps its deterministic order, and a reranker that is down silently leaves the fused order.
#[test]
fn reranker_reorders_and_degrades() {
    let db = temp_db("rerank");
    cleanup(&db);
    let (rerank_port, seen) = spawn_mock_reranker();
    let server = HttpProc::start(&db, "rerank");
    let port = server.port;

    // No embedding configuration on purpose: auto = keyword recall + rerank stage, which keeps
    // the reranker useful even where semantic recall is off.
    put_settings(
        port,
        json!({ "rerank_models": [{
            "base_url": format!("http://127.0.0.1:{rerank_port}/v1"),
            "id": "mock-rerank", "model": "mock-rerank"
        }] }),
    );

    // Three keyword hits with a deterministic fused order (summary hits outweigh content hits):
    // m1 (summary + content) > m2 (summary only) > m3 (content only)
    for (summary, content) in [
        ("alpha one", "alpha body"),
        ("alpha two", ""),
        ("zzz three", "alpha body text"),
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
    let fused = |mode: &str| {
        let (status, body, _) = request(
            port,
            "GET",
            &format!(
                "/api/memories?query={}&limit=10{}",
                encodeURIComponent("alpha"),
                mode
            ),
            None,
        );
        assert_eq!(status, 200);
        json_body(&body)
    };

    // Auto: the reranker reverses the fused order (mock scores document i with i) and names
    // itself; the scores switch to the reranker scale
    let out = fused("");
    assert_eq!(out["reranked_by"], "mock-rerank", "{out}");
    let order: Vec<String> = out["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["summary"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(order, vec!["zzz three", "alpha two", "alpha one"]);
    let scores: Vec<i64> = out["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["score"].as_i64().unwrap())
        .collect();
    assert_eq!(
        scores,
        vec![20_000, 10_000, 0],
        "relevance i (fused position) scaled by 10000, sorted descending"
    );
    // The reranker saw the query and the summary+content documents
    let seen_now = seen.lock().unwrap();
    let (query, docs) = &seen_now[seen_now.len() - 1];
    assert_eq!(query, "alpha");
    assert!(docs.iter().any(|d| d.contains("alpha one")));
    assert!(docs.iter().any(|d| d.contains("alpha body text")));
    // No content ever leaks through the rerank stage either
    assert!(out["results"]
        .as_array()
        .unwrap()
        .iter()
        .all(|r| r.get("content").is_none()));

    // Explicit keyword mode: deterministic keyword order, no rerank stage
    let out = fused("&mode=keyword");
    assert!(out.get("reranked_by").is_none());
    let order: Vec<&str> = out["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["summary"].as_str().unwrap())
        .collect();
    assert_eq!(order, vec!["alpha one", "alpha two", "zzz three"]);

    // Reranker down: auto silently keeps the fused order (degradation, not failure)
    let dead = {
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    put_settings(
        port,
        json!({ "rerank_models": [{
            "base_url": format!("http://127.0.0.1:{dead}/v1"),
            "id": "mock-rerank", "name": "mock-rerank", "model": "mock-rerank"
        }] }),
    );
    let out = fused("");
    assert!(out.get("reranked_by").is_none(), "{out}");
    let order: Vec<&str> = out["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["summary"].as_str().unwrap())
        .collect();
    assert_eq!(order, vec!["alpha one", "alpha two", "zzz three"]);

    // Connection test reports per-candidate health
    let (status, body, _) = request(port, "POST", "/api/rerank/test", None);
    assert_eq!(status, 200);
    let test = json_body(&body);
    assert_eq!(test["ok"], false, "the only candidate is down");
    assert_eq!(test["results"][0]["ok"], false);

    let (mock_port, _seen2) = spawn_mock_reranker();
    put_settings(
        port,
        json!({ "rerank_models": [{
            "base_url": format!("http://127.0.0.1:{mock_port}/v1"),
            "id": "mock-rerank", "name": "mock-rerank", "model": "mock-rerank"
        }] }),
    );
    let (status, body, _) = request(port, "POST", "/api/rerank/test", None);
    assert_eq!(status, 200);
    let test = json_body(&body);
    assert_eq!(test["ok"], true);
    assert_eq!(test["results"][0]["ok"], true);
    assert_eq!(test["results"][0]["scored"], 1);

    // An explicit `entries` array probes the caller's own values (an unsaved draft candidate)
    let (status, body, _) = request(
        port,
        "POST",
        "/api/rerank/test",
        Some(
            &json!({ "entries": [{
                "enabled": false,
                "base_url": format!("http://127.0.0.1:{mock_port}/v1"),
                "id": "draft-rerank", "name": "draft-rerank", "model": "draft-rerank"
            }] })
            .to_string(),
        ),
    );
    assert_eq!(status, 200);
    let draft = json_body(&body);
    assert_eq!(draft["results"][0]["model"], "draft-rerank");
    assert_eq!(
        draft["results"][0]["ok"], true,
        "a disabled, unsaved reranker can still be probed: {draft}"
    );
    assert_eq!(draft["results"][0]["index"], 0);

    // Reranker settings roundtrip: canonical entries echoed
    let (_, body, _) = request(port, "GET", "/api/settings", None);
    let settings = json_body(&body);
    let models = settings["rerank_models"].as_array().unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0]["model"], "mock-rerank");
    assert_eq!(models[0]["enabled"], true);

    drop(server);
    cleanup(&db);
}
