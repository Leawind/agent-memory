use crate::common::{cleanup, json_body, request, temp_db, HttpProc};
use serde_json::{json, Value};

fn api(port: u16, method: &str, path: &str, value: Option<Value>) -> (u16, Value) {
    let body = value.map(|v| v.to_string());
    let (status, bytes, _) = request(port, method, path, body.as_deref());
    (status, json_body(&bytes))
}

// Freshness decay is the only lifecycle mechanism left: an admin-configured ranking prior.
// The HTTP face must validate policies on save (bad syntax and unresolved @named references
// are rejected, not silently dead), keep them in settings, and carry them through backups.
#[test]
fn decay_policy_validation_and_backup() {
    let path = temp_db("decay-policy");
    cleanup(&path);
    let server = HttpProc::start(&path, "decay-policy");
    let port = server.port;
    // A decay rule may reference a named predicate: it binds literal tags, so seed them
    assert_eq!(
        api(
            port,
            "POST",
            "/api/memories",
            Some(json!({
                "summary": "scratch note",
                "content": "body",
                "tags": {"draft": null, "scratch": null},
                "create_missing_tags": true
            }))
        )
        .0,
        200
    );
    assert_eq!(
        api(
            port,
            "PUT",
            "/api/predicates",
            Some(json!({"name": "temporary", "predicate": "draft | scratch", "description": ""}))
        )
        .0,
        200
    );
    let policy = json!({
        "rules": [
            {"predicate": "convention", "half_life_days": null},
            {"predicate": "/^draft/ | @temporary", "half_life_days": 7}
        ],
        "default_half_life_days": 30,
        "freshness_weight": 0.3,
        "reinforcement_weight": 0.2
    });
    assert_eq!(
        api(
            port,
            "PUT",
            "/api/settings",
            Some(json!({"lifecycle_policy": policy}))
        )
        .0,
        200
    );
    let stored = api(port, "GET", "/api/settings", None).1["lifecycle_policy"].clone();
    assert_eq!(stored["rules"][0]["predicate"], "convention");
    assert_eq!(stored["rules"][0]["half_life_days"], Value::Null);
    assert_eq!(stored["rules"][1]["half_life_days"], 7.0);
    assert_eq!(stored["default_half_life_days"], 30.0);

    // key=value comparisons are query-only; unknown @references would leave dead rules
    for bad in [
        json!({"rules": [{"predicate": "a=b", "half_life_days": 7}]}),
        json!({"rules": [{"predicate": "", "half_life_days": 7}]}),
        json!({"rules": [{"predicate": "@missing", "half_life_days": 7}]}),
        json!({"rules": [{"predicate": "ok", "half_life_days": 0}]}),
        json!({"rules": [{"half_life_days": 7}]}),
    ] {
        assert_eq!(
            api(
                port,
                "PUT",
                "/api/settings",
                Some(json!({"lifecycle_policy": bad}))
            )
            .0,
            400,
            "{bad}"
        );
    }
    assert_eq!(
        api(port, "GET", "/api/settings", None).1["lifecycle_policy"],
        stored,
        "rejected writes leave the policy untouched"
    );

    // The policy is part of the dump and survives a backup restore
    let dump = api(port, "GET", "/api/export", None).1;
    let restore_path = temp_db("decay-policy-restore");
    cleanup(&restore_path);
    let restore = HttpProc::start(&restore_path, "decay-policy-restore");
    assert_eq!(api(restore.port, "POST", "/api/import", Some(dump)).0, 200);
    assert_eq!(
        api(restore.port, "GET", "/api/settings", None).1["lifecycle_policy"],
        stored
    );
    drop(restore);
    cleanup(&restore_path);
    drop(server);
    cleanup(&path);
}
