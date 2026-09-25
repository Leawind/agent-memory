//! 标签管理：增删查改（数据操作下沉到 store 的 SQL）。

use crate::model::normalize_tag_name;
use crate::store::Store;
use crate::tools::params::{opt_str, req_str, validate_description};
use crate::tools::ToolError;
use serde_json::{json, Map, Value};

pub fn tag_create(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = normalize_tag_name(&req_str(args, "name")?)?;
    let description = match opt_str(args, "description")? {
        Some(d) => validate_description(&d)?,
        None => String::new(),
    };
    st.tag_create(&name, &description)?;
    // 非阻塞提示：已存在仅大小写不同的标签时提醒 agent，避免分类体系碎片化
    // （如 "rust" 与 "Rust" 并存）。是否合并由 agent 通过 tag_rename 自行决定。
    let mut out = json!({"created": true, "tag": st.tag_view(&name)?});
    if let Some(existing) = st.find_tag_case_insensitive(&name)? {
        out["similar_existing"] = json!(existing);
        out["note"] = json!(format!(
            "tag '{}' also exists (differs only by case); consider tag_rename to merge them and keep the taxonomy tidy",
            existing
        ));
    }
    Ok(out)
}

pub fn tag_list(st: &Store) -> Result<Value, ToolError> {
    let tags = st.tag_views()?;
    let total_memories = st.stats()?["memories"].clone();
    Ok(json!({
        "total_tags": tags.len(),
        "total_memories": total_memories,
        "tags": tags,
    }))
}

pub fn tag_rename(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let old = normalize_tag_name(&req_str(args, "old_name")?)?;
    let new_name = match opt_str(args, "new_name")? {
        Some(n) => Some(normalize_tag_name(&n)?),
        None => None,
    };
    let description = match opt_str(args, "description")? {
        Some(d) => Some(validate_description(&d)?),
        None => None,
    };
    if new_name.is_none() && description.is_none() {
        return Err(ToolError::invalid(
            "nothing to change: provide new_name and/or description",
        ));
    }
    // 存在性预检：错误类别（404/400）在 handler 层显式确定
    if !st.tag_exists(&old)? {
        return Err(ToolError::not_found(format!(
            "tag '{}' not found (see tag_list)",
            old
        )));
    }
    if let Some(new) = &new_name {
        if new != &old && st.tag_exists(new)? {
            return Err(ToolError::invalid(format!("tag '{}' already exists", new)));
        }
    }
    let memories_updated = st.tag_rename(&old, new_name.as_deref(), description.as_deref())?;
    let final_name = new_name.unwrap_or_else(|| old.clone());
    Ok(json!({
        "old_name": old,
        "new_name": final_name,
        "memories_updated": memories_updated,
        "tag": st.tag_view(&final_name)?,
    }))
}

pub fn tag_delete(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = normalize_tag_name(&req_str(args, "name")?)?;
    let mode = match opt_str(args, "mode")?.as_deref() {
        None | Some("detach") => "detach",
        Some("purge") => "purge",
        Some(o) => {
            return Err(ToolError::invalid(format!(
                "mode must be 'detach' or 'purge', got '{}'",
                o
            )))
        }
    };
    if !st.tag_exists(&name)? {
        return Err(ToolError::not_found(format!(
            "tag '{}' not found (see tag_list)",
            name
        )));
    }
    match mode {
        "detach" => {
            let memories_updated = st.tag_delete_detach(&name)?;
            Ok(json!({
                "deleted_tag": name,
                "mode": "detach",
                "memories_updated": memories_updated,
            }))
        }
        _ => {
            let memories_deleted = st.tag_delete_purge(&name)?;
            Ok(json!({
                "deleted_tag": name,
                "mode": "purge",
                "memories_deleted": memories_deleted,
            }))
        }
    }
}
