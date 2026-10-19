//! Memory management: CRUD, browsing and search (the carrier of progressive disclosure).

use crate::auth::IdentityCtx;
use crate::model::{
    normalize_id, normalize_tag_name, normalize_tag_pairs, now, TagPair, RESERVED_TAG,
};
use crate::search;
use crate::store::{ListFilter, Store};
use crate::tag_expr::{self, TagExpr};
use crate::tools::params::{
    opt_bool, opt_str, opt_str_list, opt_u64, req_id_list, req_str, validate_content,
    validate_summary,
};
use crate::tools::ToolError;
use serde_json::{json, Map, Value};

/// Parse and store-validate a tag expression: syntax errors and unknown leaf tags are Invalid,
/// unknown names get the same did-you-mean hint as tag linking on writes.
fn resolve_tag_expr(st: &Store, ctx: &IdentityCtx, raw: &str) -> Result<TagExpr, ToolError> {
    let expr = st
        .resolve_named_query(
            ctx,
            st.bind_query_atoms(tag_expr::parse(raw)?)
                .map_err(ToolError::invalid)?,
        )
        .map_err(ToolError::invalid)?;
    let mut unknown: Vec<String> = Vec::new();
    // Leaf names are display forms: a valued `key:value` leaf validates its KEY against the
    // vocabulary (values are per-memory data, not vocabulary), a bare leaf validates itself.
    for name in expr.tag_names() {
        let key = match name.split_once(':') {
            Some((key, _)) => key,
            None => name,
        };
        if !st.tag_exists(key).map_err(ToolError::invalid)? {
            match st
                .find_tag_case_insensitive(key)
                .map_err(ToolError::invalid)?
            {
                Some(similar) => unknown.push(format!("'{key}' (did you mean '{similar}'?)")),
                None => unknown.push(format!("'{key}'")),
            }
        }
    }
    if !unknown.is_empty() {
        return Err(ToolError::invalid(format!(
            "unknown tags in tag_expr: {} (see tag_list for available names)",
            unknown.join(", ")
        )));
    }
    Ok(expr)
}

/// Read the optional tag_expr argument: absent, empty or whitespace-only counts as no expression
/// (the common clearing case stays lenient; a non-empty malformed expression is an error).
fn opt_tag_expr(
    st: &Store,
    ctx: &IdentityCtx,
    args: &Map<String, Value>,
) -> Result<Option<TagExpr>, ToolError> {
    let expr = match opt_str(args, "tag_expr")? {
        Some(raw) if !raw.trim().is_empty() => Some(resolve_tag_expr(st, ctx, &raw)?),
        _ => None,
    };
    Ok(expr)
}

/// Attach the typed `attributes` view (derived from the `key:value` tags + key registry) to a
/// memory view, only when the memory carries at least one registered key — sparse by default.
fn attach_attributes(st: &Store, m: &crate::model::Memory, view: &mut Value) -> Result<(), String> {
    let attributes = st.attributes_view(m)?;
    if !attributes.as_object().expect("object").is_empty() {
        view["attributes"] = attributes;
    }
    Ok(())
}

pub fn memory_create(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let summary = validate_summary(&req_str(args, "summary")?)?;
    // Content is optional: a summary-only memory stores an empty body (whitespace counts as none)
    let content = match opt_str(args, "content")? {
        Some(c) if !c.trim().is_empty() => validate_content(&c)?,
        _ => String::new(),
    };
    // One tags object: keys map to a value (string or number) or null for a bare tag. A key is
    // unique per memory, which the object encodes structurally.
    let tags: Vec<TagPair> = match args.get("tags") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Object(object)) => normalize_tag_pairs(object).map_err(ToolError::invalid)?,
        Some(other) => {
            return Err(ToolError::invalid(format!(
                "tags must be an object mapping key names to values (string, number or null), got {other}"
            )))
        }
    };
    // Default off: unknown tags are an error listing similar existing ones, keeping the taxonomy
    // from rotting through casual tagging; the agent opts into auto-creation explicitly.
    let create_missing = opt_bool(args, "create_missing_tags")?.unwrap_or(false);

    let linkage = st.link_keys(&tags, create_missing)?;
    let links: Vec<(i64, Option<String>)> = linkage
        .ids
        .iter()
        .copied()
        .zip(tags.iter().map(|(_, v)| v.clone()))
        .collect();

    // Duplicate detection (deterministic rule: titles exactly equal after normalization). Computed before insertion so the memory cannot match itself.
    // The goal is not to block storage but to remind the agent: with an identical-title memory already present, memory_update is the right call.
    let duplicate_of = st.find_duplicates_by_summary(&summary)?;

    let ts = now();
    let id = st.insert_memory(&summary, &content, &links, ts, ts)?;
    // Echo-free response: the caller just stated summary/content/tags, so none of them come back —
    // only the facts it cannot derive (id, server timestamp) plus the sparse write companions
    // (three-way tag-link classification minus the derivable reused set, duplicate hints), each
    // attached only when non-empty. The common case (all tags known, nothing duplicated) carries
    // zero scaffold tokens.
    let mut out = json!({
        "id": Store::format_id(id),
        "updated": crate::util::format_local_compact(ts),
    });
    if !linkage.autocreated.is_empty() {
        out["tags_autocreated"] = json!(linkage.autocreated);
    }
    // tags_reused is deliberately absent: it is exactly the input minus these two lists
    if !linkage.missing_description.is_empty() {
        out["tags_missing_description"] = json!(linkage.missing_description);
    }
    if !duplicate_of.is_empty() {
        out["duplicate_of"] = json!(duplicate_of);
    }
    let refs = id_like_tokens(&[&summary, &content]);
    if !refs.is_empty() {
        append_note(&mut out, id_reference_note(&refs));
    }
    Ok(out)
}

/// remove_tags carries key names: the same identifier rule as tag names (a colon inside a name
/// would make the keyed qualification ambiguous).
fn normalize_tag_name_list(names: &[String]) -> Result<Vec<String>, ToolError> {
    names
        .iter()
        .map(|n| normalize_tag_name(n).map_err(ToolError::invalid))
        .collect()
}

pub fn memory_use(
    st: &Store,
    ctx: &IdentityCtx,
    args: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let raw = req_str(args, "id")?;
    let id = Store::parse_id(&raw).ok_or_else(|| ToolError::invalid("id must follow m<N>"))?;
    if !st.memory_exists(id)? {
        return Err(ToolError::not_found(format!("memory '{raw}' not found")));
    }
    let key = req_str(args, "event_key")?;
    if key.is_empty() || key.chars().count() > 100 || key.chars().any(char::is_control) {
        return Err(ToolError::invalid(
            "event_key must contain 1..100 characters without control characters",
        ));
    }
    let (recorded, reinforced) =
        st.access_append(id, &ctx.name, crate::access::Kind::Use, Some(&key), now())?;
    Ok(json!({"recorded": recorded, "reinforced": reinforced}))
}

pub fn memory_list(
    st: &Store,
    ctx: &IdentityCtx,
    args: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let sort_opt = opt_str(args, "sort")?;
    // 'id' is the creation order (ids are monotonic at insert), so there is no separate
    // created_at sort key — and the summary rows do not carry a creation timestamp at all
    let sort = match sort_opt.as_deref() {
        None => "updated_at",
        Some(s @ ("updated_at" | "id")) => s,
        Some(o) => {
            return Err(ToolError::invalid(format!(
                "sort must be 'updated_at' or 'id', got '{o}'"
            )))
        }
    };
    let asc = match opt_str(args, "order")?.as_deref() {
        None | Some("desc") => false,
        Some("asc") => true,
        Some(o) => {
            return Err(ToolError::invalid(format!(
                "order must be 'asc' or 'desc', got '{o}'"
            )))
        }
    };
    let offset = opt_u64(args, "offset")?.unwrap_or(0);
    let limit = opt_u64(args, "limit")?.unwrap_or(20).clamp(1, 200);

    // Tag expression: resolved against the full store into a memory-id set (JSON array text),
    // handed to the same static SQL as the exact-tag filter — both AND together. Regex atoms
    // evaluate per memory tag set right here (SQL has no regex capability).
    let tag_expr = opt_tag_expr(st, ctx, args)?;
    let id_set = match &tag_expr {
        Some(expr) => {
            let memories = st.all_memories()?;
            let ids: Vec<i64> = memories
                .iter()
                .filter(|m| expr.eval(&m.tags))
                .filter_map(|m| Store::parse_id(&m.id))
                .collect();
            Some(json!(ids).to_string())
        }
        None => None,
    };
    let (total, page) = st.list_memories(
        ListFilter {
            tag: None,
            id_set: id_set.as_deref(),
        },
        sort,
        asc,
        offset,
        limit,
    )?;
    let memories: Vec<Value> = page
        .iter()
        .map(|m| {
            let mut view = m.summary_view();
            attach_attributes(st, m, &mut view)?;
            Ok(view)
        })
        .collect::<Result<_, String>>()?;

    Ok(json!({"total": total, "offset": offset, "limit": limit, "memories": memories}))
}

pub fn memory_search(
    st: &Store,
    ctx: &IdentityCtx,
    args: &Map<String, Value>,
) -> Result<Value, ToolError> {
    let query = req_str(args, "query")?;
    if query.trim().is_empty() {
        return Err(ToolError::invalid("query must not be empty"));
    }
    let tag_expr = opt_tag_expr(st, ctx, args)?;
    let limit = opt_u64(args, "limit")?.unwrap_or(10).clamp(1, 50);
    let offset = opt_u64(args, "offset")?.unwrap_or(0);
    enum SearchMode {
        Auto,
        Keyword,
        Hybrid,
    }
    let mode = match opt_str(args, "mode")?.as_deref() {
        None | Some("auto") => SearchMode::Auto,
        Some("keyword") => SearchMode::Keyword,
        Some("hybrid") => SearchMode::Hybrid,
        Some(other) => {
            return Err(ToolError::invalid(format!(
                "mode must be 'auto', 'keyword' or 'hybrid', got '{other}'"
            )))
        }
    };

    let scope = crate::search_snapshot::scope(
        st,
        ctx,
        &json!({"query":query,"tag_expr":opt_str(args,"tag_expr")?.unwrap_or_default().trim(),"mode":opt_str(args,"mode")?.unwrap_or_else(|| "auto".into())}),
    );
    let revision = st.search_revision()?;
    if let Some(cursor) = opt_str(args, "cursor")? {
        return crate::search_snapshot::get(&cursor, &scope, revision, offset, limit);
    }

    // Eligibility must precede every channel's top-k selection: filtering after
    // vector truncation can discard its entire head and hide eligible memories.
    let query_time = now();
    let memories: Vec<_> = st
        .all_memories()?
        .into_iter()
        .filter(|m| tag_expr.as_ref().is_none_or(|expr| expr.eval(&m.tags)))
        .collect();
    let keyword_hits = search::literal_channels(&memories, &query);
    // Count literal candidates once even when both independent channels support them.
    let keyword_idx: std::collections::HashSet<usize> =
        keyword_hits.iter().flatten().map(|h| h.idx).collect();
    // Page size must not change recall or the head that the reranker scores.
    let limits = st.search_limits()?;
    let vector_k = limits.semantic_candidates;

    // Semantic path: auto passes through per configuration, hybrid requires it explicitly, keyword never comes here.
    // Embedding services all unavailable (timeout/error) → fall back to the keyword pass and flag it,
    // the fallback being a promise, not an error. Semantic search not configured at all →
    // keyword under auto as well, flagged explicitly so the caller never has to infer the state
    // from mode + tool description.
    let config_result = st.embedding_configs();
    let mut used_hybrid = false;
    let mut semantic_fallback = false;
    let mut semantic_disabled = false;
    let mut vector_model: Option<String> = None;
    let vector_hits = match mode {
        SearchMode::Keyword => Vec::new(),
        SearchMode::Auto => match &config_result {
            Err(e) => {
                eprintln!(
                    "semantic search fell back to keyword (cannot read embedding config): {e}"
                );
                semantic_fallback = true;
                Vec::new()
            }
            Ok(configs) if configs.is_empty() => {
                semantic_disabled = true;
                Vec::new()
            }
            Ok(configs) => {
                let (hits, ok, model) = semantic_pass(st, configs, &memories, &query, vector_k);
                used_hybrid = ok;
                semantic_fallback = !ok;
                vector_model = model;
                hits
            }
        },
        SearchMode::Hybrid => {
            let configs = match &config_result {
                Ok(configs) if !configs.is_empty() => configs,
                Ok(_) => {
                    return Err(ToolError::invalid(
                        "mode 'hybrid' requires semantic search to be enabled and configured (embedding settings in the admin UI)",
                    ));
                }
                Err(e) => {
                    return Err(ToolError::invalid(format!(
                        "mode 'hybrid' could not read the embedding configuration: {e}"
                    )));
                }
            };
            let (hits, ok, model) = semantic_pass(st, configs, &memories, &query, vector_k);
            used_hybrid = ok;
            semantic_fallback = !ok;
            vector_model = model;
            hits
        }
    };

    let mut eligible = keyword_idx.clone();
    eligible.extend(vector_hits.iter().map(|hit| hit.idx));
    let policy = st.lifecycle_policy()?;
    let priors = crate::lifecycle::rank_channels(
        &memories,
        &eligible,
        &st.access_projection_all()?,
        &policy,
        &st.decay_plan(),
        query_time,
    );
    let hits = search::fuse(
        &memories,
        &[
            (&keyword_hits[0], 1.0),
            (&keyword_hits[1], 1.0),
            (&vector_hits, 1.0),
            (&priors[0], policy.freshness_weight),
            (&priors[1], policy.reinforcement_weight),
        ],
    );

    // Rerank stage: a configured cross-encoder re-scores the candidate pool (recall wide,
    // rerank narrow), giving one calibrated relevance scale across both channels. Explicit
    // keyword mode keeps its deterministic keyword order; everywhere else the fused order is
    // the silent fallback when no reranker answers. Filtering runs first so no rerank work is
    // spent on candidates the tag expression would drop anyway.
    let (hits, reranked_by) = if matches!(mode, SearchMode::Auto | SearchMode::Hybrid) {
        rerank_pass(st, hits, &memories, &query, &limits)
    } else {
        (hits, None)
    };

    let total = hits.len() as u64;
    let keyword_matches = hits.iter().filter(|h| keyword_idx.contains(&h.idx)).count() as u64;
    let results: Vec<Value> = hits
        .iter()
        .map(|h| {
            let m = &memories[h.idx];
            json!({
                "id": m.id,
                "tags": m.tags,
                "summary": m.summary,
                "score": h.score,
                "snippet": h.snippet,
                "updated": crate::util::format_local_compact(m.updated_at),
            })
        })
        .collect();

    let mut out = json!({
        "total_matches": total,
        "offset": offset,
        "returned": results.len(),
        "mode": if used_hybrid { "hybrid" } else { "keyword" },
        "results": results,
    });
    if semantic_fallback {
        out["semantic_fallback"] = json!(true);
    }
    if semantic_disabled {
        out["semantic"] = json!("disabled");
    }
    // Which candidate actually answered the query embedding — with several configured models
    // this is the fact a caller cannot derive from the response itself
    if let Some(model) = &vector_model {
        out["embedding_model"] = json!(model);
    }
    if let Some(model) = &reranked_by {
        out["reranked_by"] = json!(model);
    }
    // Only hybrid fuses two channels; in keyword mode the count would just duplicate total_matches
    if used_hybrid {
        out["keyword_matches"] = json!(keyword_matches);
    }
    // Progressive-disclosure guidance is carried only on the first page; by paging, the client has already read it, saving repeated context overhead
    out["hint"] = json!(
            "Summaries + snippets only (progressive disclosure). Call memory_get with the ids worth reading to reveal full content."
    );
    if total == 0 {
        out["note"] = json!(
            "no memory matched every term; drop some terms or try broader ones (matching is case-insensitive substring)"
        );
    }
    crate::search_snapshot::create(st, scope, revision, out, offset, limit)
}

pub fn memory_get(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let ids = req_id_list(args, "ids", 50)?;
    let (numeric, invalid) = split_ids(&ids);
    let (found, missing) = st.get_memories(&numeric)?;
    let memories: Vec<Value> = found
        .iter()
        .map(|m| {
            let mut view = m.full_view();
            attach_attributes(st, m, &mut view)?;
            Ok(view)
        })
        .collect::<Result<_, String>>()?;
    let mut out = json!({"memories": memories});
    // Sparse: a fully successful read is just the memories array
    if !missing.is_empty() {
        out["missing"] = json!(missing);
    }
    if !invalid.is_empty() {
        out["invalid_ids"] = json!(invalid);
    }
    attach_id_note(&mut out, !missing.is_empty(), &invalid);
    Ok(out)
}

/// Split the id list: parseable numbers versus format-invalid raw strings (missing m prefix etc.).
/// The two are reported separately — "malformed format" and "does not exist" are different errors to an agent.
fn split_ids(ids: &[String]) -> (Vec<i64>, Vec<String>) {
    let mut numeric = Vec::new();
    let mut invalid = Vec::new();
    for raw in ids {
        let id = normalize_id(raw);
        match Store::parse_id(&id) {
            Some(n) => numeric.push(n),
            None => invalid.push(id),
        }
    }
    (numeric, invalid)
}

/// Attach guiding notes to results carrying ids: the missing and invalid hints are independent and can coexist.
fn attach_id_note(out: &mut Value, has_missing: bool, invalid: &[String]) {
    let mut notes: Vec<&str> = Vec::new();
    if has_missing {
        notes.push(
            "some ids were not found; use memory_list or memory_search to discover valid ids",
        );
    }
    if !invalid.is_empty() {
        notes.push(
            "some ids are malformed; memory ids look like 'm123' (a leading 'm' is required)",
        );
    }
    if !notes.is_empty() {
        out["note"] = json!(notes.join("; "));
    }
}

pub fn memory_update(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let raw_id = normalize_id(&req_str(args, "id")?);
    // Invalid formats such as bare numbers are reported as 400 with the format requirement spelled out, not lumped into "not found" (404)
    let Some(id) = Store::parse_id(&raw_id) else {
        return Err(ToolError::invalid(format!(
            "malformed memory id '{raw_id}': ids look like 'm123' (a leading 'm' is required)"
        )));
    };
    let summary = match opt_str(args, "summary")? {
        Some(s) => Some(validate_summary(&s)?),
        None => None,
    };
    let content = match opt_str(args, "content")? {
        Some(c) if !c.trim().is_empty() => Some(validate_content(&c)?),
        // Explicit empty / whitespace content clears the body (summary-only memory)
        Some(_) => Some(String::new()),
        None => None,
    };
    // Incremental tag edits: `add_tags` maps keys to values (upserting the key's value on the
    // memory), `remove_tags` lists key names whose relation rows are dropped — one key holds one
    // value, so the key name identifies at most one tag. Restating untouched tags is never needed.
    let add_tags: Option<Vec<TagPair>> = match args.get("add_tags") {
        None | Some(Value::Null) => None,
        Some(Value::Object(object)) => {
            Some(normalize_tag_pairs(object).map_err(ToolError::invalid)?)
        }
        Some(other) => {
            return Err(ToolError::invalid(format!(
                "add_tags must be an object mapping key names to values (string, number or null), got {other}"
            )))
        }
    };
    let remove_tags = match opt_str_list(args, "remove_tags")? {
        Some(l) => Some(normalize_tag_name_list(&l)?),
        None => None,
    };
    let create_missing = opt_bool(args, "create_missing_tags")?.unwrap_or(false);
    if summary.is_none() && content.is_none() && add_tags.is_none() && remove_tags.is_none() {
        return Err(ToolError::invalid(
            "nothing to update: provide summary, content, add_tags and/or remove_tags",
        ));
    }

    // Resolve tags first (the tags table is what changes), then update the memory — the order matches the error messages.
    let mut linkage_opt = None;
    let mut add_links: Vec<(i64, Option<String>)> = Vec::new();
    if let Some(add) = &add_tags {
        let linkage = st.link_keys(add, create_missing)?;
        add_links = linkage
            .ids
            .iter()
            .copied()
            .zip(add.iter().map(|(_, v)| v.clone()))
            .collect();
        linkage_opt = Some(linkage);
    }
    let mut remove_ids: Vec<i64> = Vec::new();
    if let Some(remove) = &remove_tags {
        remove_ids = st.tag_ids_for_keys(remove)?;
    }

    if !st.memory_exists(id)? {
        return Err(ToolError::not_found(format!(
            "memory '{raw_id}' not found (use memory_list or memory_search first)"
        )));
    }

    let changed = st.update_memory(
        id,
        summary.as_deref(),
        content.as_deref(),
        &add_links,
        &remove_ids,
    )?;

    // Echo-free: no view of the stored memory — summary/content/tag names were just stated by the
    // caller, and inspecting current state is a read's job (memory_get). Only the "did anything
    // actually change" flag plus the sparse write companions (same classification as memory_create)
    let mut out = json!({"updated": changed});
    // Same three-way classification as memory_create: attached per-array, only when non-empty
    if let Some(linkage) = linkage_opt {
        if !linkage.autocreated.is_empty() {
            out["tags_autocreated"] = json!(linkage.autocreated);
        }
        // tags_reused is deliberately absent: it is exactly the input minus these two lists
        if !linkage.missing_description.is_empty() {
            out["tags_missing_description"] = json!(linkage.missing_description);
        }
    }
    // Id-reference hygiene: freshly written text containing id-shaped tokens warns; replacing the
    // content changes what inbound references to this id see, so referencing memories are reported
    let mut written: Vec<&str> = Vec::new();
    if let Some(s) = &summary {
        written.push(s);
    }
    if let Some(c) = &content {
        written.push(c);
    }
    let refs = id_like_tokens(&written);
    if !refs.is_empty() {
        append_note(&mut out, id_reference_note(&refs));
    }
    if changed && content.is_some() {
        let memories = st.all_memories()?;
        let inbound = inbound_references(&memories, &raw_id, &raw_id);
        if !inbound.is_empty() {
            out["referenced_by"] = json!(inbound
                .iter()
                .map(|(id, summary)| json!({"id": id, "summary": summary, "to": raw_id.as_str()}))
                .collect::<Vec<_>>());
            append_note(
                &mut out,
                "these memories reference this memory's id (see referenced_by) and were written against its previous content — check whether they still describe it accurately"
                    .to_string(),
            );
        }
    }
    Ok(out)
}

/// Edit one memory's content by exact string replacement (the harness file-editor pattern): the
/// caller cites the span to change instead of restating the whole body, so a one-word fix on a long
/// memory costs tokens for the span only and the untouched parts cannot silently drift. Matching is
/// byte-exact; ambiguity fails without side effects (with the match count) unless replace_all.
pub fn memory_edit(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let raw_id = normalize_id(&req_str(args, "id")?);
    // Same id discipline as memory_update: format errors are 400, not lumped into "not found" (404)
    let Some(id) = Store::parse_id(&raw_id) else {
        return Err(ToolError::invalid(format!(
            "malformed memory id '{raw_id}': ids look like 'm123' (a leading 'm' is required)"
        )));
    };
    let old_string = req_str(args, "old_string")?;
    let new_string = req_str(args, "new_string")?;
    let replace_all = opt_bool(args, "replace_all")?.unwrap_or(false);
    if old_string.is_empty() {
        return Err(ToolError::invalid(
            "old_string must not be empty (an empty match is ambiguous; to rewrite the whole content use memory_update)",
        ));
    }
    if old_string == new_string {
        return Err(ToolError::invalid(
            "old_string and new_string are identical: nothing to replace",
        ));
    }

    let (mut found, _) = st.get_memories(&[id])?;
    let memory = found.pop().ok_or_else(|| {
        ToolError::not_found(format!(
            "memory '{raw_id}' not found (use memory_list or memory_search first)"
        ))
    })?;
    let count = memory.content.matches(&old_string).count();
    if count == 0 {
        return Err(ToolError::invalid(format!(
            "old_string not found in memory '{raw_id}' (matching is byte-exact: case-sensitive, whitespace-significant); use memory_get to re-read the current content"
        )));
    }
    if count > 1 && !replace_all {
        return Err(ToolError::invalid(format!(
            "old_string matches {count} times in memory '{raw_id}': add surrounding text to make the match unique, or pass replace_all: true to replace every occurrence"
        )));
    }

    let updated = if replace_all {
        memory.content.replace(&old_string, &new_string)
    } else {
        memory.content.replacen(&old_string, &new_string, 1)
    };
    // The cap applies to the post-edit result; failing here leaves the memory untouched
    validate_content(&updated)
        .map_err(|e| ToolError::invalid(format!("edit rejected, nothing was changed: {e}")))?;
    st.update_memory(id, None, Some(&updated), &[], &[])?;

    // Echo-free: the caller computed the replacement itself, so the resulting content and the
    // untouched summary/tags are all known — only the replacement count is reported
    let mut out = json!({
        "replaced": if replace_all { count } else { 1 },
    });
    let refs = id_like_tokens(&[&new_string]);
    if !refs.is_empty() {
        append_note(&mut out, id_reference_note(&refs));
    }
    Ok(out)
}

pub fn memory_delete(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let ids = req_id_list(args, "ids", 50)?;
    let (numeric, invalid) = split_ids(&ids);
    let (deleted, missing) = st.delete_memories(&numeric)?;
    let mut out = json!({"deleted": deleted});
    if !missing.is_empty() {
        out["missing"] = json!(missing);
    }
    if !invalid.is_empty() {
        out["invalid_ids"] = json!(invalid);
    }
    attach_id_note(&mut out, !missing.is_empty(), &invalid);
    // References die with their target: surviving memories mentioning a deleted id are reported
    // (ids + summaries only, progressive disclosure) so the caller can repair the dangling mentions
    if !deleted.is_empty() {
        let memories = st.all_memories()?;
        let mut dangling = Vec::new();
        for did in &deleted {
            for (id, summary) in inbound_references(&memories, did, "") {
                dangling.push(json!({"id": id, "summary": summary, "to": did}));
            }
        }
        if !dangling.is_empty() {
            out["referenced_by"] = json!(dangling);
            append_note(
                &mut out,
                "some remaining memories reference a deleted id (see referenced_by): those mentions now dangle — repair or drop them"
                    .to_string(),
            );
        }
    }
    Ok(out)
}

/// Merge a duplicate memory into a kept one: the target survives with its id and creation time
/// intact, tags become the union, the source is deleted. This closes the loop duplicate_of /
/// similar_to open — before it, the only path away from a duplicate was delete + re-create,
/// which reset the created time.
pub fn memory_merge(
    st: &Store,
    ctx: &IdentityCtx,
    args: &Map<String, Value>,
) -> Result<Value, ToolError> {
    fn parse_id_arg(raw: &str) -> Result<i64, ToolError> {
        Store::parse_id(raw).ok_or_else(|| {
            ToolError::invalid(format!(
                "malformed memory id '{raw}': ids look like 'm123' (a leading 'm' is required)"
            ))
        })
    }
    let raw_target = normalize_id(&req_str(args, "target")?);
    let raw_source = normalize_id(&req_str(args, "source")?);
    let target_id = parse_id_arg(&raw_target)?;
    let source_id = parse_id_arg(&raw_source)?;
    if target_id == source_id {
        return Err(ToolError::invalid(
            "cannot merge a memory into itself: target and source are the same id",
        ));
    }
    let summary = match opt_str(args, "summary")? {
        Some(s) => Some(validate_summary(&s)?),
        None => None,
    };
    let explicit_content = match opt_str(args, "content")? {
        Some(c) => Some(validate_content(&c)?),
        None => None,
    };

    let (mut found_t, missing_t) = st.get_memories(&[target_id])?;
    let (mut found_s, missing_s) = st.get_memories(&[source_id])?;
    if !missing_t.is_empty() || !missing_s.is_empty() {
        let mut missing = missing_t;
        missing.extend(missing_s);
        return Err(ToolError::not_found(format!(
            "memory not found: {} (use memory_list or memory_search first)",
            missing.join(", ")
        )));
    }
    let target = found_t.pop().expect("target loaded");
    let source = found_s.pop().expect("source loaded");

    // Reserved-tag rule, data-dependent edition: the arguments name no tags, so the entry guard
    // cannot see this — merging unions the tags onto the target (an attach when the source carries
    // the convention) and deletes a resident memory when the source is one. Both are admin-only moves.
    let touches_reserved = target
        .tags
        .iter()
        .chain(source.tags.iter())
        .any(|t| t == RESERVED_TAG);
    if touches_reserved && !ctx.can(crate::auth::Cap::Admin) {
        return Err(ToolError::forbidden(format!(
            "merging memories that carry the reserved tag '{RESERVED_TAG}' requires the 'admin' permission (the resident convention is operator-curated)"
        )));
    }

    let merged_content = explicit_content
        .clone()
        .unwrap_or_else(|| format!("{}\n\n{}", target.content, source.content));
    validate_content(&merged_content)?;
    // Source tags join the target; a key the target already carries stays as it is (the target
    // is the surviving identity) and lands in keys_conflicted. No auto-creation — both memories'
    // keys already exist by definition.
    let source_rows = st.tag_rows_for(source_id)?;
    let target_keys: Vec<i64> = st
        .tag_rows_for(target_id)?
        .into_iter()
        .map(|(id, _, _)| id)
        .collect();
    let mut add_links: Vec<(i64, Option<String>)> = Vec::new();
    let mut conflicted: Vec<String> = Vec::new();
    for (id, name, value) in source_rows {
        if target_keys.contains(&id) {
            conflicted.push(name);
        } else {
            add_links.push((id, value));
        }
    }
    st.update_memory(
        target_id,
        summary.as_deref(),
        Some(&merged_content),
        &add_links,
        &[],
    )?;
    let (deleted, _) = st.delete_memories(&[source_id])?;
    if deleted.is_empty() {
        // Unreachable inside the transaction (the source was just read), but never lie about a
        // delete that did not happen
        return Err(ToolError::not_found(format!(
            "memory '{raw_source}' disappeared during merge; nothing was changed"
        )));
    }

    // Echo-free: the surviving id and the absorbed id are the caller's own arguments (target /
    // source), the union tag set is readable via memory_get — only the id-reference reports carry
    // facts the caller cannot derive. A clean merge therefore answers with an empty object.
    // Id-reference hygiene: the absorbed id dies here — surviving memories mentioning it (the
    // target included: its merged content may carry such a mention) are reported.
    let memories = st.all_memories()?;
    let inbound = inbound_references(&memories, &raw_source, "");
    let mut out = json!({});
    if !conflicted.is_empty() {
        out["keys_conflicted"] = json!(conflicted);
        append_note(
            &mut out,
            "these keys exist on both memories; the target's values were kept".to_string(),
        );
    }
    if !inbound.is_empty() {
        out["referenced_by"] = json!(inbound
            .iter()
            .map(|(id, summary)| json!({"id": id, "summary": summary, "to": raw_source.as_str()}))
            .collect::<Vec<_>>());
        append_note(
            &mut out,
            format!("the absorbed id {raw_source} is still referenced (see referenced_by): those mentions now dangle — repair or drop them"),
        );
    }
    if let Some(c) = &explicit_content {
        let refs = id_like_tokens(&[c]);
        if !refs.is_empty() {
            append_note(&mut out, id_reference_note(&refs));
        }
    }
    Ok(out)
}

/// Id-shaped token detection for freshly written text: `m123`, word-bounded and case-sensitive —
/// exactly the shape `Store::parse_id` accepts. Cross-memory id references are discouraged (ids are
/// unstable: delete/merge removes them, export/import renumbers them), so writes containing them
/// draw a warning note; fragments like "cm3" or uppercase "M4" stay unnoticed by construction.
fn id_like_tokens(texts: &[&str]) -> Vec<String> {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| regex::Regex::new(r"\bm[0-9]+\b").expect("static regex"));
    let mut found: Vec<String> = Vec::new();
    for text in texts {
        for m in re.find_iter(text) {
            let token = m.as_str().to_string();
            if !found.contains(&token) {
                found.push(token);
            }
        }
    }
    found
}

/// Compact display of a token list, capped so a pathological body cannot flood the note.
fn display_list(tokens: &[String]) -> String {
    const MAX: usize = 5;
    match tokens.len() {
        0 => String::new(),
        n if n <= MAX => tokens.join(", "),
        n => format!("{} and {} more", tokens[..MAX].join(", "), n - MAX),
    }
}

/// The guidance note for written text containing id-shaped tokens (non-blocking: the write lands).
fn id_reference_note(tokens: &[String]) -> String {
    format!(
        "{} {} like a memory-id reference{}; ids are unstable identifiers (delete/merge removes them, export/import renumbers them) — link to memories by tag or searchable keyword instead",
        display_list(tokens),
        if tokens.len() == 1 { "looks" } else { "look" },
        if tokens.len() == 1 { "" } else { "s" }
    )
}

/// Other memories (excluding `exclude_id`) whose summary or content reference `target` exactly
/// (word-bounded, so m30 does not count as a mention of m3). Reported as (id, summary) pairs —
/// progressive disclosure holds: the report never carries content.
fn inbound_references(
    memories: &[crate::model::Memory],
    target: &str,
    exclude_id: &str,
) -> Vec<(String, String)> {
    let pattern =
        regex::Regex::new(&format!(r"\b{}\b", regex::escape(target))).expect("escaped id pattern");
    memories
        .iter()
        .filter(|m| m.id != exclude_id)
        .filter(|m| pattern.is_match(&m.summary) || pattern.is_match(&m.content))
        .map(|m| (m.id.clone(), m.summary.clone()))
        .collect()
}

/// Append to the response note, joining with "; " when one is already present (the sparse
/// single-note shape shared with attach_id_note).
fn append_note(out: &mut Value, msg: String) {
    let joined = match out.get("note").and_then(Value::as_str) {
        Some(existing) => format!("{existing}; {msg}"),
        None => msg,
    };
    out["note"] = json!(joined);
}

/// Semantic pass: query embedding + cosine ranking against stored vectors, RRF-fused with the keyword pass.
/// The query embedding tries the configured candidates in priority order (see `embed::Failover`) and the
/// winner's identity selects the vector table. Any step failing (reading vectors, every candidate
/// unavailable) falls back to the pure keyword pass, returning `(hits, whether hybrid really ran, the
/// candidate that answered)` — the fallback is a normal path, not an error.
fn semantic_pass(
    st: &Store,
    configs: &[crate::embed::EmbedConfig],
    memories: &[crate::model::Memory],
    query: &str,
    vector_k: usize,
) -> (Vec<search::Hit>, bool, Option<String>) {
    let embedded = crate::embed::EMBED_FAILOVER.first_available(configs, |cfg| {
        crate::embed::embed_texts(
            cfg,
            &[crate::embed::embed_query_text(cfg, query)],
            crate::embed::QUERY_TIMEOUT,
        )
    });
    let (_, cfg, query_vecs) = match embedded {
        Ok(found) => found,
        Err(e) => {
            eprintln!("semantic search fell back to keyword (embedding services unavailable): {e}");
            return (Vec::new(), false, None);
        }
    };
    let Some(query_vec) = query_vecs.into_iter().next() else {
        return (Vec::new(), false, None);
    };
    let table = match st.embeddings_active(&cfg.vector_key(), &cfg.fingerprint()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("semantic search skipped (cannot load embeddings): {e}");
            return (Vec::new(), false, None);
        }
    };
    (
        crate::embed::semantic_hits(memories, &table, &query_vec, vector_k, cfg.min_similarity),
        true,
        Some(cfg.model.clone()),
    )
}

/// Rerank stage: the fused candidate head goes through the configured cross-encoder(s), which
/// rank evidence is fused with the prior order (weights 4:1), preserving soft priors without
/// adding raw relevance to freshness or cosine. Unscored candidates keep their prior evidence.
/// Scores remain RRF ordering values, never comparable across responses. Every failure
/// (no configuration, all candidates down, malformed answer) keeps the incoming order: reranking
/// is an improvement, never a dependency. Returns `(hits, the reranker that answered)`.
fn rerank_pass(
    st: &Store,
    hits: Vec<search::Hit>,
    memories: &[crate::model::Memory],
    query: &str,
    limits: &search::Limits,
) -> (Vec<search::Hit>, Option<String>) {
    let configs = match st.rerank_configs() {
        Ok(configs) => configs,
        Err(e) => {
            eprintln!("rerank skipped (cannot read reranker config): {e}");
            return (hits, None);
        }
    };
    if configs.is_empty() || hits.len() < 2 {
        return (hits, None);
    }
    let ceiling = if limits.adaptive.enabled {
        limits.adaptive.max_candidates
    } else {
        limits.rerank_candidates
    };
    let documents: Vec<String> = hits
        .iter()
        .take(ceiling)
        .map(|h| {
            // The reranker reads the same text the embedding side sees: title + blank line + content
            crate::embed::embed_memory_text(&memories[h.idx].summary, &memories[h.idx].content)
        })
        .collect();
    let document_chars: Vec<_> = documents.iter().map(|doc| doc.chars().count()).collect();
    let scored = crate::rerank::RERANK_FAILOVER.first_available(&configs, |cfg| {
        let pool = crate::adaptive::budget(&st.path, cfg, limits, &document_chars);
        let started = std::time::Instant::now();
        let result = crate::rerank::rerank(
            cfg,
            query,
            &documents[..pool],
            crate::rerank::RERANK_TIMEOUT,
        );
        crate::adaptive::observe(
            &st.path,
            cfg,
            limits,
            pool,
            document_chars.iter().take(pool).sum(),
            started.elapsed(),
            result.is_ok(),
        );
        result
    });
    match scored {
        Ok((_, cfg, scores)) => {
            let mut previous = None;
            let mut rank = 1usize;
            let reranked: Vec<search::Hit> = scores
                .into_iter()
                .enumerate()
                .map(|(position, (doc_idx, relevance))| {
                    if previous != Some(relevance) {
                        rank = position + 1;
                    }
                    previous = Some(relevance);
                    search::Hit {
                        idx: hits[doc_idx].idx,
                        score: -(rank as i64),
                        snippet: hits[doc_idx].snippet.clone(),
                    }
                })
                .collect();
            (
                search::fuse(memories, &[(&hits, 1.0), (&reranked, 4.0)]),
                Some(cfg.model.clone()),
            )
        }
        Err(e) => {
            eprintln!("rerank skipped (all rerankers unavailable): {e}");
            (hits, None)
        }
    }
}
