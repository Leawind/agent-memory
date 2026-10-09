//! Constraints are checked on the final tag set in the same transaction as each write.

use crate::common::{cleanup, json_body, request, temp_db, HttpProc};
use serde_json::{json, Value};

fn api(port: u16, method: &str, path: &str, body: Option<Value>) -> (u16, Value) {
    let body = body.map(|v| v.to_string());
    let (status, bytes, _) = request(port, method, path, body.as_deref());
    (status, json_body(&bytes))
}

#[test]
fn constraints_cover_publication_writes_renames_and_restore() {
    let path = temp_db("tag-rules");
    cleanup(&path);
    let server = HttpProc::start(&path, "tag-rules");
    let port = server.port;
    for tag in ["wind", "horse", "cow", "parent", "child"] {
        assert_eq!(
            api(port, "POST", "/api/tags", Some(json!({"name": tag}))).0,
            200
        );
    }
    let create = |tags: Value, summary: &str| {
        api(
            port,
            "POST",
            "/api/memories",
            Some(json!({"summary": summary, "content": "private body", "tags": tags})),
        )
    };
    let (status, a) = create(json!(["wind"]), "wind memory");
    assert_eq!(status, 200);
    let aid = a["id"].as_str().unwrap();
    let (status, b) = create(json!(["horse"]), "horse memory");
    assert_eq!(status, 200);
    let bid = b["id"].as_str().unwrap();
    let rules = json!({"constraints": [
        {"name": "animal exclusivity", "expression": "mutex(wind,horse,cow)"},
        {"name": "child requires parent", "expression": "!child|parent"}
    ]});
    assert_eq!(api(port, "PUT", "/api/tag-rules", Some(rules)).0, 200);
    assert_eq!(create(json!(["wind", "cow"]), "conflict").0, 400);
    assert_eq!(create(json!(["child"]), "missing parent").0, 400);
    assert_eq!(create(json!(["child", "parent"]), "valid family").0, 200);
    assert_eq!(create(json!([]), "zero animals allowed").0, 200);
    assert_eq!(
        api(
            port,
            "POST",
            "/api/memories",
            Some(json!({"summary": "summary-only backup"}))
        )
        .0,
        200
    );

    let (status, _) = api(
        port,
        "PUT",
        &format!("/api/memories/{aid}"),
        Some(
            json!({"summary": "must not persist", "add_tags": ["horse", "temporary"], "create_missing_tags": true}),
        ),
    );
    assert_eq!(status, 400);
    let (_, stored) = api(port, "GET", &format!("/api/memories/{aid}"), None);
    assert_eq!(stored["summary"], "wind memory");
    let (_, tags) = api(port, "GET", "/api/tags", None);
    assert!(
        !tags.to_string().contains("temporary"),
        "auto-created tags roll back with invalid writes"
    );

    // The invalid union must leave both memories and their contents intact.
    assert_eq!(
        api(
            port,
            "POST",
            "/api/memories/merge",
            Some(json!({"target": aid, "source": bid}))
        )
        .0,
        400
    );
    assert_eq!(
        api(port, "GET", &format!("/api/memories/{bid}"), None).0,
        200
    );
    assert_eq!(
        api(port, "GET", &format!("/api/memories/{aid}"), None).1["content"],
        "private body"
    );
    // Removing before validation allows an atomic switch from wind to cow.
    assert_eq!(
        api(
            port,
            "PUT",
            &format!("/api/memories/{aid}"),
            Some(json!({"add_tags": ["cow"], "remove_tags": ["wind"]}))
        )
        .0,
        200
    );

    let invalid = json!({"constraints": [{"name": "forbid cow", "expression": "!cow"}]});
    let (status, preview) = api(
        port,
        "POST",
        "/api/tag-rules/preview",
        Some(invalid.clone()),
    );
    assert_eq!(status, 200);
    assert_eq!(preview["valid"], false);
    assert_eq!(preview["total_violations"], 1);
    assert_eq!(preview["violations"][0]["id"], aid);
    assert!(!preview.to_string().contains("private body"));
    assert_eq!(api(port, "PUT", "/api/tag-rules", Some(invalid)).0, 400);
    assert_eq!(
        api(port, "GET", "/api/tag-rules", None).1["constraints"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    for expression in ["/cow/", "unknown", "mutex(cow)"] {
        assert_eq!(
            api(
                port,
                "PUT",
                "/api/tag-rules",
                Some(json!({"constraints": [{"name": "bad", "expression": expression}]}))
            )
            .0,
            400
        );
    }
    assert_eq!(
        api(
            port,
            "PUT",
            "/api/tags/cow",
            Some(json!({"new_name": "ox"}))
        )
        .0,
        200
    );
    let (_, renamed) = api(port, "GET", "/api/tag-rules", None);
    assert_eq!(
        renamed["constraints"][0]["expression"],
        "mutex(wind,horse,ox)"
    );
    assert_eq!(api(port, "DELETE", "/api/tags/ox", None).0, 400);
    assert_eq!(create(json!(["ox", "horse"]), "renamed conflict").0, 400);

    let (status, dump) = api(port, "GET", "/api/export", None);
    assert_eq!(status, 200);
    assert_eq!(dump["tag_rules"], renamed);
    let restore_path = temp_db("tag-rules-restore");
    cleanup(&restore_path);
    let restored = HttpProc::start(&restore_path, "tag-rules-restore");
    let mut corrupt = dump.clone();
    corrupt["tag_rules"] =
        json!({"constraints": [{"name": "reject restore", "expression": "!horse"}]});
    assert_eq!(
        api(restored.port, "POST", "/api/import", Some(corrupt)).0,
        400
    );
    assert_eq!(
        api(restored.port, "GET", "/api/stats", None).1["memories"],
        0
    );
    assert_eq!(
        api(restored.port, "GET", "/api/tag-rules", None).1["constraints"],
        json!([])
    );
    assert_eq!(api(restored.port, "POST", "/api/import", Some(dump)).0, 200);
    assert_eq!(api(restored.port, "GET", "/api/tag-rules", None).1, renamed);
    assert_eq!(
        api(
            restored.port,
            "POST",
            "/api/memories",
            Some(json!({"summary": "invalid restored write", "tags": ["ox", "wind"]}))
        )
        .0,
        400
    );
    drop(restored);
    cleanup(&restore_path);
    drop(server);
    cleanup(&path);
}

#[test]
fn derived_tags_are_searchable_retractable_and_never_promoted_by_merge_or_backup() {
    let path = temp_db("derived-tags");
    cleanup(&path);
    let server = HttpProc::start(&path, "derived-tags");
    let port = server.port;
    for tag in [
        "vue3",
        "web",
        "frontend",
        "backend",
        "mcmod",
        "minecraft",
        "modding",
    ] {
        assert_eq!(
            api(port, "POST", "/api/tags", Some(json!({"name": tag}))).0,
            200
        );
    }
    let create = |tags: Value| {
        let (status, result) = api(
            port,
            "POST",
            "/api/memories",
            Some(json!({"summary": "a fact", "tags": tags})),
        );
        assert_eq!(status, 200, "{result}");
        result["id"].as_str().unwrap().to_string()
    };
    let id = create(json!(["vue3"]));
    let rules = json!({"constraints": [{"name": "platform mutex", "expression": "mutex(vue3,backend)"}], "derivations": [
        {"name": "framework domain", "expression": "vue3 => web"},
        {"name": "domain category", "expression": "web => frontend"},
        {"name": "positive cycle", "expression": "frontend => web"},
        {"name": "mod equivalence", "expression": "mcmod <=> minecraft, modding"}
    ]});
    assert_eq!(
        api(port, "POST", "/api/tag-rules/preview", Some(rules.clone())).1["positive_cycles"],
        true
    );
    assert_eq!(
        api(port, "PUT", "/api/tag-rules", Some(rules.clone())).0,
        200
    );
    let get = |id: &str| api(port, "GET", &format!("/api/memories/{id}"), None).1;
    let update = |id: &str, value| api(port, "PUT", &format!("/api/memories/{id}"), Some(value));
    let full = get(&id);
    assert_eq!(full["tags"], json!(["frontend", "vue3", "web"]));
    assert_eq!(full["original_tags"], json!(["vue3"]));
    assert_eq!(
        full["derived_tags"][1],
        json!({"tag": "web", "rules": ["framework domain", "positive cycle"]})
    );
    assert_eq!(
        api(port, "GET", "/api/memories?tag_expr=frontend", None).1["total"],
        1
    );
    assert_eq!(
        api(port, "GET", "/api/memories?query=web&mode=keyword", None).1["total_matches"],
        1
    );
    assert_eq!(update(&id, json!({"remove_tags": ["web"]})).0, 400);
    assert_eq!(update(&id, json!({"add_tags": ["web"]})).0, 200);
    assert_eq!(get(&id)["original_tags"], json!(["vue3", "web"]));
    assert_eq!(
        get(&id)["derived_tags"].as_array().unwrap().len(),
        2,
        "original and derived sources coexist"
    );
    assert_eq!(update(&id, json!({"remove_tags": ["vue3"]})).0, 200);
    assert_eq!(get(&id)["tags"], json!(["frontend", "web"]));
    assert_eq!(update(&id, json!({"remove_tags": ["web"]})).0, 200);
    assert_eq!(
        get(&id)["tags"],
        json!([]),
        "unsupported cycle must disappear"
    );

    let alias = create(json!(["mcmod"]));
    assert_eq!(
        get(&alias)["tags"],
        json!(["mcmod", "minecraft", "modding"])
    );
    assert_eq!(update(&alias, json!({"remove_tags": ["mcmod"]})).0, 200);
    assert_eq!(get(&alias)["tags"], json!([]));
    let decomposed = create(json!(["minecraft", "modding"]));
    assert_eq!(get(&decomposed)["derived_tags"][0]["tag"], "mcmod");
    assert_eq!(
        update(&decomposed, json!({"remove_tags": ["minecraft"]})).0,
        200
    );
    assert_eq!(get(&decomposed)["tags"], json!(["modding"]));

    let target = create(json!(["vue3"]));
    let source = create(json!(["vue3"]));
    assert_eq!(
        api(
            port,
            "POST",
            "/api/memories/merge",
            Some(json!({"target": target, "source": source}))
        )
        .0,
        200
    );
    assert_eq!(get(&target)["original_tags"], json!(["vue3"]));
    assert_eq!(update(&target, json!({"remove_tags": ["vue3"]})).0, 200);
    assert_eq!(get(&target)["tags"], json!([]));
    let rooted = create(json!(["vue3"]));
    let mut conflict = rules;
    conflict["derivations"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "invalid inference", "expression": "vue3 => backend"}));
    assert_eq!(api(port, "PUT", "/api/tag-rules", Some(conflict)).0, 400);
    assert_eq!(get(&rooted)["tags"], json!(["frontend", "vue3", "web"]));
    assert_eq!(
        api(
            port,
            "PUT",
            "/api/tags/web",
            Some(json!({"new_name": "website"}))
        )
        .0,
        200
    );
    assert_eq!(get(&rooted)["tags"], json!(["frontend", "vue3", "website"]));

    let dump = api(port, "GET", "/api/export", None).1;
    assert_eq!(
        dump["memories"][&rooted]["tags"].as_array().unwrap().len(),
        1
    );
    let restore_path = temp_db("derived-restore");
    cleanup(&restore_path);
    let restore = HttpProc::start(&restore_path, "derived-restore");
    assert_eq!(api(restore.port, "POST", "/api/import", Some(dump)).0, 200);
    let restored = api(restore.port, "GET", "/api/memories?tag_expr=vue3", None).1;
    let restored_id = restored["memories"][0]["id"].as_str().unwrap();
    assert_eq!(
        api(
            restore.port,
            "GET",
            &format!("/api/memories/{restored_id}"),
            None
        )
        .1["original_tags"],
        json!(["vue3"])
    );
    assert_eq!(
        api(
            restore.port,
            "PUT",
            &format!("/api/memories/{restored_id}"),
            Some(json!({"remove_tags": ["vue3"]}))
        )
        .0,
        200
    );
    assert_eq!(
        api(
            restore.port,
            "GET",
            &format!("/api/memories/{restored_id}"),
            None
        )
        .1["tags"],
        json!([])
    );
    drop(restore);
    cleanup(&restore_path);
    assert_eq!(
        api(
            port,
            "PUT",
            "/api/tag-rules",
            Some(json!({"constraints": [], "derivations": []}))
        )
        .0,
        200
    );
    assert_eq!(
        get(&rooted)["tags"],
        json!(["vue3"]),
        "rule deletion retracts only derived sources"
    );
    drop(server);
    cleanup(&path);
}
