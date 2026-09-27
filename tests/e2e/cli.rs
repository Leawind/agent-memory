//! CLI 运维子命令（stats / doctor / export / import）与 REST 导入端点。

use serde_json::Value;

use crate::common::{cleanup, json_body, request, run_cli, temp_db, try_request, HttpProc};

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
    let dump_text = std::fs::read_to_string(&export_path).unwrap();
    let dump: Value = serde_json::from_str(&dump_text).unwrap();
    // 紧凑 JSON（单行）且以 id 为键，无派生字段
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
