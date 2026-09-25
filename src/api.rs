//! 管理后端（`/api/*`）：内嵌管理界面的数据通道。
//!
//! 全部复用工具层 handler（`tools::execute`），保证与 MCP 端点完全一致的
//! 校验与提示语义；错误按 `ToolError` 类别映射 HTTP 状态码
//! （NotFound → 404，Invalid → 400），不做文本匹配。
//! URL 段与查询串值在此层做百分号解码（tiny_http 不解码，且其请求行不接受
//! 原始非 ASCII 字节）。事务模式按 HTTP 方法划分：GET 为只读快照（DEFERRED，
//! 不抢写锁），其余为写锁（IMMEDIATE）。

use crate::store::{self, TxMode};
use crate::tools::{self, ToolError};
use serde_json::{json, Map, Value};
use std::path::Path;

/// 处理 `/api/*`（`/api/export` 由 http 层先行拦截为附件下载）。
pub fn handle(db_path: &Path, method: &str, path: &str, query: &str, body: &[u8]) -> (u16, Value) {
    let tx_mode = if method == "GET" {
        TxMode::ReadOnly
    } else {
        TxMode::Write
    };
    let segments_owned: Vec<String> = path
        .strip_prefix("/api/")
        .unwrap_or("")
        .split('/')
        .filter(|s| !s.is_empty())
        .map(percent_decode)
        .collect();
    let segments: Vec<&str> = segments_owned.iter().map(String::as_str).collect();

    let args_from_body = || -> Result<Map<String, Value>, ToolError> {
        if body.is_empty() {
            return Ok(Map::new());
        }
        match serde_json::from_slice::<Value>(body) {
            Ok(Value::Object(m)) => Ok(m),
            Ok(_) => Err(ToolError::invalid("request body must be a JSON object")),
            Err(e) => Err(ToolError::invalid(format!("invalid JSON body: {e}"))),
        }
    };

    let result: Result<(u16, Value), ToolError> = match (method, segments.as_slice()) {
        ("GET", ["stats"]) => db_tx(db_path, tx_mode, |st| {
            st.stats().map_err(ToolError::from).map(|v| (200, v))
        }),
        ("POST", ["import"]) => {
            let dump = match args_from_body() {
                Ok(m) => Value::Object(m),
                Err(e) => return bad_request(e),
            };
            db_tx(db_path, TxMode::Write, |st| {
                st.import_dump(&dump)
                    .map_err(ToolError::invalid)
                    .map(|(memories, tags)| {
                        (
                            200,
                            json!({"imported_memories": memories, "imported_tags": tags}),
                        )
                    })
            })
        }
        ("GET", ["doctor"]) => db_tx(db_path, tx_mode, |st| {
            st.hygiene_issues().map_err(ToolError::from).map(|issues| {
                let ok = issues.is_empty();
                (200, json!({ "ok": ok, "issues": issues }))
            })
        }),
        ("GET", ["tags"]) => db_tx(db_path, tx_mode, |st| {
            tools::execute(st, "tag_list", &json!({})).map(|v| (200, v))
        }),
        ("POST", ["tags"]) => {
            let args = match args_from_body() {
                Ok(m) => m,
                Err(e) => return bad_request(e),
            };
            db_tx(db_path, tx_mode, |st| {
                tools::execute(st, "tag_create", &Value::Object(args.clone())).map(|v| (200, v))
            })
        }
        ("PUT", ["tags", tag_name]) => {
            let args = match args_from_body() {
                Ok(m) => m,
                Err(e) => return bad_request(e),
            };
            let mut full = Map::new();
            full.insert("old_name".into(), json!(tag_name));
            for (k, v) in args {
                if k == "new_name" || k == "description" {
                    full.insert(k, v);
                }
            }
            db_tx(db_path, tx_mode, |st| {
                tools::execute(st, "tag_rename", &Value::Object(full.clone())).map(|v| (200, v))
            })
        }
        ("DELETE", ["tags", tag_name]) => {
            let mode = query_get(query, "mode").unwrap_or_else(|| "detach".into());
            db_tx(db_path, tx_mode, |st| {
                tools::execute(st, "tag_delete", &json!({ "name": tag_name, "mode": mode }))
                    .map(|v| (200, v))
            })
        }
        ("GET", ["memories"]) => {
            let args = list_or_search_args(query);
            db_tx(db_path, tx_mode, |st| {
                let name = if query_get(query, "query").is_some() {
                    "memory_search"
                } else {
                    "memory_list"
                };
                tools::execute(st, name, &args).map(|v| (200, v))
            })
        }
        ("POST", ["memories"]) => {
            let args = match args_from_body() {
                Ok(m) => m,
                Err(e) => return bad_request(e),
            };
            db_tx(db_path, tx_mode, |st| {
                tools::execute(st, "memory_create", &Value::Object(args.clone())).map(|v| (200, v))
            })
        }
        ("GET", ["memories", mem_id]) => db_tx(db_path, tx_mode, |st| {
            tools::execute(st, "memory_get", &json!({ "ids": [mem_id] })).map(|v| {
                if v["missing"].as_array().is_some_and(|m| !m.is_empty()) {
                    (
                        404,
                        json!({ "error": format!("memory '{}' not found", mem_id) }),
                    )
                } else {
                    (200, v["memories"][0].clone())
                }
            })
        }),
        ("PUT", ["memories", mem_id]) => {
            let args = match args_from_body() {
                Ok(m) => m,
                Err(e) => return bad_request(e),
            };
            let mut full = Map::new();
            full.insert("id".into(), json!(mem_id));
            for (k, v) in args {
                full.insert(k, v);
            }
            db_tx(db_path, tx_mode, |st| {
                tools::execute(st, "memory_update", &Value::Object(full.clone())).map(|v| (200, v))
            })
        }
        ("DELETE", ["memories", mem_id]) => db_tx(db_path, tx_mode, |st| {
            tools::execute(st, "memory_delete", &json!({ "ids": [mem_id] })).map(|v| {
                if v["deleted"].as_array().is_some_and(|d| d.is_empty()) {
                    (
                        404,
                        json!({ "error": format!("memory '{}' not found", mem_id) }),
                    )
                } else {
                    (200, v)
                }
            })
        }),
        _ => {
            return bad_request(ToolError::invalid(format!(
                "no such API route: {method} {path}"
            )))
        }
    };

    // 错误分类是结构化的：NotFound → 404，其余业务错误 → 400
    match result {
        Ok((status, v)) => (status, v),
        Err(e) => {
            let status = match e {
                ToolError::NotFound(_) => 404,
                ToolError::Invalid(_) => 400,
            };
            (status, json!({ "error": e.message() }))
        }
    }
}

/// 导出备份的字节流（附件下载的内容，独立于 JSON 响应通道）。
pub fn export_bytes(db_path: &Path) -> Result<Vec<u8>, String> {
    store::with_db_in(db_path, TxMode::ReadOnly, |st| st.export_dump()).map(|dump| {
        let mut bytes = serde_json::to_vec_pretty(&dump).unwrap_or_default();
        bytes.push(b'\n');
        bytes
    })
}

/// 在单事务里执行一个工具并附带 200 状态码（状态码由各路由按需覆盖）。
fn db_tx(
    db_path: &Path,
    mode: TxMode,
    f: impl FnOnce(&store::Store) -> Result<(u16, Value), ToolError>,
) -> Result<(u16, Value), ToolError> {
    store::with_db_in(db_path, mode, f)
}

fn bad_request(e: ToolError) -> (u16, Value) {
    (400, json!({ "error": e.message() }))
}

/// GET /api/memories 的查询串 → memory_list / memory_search 参数。
fn list_or_search_args(query: &str) -> Value {
    let mut args = Map::new();
    for key in ["query", "tag", "sort", "order", "offset", "limit", "tags"] {
        if let Some(v) = query_get(query, key) {
            let value = if key == "tags" {
                // 逗号分隔的多标签过滤
                Value::Array(v.split(',').map(|s| json!(s)).collect())
            } else if key == "offset" || key == "limit" {
                match v.parse::<u64>() {
                    Ok(n) => json!(n),
                    Err(_) => continue,
                }
            } else {
                json!(v)
            };
            args.insert(key.to_string(), value);
        }
    }
    Value::Object(args)
}

fn query_get(query: &str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if k == key {
            return Some(percent_decode(v));
        }
    }
    None
}

/// 百分号解码（含非法序列的宽容处理）。
pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = &s[i + 1..i + 3];
            match u8::from_str_radix(hex, 16) {
                Ok(b) => {
                    out.push(b);
                    i += 3;
                }
                Err(_) => {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_decode_handles_utf8_and_invalid() {
        assert_eq!(percent_decode("%E9%A1%B9%E7%9B%AE"), "项目");
        assert_eq!(percent_decode("plain"), "plain");
        assert_eq!(percent_decode("a%2Fb"), "a/b");
        // 非法序列原样保留
        assert_eq!(percent_decode("a%ZZb"), "a%ZZb");
        assert_eq!(percent_decode("a%2"), "a%2");
    }

    #[test]
    fn query_get_parses_pairs() {
        let q = "tag=%E9%A1%B9%E7%9B%AE&limit=20&empty=";
        assert_eq!(query_get(q, "tag").as_deref(), Some("项目"));
        assert_eq!(query_get(q, "limit").as_deref(), Some("20"));
        assert_eq!(query_get(q, "empty").as_deref(), Some(""));
        assert_eq!(query_get(q, "missing"), None);
    }

    #[test]
    fn list_args_maps_query_params() {
        let args = list_or_search_args("query=rust&limit=5&offset=2&tags=a,b");
        assert_eq!(args["query"], "rust");
        assert_eq!(args["limit"], 5);
        assert_eq!(args["offset"], 2);
        assert_eq!(args["tags"][0], "a");
        assert_eq!(args["tags"][1], "b");
        let list_args = list_or_search_args("tag=x&sort=created_at&order=asc");
        assert_eq!(list_args["tag"], "x");
        assert_eq!(list_args["sort"], "created_at");
        assert_eq!(list_args["order"], "asc");
    }

    #[test]
    fn api_routes_return_400_for_unknown_paths() {
        let db = std::env::temp_dir().join(format!("agent-memory-api-{}.db", std::process::id()));
        let (status, v) = handle(&db, "GET", "/api/nope", "", &[]);
        assert_eq!(status, 400);
        assert!(v["error"].as_str().unwrap().contains("no such API route"));
        let _ = std::fs::remove_file(&db);
    }

    /// 404 映射约定：handler 错误消息含 "not found" 时 REST 层回 404，
    /// 其余业务错误一律 400。锁住这条约定，防止错误文案改动悄悄改变状态码。
    #[test]
    fn missing_entities_map_to_404_and_validation_to_400() {
        let db =
            std::env::temp_dir().join(format!("agent-memory-api404-{}.db", std::process::id()));
        let cleanup = || {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{}", db.display(), suffix));
            }
        };
        cleanup();

        // 不存在的标签删除 → 404
        let (status, _) = handle(&db, "DELETE", "/api/tags/nope", "mode=purge", &[]);
        assert_eq!(status, 404);
        // 不存在的记忆更新 → 404
        let (status, _) = handle(&db, "PUT", "/api/memories/m999", "", br#"{"summary": "x"}"#);
        assert_eq!(status, 404);
        // 校验失败（缺必填参数）→ 400
        let (status, _) = handle(&db, "POST", "/api/memories", "", br#"{"summary": "only"}"#);
        assert_eq!(status, 400);
        // 非 JSON body → 400
        let (status, _) = handle(&db, "POST", "/api/tags", "", b"not json");
        assert_eq!(status, 400);

        cleanup();
    }
}
