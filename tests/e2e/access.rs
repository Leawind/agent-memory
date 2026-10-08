use crate::common::{cleanup, json_body, mcp_post, request, temp_db, tool_data, HttpProc};
use serde_json::json;

#[test]
fn access_events_agree_across_tools_rest_and_resources() {
    let db = temp_db("access");
    cleanup(&db);
    let server = HttpProc::start(&db, "access");
    let port = server.port;
    assert_eq!(
        request(
            port,
            "POST",
            "/api/memories",
            Some(r#"{"summary":"Test","content":"private body"}"#)
        )
        .0,
        200
    );
    let rpc = |method, params| {
        let (status, value) = mcp_post(port, json!(1), method, params, &[]);
        assert_eq!(status, 200, "{value}");
        value
    };
    let listing = rpc("tools/call", json!({"name":"memory_list", "arguments":{}}));
    assert!(listing.to_string().find("private body").is_none());
    rpc(
        "tools/call",
        json!({"name":"memory_search", "arguments":{"query":"Test", "mode":"keyword"}}),
    );
    let stats = || json_body(&request(port, "GET", "/api/access/stats", None).1);
    assert_eq!(stats()["raw_events"], 0);
    rpc(
        "tools/call",
        json!({"name":"memory_get", "arguments":{"ids":["m1","m1","m999"]}}),
    );
    rpc("resources/read", json!({"uri":"memory://memories/m1"}));
    assert_eq!(request(port, "GET", "/api/memories/m1", None).0, 200);
    let use_result = rpc(
        "tools/call",
        json!({"name":"memory_use", "arguments":{"id":"m1", "event_key":"decision"}}),
    );
    assert_eq!(tool_data(&use_result)["reinforced"], true);
    let repeated = request(
        port,
        "POST",
        "/api/memories/m1/use",
        Some(r#"{"event_key":"decision"}"#),
    );
    assert_eq!(json_body(&repeated.1)["recorded"], false);
    let usage = json_body(&request(port, "GET", "/api/memories/m1/usage", None).1);
    assert_eq!(
        (usage["reads"].as_i64(), usage["uses"].as_i64()),
        (Some(3), Some(1))
    );
    assert!((usage["score"].as_f64().unwrap() - 1.25).abs() < 1e-4);
    assert_eq!(stats()["raw_events"], 4);
    assert_eq!(
        request(port, "POST", "/api/access/rebuild", Some("{}")).0,
        200
    );
    assert_eq!(stats()["raw_events"], 4);
    drop(server);
    cleanup(&db);
}
