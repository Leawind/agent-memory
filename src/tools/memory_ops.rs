//! Memory management: CRUD, browsing and search (the carrier of progressive disclosure).

use crate::auth::IdentityCtx;
use crate::model::{normalize_id, normalize_tag_name, now, RESERVED_TAG};
use crate::search;
use crate::store::Store;
use crate::tools::params::{
    normalize_tag_list, opt_bool, opt_regex, opt_str, opt_str_list, opt_u64, req_id_list, req_str,
    validate_content, validate_summary,
};
use crate::tools::ToolError;
use serde_json::{json, Map, Value};

pub fn memory_create(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let summary = validate_summary(&req_str(args, "summary")?)?;
    let content = validate_content(&req_str(args, "content")?)?;
    let raw_tags = opt_str_list(args, "tags")?.unwrap_or_default();
    let tags = normalize_tag_list(&raw_tags)?;
    // Default off: unknown tags are an error listing similar existing ones, keeping the taxonomy
    // from rotting through casual tagging; the agent opts into auto-creation explicitly.
    let create_missing = opt_bool(args, "create_missing_tags")?.unwrap_or(false);

    let linkage = st.link_tags(&tags, create_missing)?;

    // Duplicate detection (deterministic rule: titles exactly equal after normalization). Computed before insertion so the memory cannot match itself.
    // The goal is not to block storage but to remind the agent: with an identical-title memory already present, memory_update is the right call.
    let duplicate_of = st.find_duplicates_by_summary(&summary)?;

    let id = st.insert_memory(&summary, &content, &linkage.ids, now(), now())?;
    let view = memory_view(st, id)?;
    // Sparse response: the three-way tag-link classification (created / reused /
    // reused-but-missing-description, sparing a follow-up tag_list) and the duplicate hint attach
    // only when non-empty — the common case (all tags known, nothing duplicated) carries zero
    // scaffold tokens
    let mut out = json!({ "memory": view });
    if !linkage.autocreated.is_empty() {
        out["tags_autocreated"] = json!(linkage.autocreated);
    }
    if !linkage.reused.is_empty() {
        out["tags_reused"] = json!(linkage.reused);
    }
    if !linkage.missing_description.is_empty() {
        out["tags_missing_description"] = json!(linkage.missing_description);
    }
    if !duplicate_of.is_empty() {
        out["duplicate_of"] = json!(duplicate_of);
    }
    Ok(out)
}

pub fn memory_list(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let tag = match opt_str(args, "tag")? {
        Some(t) => Some(normalize_tag_name(&t)?),
        None => None,
    };
    let tag_re = opt_regex(args, "tag_filter").map_err(ToolError::invalid)?;
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

    // The regex is first resolved over the full tag set into a set of tag names, then translated into internal id sets
    // (SQL has no regex capability; expanding with json_each keeps the SQL static)
    let tag_names = match &tag_re {
        Some(re) => Some(st.tag_names_matching(re)?),
        None => None,
    };
    let tag_set = match &tag_names {
        Some(names) => {
            let ids = st.tag_ids_for_names(names)?;
            Some(json!(ids).to_string())
        }
        None => None,
    };
    let (total, page) =
        st.list_memories(tag.as_deref(), tag_set.as_deref(), sort, asc, offset, limit)?;
    let memories: Vec<Value> = page.iter().map(|m| m.summary_view()).collect();

    let mut out = json!({"total": total, "offset": offset, "limit": limit, "memories": memories});
    if let Some(t) = &tag {
        if !st.tag_exists(t)? {
            out["note"] = json!(format!("tag '{}' does not exist yet; see tag_list", t));
        } else if total == 0 {
            out["note"] = json!(format!("tag '{}' exists but currently has no memories", t));
        }
    }
    if let Some(re) = &tag_re {
        if tag_names.as_ref().is_none_or(|n| n.is_empty()) {
            out["note"] = json!(format!(
                "tag_filter '{}' matched no tags; see tag_list for available names",
                re.as_str()
            ));
        }
    }
    Ok(out)
}

pub fn memory_search(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let query = req_str(args, "query")?;
    if query.trim().is_empty() {
        return Err(ToolError::invalid("query must not be empty"));
    }
    let raw_tags = opt_str_list(args, "tags")?.unwrap_or_default();
    let tag_filter = normalize_tag_list(&raw_tags)?;
    let tag_re = opt_regex(args, "tag_filter").map_err(ToolError::invalid)?;
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

    let memories = st.all_memories()?;
    let keyword_hits = search::run(&memories, &query, &tag_filter, tag_re.as_ref());

    // Semantic path: auto passes through per configuration, hybrid requires it explicitly, keyword never comes here.
    // Embedding service unavailable (timeout/error/database read failure) → fall back to the keyword pass and flag it,
    // the fallback being a promise, not an error path.
    let config = st.embedding_config().map_err(ToolError::invalid)?;
    let mut used_hybrid = false;
    let mut semantic_fallback = false;
    let hits = match mode {
        SearchMode::Keyword => keyword_hits,
        SearchMode::Auto => match &config {
            None => keyword_hits,
            Some(cfg) => {
                let (hits, ok) = semantic_pass(
                    st,
                    cfg,
                    &memories,
                    keyword_hits,
                    &query,
                    &tag_filter,
                    tag_re.as_ref(),
                );
                used_hybrid = ok;
                semantic_fallback = !ok;
                hits
            }
        },
        SearchMode::Hybrid => {
            let Some(cfg) = &config else {
                return Err(ToolError::invalid(
                    "mode 'hybrid' requires semantic search to be enabled and configured (embedding settings in the admin UI)",
                ));
            };
            let (hits, ok) = semantic_pass(
                st,
                cfg,
                &memories,
                keyword_hits,
                &query,
                &tag_filter,
                tag_re.as_ref(),
            );
            used_hybrid = ok;
            semantic_fallback = !ok;
            hits
        }
    };

    let total = hits.len() as u64;
    let results: Vec<Value> = hits
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
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
    // Progressive-disclosure guidance is carried only on the first page; by paging, the client has already read it, saving repeated context overhead
    if offset == 0 {
        out["hint"] = json!(
            "Summaries + snippets only (progressive disclosure). Call memory_get with the ids worth reading to reveal full content."
        );
    }
    if total == 0 {
        out["note"] = json!(
            "no memory matched every term; drop some terms or try broader ones (matching is case-insensitive substring)"
        );
    }
    Ok(out)
}

pub fn memory_get(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let ids = req_id_list(args, "ids", 50)?;
    let (numeric, invalid) = split_ids(&ids);
    let (found, missing) = st.get_memories(&numeric)?;
    let memories: Vec<Value> = found.iter().map(|m| m.full_view()).collect();
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
        Some(c) => Some(validate_content(&c)?),
        None => None,
    };
    let add_tags = match opt_str_list(args, "add_tags")? {
        Some(l) => Some(normalize_tag_list(&l)?),
        None => None,
    };
    let remove_tags = match opt_str_list(args, "remove_tags")? {
        Some(l) => Some(normalize_tag_list(&l)?),
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
    let mut remove_ids: Vec<i64> = Vec::new();
    if let Some(add) = &add_tags {
        linkage_opt = Some(st.link_tags(add, create_missing)?);
    }
    if let Some(remove) = &remove_tags {
        remove_ids = st.tag_ids_for_names(remove)?;
    }

    if !st.memory_exists(id)? {
        return Err(ToolError::not_found(format!(
            "memory '{raw_id}' not found (use memory_list or memory_search first)"
        )));
    }

    let add_ids = linkage_opt
        .as_ref()
        .map(|l| l.ids.clone())
        .unwrap_or_default();
    let changed = st.update_memory(
        id,
        summary.as_deref(),
        content.as_deref(),
        &add_ids,
        &remove_ids,
    )?;

    let mut out = json!({"updated": changed, "memory": memory_view(st, id)?});
    // Same three-way classification as memory_create: attached per-array, only when non-empty
    if let Some(linkage) = linkage_opt {
        if !linkage.autocreated.is_empty() {
            out["tags_autocreated"] = json!(linkage.autocreated);
        }
        if !linkage.reused.is_empty() {
            out["tags_reused"] = json!(linkage.reused);
        }
        if !linkage.missing_description.is_empty() {
            out["tags_missing_description"] = json!(linkage.missing_description);
        }
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
    // conventions) and deletes a resident memory when the source is one. Both are admin-only moves.
    let touches_reserved = target
        .tags
        .iter()
        .chain(source.tags.iter())
        .any(|t| t == RESERVED_TAG);
    if touches_reserved && !ctx.can(crate::auth::Cap::Admin) {
        return Err(ToolError::forbidden(format!(
            "merging memories that carry the reserved tag '{RESERVED_TAG}' requires the 'admin' permission (conventions are operator-curated)"
        )));
    }

    let merged_content = explicit_content
        .clone()
        .unwrap_or_else(|| format!("{}\n\n{}", target.content, source.content));
    validate_content(&merged_content)?;
    // Source tag ids join the target (INSERT OR IGNORE dedups the overlap); no auto-creation —
    // both memories' tags already exist by definition
    let source_tag_ids = st.tag_ids_for_names(&source.tags)?;
    st.update_memory(
        target_id,
        summary.as_deref(),
        Some(&merged_content),
        &source_tag_ids,
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

    let view = memory_view(st, target_id)?;
    // No "merged": true echo - a non-error result already means success
    Ok(json!({
        "memory": view,
        "removed": raw_source,
        "content_appended": explicit_content.is_none(),
    }))
}

/// Semantic pass: query embedding + cosine ranking against stored vectors, RRF-fused with the keyword pass.
/// Any step failing (reading vectors, embedding service timeout/error) falls back to the pure keyword pass,
/// returning `(hits, whether hybrid really ran)` — the fallback is a normal path, not an error.
fn semantic_pass(
    st: &Store,
    cfg: &crate::embed::EmbedConfig,
    memories: &[crate::model::Memory],
    keyword_hits: Vec<search::Hit>,
    query: &str,
    tag_filter: &[String],
    tag_re: Option<&regex::Regex>,
) -> (Vec<search::Hit>, bool) {
    let table = match st.embeddings_active(&cfg.model) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("semantic search skipped (cannot load embeddings): {e}");
            return (keyword_hits, false);
        }
    };
    let query_vecs = match crate::embed::embed_texts(
        cfg,
        &[crate::embed::embed_memory_text(query, "")],
        crate::embed::QUERY_TIMEOUT,
    ) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("semantic search fell back to keyword (embedding service unavailable): {e}");
            return (keyword_hits, false);
        }
    };
    let Some(query_vec) = query_vecs.into_iter().next() else {
        return (keyword_hits, false);
    };
    (
        crate::embed::hybrid_hits(
            memories,
            keyword_hits,
            &table,
            &query_vec,
            tag_filter,
            tag_re,
        ),
        true,
    )
}

/// Fetch one memory's summary view (existence ensured; a uniform error when missing).
fn memory_view(st: &Store, id: i64) -> Result<Value, ToolError> {
    let (mut found, _) = st.get_memories(&[id])?;
    found
        .pop()
        .map(|m| m.summary_view())
        .ok_or_else(|| ToolError::not_found(format!("memory '{}' not found", Store::format_id(id))))
}
