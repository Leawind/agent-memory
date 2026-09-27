//! 记忆管理：增删改查、浏览与搜索（渐进式披露的载体）。

use crate::model::{normalize_id, normalize_tag_name, now};
use crate::search;
use crate::store::Store;
use crate::tools::params::{
    normalize_tag_list, opt_regex, opt_str, opt_str_list, opt_u64, req_id_list, req_str,
    validate_content, validate_summary,
};
use crate::tools::ToolError;
use serde_json::{json, Map, Value};

pub fn memory_create(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let summary = validate_summary(&req_str(args, "summary")?)?;
    let content = validate_content(&req_str(args, "content")?)?;
    let raw_tags = opt_str_list(args, "tags")?.unwrap_or_default();
    let tags = normalize_tag_list(&raw_tags)?;

    let linkage = st.link_tags(&tags)?;

    // 重复检测（确定性规则：摘要归一化后完全相等）。在插入前计算，避免匹配到自己。
    // 目的不是阻止存储，而是提醒 agent：已有同摘要记忆时应改用 memory_update。
    let duplicate_of = st.find_duplicates_by_summary(&summary)?;

    let id = st.insert_memory(&summary, &content, &linkage.ids, now(), now())?;
    let view = memory_view(st, id)?;
    // 标签挂载三分类始终回传：新建 / 复用 / 复用但缺描述（提示用 tag_update
    // 补写），让 agent 不必事后 tag_list 核对挂载是否齐全
    Ok(json!({
        "memory": view,
        "tags_autocreated": linkage.autocreated,
        "tags_reused": linkage.reused,
        "tags_missing_description": linkage.missing_description,
        "duplicate_of": duplicate_of,
    }))
}

pub fn memory_list(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let tag = match opt_str(args, "tag")? {
        Some(t) => Some(normalize_tag_name(&t)?),
        None => None,
    };
    let tag_re = opt_regex(args, "tag_filter").map_err(ToolError::invalid)?;
    let sort_opt = opt_str(args, "sort")?;
    let sort = match sort_opt.as_deref() {
        None => "updated_at",
        Some(s @ ("updated_at" | "created_at" | "id")) => s,
        Some(o) => {
            return Err(ToolError::invalid(format!(
                "sort must be 'updated_at', 'created_at' or 'id', got '{o}'"
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

    // 正则先在标签全集上解析成标签名集合，再换算成内部 id 集合
    // （SQL 无正则能力；json_each 展开保持 SQL 静态）
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

    // 语义路：auto 按配置直通，hybrid 显式要求配置，keyword 永不走。
    // embedding 服务不可用（超时/报错/读库失败）→ 回退关键词趟并打标，
    // 回退是承诺而不是报错路径。
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
                "updated_at": m.updated_at,
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
    // 渐进式披露引导只在第一页携带；翻页时客户端已读过，省掉重复上下文开销
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
    let mut out = json!({"memories": memories, "missing": missing});
    if !invalid.is_empty() {
        out["invalid_ids"] = json!(invalid);
    }
    attach_id_note(&mut out, !missing.is_empty(), &invalid);
    Ok(out)
}

/// 拆分 id 列表：可解析的数字与格式非法的原始串（缺 m 前缀等）。
/// 两者分开呈现——"格式写错"和"不存在"对 agent 是不同的错误。
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

/// 给含 id 的结果附上引导 note：missing 与 invalid 的提示各自独立、可并存。
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
    // 裸数字等非法格式按 400 报出并写明格式要求，而不是混进"不存在"（404）
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
    if summary.is_none() && content.is_none() && add_tags.is_none() && remove_tags.is_none() {
        return Err(ToolError::invalid(
            "nothing to update: provide summary, content, add_tags and/or remove_tags",
        ));
    }

    // 先解析标签（改的是 tags 表），再改记忆，顺序与错误信息一致。
    let mut linkage_opt = None;
    let mut remove_ids: Vec<i64> = Vec::new();
    if let Some(add) = &add_tags {
        linkage_opt = Some(st.link_tags(add)?);
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
    // 与 memory_create 同款三分类；仅在本次确实新增了标签时携带
    if let Some(linkage) = linkage_opt {
        out["tags_autocreated"] = json!(linkage.autocreated);
        out["tags_reused"] = json!(linkage.reused);
        out["tags_missing_description"] = json!(linkage.missing_description);
    }
    Ok(out)
}

pub fn memory_delete(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let ids = req_id_list(args, "ids", 50)?;
    let (numeric, invalid) = split_ids(&ids);
    let (deleted, missing) = st.delete_memories(&numeric)?;
    let mut out = json!({"deleted": deleted, "missing": missing});
    if !invalid.is_empty() {
        out["invalid_ids"] = json!(invalid);
    }
    attach_id_note(&mut out, !missing.is_empty(), &invalid);
    Ok(out)
}

/// 语义趟：查询向量化 + 与库存向量的余弦排名，与关键词趟 RRF 融合。
/// 任何一步失败（读向量、embedding 服务超时/报错）都回退纯关键词趟，
/// 返回 `(hits, 是否真正走了混合)`——回退是正常路径而非错误。
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

/// 取一条记忆的摘要视图（确保存在，不存在时给统一错误）。
fn memory_view(st: &Store, id: i64) -> Result<Value, ToolError> {
    let (mut found, _) = st.get_memories(&[id])?;
    found
        .pop()
        .map(|m| m.summary_view())
        .ok_or_else(|| ToolError::not_found(format!("memory '{}' not found", Store::format_id(id))))
}
