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
use crate::tools::{
    self, ToolError, MEMORY_CREATE, MEMORY_DELETE, MEMORY_GET, MEMORY_LIST, MEMORY_MERGE,
    MEMORY_SEARCH, MEMORY_UPDATE, TAG_CREATE, TAG_DELETE, TAG_LIST, TAG_UPDATE,
};
use crate::util::percent_decode_lenient;
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
        .map(percent_decode_lenient)
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
                store::with_db_in(db_path, tx_mode, |st| {
                    st.stats().map_err(ToolError::from).map(|v| (200, v))
                })
            }
            ("POST", ["import"]) => {
                ctx.require(Cap::Admin)?;
                let dump = match args_from_body() {
                    Ok(m) => Value::Object(m),
                    Err(e) => return Ok(bad_request(e)),
                };
                store::with_db_in(db_path, TxMode::Write, |st| {
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
                store::with_db_in(db_path, tx_mode, |st| {
                    st.hygiene_checks().map_err(ToolError::from).map(|checks| {
                        let ok = checks.iter().all(|c| c.issues.is_empty());
                        let checks: Vec<Value> = checks
                            .into_iter()
                            .map(|c| {
                                json!({
                                    "id": c.id,
                                    "ok": c.issues.is_empty(),
                                    "issues": c.issues,
                                })
                            })
                            .collect();
                        (200, json!({ "ok": ok, "checks": checks }))
                    })
                })
            }
            ("GET", ["identities"]) => {
                ctx.require(Cap::Admin)?;
                store::with_db_in(db_path, tx_mode, |st| {
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
                store::with_db_in(db_path, TxMode::Write, |st| {
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
                store::with_db_in(db_path, TxMode::Write, |st| {
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
                store::with_db_in(db_path, TxMode::Write, |st| {
                    if !st
                        .identity_set_permissions(name, &perms)
                        .map_err(ToolError::from)?
                    {
                        return Err(ToolError::not_found(format!("identity '{name}' not found")));
                    }
                    // Echo-free like the other write endpoints: the permissions were just stated
                    // by the caller — the refreshed row comes from the list the UI reloads
                    Ok((200, json!({ "saved": true })))
                })
            }
            ("DELETE", ["identities", name]) => {
                ctx.require(Cap::Admin)?;
                store::with_db_in(db_path, TxMode::Write, |st| {
                    if !st.identity_delete(name).map_err(ToolError::from)? {
                        return Err(ToolError::not_found(format!("identity '{name}' not found")));
                    }
                    Ok((200, json!({ "deleted": name })))
                })
            }
            ("GET", ["settings"]) => {
                ctx.require(Cap::Admin)?;
                store::with_db_in(db_path, tx_mode, |st| {
                    // Empty strings normalize to null: means "unset (use default)", the UI shows a placeholder
                    let instructions = st
                        .settings_get("instructions")
                        .map_err(ToolError::from)?
                        .filter(|s| !s.is_empty());
                    // The ordered model-candidate lists are returned as native arrays (canonical
                    // entry shapes via the entry types): the endpoint is Admin-only and
                    // api_key and instructions are both "server secrets", no extra masking
                    let embedding_models: Vec<Value> = st
                        .embedding_entries()
                        .map_err(ToolError::from)?
                        .iter()
                        .map(|e| e.to_json())
                        .collect();
                    let rerank_models: Vec<Value> = st
                        .rerank_entries()
                        .map_err(ToolError::from)?
                        .iter()
                        .map(|e| e.to_json())
                        .collect();
                    Ok((
                        200,
                        json!({
                            "instructions": instructions,
                            // Auth switch: the auth boundary is decided by this explicit switch,
                            "auth_required": st.auth_required()?,
                            // Anonymous identity capability set: tokenless requests are resolved
                            // against it once auth is enabled; null = unset (anonymous rejected)
                            "anonymous_permissions": st
                                .anonymous_permissions()?
                                .map(|p| p.to_json()),
                            "embedding_models": embedding_models,
                            "rerank_models": rerank_models,
                            "search_limits": st.search_limits()?.to_json(),
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
                    "auth_required",
                    store::Store::SETTING_ANONYMOUS_PERMISSIONS,
                    store::Store::SETTING_EMBEDDING_MODELS,
                    store::Store::SETTING_RERANK_MODELS,
                    store::Store::SETTING_SEARCH_LIMITS,
                ];
                for key in args.keys() {
                    if !VALID_KEYS.contains(&key.as_str()) {
                        return Ok(bad_request(ToolError::invalid(format!(
                            "unknown settings key '{key}' (valid: {})",
                            VALID_KEYS.join(", ")
                        ))));
                    }
                }
                // Boolean keys: auth_required; text items go through the String branch;
                // anonymous_permissions is an object/null special case and the ordered model
                // lists are arrays, each collected separately
                const BOOL_KEYS: &[&str] = &["auth_required"];
                let mut updates: Vec<(&str, String)> = Vec::new();
                // Outer Some = the request carried this key; inner None = clear (anonymous rejected)
                let mut anon_perms: Option<Option<Permissions>> = None;
                for key in VALID_KEYS {
                    match args.get(*key) {
                        None => continue, // omitted = leave this item unchanged
                        Some(v) if *key == store::Store::SETTING_SEARCH_LIMITS => {
                            let limits = match crate::search::Limits::parse(v) {
                                Ok(limits) => limits,
                                Err(e) => return Ok(bad_request(ToolError::invalid(e))),
                            };
                            updates.push((key, limits.to_json().to_string()));
                        }
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
                        Some(v) if *key == store::Store::SETTING_EMBEDDING_MODELS => {
                            // Validated + canonicalized here; the canonical JSON is what gets stored
                            let entries = match parse_model_entries(key, v) {
                                Ok(entries) => entries,
                                Err(e) => return Ok(bad_request(e)),
                            };
                            if let Err(e) = push_canonical(
                                key,
                                &entries,
                                crate::embed::EmbedEntry::to_json,
                                &mut updates,
                            ) {
                                return Ok(bad_request(e));
                            }
                        }
                        Some(v) if *key == store::Store::SETTING_RERANK_MODELS => {
                            let entries = match parse_rerank_entries(key, v) {
                                Ok(entries) => entries,
                                Err(e) => return Ok(bad_request(e)),
                            };
                            if let Err(e) = push_canonical(
                                key,
                                &entries,
                                crate::rerank::RerankEntry::to_json,
                                &mut updates,
                            ) {
                                return Ok(bad_request(e));
                            }
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
                store::with_db_in(db_path, TxMode::Write, |st| {
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
                // An explicit `model_key` targets one cache identity (per-model backfill);
                // without one the highest-priority available candidate's cache is drained.
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                let batch = args
                    .get("batch")
                    .and_then(Value::as_u64)
                    .unwrap_or(crate::embed::MAX_BATCH as u64) as usize;
                let outcome = match args.get("model_key").and_then(Value::as_str) {
                    Some(key) => {
                        let cfg = store::with_db_in(db_path, TxMode::ReadOnly, |st| {
                            Ok::<_, String>(
                                st.embedding_entries()?
                                    .into_iter()
                                    .find(|e| e.vector_key() == key)
                                    .and_then(|e| e.usable()),
                            )
                        })
                        .map_err(ToolError::from)?;
                        match cfg {
                            Some(cfg) => crate::embed::process_pending_for(db_path, batch, &cfg)
                                .unwrap_or_else(crate::embed::EmbedOutcome::Failed),
                            None => {
                                return Ok(bad_request(ToolError::invalid(format!(
                                    "no enabled embedding entry matches cache key '{key}' (its cache can only be deleted)"
                                ))));
                            }
                        }
                    }
                    None => crate::embed::process_pending(db_path, batch),
                };
                Ok((200, outcome.to_json()))
            }
            ("POST", ["embeddings", "test"]) => {
                ctx.require(Cap::Admin)?;
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                // Two modes: an explicit `entries` array probes exactly what the caller sent (the
                // admin editor verifies the values on screen — a candidate that is disabled or not
                // saved yet is the interesting case), otherwise every enabled candidate in the
                // settings list is probed: the list the search-time failover would walk.
                let entries: Vec<crate::embed::EmbedEntry> = match args.get("entries") {
                    Some(v) => match parse_probe_entries(v, crate::embed::EmbedEntry::from_json) {
                        Ok(entries) => entries,
                        Err(e) => return Ok(bad_request(e)),
                    },
                    None => store::with_db_in(db_path, TxMode::ReadOnly, |st| {
                        st.embedding_entries().map_err(ToolError::from)
                    })?
                    .into_iter()
                    .filter(|e| e.usable().is_some())
                    .collect(),
                };
                if entries.is_empty() {
                    return Ok(bad_request(ToolError::invalid(
                        "nothing to test: no enabled, fully specified embedding entry exists (embedding settings)",
                    )));
                }
                let mut results = Vec::with_capacity(entries.len());
                let mut all_ok = true;
                for (index, entry) in entries.iter().enumerate() {
                    // An incomplete candidate is reported in place, never skipped: the reply keeps
                    // one verdict per request position so the UI can line them up with its rows
                    let Some(cfg) = entry.config() else {
                        all_ok = false;
                        results.push(json!({
                            "index": index,
                            "model": entry.model,
                            "ok": false,
                            "error": "incomplete candidate: base_url and model are both required",
                        }));
                        continue;
                    };
                    let start = std::time::Instant::now();
                    match crate::embed::embed_texts(
                        &cfg,
                        // The probe text goes through the query side: the prefix is part of the
                        // effective configuration being tested
                        &[crate::embed::embed_query_text(
                            &cfg,
                            "connection test 连接测试",
                        )],
                        crate::embed::BATCH_TIMEOUT,
                    ) {
                        Ok(vectors) => results.push(json!({
                            "index": index,
                            "key": cfg.vector_key(),
                            "model": cfg.model,
                            "ok": true,
                            "dim": vectors.first().map(|v| v.len()).unwrap_or(0),
                            "elapsed_ms": start.elapsed().as_millis() as u64,
                        })),
                        Err(e) => {
                            all_ok = false;
                            results.push(json!({
                                "index": index,
                                "key": cfg.vector_key(),
                                "model": cfg.model,
                                "ok": false,
                                "error": e,
                            }));
                        }
                    }
                }
                Ok((200, json!({ "ok": all_ok, "results": results })))
            }
            ("POST", ["rerank", "test"]) => {
                ctx.require(Cap::Admin)?;
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                // Same two modes as the embedding probe (see above), with a one-document rerank
                // as the exercised operation
                let entries: Vec<crate::rerank::RerankEntry> = match args.get("entries") {
                    Some(v) => {
                        match parse_probe_entries(v, crate::rerank::RerankEntry::from_json) {
                            Ok(entries) => entries,
                            Err(e) => return Ok(bad_request(e)),
                        }
                    }
                    None => store::with_db_in(db_path, TxMode::ReadOnly, |st| {
                        st.rerank_entries().map_err(ToolError::from)
                    })?
                    .into_iter()
                    .filter(|e| e.usable().is_some())
                    .collect(),
                };
                if entries.is_empty() {
                    return Ok(bad_request(ToolError::invalid(
                        "nothing to test: no enabled, fully specified reranker entry exists (reranker settings)",
                    )));
                }
                let mut results = Vec::with_capacity(entries.len());
                let mut all_ok = true;
                for (index, entry) in entries.iter().enumerate() {
                    let Some(cfg) = entry.config() else {
                        all_ok = false;
                        results.push(json!({
                            "index": index,
                            "model": entry.model,
                            "ok": false,
                            "error": "incomplete candidate: base_url and model are both required",
                        }));
                        continue;
                    };
                    let start = std::time::Instant::now();
                    match crate::rerank::rerank(
                        &cfg,
                        "connection test 连接测试",
                        &["a relevant document 一篇相关文档".to_string()],
                        crate::rerank::RERANK_TIMEOUT,
                    ) {
                        Ok(scores) => results.push(json!({
                            "index": index,
                            "model": cfg.model,
                            "ok": true,
                            "scored": scores.len(),
                            "elapsed_ms": start.elapsed().as_millis() as u64,
                        })),
                        Err(e) => {
                            all_ok = false;
                            results.push(json!({
                                "index": index,
                                "model": cfg.model,
                                "ok": false,
                                "error": e,
                            }));
                        }
                    }
                }
                Ok((200, json!({ "ok": all_ok, "results": results })))
            }
            ("GET", ["embeddings", "caches"]) => {
                ctx.require(Cap::Admin)?;
                store::with_db_in(db_path, TxMode::ReadOnly, |st| {
                    let entries = st.embedding_entries().map_err(ToolError::from)?;
                    let cached = st.embedding_cached_models().map_err(ToolError::from)?;
                    // Union of identities with rows and identities configured (a configured but
                    // never-backfilled model still shows up, with zero embedded)
                    let mut keys: Vec<String> = entries
                        .iter()
                        .filter(|e| !e.id.is_empty())
                        .map(|e| e.vector_key())
                        .collect();
                    for (key, _) in &cached {
                        if !keys.contains(key) {
                            keys.push(key.clone());
                        }
                    }
                    keys.sort();
                    keys.dedup();
                    let mut caches = Vec::with_capacity(keys.len());
                    for key in keys {
                        let entry = entries.iter().find(|e| e.id == key);
                        let fingerprint = entry.and_then(|e| e.fingerprint());
                        let embedded = st
                            .embedding_embedded_count(&key, fingerprint.as_deref())
                            .map_err(ToolError::from)?;
                        caches.push(json!({
                            "key": key,
                            "model": entry.map(|e| e.model.as_str()).unwrap_or(&key),
                            "name": entry.map(|e| e.name.as_str()).unwrap_or(&key),
                            "embedded": embedded,
                            "pending": st.embedding_pending_count(&key, fingerprint.as_deref()).map_err(ToolError::from)?,
                            "configured": entries
                                .iter()
                                .any(|e| e.vector_key() == key && e.usable().is_some()),
                        }));
                    }
                    Ok((200, json!({ "caches": caches })))
                })
            }
            ("DELETE", ["embeddings", "caches"]) => {
                ctx.require(Cap::Admin)?;
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                let Some(key) = args.get("model_key").and_then(Value::as_str) else {
                    return Ok(bad_request(ToolError::invalid(
                        "model_key is required (see GET /api/embeddings/caches)",
                    )));
                };
                store::with_db_in(db_path, tx_mode, |st| {
                    let deleted = st.embedding_delete_model(key).map_err(ToolError::from)?;
                    Ok((200, json!({ "deleted": deleted })))
                })
            }
            ("GET", ["tags"]) => {
                let mut args = Map::new();
                if let Some(f) = query_get(query, "filter") {
                    args.insert("filter".into(), json!(f));
                }
                store::with_db_in(db_path, tx_mode, |st| {
                    tools::execute(st, ctx, TAG_LIST, &Value::Object(args)).map(|v| (200, v))
                })
            }
            ("POST", ["tags"]) => {
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                tool_write(db_path, ctx, TAG_CREATE, &Value::Object(args))
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
                tool_write(db_path, ctx, TAG_UPDATE, &Value::Object(full))
            }
            ("DELETE", ["tags", tag_name]) => {
                let mode = query_get(query, "mode").unwrap_or_else(|| "detach".into());
                let mut args = json!({ "name": tag_name, "mode": mode });
                // The purge preview must be reachable from the REST face too — ignoring the flag
                // here would turn an intended preview into a real, irreversible deletion
                if query_get(query, "dry_run").is_some_and(|v| v == "true") {
                    args["dry_run"] = json!(true);
                }
                tool_write(db_path, ctx, TAG_DELETE, &args)
            }
            ("GET", ["memories"]) => {
                let args = list_or_search_args(query);
                store::with_db_in(db_path, tx_mode, |st| {
                    let name = if query_get(query, "query").is_some() {
                        MEMORY_SEARCH
                    } else {
                        MEMORY_LIST
                    };
                    tools::execute(st, ctx, name, &args).map(|v| (200, v))
                })
            }
            ("POST", ["memories"]) => {
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                tool_write(db_path, ctx, MEMORY_CREATE, &Value::Object(args))
            }
            ("POST", ["memories", "merge"]) => {
                let args = match args_from_body() {
                    Ok(m) => m,
                    Err(e) => return Ok(bad_request(e)),
                };
                tool_write(db_path, ctx, MEMORY_MERGE, &Value::Object(args))
            }
            ("GET", ["memories", mem_id]) => store::with_db_in(db_path, tx_mode, |st| {
                tools::execute(st, ctx, MEMORY_GET, &json!({ "ids": [mem_id] })).map(|v| {
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
                tool_write(db_path, ctx, MEMORY_UPDATE, &Value::Object(full))
            }
            ("DELETE", ["memories", mem_id]) => {
                let out = tool_write(db_path, ctx, MEMORY_DELETE, &json!({ "ids": [mem_id] }))?;
                // Nothing was deleted: the REST face surfaces that as 404 (the tool result reports
                // it in `missing`; the notification hook already no-opped on the empty deletion)
                if out.1["deleted"].as_array().is_some_and(|d| d.is_empty()) {
                    Ok((
                        404,
                        json!({ "error": format!("memory '{}' not found", mem_id) }),
                    ))
                } else {
                    Ok(out)
                }
            }
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

/// One write-tool call on the REST face, carrying the same post-commit hook sequence as the MCP
/// path (`execute_with_db` — literally: the shared `tools::after_commit`). Routes only assemble
/// the arguments and, where the REST surface demands it, adjust the returned status.
fn tool_write(
    db_path: &Path,
    ctx: &IdentityCtx,
    tool: &str,
    args: &Value,
) -> Result<(u16, Value), ToolError> {
    let pre = crate::notify::capture(db_path, tool, args);
    let mut out = store::with_db_in(db_path, TxMode::Write, |st| {
        tools::execute(st, ctx, tool, args)
    })?;
    tools::after_commit(db_path, tool, args, &mut out, pre);
    Ok((200, out))
}

fn bad_request(e: ToolError) -> (u16, Value) {
    (400, json!({ "error": e.message() }))
}

/// Canonical JSON of a validated candidate list — the form that gets stored and later echoed
/// by GET (shared by both ordered model lists).
fn push_canonical<'a, T>(
    key: &'a str,
    entries: &[T],
    to_json: impl Fn(&T) -> Value,
    updates: &mut Vec<(&'a str, String)>,
) -> Result<(), ToolError> {
    let values: Vec<Value> = entries.iter().map(to_json).collect();
    let canonical =
        serde_json::to_string(&values).map_err(|e| ToolError::invalid(format!("{key}: {e}")))?;
    updates.push((key, canonical));
    Ok(())
}

/// Parse the explicit `entries` array of a connection-probe request: one leniently parsed element
/// per array element, in request order (the reply keeps a verdict per position, so the caller can
/// line the results up with its rows). The write-path validation is deliberately not applied here:
/// probing what is on screen is exactly how a half-filled candidate gets diagnosed.
fn parse_probe_entries<T>(
    v: &Value,
    parse: impl Fn(&Value) -> Option<T>,
) -> Result<Vec<T>, ToolError> {
    let arr = v
        .as_array()
        .ok_or_else(|| ToolError::invalid("'entries' must be an array"))?;
    let entries: Vec<T> = arr.iter().filter_map(parse).collect();
    if entries.len() != arr.len() {
        return Err(ToolError::invalid(
            "every 'entries' element must be an object",
        ));
    }
    Ok(entries)
}

/// Validate and canonicalize one ordered model-candidate list from the settings PUT body.
/// Entries must be objects; an enabled entry must carry non-empty base_url and model; the
/// per-entry similarity floor must lie in 0..=1 (absent = built-in default). Unknown fields
/// are dropped — the canonical entry shape is what gets stored and later echoed by GET.
fn parse_model_entries(key: &str, v: &Value) -> Result<Vec<crate::embed::EmbedEntry>, ToolError> {
    let arr = v
        .as_array()
        .ok_or_else(|| ToolError::invalid(format!("{key} must be an array")))?;
    let mut entries = Vec::with_capacity(arr.len());
    let mut ids = std::collections::HashSet::new();
    for (i, item) in arr.iter().enumerate() {
        let obj = item
            .as_object()
            .ok_or_else(|| ToolError::invalid(format!("{key}[{i}] must be an object")))?;
        let opt_string = |field: &str| -> Result<Option<String>, ToolError> {
            match obj.get(field) {
                None | Some(Value::Null) => Ok(None),
                Some(Value::String(s)) if !s.is_empty() => Ok(Some(s.clone())),
                Some(Value::String(_)) => Ok(None),
                Some(_) => Err(ToolError::invalid(format!(
                    "{key}[{i}].{field} must be a string"
                ))),
            }
        };
        let enabled = match obj.get("enabled") {
            None | Some(Value::Null) => true,
            Some(Value::Bool(b)) => *b,
            Some(_) => {
                return Err(ToolError::invalid(format!(
                    "{key}[{i}].enabled must be a boolean"
                )))
            }
        };
        let base_url = opt_string("base_url")?.unwrap_or_default();
        let model = opt_string("model")?.unwrap_or_default();
        let (id, name) = parse_model_identity(key, i, item, &mut ids)?;
        if enabled && (base_url.trim().is_empty() || model.trim().is_empty()) {
            return Err(ToolError::invalid(format!(
                "{key}[{i}]: an enabled entry needs a non-empty base_url and model"
            )));
        }
        let min_similarity = match obj.get("min_similarity") {
            None | Some(Value::Null) => None,
            Some(Value::Number(n)) => {
                let f = n.as_f64().unwrap_or(f64::NAN);
                if !(0.0..=1.0).contains(&f) {
                    return Err(ToolError::invalid(format!(
                        "{key}[{i}].min_similarity must be a number between 0 and 1 (0 disables the floor)"
                    )));
                }
                Some(f as f32)
            }
            Some(_) => {
                return Err(ToolError::invalid(format!(
                    "{key}[{i}].min_similarity must be a number or null"
                )))
            }
        };
        entries.push(crate::embed::EmbedEntry {
            id,
            name,
            enabled,
            base_url,
            model,
            api_key: opt_string("api_key")?,
            query_prefix: opt_string("query_prefix")?,
            passage_prefix: opt_string("passage_prefix")?,
            min_similarity,
        });
    }
    Ok(entries)
}

/// Validate and canonicalize the ordered reranker-candidate list (same rules as
/// `parse_model_entries` minus the embedding-specific fields).
fn parse_rerank_entries(
    key: &str,
    v: &Value,
) -> Result<Vec<crate::rerank::RerankEntry>, ToolError> {
    let arr = v
        .as_array()
        .ok_or_else(|| ToolError::invalid(format!("{key} must be an array")))?;
    let mut entries = Vec::with_capacity(arr.len());
    let mut ids = std::collections::HashSet::new();
    for (i, item) in arr.iter().enumerate() {
        let obj = item
            .as_object()
            .ok_or_else(|| ToolError::invalid(format!("{key}[{i}] must be an object")))?;
        let opt_string = |field: &str| -> Result<Option<String>, ToolError> {
            match obj.get(field) {
                None | Some(Value::Null) => Ok(None),
                Some(Value::String(s)) if !s.is_empty() => Ok(Some(s.clone())),
                Some(Value::String(_)) => Ok(None),
                Some(_) => Err(ToolError::invalid(format!(
                    "{key}[{i}].{field} must be a string"
                ))),
            }
        };
        let enabled = match obj.get("enabled") {
            None | Some(Value::Null) => true,
            Some(Value::Bool(b)) => *b,
            Some(_) => {
                return Err(ToolError::invalid(format!(
                    "{key}[{i}].enabled must be a boolean"
                )))
            }
        };
        let base_url = opt_string("base_url")?.unwrap_or_default();
        let model = opt_string("model")?.unwrap_or_default();
        let (id, name) = parse_model_identity(key, i, item, &mut ids)?;
        if enabled && (base_url.trim().is_empty() || model.trim().is_empty()) {
            return Err(ToolError::invalid(format!(
                "{key}[{i}]: an enabled entry needs a non-empty base_url and model"
            )));
        }
        entries.push(crate::rerank::RerankEntry {
            id,
            name,
            enabled,
            base_url,
            model,
            api_key: opt_string("api_key")?,
        });
    }
    Ok(entries)
}

fn parse_model_identity(
    key: &str,
    index: usize,
    item: &Value,
    ids: &mut std::collections::HashSet<String>,
) -> Result<(String, String), ToolError> {
    let required = |field: &str| {
        item.get(field)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                ToolError::invalid(format!("{key}[{index}].{field} must be a non-empty string"))
            })
    };
    let id = required("id")?;
    let name = match item.get("name") {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.trim().to_string(),
        Some(_) => {
            return Err(ToolError::invalid(format!(
                "{key}[{index}].name must be a string or null"
            )))
        }
    };
    if !ids.insert(id.clone()) {
        return Err(ToolError::invalid(format!(
            "{key}[{index}]: duplicate id '{id}'"
        )));
    }
    Ok((id, name))
}

/// Query string of GET /api/memories → memory_list / memory_search parameters.
fn list_or_search_args(query: &str) -> Value {
    let mut args = Map::new();
    for key in [
        "query", "tag_expr", "sort", "order", "offset", "limit", "mode",
    ] {
        if let Some(v) = query_get(query, key) {
            let value = if key == "offset" || key == "limit" {
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
            return Some(percent_decode_lenient(v));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::Cap;
    use serde_json::json;

    #[test]
    fn model_lists_require_unique_ids_and_allow_optional_display_names() {
        for key in ["embedding_models", "rerank_models"] {
            let parse = |value: &Value| -> Result<Vec<Value>, ToolError> {
                if key == "embedding_models" {
                    parse_model_entries(key, value)
                        .map(|entries| entries.iter().map(|e| e.to_json()).collect())
                } else {
                    parse_rerank_entries(key, value)
                        .map(|entries| entries.iter().map(|e| e.to_json()).collect())
                }
            };
            let valid = json!([
                {"id":" local ", "name":" Local model ", "base_url":"http://x", "model":"shared"},
                {"id":"cloud", "name":"Cloud model", "base_url":"http://y", "model":"shared"}
            ]);
            let entries = parse(&valid).unwrap();
            assert_eq!(entries[0]["id"], "local");
            assert_eq!(entries[0]["name"], "Local model");
            for invalid in [Value::Null, json!("  "), json!(3)] {
                let mut candidate = valid.clone();
                candidate[0]["id"] = invalid;
                assert!(parse(&candidate).is_err(), "{key}: id");
            }
            let mut missing_id = valid.clone();
            missing_id[0].as_object_mut().unwrap().remove("id");
            assert!(parse(&missing_id).is_err());
            for name in [Value::Null, json!(""), json!("  ")] {
                let mut candidate = valid.clone();
                candidate[0]["name"] = name;
                assert_eq!(parse(&candidate).unwrap()[0]["name"], "");
            }
            let mut unnamed = valid.clone();
            unnamed[0].as_object_mut().unwrap().remove("name");
            assert_eq!(parse(&unnamed).unwrap()[0]["name"], "");
            unnamed[0]["name"] = json!(3);
            assert!(parse(&unnamed).is_err());
            let mut duplicate = valid;
            duplicate[1]["id"] = json!("local");
            assert!(parse(&duplicate).is_err());
        }
    }

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
    fn query_get_parses_pairs() {
        let q = "tag=%E9%A1%B9%E7%9B%AE&limit=20&empty=";
        assert_eq!(query_get(q, "tag").as_deref(), Some("项目"));
        assert_eq!(query_get(q, "limit").as_deref(), Some("20"));
        assert_eq!(query_get(q, "empty").as_deref(), Some(""));
        assert_eq!(query_get(q, "missing"), None);
    }

    #[test]
    fn list_args_maps_query_params() {
        let args = list_or_search_args("query=rust&limit=5&offset=2&tag_expr=%28a%26b%29%7Cc");
        assert_eq!(args["query"], "rust");
        assert_eq!(args["limit"], 5);
        assert_eq!(args["offset"], 2);
        assert_eq!(args["tag_expr"], "(a&b)|c");
        // The retired tag/tags query parameters are no longer forwarded: tag filtering has one
        // syntax (tag_expr), unknown parameters are rejected at the tool boundary
        let legacy = list_or_search_args("tag=x&tags=a,b&sort=created_at&order=asc");
        assert!(legacy.get("tag").is_none());
        assert!(legacy.get("tags").is_none());
        assert_eq!(legacy["sort"], "created_at");
        assert_eq!(legacy["order"], "asc");
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
        // Validation failure (missing required argument) → 400 (content is optional; summary is not)
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "POST",
            "/api/memories",
            "",
            br#"{"content": "body without summary"}"#,
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
        // Echo-free: the saved ack replaces the view (the refreshed row comes from the list)
        assert_eq!(v["saved"], true);
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

        // Settings: write (custom prompt) → read back; unknown keys rejected; empty values are also valid (falls back to default)
        let settings_body =
            serde_json::to_vec(&json!({ "instructions": "团队共享库规范" })).unwrap();
        let (status, v) = handle(&db, &open_ctx(), "PUT", "/api/settings", "", &settings_body);
        assert_eq!(status, 200, "{v}");
        let (status, v) = handle(&db, &open_ctx(), "GET", "/api/settings", "", &[]);
        assert_eq!(status, 200);
        assert_eq!(v["instructions"], "团队共享库规范");
        assert!(
            v.get("conventions").is_none(),
            "the retired conventions key is gone (conventions live as a reserved tag now)"
        );
        assert!(
            v["default_instructions"].as_str().unwrap().len() > 50,
            "GET must carry the built-in default for the UI's restore action"
        );
        // Unknown keys rejected — including the retired conventions key
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"nope": "x"}"#,
        );
        assert_eq!(status, 400);
        let (status, _) = handle(
            &db,
            &open_ctx(),
            "PUT",
            "/api/settings",
            "",
            br#"{"conventions": "obsolete"}"#,
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
