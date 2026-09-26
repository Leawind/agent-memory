//! 管理后端（`/api/*`）：内嵌管理界面的数据通道。
//!
//! 记忆/标签端点全部复用工具层 handler（`tools::execute`），保证与 MCP 端点
//! 完全一致的校验与提示语义；错误按 `ToolError` 类别映射 HTTP 状态码
//! （NotFound → 404，Invalid → 400，Forbidden → 403），不做文本匹配。
//! 身份与设置管理（identities / settings）是刻意例外：它们不属于 agent 的
//! 工具面（agent 不管理权限），走专用 handler，但校验逻辑同样单一来源
//! （`auth::Permissions` 登记表 + store 层）。
//! URL 段与查询串值在此层做百分号解码（tiny_http 不解码，且其请求行不接受
//! 原始非 ASCII 字节）。事务模式按 HTTP 方法划分：GET 为只读快照（DEFERRED，
//! 不抢写锁），其余为写锁（IMMEDIATE）。调用方身份由 HTTP 层解析后传入。

use crate::auth::{Cap, IdentityCtx, Permissions};
use crate::model::{normalize_identity_name, MAX_INSTRUCTIONS_CHARS};
use crate::store::{self, TxMode};
use crate::tools::{self, ToolError};
use serde_json::{json, Map, Value};
use std::path::Path;

/// 处理 `/api/*`（`/api/export` 由 http 层先行拦截为附件下载）。
pub fn handle(
    db_path: &Path,
    ctx: &IdentityCtx,
    method: &str,
    path: &str,
    query: &str,
    body: &[u8],
) -> (u16, Value) {
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

    // match 臂内多处用 `?`（能力守卫、参数解析）：闭包承载，短路时整个
    // 请求以对应错误类别返回。
    let result: Result<(u16, Value), ToolError> = (|| -> Result<(u16, Value), ToolError> {
        match (method, segments.as_slice()) {
            ("GET", ["whoami"]) => Ok((200, ctx.summary())),
            ("GET", ["stats"]) => {
                ctx.require(Cap::Read)?;
                db_tx(db_path, tx_mode, |st| {
                    st.stats().map_err(ToolError::from).map(|v| (200, v))
                })
            }
            ("POST", ["import"]) => {
                ctx.require(Cap::Admin)?;
                let dump = match args_from_body() {
                    Ok(m) => Value::Object(m),
                    Err(e) => return Ok(bad_request(e)),
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
            ("GET", ["doctor"]) => {
                ctx.require(Cap::Admin)?;
                db_tx(db_path, tx_mode, |st| {
                    st.hygiene_issues().map_err(ToolError::from).map(|issues| {
                        let ok = issues.is_empty();
                        (200, json!({ "ok": ok, "issues": issues }))
                    })
                })
            }
            ("GET", ["identities"]) => {
                ctx.require(Cap::Admin)?;
                db_tx(db_path, tx_mode, |st| {
                    st.identity_list()
                        .map_err(ToolError::from)
                        .map(|identities| (200, json!({ "identities": identities })))
                })
            }
            ("POST", ["identities"]) => {
                ctx.require(Cap::Admin)?;
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                let name = match args.get("name").and_then(Value::as_str) {
                    Some(n) => normalize_identity_name(n).map_err(ToolError::invalid)?,
                    None => {
                        return Ok(bad_request(ToolError::invalid(
                            "missing required argument: name",
                        )));
                    }
                };
                let perms = Permissions::from_json(args.get("permissions").unwrap_or(&Value::Null))
                    .map_err(ToolError::invalid)?;
                db_tx(db_path, TxMode::Write, |st| {
                    st.identity_create(&name, &perms)
                        .map_err(ToolError::from)
                        .map(|(token, mut view)| {
                            // token 明文仅随创建响应出现一次，库内只存哈希
                            view["token"] = json!(token);
                            (200, view)
                        })
                })
            }
            ("POST", ["identities", name, "token-reset"]) => {
                ctx.require(Cap::Admin)?;
                db_tx(db_path, TxMode::Write, |st| {
                    st.identity_reset_token(name)
                        .map_err(ToolError::from)?
                        .map(|token| (200, json!({ "token": token })))
                        .ok_or_else(|| ToolError::not_found(format!("identity '{name}' not found")))
                })
            }
            ("PUT", ["identities", name]) => {
                ctx.require(Cap::Admin)?;
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                let perms = Permissions::from_json(args.get("permissions").unwrap_or(&Value::Null))
                    .map_err(ToolError::invalid)?;
                db_tx(db_path, TxMode::Write, |st| {
                    if !st
                        .identity_set_permissions(name, &perms)
                        .map_err(ToolError::from)?
                    {
                        return Err(ToolError::not_found(format!("identity '{name}' not found")));
                    }
                    st.identity_view(name)
                        .map_err(ToolError::from)?
                        .map(|view| (200, view))
                        .ok_or_else(|| ToolError::not_found(format!("identity '{name}' not found")))
                })
            }
            ("DELETE", ["identities", name]) => {
                ctx.require(Cap::Admin)?;
                db_tx(db_path, TxMode::Write, |st| {
                    if !st.identity_delete(name).map_err(ToolError::from)? {
                        return Err(ToolError::not_found(format!("identity '{name}' not found")));
                    }
                    Ok((200, json!({ "deleted": name })))
                })
            }
            ("GET", ["settings"]) => {
                ctx.require(Cap::Admin)?;
                db_tx(db_path, tx_mode, |st| {
                    // 空字符串归一为 null：语义是"未设置（走默认）"，UI 显示占位符
                    let instructions = st
                        .settings_get("instructions")
                        .map_err(ToolError::from)?
                        .filter(|s| !s.is_empty());
                    let conventions = st
                        .settings_get("conventions")
                        .map_err(ToolError::from)?
                        .filter(|s| !s.is_empty());
                    Ok((
                        200,
                        json!({
                            "instructions": instructions,
                            "conventions": conventions,
                            // 内置默认提示词：UI 展示"恢复默认"的目标
                            "default_instructions": tools::INSTRUCTIONS,
                        }),
                    ))
                })
            }
            ("PUT", ["settings"]) => {
                ctx.require(Cap::Admin)?;
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                for key in args.keys() {
                    if key != "instructions" && key != "conventions" {
                        return Ok(bad_request(ToolError::invalid(format!(
                            "unknown settings key '{key}' (valid: instructions, conventions)"
                        ))));
                    }
                }
                let mut updates: Vec<(&str, String)> = Vec::new();
                for key in ["instructions", "conventions"] {
                    match args.get(key) {
                        None => continue, // 省略 = 不改动该项
                        Some(Value::String(s)) => {
                            if s.chars().count() > MAX_INSTRUCTIONS_CHARS {
                                return Ok(bad_request(ToolError::invalid(format!(
                                    "{key} is too long (max {MAX_INSTRUCTIONS_CHARS} characters)"
                                ))));
                            }
                            updates.push((key, s.clone()));
                        }
                        Some(_) => {
                            return Ok(bad_request(ToolError::invalid(format!(
                                "{key} must be a string"
                            ))))
                        }
                    }
                }
                if updates.is_empty() {
                    return Ok(bad_request(ToolError::invalid(
                        "nothing to update: provide instructions and/or conventions",
                    )));
                }
                db_tx(db_path, TxMode::Write, |st| {
                    for (key, value) in &updates {
                        st.settings_put(key, value).map_err(ToolError::from)?;
                    }
                    Ok((200, json!({ "saved": true })))
                })
            }
            ("GET", ["tags"]) => db_tx(db_path, tx_mode, |st| {
                tools::execute(st, ctx, "tag_list", &json!({})).map(|v| (200, v))
            }),
            ("POST", ["tags"]) => {
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                db_tx(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, "tag_create", &Value::Object(args.clone()))
                        .map(|v| (200, v))
                })
            }
            ("PUT", ["tags", tag_name]) => {
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                let mut full = Map::new();
                full.insert("old_name".into(), json!(tag_name));
                for (k, v) in args {
                    if k == "new_name" || k == "description" {
                        full.insert(k, v);
                    }
                }
                db_tx(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, "tag_rename", &Value::Object(full.clone()))
                        .map(|v| (200, v))
                })
            }
            ("DELETE", ["tags", tag_name]) => {
                let mode = query_get(query, "mode").unwrap_or_else(|| "detach".into());
                db_tx(db_path, tx_mode, |st| {
                    tools::execute(
                        st,
                        ctx,
                        "tag_delete",
                        &json!({ "name": tag_name, "mode": mode }),
                    )
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
                    tools::execute(st, ctx, name, &args).map(|v| (200, v))
                })
            }
            ("POST", ["memories"]) => {
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                db_tx(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, "memory_create", &Value::Object(args.clone()))
                        .map(|v| (200, v))
                })
            }
            ("GET", ["memories", mem_id]) => db_tx(db_path, tx_mode, |st| {
                tools::execute(st, ctx, "memory_get", &json!({ "ids": [mem_id] })).map(|v| {
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
                    Err(e) => return Ok(bad_request(e)),
                };
                let mut full = Map::new();
                full.insert("id".into(), json!(mem_id));
                for (k, v) in args {
                    full.insert(k, v);
                }
                db_tx(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, "memory_update", &Value::Object(full.clone()))
                        .map(|v| (200, v))
                })
            }
            ("DELETE", ["memories", mem_id]) => db_tx(db_path, tx_mode, |st| {
                tools::execute(st, ctx, "memory_delete", &json!({ "ids": [mem_id] })).map(|v| {
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
            _ => Ok(bad_request(ToolError::invalid(format!(
                "no such API route: {method} {path}"
            )))),
        }
    })();

    // 错误分类是结构化的：NotFound → 404，Forbidden → 403，其余业务错误 → 400
    match result {
        Ok((status, v)) => (status, v),
        Err(e) => {
            let status = match e {
                ToolError::NotFound(_) => 404,
                ToolError::Forbidden(_) => 403,
                ToolError::Invalid(_) => 400,
            };
            (status, json!({ "error": e.message() }))
        }
    }
}

/// 导出备份的字节流（附件下载的内容，独立于 JSON 响应通道）。
/// 调用方的 admin 能力由 http 层在拦截时校验。
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
    use crate::auth::Cap;
    use serde_json::json;

    fn open_ctx() -> IdentityCtx {
        IdentityCtx::open_mode()
    }

    /// 指定能力集的身份（权限边界测试用）。
    fn ctx_with(caps: &[Cap]) -> IdentityCtx {
        let mut obj = serde_json::Map::new();
        for c in caps {
            obj.insert(c.as_str().to_string(), json!(true));
        }
        let perms = Permissions::from_json(&Value::Object(obj)).unwrap();
        IdentityCtx::new("tester", perms)
    }

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
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/nope", "", &[]);
        assert_eq!(status, 400);
        assert!(v["error"].as_str().unwrap().contains("no such API route"));
        let _ = std::fs::remove_file(&db);
    }

    /// 404/403/400 映射约定：NotFound → 404、Forbidden → 403、
    /// 其余业务错误一律 400。锁住这条约定，防止错误文案改动悄悄改变状态码。
    #[test]
    fn error_classes_map_to_distinct_status_codes() {
        let db =
            std::env::temp_dir().join(format!("agent-memory-api404-{}.db", std::process::id()));
        let cleanup = || {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{}", db.display(), suffix));
            }
        };
        cleanup();

        // 不存在的标签删除 → 404
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "DELETE",
            "/api/tags/nope",
            "mode=purge",
            &[],
        );
        assert_eq!(status, 404);
        // 不存在的记忆更新 → 404
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/memories/m999",
            "",
            br#"{"summary": "x"}"#,
        );
        assert_eq!(status, 404);
        // 校验失败（缺必填参数）→ 400
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/memories",
            "",
            br#"{"summary": "only"}"#,
        );
        assert_eq!(status, 400);
        // 非 JSON body → 400
        let (status, _) = handle(&db, &open_ctx(), "POST", "/api/tags", "", b"not json");
        assert_eq!(status, 400);

        cleanup();
    }

    /// 能力不足 → 403：写端点对只读身份、管理端点对非 admin 身份。
    #[test]
    fn forbidden_maps_to_403() {
        let db =
            std::env::temp_dir().join(format!("agent-memory-api403-{}.db", std::process::id()));
        let cleanup = || {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{}", db.display(), suffix));
            }
        };
        cleanup();

        let reader = ctx_with(&[Cap::Read]);
        // 只读身份写记忆 → 403
        let (status, v) = handle(
            &db,
            &reader,
            "POST",
            "/api/memories",
            "",
            br#"{"summary": "s", "content": "c"}"#,
        );
        assert_eq!(status, 403, "{v}");
        // 只读身份管身份 → 403
        let (status, _) = handle(&db, &reader, "GET", "/api/identities", "", &[]);
        assert_eq!(status, 403);

        cleanup();
    }

    /// 身份与设置端点：admin 创建 → 列表可见 → 改权限 → 删除；
    /// whoami 对任何身份可用。
    #[test]
    fn identity_admin_lifecycle() {
        let db =
            std::env::temp_dir().join(format!("agent-memory-api-id-{}.db", std::process::id()));
        let cleanup = || {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{}", db.display(), suffix));
            }
        };
        cleanup();

        // whoami：开放模式
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/whoami", "", &[]);
        assert_eq!(status, 200);
        assert_eq!(v["mode"], "open");

        // 创建（未知能力名 → 400；合法 → 200 且带 token）
        let (status, v) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities",
            "",
            br#"{"name": "alice", "permissions": {"read": true, "create": true, "hax": true}}"#,
        );
        assert_eq!(status, 400, "{v}");
        let (status, v) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities",
            "",
            br#"{"name": "alice", "permissions": {"read": true, "create": true}}"#,
        );
        assert_eq!(status, 200, "{v}");
        assert!(v["token"].as_str().unwrap().len() == 64);
        assert_eq!(v["permissions"]["read"], true);
        assert_eq!(v["permissions"]["delete"], false);

        // 重名 → 400
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities",
            "",
            br#"{"name": "alice", "permissions": {"read": true}}"#,
        );
        assert_eq!(status, 400);

        // 列表只含尾缀提示，不含 token 明文
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/identities", "", &[]);
        assert_eq!(status, 200);
        assert_eq!(v["identities"].as_array().unwrap().len(), 1);
        assert!(v["identities"][0]["token"].is_null());
        assert_eq!(v["identities"][0]["token_hint"].as_str().unwrap().len(), 4);

        // 重置 token：返回新明文一次，旧明文立即失效
        let (status, v) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities/alice/token-reset",
            "",
            &[],
        );
        assert_eq!(status, 200, "{v}");
        assert!(v["token"].as_str().unwrap().len() == 64);
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities/nope/token-reset",
            "",
            &[],
        );
        assert_eq!(status, 404);

        // 改权限：全能力
        let (status, v) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/identities/alice",
            "",
            br#"{"permissions": {"admin": true, "read": true}}"#,
        );
        assert_eq!(status, 200, "{v}");
        assert_eq!(v["permissions"]["admin"], true);
        // 不存在的身份 → 404
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/identities/nope",
            "",
            br#"{"permissions": {}}"#,
        );
        assert_eq!(status, 404);

        // 设置：写入（基础 + 附加规范）→ 读回；未知键拒绝；空值也合法（回退默认）
        let settings_body = serde_json::to_vec(
            &json!({ "instructions": "团队共享库规范", "conventions": "标签小写" }),
        )
        .unwrap();
        let (status, v) = handle(&db, &open_ctx(), "PUT", "/api/settings", "", &settings_body);
        assert_eq!(status, 200, "{v}");
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(status, 200);
        assert_eq!(v["instructions"], "团队共享库规范");
        assert_eq!(v["conventions"], "标签小写");
        assert!(
            v["default_instructions"].as_str().unwrap().len() > 50,
            "GET must carry the built-in default for the UI's restore action"
        );
        // 未知键拒绝
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"nope": "x"}"#,
        );
        assert_eq!(status, 400);
        // 恢复默认 = 写空字符串
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"instructions": ""}"#,
        );
        assert_eq!(status, 200);
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(status, 200);
        assert_eq!(v["instructions"], json!(null));
        assert_eq!(v["conventions"], "标签小写");

        // 删除 → 再删 404
        let (status, _) = handle(&db, &open_ctx(), "DELETE", "/api/identities/alice", "", &[]);
        assert_eq!(status, 200);
        let (status, _) = handle(&db, &open_ctx(), "DELETE", "/api/identities/alice", "", &[]);
        assert_eq!(status, 404);

        cleanup();
    }
}
