//! Concurrency correctness with multiple processes sharing one database.

use serde_json::json;

use crate::common::{cleanup, encodeURIComponent, json_body, request, temp_db, HttpProc};

#[test]
fn rest_reads_succeed_while_a_maintenance_writer_holds_the_database() {
    let db = temp_db("maintenance-read");
    cleanup(&db);
    let server = HttpProc::start(&db, "maintenance-read");
    assert_eq!(
        request(
            server.port,
            "POST",
            "/api/memories",
            Some(r#"{"summary":"committed topic","content":"body"}"#)
        )
        .0,
        200
    );
    let mut connection = rusqlite::Connection::open(&db).unwrap();
    let writer = connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
        .unwrap();
    let started = std::time::Instant::now();
    let search = request(
        server.port,
        "GET",
        "/api/memories?query=topic&mode=keyword",
        None,
    );
    assert_eq!(search.0, 200);
    assert_eq!(json_body(&search.1)["total_matches"], 1);
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
    let started = std::time::Instant::now();
    let full = request(server.port, "GET", "/api/memories/m1", None);
    assert_eq!(full.0, 200);
    assert_eq!(json_body(&full.1)["content"], "body");
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
    let stats = json_body(&request(server.port, "GET", "/api/access/stats", None).1);
    assert_eq!(stats["raw_events"], 0);
    assert_eq!(stats["dropped_read_events"], 1);
    writer.rollback().unwrap();
    drop(connection);
    assert_eq!(request(server.port, "GET", "/api/memories/m1", None).0, 200);
    let stats = json_body(&request(server.port, "GET", "/api/access/stats", None).1);
    assert_eq!(stats["raw_events"], 1);
    drop(server);
    cleanup(&db);
}

#[test]
fn two_server_processes_share_one_db() {
    let db = temp_db("share");
    cleanup(&db);
    let a = HttpProc::start(&db, "share-a");
    let b = HttpProc::start(&db, "share-b");

    // A writes, B sees it immediately (WAL read/write concurrency)
    let (status, _, _) = request(
        a.port,
        "POST",
        "/api/memories",
        Some(
            r#"{"summary": "written by A", "content": "a", "tags": ["share"], "create_missing_tags": true}"#,
        ),
    );
    assert_eq!(status, 200);
    let (status, body, _) = request(b.port, "GET", "/api/memories?query=written", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total_matches"], 1);

    // B renames the tag, A's data stays in sync
    let (status, _, _) = request(
        b.port,
        "PUT",
        &format!("/api/tags/{}", encodeURIComponent("share")),
        Some(r#"{"new_name": "shared-tag"}"#),
    );
    assert_eq!(status, 200);
    let (status, body, _) = request(a.port, "GET", "/api/memories?tag_expr=shared-tag", None);
    assert_eq!(status, 200);
    assert_eq!(json_body(&body)["total"], 1);

    // 4 threads write concurrently via REST (each process taking half), zero loss;
    // meanwhile a reader thread keeps searching -- read-only transactions (DEFERRED snapshot) never block writers
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
