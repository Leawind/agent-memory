//! 多进程共享同一数据库的并发正确性。

use serde_json::json;

use crate::common::{cleanup, encodeURIComponent, json_body, request, temp_db, HttpProc};

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
