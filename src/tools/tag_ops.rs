//! Tag and scope management: create/read/update/delete (data operations are delegated to the
//! store's SQL). A tag name is bare (`rust`) or scoped (`system::pc1sw0`); the scope is an entity
//! of its own, created explicitly by `scope_set` — tagging never invents one.

use crate::model::{normalize_ident_word, normalize_tag_name};
use crate::store::Store;
use crate::tools::params::{opt_bool, opt_regex, opt_str, req_str, validate_description};
use crate::tools::ToolError;
use serde_json::{json, Map, Value};

pub fn tag_create(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = normalize_tag_name(&req_str(args, "name")?)?;
    // The shape is validated above; a scoped name additionally needs its scope to exist, which the
    // store refuses by name (a bare name never consults the scopes: the namespaces are separate).
    let description = match opt_str(args, "description")? {
        Some(d) => validate_description(&d)?,
        None => String::new(),
    };
    st.tag_create(&name, &description)?;
    // Non-blocking hint: when a tag differing only in case already exists, remind the agent to keep the taxonomy
    // from fragmenting (e.g. "rust" and "Rust" coexisting). Whether to merge is the agent's call, via tag_update.
    // Echo-free: name and description were just stated by the caller, so a clean create answers
    // with an empty object — only the case-variant hint carries news.
    let mut out = json!({});
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
    // Two namespaces side by side: tags in browse order (count desc, name asc, as the store returns
    // them) and scopes in the store's own order. No role split survives in the new model, so there
    // is no `all` switch and no total_* echoes (the arrays already say how many there are).
    let mut tags = st.tag_views()?;
    let mut scopes = st.scope_views()?;
    // The filter narrows both lists by name, case-insensitively (like every other matcher).
    if let Some(re) = &filter {
        tags.retain(|t| re.is_match(t["name"].as_str().unwrap_or_default()));
        scopes.retain(|s| re.is_match(s["name"].as_str().unwrap_or_default()));
    }
    Ok(json!({ "tags": tags, "scopes": scopes }))
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
    // The requirement is a property of the scope, not of the tag, so a rename keeps it in place
    // whatever scope the tag moves to or from.
    let (renamed, description_updated) =
        st.tag_update(&name, new_name.as_deref(), description.as_deref())?;
    // Echo-free: the final name and description are the caller's own inputs (name / new_name /
    // description) — only the which-parts-actually-changed flags are news
    Ok(json!({
        "renamed": renamed,
        "description_updated": description_updated,
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
    // Echo-free throughout: the tag name and mode are the caller's own arguments.
    if dry_run {
        st.assert_tag_deletable(&name)?;
        let ids = st.tag_memory_ids(&name)?;
        let mut out = json!({ "memories_affected": ids.len() });
        if mode == "purge" {
            out["memory_ids"] = json!(ids);
        }
        return Ok(out);
    }
    match mode {
        "detach" => {
            let memories_updated = st.tag_delete_detach(&name)?;
            Ok(json!({ "memories_updated": memories_updated }))
        }
        _ => {
            let memories_deleted = st.tag_delete_purge(&name)?;
            Ok(json!({ "memories_deleted": memories_deleted }))
        }
    }
}

/// Create a scope, or update the existing one's description and its "required on every write"
/// contract. A scope name is one identifier word, so `::` cannot appear in it (that separator
/// belongs to the tag names pointing into the scope).
pub fn scope_set(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let name = normalize_ident_word(&req_str(args, "name")?, "scope name")?;
    let description = match opt_str(args, "description")? {
        Some(d) => Some(validate_description(&d)?),
        None => None,
    };
    let required = opt_bool(args, "required")?;
    if !st.scope_exists(&name)? {
        st.scope_create(&name, description.as_deref().unwrap_or_default())?;
        // A fresh scope starts out not required; a requested requirement takes effect at once, so
        // the flag is never silently dropped. Echo-free: the create is the news.
        if let Some(flag) = required {
            st.scope_update(&name, None, Some(flag))?;
        }
        return Ok(json!({ "created": true }));
    }
    // Never an upsert: the scope exists, so this reports the halves that were written. The
    // requirement starts at this moment, leaving older memories as unclassified stock.
    let (description_updated, required_updated) =
        st.scope_update(&name, description.as_deref(), required)?;
    Ok(json!({
        "description_updated": description_updated,
        "required_updated": required_updated,
    }))
}
