//! Argument parsing and validation: safely extract values from a JSON object and produce readable, agent-facing errors.
//!
//! Principle: the schema is a strict contract (additionalProperties: false). The server is strict about
//! parameter names (a misspelled name errors out immediately so the agent never believes it took effect),
//! and lenient about value forms (a single string is automatically treated as a one-element array; ids are the
//! exception — the format is strictly "m{n}" and bare numbers without the m prefix are rejected).

use crate::model::{normalize_tag_name, MAX_CONTENT_CHARS, MAX_SUMMARY_CHARS, MAX_TAG_DESC_CHARS};
use serde_json::{Map, Value};

pub fn req_str(args: &Map<String, Value>, key: &str) -> Result<String, String> {
    match args.get(key) {
        None | Some(Value::Null) => Err(format!("missing required parameter '{key}'")),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => Err(format!("parameter '{key}' must be a string")),
    }
}

pub fn opt_str(args: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(format!("parameter '{key}' must be a string")),
    }
}

/// Optional regex argument: take the string and compile it (invalid pattern → readable error, mapped to 400 by the handler).
/// Matching is case-insensitive, mirroring content search's case folding — a taxonomy that sprouted
/// case variants (the exact problem similar_existing warns about) must not also split the filters.
pub fn opt_regex(args: &Map<String, Value>, key: &str) -> Result<Option<regex::Regex>, String> {
    match opt_str(args, key)? {
        None => Ok(None),
        Some(pattern) => regex::RegexBuilder::new(&pattern)
            .case_insensitive(true)
            .build()
            .map(Some)
            .map_err(|e| format!("parameter '{key}' is not a valid regular expression: {e}")),
    }
}

/// String array argument; tolerates the single-string form (wrapped into a one-element array automatically).
pub fn opt_str_list(args: &Map<String, Value>, key: &str) -> Result<Option<Vec<String>>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(vec![s.clone()])),
        Some(Value::Array(items)) => {
            let mut v = Vec::with_capacity(items.len());
            for it in items {
                match it {
                    Value::String(s) => v.push(s.clone()),
                    _ => return Err(format!("parameter '{key}' must be an array of strings")),
                }
            }
            Ok(Some(v))
        }
        Some(_) => Err(format!("parameter '{key}' must be an array of strings")),
    }
}

/// Required id list (1..=max items); tolerates the single-string form.
pub fn req_id_list(
    args: &Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<Vec<String>, String> {
    match opt_str_list(args, key)? {
        None => Err(format!("missing required parameter '{key}'")),
        Some(v) if v.is_empty() => Err(format!("parameter '{key}' must not be empty")),
        Some(v) if v.len() > max => Err(format!("parameter '{key}' accepts at most {max} ids")),
        Some(v) => Ok(v),
    }
}

pub fn opt_u64(args: &Map<String, Value>, key: &str) -> Result<Option<u64>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) if n.is_u64() => Ok(Some(n.as_u64().unwrap())),
        Some(_) => Err(format!("parameter '{key}' must be a non-negative integer")),
    }
}

pub fn opt_bool(args: &Map<String, Value>, key: &str) -> Result<Option<bool>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(format!("parameter '{key}' must be a boolean")),
    }
}

pub fn validate_summary(s: &str) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("summary must not be empty".into());
    }
    if t.chars().count() > MAX_SUMMARY_CHARS {
        return Err(format!(
            "summary is too long (max {MAX_SUMMARY_CHARS} characters)"
        ));
    }
    Ok(t.to_string())
}

/// Content may be empty: a summary-only memory stores an empty body (the handlers normalize
/// whitespace-only input to ""). Only the size cap is enforced here.
pub fn validate_content(s: &str) -> Result<String, String> {
    if s.chars().count() > MAX_CONTENT_CHARS {
        return Err(format!(
            "content is too long (max {MAX_CONTENT_CHARS} characters)"
        ));
    }
    Ok(s.to_string())
}

pub fn validate_description(s: &str) -> Result<String, String> {
    if s.chars().count() > MAX_TAG_DESC_CHARS {
        return Err(format!(
            "description is too long (max {MAX_TAG_DESC_CHARS} characters)"
        ));
    }
    Ok(s.to_string())
}

/// Normalize a set of tag names: trim, cap length, deduplicate (preserving first-occurrence order).
pub fn normalize_tag_list(raw: &[String]) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::with_capacity(raw.len());
    for t in raw {
        let n = normalize_tag_name(t)?;
        if !out.contains(&n) {
            out.push(n);
        }
    }
    Ok(out)
}
