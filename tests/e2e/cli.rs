//! CLI ops subcommands (stats / doctor / export / import) and the REST import endpoint.

use serde_json::Value;

use crate::common::{cleanup, json_body, request, run_cli, temp_db, try_request, HttpProc};

#[test]
fn cli_subcommands_work() {
    let db = temp_db("cli");
    cleanup(&db);
    let db_s = db.display().to_string();

    // Prepare data: start a temporary server and write via REST
    {
        let server = HttpProc::start(&db, "cli-writer");
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "cli test memory", "content": "body", "tags": {"cli": null}, "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        drop(server);
    }

    // stats
    let out = run_cli(&["stats", "--db", &db_s]);
    assert!(out.status.success(), "stats failed: {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("memories: 1"), "stats output: {stdout}");
    // "cli" + the always-present reserved fixture
    assert!(stdout.contains("tags: 2"), "stats output: {stdout}");

    // doctor: clean db -> exit code 0
    let out = run_cli(&["doctor", "--db", &db_s]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("no issues found"));

    // export: first export succeeds -> a second export refuses to overwrite
    let export_path = temp_db("export-out");
    let out = run_cli(&["export", export_path.to_str().unwrap(), "--db", &db_s]);
    assert!(out.status.success(), "export failed");
    assert!(export_path.exists());
    let dump_text = std::fs::read_to_string(&export_path).unwrap();
    let dump: Value = serde_json::from_str(&dump_text).unwrap();
    // Compact JSON (single line) keyed by id, no derived fields
    assert_eq!(dump_text.trim_end().lines().count(), 1, "must be compact");
    assert!(dump.get("total_memories").is_none());
    assert_eq!(dump["memories"].as_object().unwrap().len(), 1);
    assert_eq!(dump["memories"]["m1"]["content"], "body");

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

    // Prepare data in the source db
    {
        let server = HttpProc::start(&src, "roundtrip-src");
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "roundtrip 记忆甲", "content": "内容甲", "tags": {"tag甲": null}, "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "roundtrip 记忆乙", "content": "内容乙", "tags": {"tag乙": null}, "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        drop(server);
    }

    // CLI export -> CLI import into a fresh db
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

    // Imported db is fully usable (content + tags + timestamps preserved)
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
    // tag甲 + tag乙 + the always-present reserved fixture
    assert_eq!(json_body(&body)["tags"], 3);
    drop(server);

    // Non-empty target db -> a second import is rejected
    let out = run_cli(&["import", export_path.to_str().unwrap(), "--db", &dst_s]);
    assert!(!out.status.success(), "import into non-empty db must fail");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("not empty"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // REST import endpoint (the UI's import goes through this path): restore into a fresh db and verify
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
        // Importing again into a non-empty db -> 400
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

/// Predicate-filtered export and merge-mode import, over both the CLI and the REST face.
#[test]
fn backup_filter_and_merge_work_end_to_end() {
    let src = temp_db("filter-src");
    let dst = temp_db("filter-dst");
    cleanup(&src);
    cleanup(&dst);
    let src_s = src.display().to_string();
    let dst_s = dst.display().to_string();

    // Two memories in the source: one matching the filter, one not
    {
        let server = HttpProc::start(&src, "filter-src");
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "keep me", "content": "alpha", "tags": {"proj": "one"}, "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "drop me", "content": "beta", "tags": {"other": null}, "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        drop(server);
    }

    // CLI export with a predicate: only the matching memory travels, no configuration facets
    let export_path = temp_db("filter-export");
    let out = run_cli(&[
        "export",
        export_path.to_str().unwrap(),
        "--predicate",
        "proj=one",
        "--db",
        &src_s,
    ]);
    assert!(
        out.status.success(),
        "filtered export failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let dump: Value =
        serde_json::from_str(&std::fs::read_to_string(&export_path).unwrap()).unwrap();
    let memories = dump["memories"].as_object().unwrap();
    assert_eq!(memories.len(), 1);
    assert!(memories.contains_key("m1"));
    assert!(dump.get("predicates").is_none());
    assert!(dump.get("lifecycle_policy").is_none());

    // A bad predicate fails the export outright (CLI: nonzero exit; REST: 400)
    let bad_export_path = temp_db("bad-export");
    let out = run_cli(&[
        "export",
        bad_export_path.to_str().unwrap(),
        "--predicate",
        "nosuchtag",
        "--db",
        &src_s,
    ]);
    assert!(!out.status.success(), "bad predicate must fail the export");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("unknown tags in tag_expr"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!bad_export_path.exists(), "no file is created on failure");
    let _ = std::fs::remove_file(&bad_export_path);

    {
        let server = HttpProc::start(&src, "filter-rest");
        let (status, body, _) =
            request(server.port, "GET", "/api/export?predicate=proj%3Done", None);
        assert_eq!(status, 200);
        let rest_dump: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(rest_dump["memories"].as_object().unwrap().len(), 1);
        let (status, _, _) = request(server.port, "GET", "/api/export?predicate=nosuchtag", None);
        assert_eq!(status, 400);
        drop(server);
    }

    // Merge import over CLI: an empty target accepts the merge like a data-only restore...
    let out = run_cli(&[
        "import",
        export_path.to_str().unwrap(),
        "--merge",
        "--db",
        &dst_s,
    ]);
    assert!(
        out.status.success(),
        "merge import failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("imported 1 memories, 2 tags"),
        "stdout: {stdout}"
    );
    // ...and a second merge finds the same memory verbatim and skips it
    let out = run_cli(&[
        "import",
        export_path.to_str().unwrap(),
        "--merge",
        "--db",
        &dst_s,
    ]);
    assert!(out.status.success(), "second merge import failed");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("imported 0 memories"), "stdout: {stdout}");
    assert!(stdout.contains("1 duplicates skipped"), "stdout: {stdout}");

    // REST merge import into the populated target reports the skips
    {
        let server = HttpProc::start(&dst, "filter-dst");
        let dump_text = std::fs::read_to_string(&export_path).unwrap();
        let (status, body, _) = try_request(
            server.port,
            "POST",
            "/api/import?merge=1",
            Some(dump_text.trim()),
            &[],
        )
        .unwrap();
        assert_eq!(status, 200);
        let report = json_body(&body);
        assert_eq!(report["imported_memories"], 0);
        assert_eq!(report["imported_tags"], 0);
        assert_eq!(report["skipped_memories"], 1);
        drop(server);
    }

    let _ = std::fs::remove_file(&export_path);
    cleanup(&src);
    cleanup(&dst);
}
