//! 标签管理：增删查改。

use crate::model::{normalize_tag_name, now, Tag};
use crate::store::Store;
use crate::tools::params::{opt_str, req_str, validate_description};
use serde_json::{json, Map, Value};

pub fn tag_create(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
    let name = normalize_tag_name(&req_str(args, "name")?)?;
    let description = match opt_str(args, "description")? {
        Some(d) => validate_description(&d)?,
        None => String::new(),
    };
    if st.tags.contains_key(&name) {
        return Err(format!(
            "tag '{}' already exists (rename it with tag_rename, or see tag_list)",
            name
        ));
    }
    st.tags.insert(
        name.clone(),
        Tag {
            name: name.clone(),
            description,
            created_at: now(),
        },
    );
    Ok(json!({"created": true, "tag": tag_view(st, &name)}))
}

pub fn tag_list(st: &mut Store) -> Result<Value, String> {
    let mut items: Vec<Value> = st.tags.keys().map(|n| tag_view(st, n)).collect();
    items.sort_by(|a, b| {
        b["memory_count"]
            .as_u64()
            .cmp(&a["memory_count"].as_u64())
            .then(a["name"].as_str().cmp(&b["name"].as_str()))
    });
    Ok(json!({
        "total_tags": items.len(),
        "total_memories": st.memories.len(),
        "tags": items,
    }))
}

pub fn tag_rename(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
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
        return Err("nothing to change: provide new_name and/or description".into());
    }
    if !st.tags.contains_key(&old) {
        return Err(format!("tag '{}' not found (see tag_list)", old));
    }
    let final_name = new_name.clone().unwrap_or_else(|| old.clone());
    if new_name.is_some() && final_name != old && st.tags.contains_key(&final_name) {
        return Err(format!("tag '{}' already exists", final_name));
    }

    let mut tag = st.tags.remove(&old).expect("checked above");
    if let Some(d) = description {
        tag.description = d;
    }
    tag.name = final_name.clone();
    st.tags.insert(final_name.clone(), tag);

    let mut memories_updated = 0u64;
    if final_name != old {
        for m in &mut st.memories {
            let mut changed = false;
            for t in m.tags.iter_mut() {
                if *t == old {
                    *t = final_name.clone();
                    changed = true;
                }
            }
            if changed {
                // 重命名可能让同一记忆出现重复标签，去重但保持顺序。
                dedup_tags(m);
                memories_updated += 1;
            }
        }
    }
    Ok(json!({
        "old_name": old,
        "new_name": final_name,
        "memories_updated": memories_updated,
        "tag": tag_view(st, &final_name),
    }))
}

pub fn tag_delete(st: &mut Store, args: &Map<String, Value>) -> Result<Value, String> {
    let name = normalize_tag_name(&req_str(args, "name")?)?;
    let mode = match opt_str(args, "mode")?.as_deref() {
        None | Some("detach") => "detach",
        Some("purge") => "purge",
        Some(o) => return Err(format!("mode must be 'detach' or 'purge', got '{}'", o)),
    };
    if !st.tags.contains_key(&name) {
        return Err(format!("tag '{}' not found (see tag_list)", name));
    }
    match mode {
        "detach" => {
            let mut memories_updated = 0u64;
            for m in &mut st.memories {
                let before = m.tags.len();
                m.tags.retain(|t| *t != name);
                if m.tags.len() != before {
                    memories_updated += 1;
                }
            }
            st.tags.remove(&name);
            Ok(json!({
                "deleted_tag": name,
                "mode": "detach",
                "memories_updated": memories_updated,
            }))
        }
        _ => {
            let mut memories_deleted: Vec<String> = Vec::new();
            st.memories.retain(|m| {
                if m.tags.contains(&name) {
                    memories_deleted.push(m.id.clone());
                    false
                } else {
                    true
                }
            });
            st.tags.remove(&name);
            Ok(json!({
                "deleted_tag": name,
                "mode": "purge",
                "memories_deleted": memories_deleted,
            }))
        }
    }
}

/// 去除记忆条目中的重复标签，保持首次出现顺序。
pub fn dedup_tags(m: &mut crate::model::Memory) {
    let mut seen: Vec<String> = Vec::new();
    m.tags.retain(|t| {
        if seen.contains(t) {
            false
        } else {
            seen.push(t.clone());
            true
        }
    });
}

/// 统计标签的引用情况：记忆条数与最近使用时间（无引用时为 0）。
fn tag_usage(st: &Store, name: &str) -> (u64, u64) {
    let mut count = 0u64;
    let mut last = 0u64;
    for m in &st.memories {
        if m.tags.iter().any(|t| t == name) {
            count += 1;
            last = last.max(m.updated_at);
        }
    }
    (count, last)
}

fn tag_view(st: &Store, name: &str) -> Value {
    let (count, last) = tag_usage(st, name);
    match st.tags.get(name) {
        Some(t) => json!({
            "name": t.name,
            "description": t.description,
            "memory_count": count,
            "last_used_at": if last > 0 { json!(last) } else { Value::Null },
        }),
        None => Value::Null,
    }
}
