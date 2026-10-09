use super::{
    params::{opt_str, req_str, validate_description},
    ToolError,
};
use crate::{auth::IdentityCtx, model::normalize_tag_name, store::Store};
use serde_json::{Map, Value};

pub fn scope(args: &Map<String, Value>) -> Result<String, ToolError> {
    let scope = opt_str(args, "scope")?.unwrap_or_else(|| "user".into());
    if scope != "user" && scope != "global" {
        return Err(ToolError::invalid("scope must be user or global"));
    }
    Ok(scope)
}

pub fn set(st: &Store, ctx: &IdentityCtx, args: &Map<String, Value>) -> Result<Value, ToolError> {
    let scope = scope(args)?;
    let name = normalize_tag_name(&req_str(args, "name")?)?;
    let description = validate_description(&req_str(args, "description")?)?;
    st.predicate_set(
        ctx,
        &scope,
        &name,
        &req_str(args, "predicate")?,
        &description,
    )
    .map_err(ToolError::from)
}

pub fn delete(
    st: &Store,
    ctx: &IdentityCtx,
    args: &Map<String, Value>,
) -> Result<Value, ToolError> {
    st.predicate_delete(
        ctx,
        &scope(args)?,
        &normalize_tag_name(&req_str(args, "name")?)?,
    )
    .map_err(ToolError::from)
}
