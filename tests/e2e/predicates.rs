use crate::common::{cleanup, encodeURIComponent, json_body, temp_db, try_request, HttpProc};
use serde_json::{json, Value};

#[test]
fn rest_predicates_enforce_independent_permissions_and_feed_queries_rules_and_policies() {
    let path = temp_db("predicates-rest");
    cleanup(&path);
    let server = HttpProc::start(&path, "predicates-rest");
    let call = |method: &str, url: &str, value: Value, token: Option<&str>| {
        let body = if value.is_null() {
            None
        } else {
            Some(value.to_string())
        };
        let bearer = token.map(|t| format!("Bearer {t}"));
        let headers = bearer
            .as_deref()
            .map(|v| vec![("Authorization", v)])
            .unwrap_or_default();
        let (code, bytes, _) =
            try_request(server.port, method, url, body.as_deref(), &headers).unwrap();
        (code, json_body(&bytes))
    };
    assert_eq!(call("POST","/api/memories",json!({"summary":"topic", "content":"private body", "tags":["a"],"create_missing_tags":true}),None).0,200);
    call("POST", "/api/tags", json!({"name":"b"}), None);
    let admin = call(
        "POST",
        "/api/identities",
        json!({"name":"admin","permissions":{"read":true,"admin":true}}),
        None,
    )
    .1["token"]
        .as_str()
        .unwrap()
        .to_string();
    let manager = call(
        "POST",
        "/api/identities",
        json!({"name":"manager","permissions":{"read":true,"predicate_manage_global":true}}),
        None,
    )
    .1["token"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        call("PUT", "/api/settings", json!({"auth_required":true}), None).0,
        200
    );
    let predicate =
        json!({"scope":"global","name":"topic","predicate":"a","description":"topic selector"});
    assert_eq!(
        call("PUT", "/api/predicates", predicate.clone(), Some(&admin)).0,
        403
    );
    assert_eq!(
        call("PUT", "/api/predicates", predicate, Some(&manager)).0,
        200
    );
    let listing = call("GET", "/api/predicates", Value::Null, Some(&manager));
    assert_eq!(listing.0, 200);
    assert_eq!(
        call(
            "POST",
            "/api/import",
            json!({"predicates":[{"name":"other","predicate":"a","description":""}]}),
            Some(&admin)
        )
        .0,
        403
    );
    assert!(!listing.1.to_string().contains("private body"));
    let url = format!("/api/memories?tag_expr={}", encodeURIComponent("@topic"));
    assert_eq!(call("GET", &url, Value::Null, Some(&manager)).1["total"], 1);
    assert_eq!(call("PUT","/api/tag-rules",json!({"constraints":[],"derivations":[{"name":"derive","predicate":"@topic","derived":["b"]}]}),Some(&admin)).0,200);
    assert_eq!(
        call(
            "PUT",
            "/api/settings",
            json!({"lifecycle_policy":{"rules":[{"predicate":"@topic&b","half_life_days":7}]}}),
            Some(&admin)
        )
        .0,
        200
    );
    assert_eq!(
        call(
            "GET",
            "/api/memories?tag_expr=b",
            Value::Null,
            Some(&manager)
        )
        .1["total"],
        1
    );
    assert_eq!(
        call(
            "PUT",
            "/api/predicates",
            json!({"scope":"global","name":"topic","predicate":"!a","description":""}),
            Some(&manager)
        )
        .0,
        400
    );
    assert_eq!(
        call(
            "DELETE",
            "/api/predicates",
            json!({"scope":"global","name":"topic"}),
            Some(&manager)
        )
        .0,
        400
    );
    assert_eq!(call("GET", &url, Value::Null, Some(&manager)).1["total"], 1);
    drop(server);
    cleanup(&path);
}
