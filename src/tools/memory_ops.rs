//! 记忆管理：增删改查、浏览与搜索（渐进式披露的载体）。

use crate::model::{normalize_id, normalize_tag_name, now, Memory, Tag};
use crate::search;
use crate::store::Store;
use crate::tools::params::{
    normalize_tag_list, opt_str, opt_str_list, opt_u64, req_id_list, req_str, validate_content,
    validate_summary,
};
use crate::tools::tag_ops::dedup_tags;
use serde_json::{json, Map, Value};

pub fn memory_create(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
    let summary = validate_summary(&req_str(args, "summary")?)?;
    let content = validate_content(&req_str(args, "content")?)?;
    let raw_tags = opt_str_list(args, "tags")?.unwrap_or_default();
    let tags = normalize_tag_list(&raw_tags)?;

    let mut autocreated = Vec::new();
    ensure_tags_exist(st, &tags, &mut autocreated);

    // 重复检测（确定性规则：摘要归一化后完全相等）。在插入前计算，避免匹配到自己。
    // 目的不是阻止存储，而是提醒 agent：已有同摘要记忆时应改用 memory_update。
    let summary_norm = summary.to_lowercase();
    let duplicate_of: Vec<String> = st
        .memories
        .iter()
        .filter(|m| m.summary.to_lowercase() == summary_norm)
        .map(|m| m.id.clone())
        .collect();

    let t = now();
    let m = Memory {
        id: st.new_id(),
        summary,
        content,
        tags,
        created_at: t,
        updated_at: t,
    };
    let view = m.summary_view();
    st.memories.push(m);
    Ok(json!({"memory": view, "tags_autocreated": autocreated, "duplicate_of": duplicate_of}))
}

pub fn memory_list(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
    let tag = match opt_str(args, "tag")? {
        Some(t) => Some(normalize_tag_name(&t)?),
        None => None,
    };
    let sort_opt = opt_str(args, "sort")?;
    let sort = match sort_opt.as_deref() {
        None => "updated_at",
        Some(s @ ("updated_at" | "created_at")) => s,
        Some(o) => {
            return Err(format!(
                "sort must be 'updated_at' or 'created_at', got '{}'",
                o
            ))
        }
    };
    let asc = match opt_str(args, "order")?.as_deref() {
        None | Some("desc") => false,
        Some("asc") => true,
        Some(o) => return Err(format!("order must be 'asc' or 'desc', got '{}'", o)),
    };
    let offset = opt_u64(args, "offset")?.unwrap_or(0);
    let limit = opt_u64(args, "limit")?.unwrap_or(20).clamp(1, 200);

    let mut items: Vec<&Memory> = st
        .memories
        .iter()
        .filter(|m| tag.as_ref().is_none_or(|t| m.tags.contains(t)))
        .collect();
    match (sort, asc) {
        ("updated_at", false) => {
            items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(b.id.cmp(&a.id)))
        }
        ("updated_at", true) => {
            items.sort_by(|a, b| a.updated_at.cmp(&b.updated_at).then(a.id.cmp(&b.id)))
        }
        ("created_at", false) => {
            items.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)))
        }
        ("created_at", true) => {
            items.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)))
        }
        _ => unreachable!(),
    }

    let total = items.len() as u64;
    let page: Vec<Value> = items
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|m| m.summary_view())
        .collect();

    let mut out = json!({"total": total, "offset": offset, "limit": limit, "memories": page});
    if let Some(t) = &tag {
        if !st.tags.contains_key(t) {
            out["note"] = json!(format!("tag '{}' does not exist yet; see tag_list", t));
        } else if total == 0 {
            out["note"] = json!(format!("tag '{}' exists but currently has no memories", t));
        }
    }
    Ok(out)
}

pub fn memory_search(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
    let query = req_str(args, "query")?;
    if query.trim().is_empty() {
        return Err("query must not be empty".into());
    }
    let raw_tags = opt_str_list(args, "tags")?.unwrap_or_default();
    let tag_filter = normalize_tag_list(&raw_tags)?;
    let limit = opt_u64(args, "limit")?.unwrap_or(10).clamp(1, 50);
    let offset = opt_u64(args, "offset")?.unwrap_or(0);

    let hits = search::run(&st.memories, &query, &tag_filter);
    let total = hits.len() as u64;
    let results: Vec<Value> = hits
        .iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|h| {
            let m = &st.memories[h.idx];
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

    Ok(json!({
        "query": query,
        "total_matches": total,
        "offset": offset,
        "returned": results.len(),
        "hint": "Summaries + snippets only (progressive disclosure). Call memory_get with the ids worth reading to reveal full content.",
        "results": results,
    }))
}

pub fn memory_get(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
    let ids = req_id_list(args, "ids", 50)?;
    let mut found = Vec::new();
    let mut missing = Vec::new();
    for raw in ids {
        let id = normalize_id(&raw);
        match st.memories.iter().find(|m| m.id == id) {
            Some(m) => found.push(m.full_view()),
            None => missing.push(id),
        }
    }
    let mut out = json!({"memories": found, "missing": missing});
    if !missing.is_empty() {
        out["note"] = json!(
            "some ids were not found; use memory_list or memory_search to discover valid ids"
        );
    }
    Ok(out)
}

pub fn memory_update(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
    let id = normalize_id(&req_str(args, "id")?);
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
        return Err(
            "nothing to update: provide summary, content, add_tags and/or remove_tags".into(),
        );
    }

    // 先确保新标签存在（改的是 tags 表），再取记忆条目的可变借用，避免双重可变借用。
    let mut autocreated = Vec::new();
    if let Some(add) = &add_tags {
        ensure_tags_exist(st, add, &mut autocreated);
    }

    let m = st.memories.iter_mut().find(|m| m.id == id).ok_or_else(|| {
        format!(
            "memory '{}' not found (use memory_list or memory_search first)",
            id
        )
    })?;

    let mut changed = false;
    if let Some(s) = summary {
        m.summary = s;
        changed = true;
    }
    if let Some(c) = content {
        m.content = c;
        changed = true;
    }
    if let Some(add) = &add_tags {
        for t in add {
            if !m.tags.contains(t) {
                m.tags.push(t.clone());
                changed = true;
            }
        }
    }
    if let Some(rem) = &remove_tags {
        let before = m.tags.len();
        m.tags.retain(|t| !rem.contains(t));
        if m.tags.len() != before {
            changed = true;
        }
    }
    if changed {
        m.updated_at = now();
    }
    dedup_tags(m);

    let mut out = json!({"updated": changed, "memory": m.summary_view()});
    if !autocreated.is_empty() {
        out["tags_autocreated"] = json!(autocreated);
    }
    Ok(out)
}

pub fn memory_delete(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
    let ids = req_id_list(args, "ids", 50)?;
    let targets: Vec<String> = ids.iter().map(|s| normalize_id(s)).collect();
    let mut deleted = Vec::new();
    let mut missing = Vec::new();
    for id in &targets {
        if st.memories.iter().any(|m| &m.id == id) {
            deleted.push(id.clone());
        } else {
            missing.push(id.clone());
        }
    }
    if !deleted.is_empty() {
        st.memories.retain(|m| !deleted.contains(&m.id));
    }
    Ok(json!({"deleted": deleted, "missing": missing}))
}

/// 确保标签表中存在这些标签（自动创建，空描述），并汇报哪些是新建的。
fn ensure_tags_exist(st: &mut Store, names: &[String], autocreated: &mut Vec<String>) {
    for n in names {
        if !st.tags.contains_key(n) {
            st.tags.insert(n.clone(), Tag::new(n.clone()));
            autocreated.push(n.clone());
        }
    }
}
