use super::{
    params::{req_str, validate_description},
    ToolError,
};
use crate::{model::normalize_tag_name, store::Store};
use serde_json::{Map, Value};

pub fn set(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    st.predicate_set(
        &normalize_tag_name(&req_str(args, "name")?)?,
        &req_str(args, "predicate")?,
        &validate_description(&req_str(args, "description")?)?,
    )
    .map_err(ToolError::from)
}

pub fn delete(st: &Store, args: &Map<String, Value>) -> Result<Value, ToolError> {
    st.predicate_delete(&normalize_tag_name(&req_str(args, "name")?)?)
        .map_err(ToolError::from)
}
