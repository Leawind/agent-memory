//! Backup round trips over the REST face: export (optionally predicate-filtered), restore into a
//! fresh database, the non-empty target rejection, and merge-mode duplicate skipping.

use serde_json::Value;

use crate::common::{cleanup, json_body, request, temp_db, try_request, HttpProc};

#[test]
fn export_and_import_roundtrip_work() {
    let src = temp_db("roundtrip-src");
    let dst = temp_db("roundtrip-dst");
    cleanup(&src);
    cleanup(&dst);

    // Prepare data in the source db
    {
        let server = HttpProc::start(&src, "roundtrip-src");
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "roundtrip 记忆甲", "content": "内容甲", "tags": ["tag甲"], "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "roundtrip 记忆乙", "content": "内容乙", "tags": ["tag乙"], "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        drop(server);
    }

    // Export is compact JSON keyed by id, with no derived fields
    let export = {
        let server = HttpProc::start(&src, "roundtrip-src");
        let (status, body, _) = request(server.port, "GET", "/api/export", None);
        assert_eq!(status, 200);
        drop(server);
        let text = String::from_utf8(body.clone()).unwrap();
        let dump: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(text.trim_end().lines().count(), 1, "must be compact");
        assert!(dump.get("total_memories").is_none());
        assert_eq!(dump["memories"].as_object().unwrap().len(), 2);
        assert_eq!(dump["memories"]["m1"]["content"], "内容甲");
        text
    };

    // Restore into a fresh db; the imported db is fully usable (content + tags + timestamps)
    {
        let server = HttpProc::start(&dst, "roundtrip-verify");
        let (status, _, _) =
            try_request(server.port, "POST", "/api/import", Some(export.trim()), &[]).unwrap();
        assert_eq!(status, 200);
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
        // Importing again into a non-empty db -> 400
        let (status, _, _) =
            try_request(server.port, "POST", "/api/import", Some(export.trim()), &[]).unwrap();
        assert_eq!(status, 400);
        drop(server);
    }

    cleanup(&src);
    cleanup(&dst);
}

/// Predicate-filtered export and merge-mode import.
#[test]
fn backup_filter_and_merge_work() {
    let src = temp_db("filter-src");
    let dst = temp_db("filter-dst");
    cleanup(&src);
    cleanup(&dst);

    // Two memories in the source: one matching the filter, one not
    {
        let server = HttpProc::start(&src, "filter-src");
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "keep me", "content": "alpha", "attributes": {"proj": "one"}, "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        let (status, _, _) = request(
            server.port,
            "POST",
            "/api/memories",
            Some(
                r#"{"summary": "drop me", "content": "beta", "tags": ["other"], "create_missing_tags": true}"#,
            ),
        );
        assert_eq!(status, 200);
        drop(server);
    }

    let filtered = {
        let server = HttpProc::start(&src, "filter-rest");
        let (status, body, _) = request(
            server.port,
            "GET",
            "/api/export?predicate=proj%3A%3Aone",
            None,
        );
        assert_eq!(status, 200);
        let dump: Value = serde_json::from_slice(&body).unwrap();
        let memories = dump["memories"].as_object().unwrap();
        assert_eq!(memories.len(), 1);
        assert!(memories.contains_key("m1"));
        // The filtered backup carries data only
        assert!(dump.get("predicates").is_none());
        assert!(dump.get("lifecycle_policy").is_none());
        // A bad predicate fails the export outright
        let (status, _, _) = request(server.port, "GET", "/api/export?predicate=nosuchtag", None);
        assert_eq!(status, 400);
        drop(server);
        String::from_utf8(body).unwrap()
    };

    {
        let server = HttpProc::start(&dst, "filter-dst");
        // Merge into an empty target works like a data-only restore...
        let (status, body, _) = try_request(
            server.port,
            "POST",
            "/api/import?merge=1",
            Some(filtered.trim()),
            &[],
        )
        .unwrap();
        assert_eq!(status, 200);
        let report = json_body(&body);
        assert_eq!(report["imported_memories"], 1);
        // One scoped tag, not a key plus a value: the vocabulary entry is the composed name
        assert_eq!(report["imported_tags"], 1);
        assert_eq!(report["skipped_memories"], 0);
        // ...and a second merge finds the same memory verbatim and skips it
        let (status, body, _) = try_request(
            server.port,
            "POST",
            "/api/import?merge=1",
            Some(filtered.trim()),
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

    cleanup(&src);
    cleanup(&dst);
}
