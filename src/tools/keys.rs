//! Key registry tools: the shared schema contract that agents read and admins reconfigure.

use super::{
    params::{opt_bool, opt_str, req_str},
    ToolError,
};
use crate::{model::MAX_TAG_DESC_CHARS, store::Store};
use serde_json::{json, Map, Value};

pub fn key_set(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let key_type = opt_str(args, "key_type")?.unwrap_or_else(|| "enum".into());
    let multi = opt_bool(args, "multi")?.unwrap_or(false);
    let description = opt_str(args, "description")?.unwrap_or_default();
    if description.chars().count() > MAX_TAG_DESC_CHARS {
        return Err(ToolError::invalid(format!(
            "description exceeds {MAX_TAG_DESC_CHARS} characters"
        )));
    }
    st.key_set(&req_str(args, "name")?, &key_type, multi, &description)
        .map_err(ToolError::from)?;
    // Echo-free: the caller just stated name/type/multi/description, so a clean set answers
    // with an empty object.
    Ok(json!({}))
}

pub fn key_list(st: &Store, _args: &Map<String, Value>) -> Result<Value, ToolError> {
    let mut rows = Vec::new();
    for k in st.key_rows().map_err(ToolError::from)? {
        let mut row = json!({
            "name": k.name,
            "type": k.key_type,
            "multi": k.multi,
            "description": k.description,
        });
        // Values are the 'key:value' tags: the value halves are the known vocabulary (enum
        // words, or numbers in canonical form) an agent can reuse.
        let values = st
            .key_value_tags(&k.name)
            .map_err(ToolError::from)?
            .into_iter()
            .map(|tag| match tag.split_once(':') {
                Some((_, v)) => v.to_string(),
                None => tag,
            })
            .collect::<Vec<_>>();
        if !values.is_empty() {
            row["values"] = json!(values);
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(json!({ "keys": rows }))
}

pub fn key_delete(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    st.key_delete(&req_str(args, "name")?)
        .map_err(ToolError::from)
}
