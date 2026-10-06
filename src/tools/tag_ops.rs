//! Tag management: create/read/update/delete (data operations are delegated to the store's SQL).

use crate::model::normalize_tag_name;
use crate::store::Store;
use crate::tools::params::{opt_bool, opt_regex, opt_str, req_str, validate_description};
use crate::tools::ToolError;
use serde_json::{json, Map, Value};

pub fn tag_create(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = normalize_tag_name(&req_str(args, "name")?)?;
    let description = match opt_str(args, "description")? {
        Some(d) => validate_description(&d)?,
        None => String::new(),
    };
    st.tag_create(&name, &description)?;
    // Non-blocking hint: when a tag differing only in case already exists, remind the agent to keep the taxonomy
    // from fragmenting (e.g. "rust" and "Rust" coexisting). Whether to merge is the agent's call, via tag_update.
    // No "created": true echo — a non-error result already means success.
    let mut out = json!({"tag": st.tag_view(&name)?});
    if let Some(existing) = st.find_tag_case_insensitive(&name)? {
        out["similar_existing"] = json!(existing);
        out["note"] = json!(format!(
            "tag '{}' also exists (differs only by case); consider tag_update to merge them and keep the taxonomy tidy",
            existing
        ));
    }
    Ok(out)
}

pub fn tag_list(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let filter = opt_regex(args, "filter").map_err(ToolError::invalid)?;
    let mut tags = st.tag_views()?;
    if let Some(re) = &filter {
        tags.retain(|t| re.is_match(t["name"].as_str().unwrap_or_default()));
    }
    // The reserved tag always has a row (seeded at open), so it flows through like any other
    // name — the filter applies to it unchanged. Restore the documented browse order
    // (count desc, name asc).
    fn sort_key(v: &Value) -> (u64, &str) {
        (
            v["count"].as_u64().unwrap_or(0),
            v["name"].as_str().unwrap_or_default(),
        )
    }
    tags.sort_by(|a, b| {
        let (count_a, name_a) = sort_key(a);
        let (count_b, name_b) = sort_key(b);
        count_b.cmp(&count_a).then(name_a.cmp(name_b))
    });
    // No total_* echoes: total_tags duplicates the array length and the store-wide memory count
    // is the stats tool's job, not something a tag browser acts on.
    Ok(json!({ "tags": tags }))
}

pub fn tag_update(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = normalize_tag_name(&req_str(args, "name")?)?;
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
    // Existence pre-check: the error kind (404/400) is decided explicitly at the handler layer
    if !st.tag_exists(&name)? {
        return Err(ToolError::not_found(format!(
            "tag '{name}' not found (see tag_list)"
        )));
    }
    if let Some(new) = &new_name {
        if new != &name && st.tag_exists(new)? {
            return Err(ToolError::invalid(format!("tag '{new}' already exists")));
        }
    }
    let (renamed, description_updated) =
        st.tag_update(&name, new_name.as_deref(), description.as_deref())?;
    let final_name = new_name.unwrap_or_else(|| name.clone());
    Ok(json!({
        "name": final_name,
        "renamed": renamed,
        "description_updated": description_updated,
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
                "mode must be 'detach' or 'purge', got '{o}'"
            )))
        }
    };
    let dry_run = opt_bool(args, "dry_run")?.unwrap_or(false);
    if !st.tag_exists(&name)? {
        return Err(ToolError::not_found(format!(
            "tag '{name}' not found (see tag_list)"
        )));
    }
    // Preview before impact: purge deletes irreversibly, so the caller can price it first
    // (count for both modes, plus the exact ids that would die under purge).
    if dry_run {
        let ids = st.tag_memory_ids(&name)?;
        return Ok(json!({
            "dry_run": true,
            "name": name,
            "mode": mode,
            "memories_affected": ids.len(),
            "memory_ids": if mode == "purge" { json!(ids) } else { Value::Null },
        }));
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
