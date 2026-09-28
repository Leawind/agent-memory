//! Admin backend (`/api/*`): the data channel for the embedded management UI.
//!
//! Memory/tag endpoints all reuse the tool-layer handlers (`tools::execute`) so
//! that validation and error semantics match the MCP endpoints exactly; errors
//! are mapped to HTTP status codes by `ToolError` kind (NotFound → 404, Invalid → 400,
//! Forbidden → 403), never by text matching.
//! Identity and settings management (identities / settings) is a deliberate exception:
//! they are not part of the agent tool surface (agents do not manage permissions)
//! and use dedicated handlers, but validation still has a single source of truth
//! (the `auth::Permissions` registry + the store layer). URL segments and query
//! string values are percent-decoded in this layer (tiny_http does not decode, and
//! its request line does not accept raw non-ASCII bytes). Transaction mode follows
//! the HTTP method: GET takes a read-only snapshot (DEFERRED, no write lock),
//! everything else takes the write lock (IMMEDIATE). Caller identity is resolved
//! by the HTTP layer and passed in.

use crate::auth::{Cap, IdentityCtx, Permissions};
use crate::model::{normalize_identity_name, MAX_INSTRUCTIONS_CHARS};
use crate::store::{self, TxMode};
use crate::tools::{self, ToolError};
use serde_json::{json, Map, Value};
use std::path::Path;

/// Handles `/api/*` (`/api/export` is intercepted upstream by the http layer as an attachment download).
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

    // Several match arms use `?` (capability guards, argument parsing): a closure
    // carries them so a short-circuit returns the whole request as that error kind.
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
                            // token plaintext appears only in the create response; the DB stores a hash
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
                    // Empty strings normalize to null: means "unset (use default)", the UI shows a placeholder
                    let instructions = st
                        .settings_get("instructions")
                        .map_err(ToolError::from)?
                        .filter(|s| !s.is_empty());
                    let conventions = st
                        .settings_get("conventions")
                        .map_err(ToolError::from)?
                        .filter(|s| !s.is_empty());
                    // Semantic search config is returned as-is: the endpoint is Admin-only and
                    // api_key and instructions are both "server secrets", no extra masking
                    let embed = &store::Store::SETTING_EMBEDDING_BASE_URL;
                    let base_url = st.settings_get(embed).map_err(ToolError::from)?;
                    let model = st
                        .settings_get(store::Store::SETTING_EMBEDDING_MODEL)
                        .map_err(ToolError::from)?;
                    let api_key = st
                        .settings_get(store::Store::SETTING_EMBEDDING_API_KEY)
                        .map_err(ToolError::from)?;
                    Ok((
                        200,
                        json!({
                            "instructions": instructions,
                            "conventions": conventions,
                                                        // Auth switch: the auth boundary is decided by this explicit switch,
                            "auth_required": st.auth_required()?,
                                                        // Anonymous identity capability set: tokenless requests are resolved
                                                        // against it once auth is enabled; null = unset (anonymous rejected)
                            "anonymous_permissions": st
                                .anonymous_permissions()?
                                .map(|p| p.to_json()),
                            "embedding_enabled": st.embedding_enabled()?,
                            "embedding_base_url": base_url,
                            "embedding_model": model,
                            "embedding_api_key": api_key,
                                                        // Built-in default prompt: what the UI shows as the "restore default" target
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
                const VALID_KEYS: &[&str] = &[
                    "instructions",
                    "conventions",
                    "auth_required",
                    store::Store::SETTING_ANONYMOUS_PERMISSIONS,
                    store::Store::SETTING_EMBEDDING_ENABLED,
                    store::Store::SETTING_EMBEDDING_BASE_URL,
                    store::Store::SETTING_EMBEDDING_MODEL,
                    store::Store::SETTING_EMBEDDING_API_KEY,
                ];
                for key in args.keys() {
                    if !VALID_KEYS.contains(&key.as_str()) {
                        return Ok(bad_request(ToolError::invalid(format!(
                            "unknown settings key '{key}' (valid: {})",
                            VALID_KEYS.join(", ")
                        ))));
                    }
                }
                // Boolean keys: auth_required and embedding_enabled; text items go through the String branch;
                // anonymous_permissions is an object/null special case, collected separately
                const BOOL_KEYS: &[&str] =
                    &["auth_required", store::Store::SETTING_EMBEDDING_ENABLED];
                let mut updates: Vec<(&str, String)> = Vec::new();
                // Outer Some = the request carried this key; inner None = clear (anonymous rejected)
                let mut anon_perms: Option<Option<Permissions>> = None;
                for key in VALID_KEYS {
                    match args.get(*key) {
                        None => continue, // omitted = leave this item unchanged
                        Some(v) if *key == store::Store::SETTING_ANONYMOUS_PERMISSIONS => {
                            let parsed = match v {
                                Value::Null => None,
                                Value::Object(_) => {
                                    Some(Permissions::from_json(v).map_err(ToolError::invalid)?)
                                }
                                _ => {
                                    return Ok(bad_request(ToolError::invalid(format!(
                                        "{key} must be a permissions object or null"
                                    ))))
                                }
                            };
                            anon_perms = Some(parsed);
                        }
                        Some(v) if BOOL_KEYS.contains(key) => match v.as_bool() {
                            Some(on) => {
                                updates.push((key, if on { "true" } else { "false" }.into()))
                            }
                            None => {
                                return Ok(bad_request(ToolError::invalid(format!(
                                    "{key} must be a boolean"
                                ))))
                            }
                        },
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
                if updates.is_empty() && anon_perms.is_none() {
                    return Ok(bad_request(ToolError::invalid(
                        "nothing to update: provide at least one settings key",
                    )));
                }
                db_tx(db_path, TxMode::Write, |st| {
                    // An admin identity must exist before enabling auth, otherwise every request
                    // would 401 with no one able to manage (token plaintext appears only once at creation). Checking in the same transaction avoids concurrent bypass.
                    let enabling = updates
                        .iter()
                        .any(|(k, v)| *k == store::Store::SETTING_AUTH_REQUIRED && v == "true");
                    if enabling && !st.has_admin_identity()? {
                        return Err(ToolError::invalid(
                            "cannot enable auth_required: no admin identity exists; create one first (open mode has full capabilities)",
                        ));
                    }
                    for (key, value) in &updates {
                        if *key == store::Store::SETTING_AUTH_REQUIRED {
                            st.set_auth_required(value == "true")
                                .map_err(ToolError::from)?;
                        } else {
                            st.settings_put(key, value).map_err(ToolError::from)?;
                        }
                    }
                    if let Some(perms) = anon_perms {
                        st.set_anonymous_permissions(perms.as_ref())
                            .map_err(ToolError::from)?;
                    }
                    Ok((200, json!({ "saved": true })))
                })
            }
            ("POST", ["embeddings", "backfill"]) => {
                ctx.require(Cap::Admin)?;
                // Bounded batch: each request processes a small batch and returns the remaining
                // count, which the UI calls in a loop. Sidesteps the long-running task vs. one-transaction-per-request model conflict.
                let batch = query_get(query, "batch")
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(crate::embed::MAX_BATCH);
                Ok((200, crate::embed::process_pending(db_path, batch).to_json()))
            }
            ("POST", ["embeddings", "test"]) => {
                ctx.require(Cap::Admin)?;
                let cfg = store::with_db_in(db_path, TxMode::ReadOnly, |st| {
                    st.embedding_config().map_err(ToolError::from)
                })?;
                let Some(cfg) = cfg else {
                    return Ok(bad_request(ToolError::invalid(
                        "semantic search is not enabled or not fully configured (embedding settings)",
                    )));
                };
                let start = std::time::Instant::now();
                match crate::embed::embed_texts(
                    &cfg,
                    &["connection test 连接测试".to_string()],
                    crate::embed::BATCH_TIMEOUT,
                ) {
                    Ok(vectors) => Ok((
                        200,
                        json!({
                            "ok": true,
                            "dim": vectors.first().map(|v| v.len()).unwrap_or(0),
                            "elapsed_ms": start.elapsed().as_millis() as u64,
                        }),
                    )),
                    Err(e) => Ok((200, json!({ "ok": false, "error": e }))),
                }
            }
            ("GET", ["tags"]) => {
                let mut args = Map::new();
                if let Some(f) = query_get(query, "filter") {
                    args.insert("filter".into(), json!(f));
                }
                db_tx(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, "tag_list", &Value::Object(args)).map(|v| (200, v))
                })
            }
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
                full.insert("name".into(), json!(tag_name));
                for (k, v) in args {
                    if k == "new_name" || k == "description" {
                        full.insert(k, v);
                    }
                }
                db_tx(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, "tag_update", &Value::Object(full.clone()))
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
                let out = db_tx(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, "memory_create", &Value::Object(args.clone()))
                        .map(|v| (200, v))
                })?;
                // Embedding runs after the transaction commits (same hook semantics as the MCP path)
                crate::embed::after_write(db_path);
                Ok(out)
            }
            ("GET", ["memories", mem_id]) => db_tx(db_path, tx_mode, |st| {
                tools::execute(st, ctx, "memory_get", &json!({ "ids": [mem_id] })).map(|v| {
                    let not_found = v["missing"].as_array().is_some_and(|m| !m.is_empty())
                        || v["invalid_ids"].as_array().is_some_and(|m| !m.is_empty());
                    if not_found {
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
                let out = db_tx(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, "memory_update", &Value::Object(full.clone()))
                        .map(|v| (200, v))
                })?;
                crate::embed::after_write(db_path);
                Ok(out)
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

    // Error classification is structural: NotFound → 404, Forbidden → 403, all other business errors → 400
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

/// Byte stream of the export backup (the attachment download payload, separate from the JSON response channel).
/// The caller's admin capability is verified by the http layer at interception time. Compact JSON, same as the CLI export.
pub fn export_bytes(db_path: &Path) -> Result<Vec<u8>, String> {
    store::with_db_in(db_path, TxMode::ReadOnly, |st| st.export_dump()).map(|dump| {
        let mut bytes = serde_json::to_vec(&dump).unwrap_or_default();
        bytes.push(b'\n');
        bytes
    })
}

/// Executes one tool in a single transaction with a 200 status code (routes override the status as needed).
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

/// Query string of GET /api/memories → memory_list / memory_search parameters.
fn list_or_search_args(query: &str) -> Value {
    let mut args = Map::new();
    for key in [
        "query",
        "tag",
        "tag_filter",
        "sort",
        "order",
        "offset",
        "limit",
        "tags",
        "mode",
    ] {
        if let Some(v) = query_get(query, key) {
            let value = if key == "tags" {
                // Comma-separated multi-tag filter
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

/// Percent-decoding (lenient handling of invalid sequences).
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

    /// Identity with a given capability set (for permission boundary tests).
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
        // Invalid sequences are kept as-is
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

    /// 404/403/400 mapping convention: NotFound → 404, Forbidden → 403,
    /// all other business errors → 400. Pinning this convention so error-message edits cannot silently change status codes.
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

        // Deleting a nonexistent tag → 404
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "DELETE",
            "/api/tags/nope",
            "mode=purge",
            &[],
        );
        assert_eq!(status, 404);
        // Updating a nonexistent memory → 404
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/memories/m999",
            "",
            br#"{"summary": "x"}"#,
        );
        assert_eq!(status, 404);
        // Validation failure (missing required argument) → 400
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/memories",
            "",
            br#"{"summary": "only"}"#,
        );
        assert_eq!(status, 400);
        // Non-JSON body → 400
        let (status, _) = handle(&db, &open_ctx(), "POST", "/api/tags", "", b"not json");
        assert_eq!(status, 400);

        cleanup();
    }

    /// Insufficient capabilities → 403: write endpoints against a read-only identity,
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
        // Read-only identity writing a memory → 403
        let (status, v) = handle(
            &db,
            &reader,
            "POST",
            "/api/memories",
            "",
            br#"{"summary": "s", "content": "c"}"#,
        );
        assert_eq!(status, 403, "{v}");
        // Read-only identity managing identities → 403
        let (status, _) = handle(&db, &reader, "GET", "/api/identities", "", &[]);
        assert_eq!(status, 403);

        cleanup();
    }

    /// Identity and settings endpoints: admin creates → visible in list → permissions changed → deleted;
    /// whoami works for any identity.
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

        // whoami: open mode
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/whoami", "", &[]);
        assert_eq!(status, 200);
        assert_eq!(v["mode"], "open");

        // Create (unknown capability name → 400; valid → 200 with token)
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
        let token = v["token"].as_str().unwrap();
        assert!(
            token.starts_with("sk_") && token.len() == 65,
            "token = {token}"
        );
        assert_eq!(v["permissions"]["read"], true);
        assert_eq!(v["permissions"]["delete"], false);

        // Duplicate name → 400
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities",
            "",
            br#"{"name": "alice", "permissions": {"read": true}}"#,
        );
        assert_eq!(status, 400);

        // List contains only the suffix hint, not the token plaintext
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/identities", "", &[]);
        assert_eq!(status, 200);
        assert_eq!(v["identities"].as_array().unwrap().len(), 1);
        assert!(v["identities"][0]["token"].is_null());
        assert_eq!(v["identities"][0]["token_hint"].as_str().unwrap().len(), 4);

        // Reset token: returns the new plaintext once; the old one is invalidated immediately
        let (status, v) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities/alice/token-reset",
            "",
            &[],
        );
        assert_eq!(status, 200, "{v}");
        let token = v["token"].as_str().unwrap();
        assert!(
            token.starts_with("sk_") && token.len() == 65,
            "token = {token}"
        );
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities/nope/token-reset",
            "",
            &[],
        );
        assert_eq!(status, 404);

        // Change permissions: full capabilities
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
        // Nonexistent identity → 404
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/identities/nope",
            "",
            br#"{"permissions": {}}"#,
        );
        assert_eq!(status, 404);

        // Settings: write (base + additional conventions) → read back; unknown keys rejected; empty values are also valid (falls back to default)
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
        // Unknown keys rejected
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"nope": "x"}"#,
        );
        assert_eq!(status, 400);
        // Auth switch: boolean round-trip (alice is admin by now, guard passes); non-boolean rejected
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"auth_required": true}"#,
        );
        assert_eq!(status, 200);
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(status, 200);
        assert_eq!(v["auth_required"], true);
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"auth_required": "yes"}"#,
        );
        assert_eq!(status, 400);
        // Restore default = write an empty string
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

        // Delete → deleting again gives 404
        let (status, _) = handle(&db, &open_ctx(), "DELETE", "/api/identities/alice", "", &[]);
        assert_eq!(status, 200);
        let (status, _) = handle(&db, &open_ctx(), "DELETE", "/api/identities/alice", "", &[]);
        assert_eq!(status, 404);

        cleanup();
    }

    /// Guard for enabling the auth switch: rejected when no identity with admin
    /// capabilities exists (empty table or read-only-only both fail), allowed once an admin is created. Prevents locking the admin surface with no token holder after the switch is flipped.
    #[test]
    fn settings_enable_auth_requires_admin_identity() {
        let db =
            std::env::temp_dir().join(format!("agent-memory-api-guard-{}.db", std::process::id()));
        let cleanup = || {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{}", db.display(), suffix));
            }
        };
        cleanup();

        let enable = |db: &std::path::Path| {
            handle(
                db,
                &open_ctx(),
                "PUT",
                "/api/settings",
                "",
                br#"{"auth_required": true}"#,
            )
        };

        // Empty table → 400 and the switch stays off
        let (status, v) = enable(&db);
        assert_eq!(status, 400, "{v}");
        assert!(v.to_string().contains("no admin identity exists"));
        let (_, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(v["auth_required"], false, "guard must leave the switch off");

        // Only non-admin identities → still 400
        let (status, v) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities",
            "",
            br#"{"name": "lone-viewer", "permissions": {"read": true}}"#,
        );
        assert_eq!(status, 200, "{v}");
        let (status, v) = enable(&db);
        assert_eq!(status, 400, "{v}");

        // Create admin → allowed, the switch takes effect
        let (status, v) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/identities",
            "",
            br#"{"name": "boss", "permissions": {"read": true, "admin": true}}"#,
        );
        assert_eq!(status, 200, "{v}");
        let (status, v) = enable(&db);
        assert_eq!(status, 200, "{v}");
        let (_, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(v["auth_required"], true);

        cleanup();
    }

    /// Settings endpoint round-trip for the anonymous capability set: partial-key object write → GET returns the full-key object;
    /// null clears (GET returns null); unknown capability names / non-boolean values / non-object types rejected;
    /// writing only this key is also a valid update.
    #[test]
    fn settings_anonymous_permissions_roundtrip() {
        let db =
            std::env::temp_dir().join(format!("agent-memory-api-anon-{}.db", std::process::id()));
        let cleanup = || {
            for suffix in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{}", db.display(), suffix));
            }
        };
        cleanup();

        // Unset → null
        let (_, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(v["anonymous_permissions"], json!(null));

        // Partial-key object write → read back full-key object (missing keys = false)
        let (status, v) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"anonymous_permissions": {"read": true}}"#,
        );
        assert_eq!(status, 200, "{v}");
        let (_, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(v["anonymous_permissions"]["read"], true);
        assert_eq!(v["anonymous_permissions"]["create"], false);
        assert_eq!(v["anonymous_permissions"]["admin"], false);

        // Unknown capability name → 400
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"anonymous_permissions": {"riter": true}}"#,
        );
        assert_eq!(status, 400);
        // Non-boolean value → 400
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"anonymous_permissions": {"read": "yes"}}"#,
        );
        assert_eq!(status, 400);
        // Neither object nor null → 400
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"anonymous_permissions": "read"}"#,
        );
        assert_eq!(status, 400);

        // null clears → GET returns null
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"anonymous_permissions": null}"#,
        );
        assert_eq!(status, 200);
        let (_, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(v["anonymous_permissions"], json!(null));

        cleanup();
    }
}
