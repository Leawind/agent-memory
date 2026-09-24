//! 参数解析与校验：从 JSON 对象中安全取值，输出面向 agent 的可读错误。
//!
//! 原则：schema 是严格契约（additionalProperties: false），服务器在参数名上
//! 从严（拼错立刻报错，避免 agent 以为参数生效了）；在值的写法上从宽
//! （单个字符串自动当作单元素数组，id 带不带 m 前缀都接受）。

use crate::model::{normalize_tag_name, MAX_CONTENT_CHARS, MAX_SUMMARY_CHARS, MAX_TAG_DESC_CHARS};
use serde_json::{Map, Value};

pub fn req_str(args: &Map<String, Value>, key: &str) -> Result<String, String> {
    match args.get(key) {
        None | Some(Value::Null) => Err(format!("missing required parameter '{}'", key)),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => Err(format!("parameter '{}' must be a string", key)),
    }
}

pub fn opt_str(args: &Map<String, Value>, key: &str) -> Result<Option<String>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(format!("parameter '{}' must be a string", key)),
    }
}

/// 字符串数组参数；容忍单个字符串写法（自动包装为单元素数组）。
pub fn opt_str_list(args: &Map<String, Value>, key: &str) -> Result<Option<Vec<String>>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(vec![s.clone()])),
        Some(Value::Array(items)) => {
            let mut v = Vec::with_capacity(items.len());
            for it in items {
                match it {
                    Value::String(s) => v.push(s.clone()),
                    _ => return Err(format!("parameter '{}' must be an array of strings", key)),
                }
            }
            Ok(Some(v))
        }
        Some(_) => Err(format!("parameter '{}' must be an array of strings", key)),
    }
}

/// 必填的 id 列表（1..=max 条）；容忍单个字符串写法。
pub fn req_id_list(
    args: &Map<String, Value>,
    key: &str,
    max: usize,
) -> Result<Vec<String>, String> {
    match opt_str_list(args, key)? {
        None => Err(format!("missing required parameter '{}'", key)),
        Some(v) if v.is_empty() => Err(format!("parameter '{}' must not be empty", key)),
        Some(v) if v.len() > max => Err(format!("parameter '{}' accepts at most {} ids", key, max)),
        Some(v) => Ok(v),
    }
}

pub fn opt_u64(args: &Map<String, Value>, key: &str) -> Result<Option<u64>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) if n.is_u64() => Ok(Some(n.as_u64().unwrap())),
        Some(_) => Err(format!(
            "parameter '{}' must be a non-negative integer",
            key
        )),
    }
}

pub fn validate_summary(s: &str) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err("summary must not be empty".into());
    }
    if t.chars().count() > MAX_SUMMARY_CHARS {
        return Err(format!(
            "summary is too long (max {} characters)",
            MAX_SUMMARY_CHARS
        ));
    }
    Ok(t.to_string())
}

pub fn validate_content(s: &str) -> Result<String, String> {
    if s.trim().is_empty() {
        return Err("content must not be empty".into());
    }
    if s.chars().count() > MAX_CONTENT_CHARS {
        return Err(format!(
            "content is too long (max {} characters)",
            MAX_CONTENT_CHARS
        ));
    }
    Ok(s.to_string())
}

pub fn validate_description(s: &str) -> Result<String, String> {
    if s.chars().count() > MAX_TAG_DESC_CHARS {
        return Err(format!(
            "description is too long (max {} characters)",
            MAX_TAG_DESC_CHARS
        ));
    }
    Ok(s.to_string())
}

/// 归一化一组标签名：trim、限长、去重（保持首次出现顺序）。
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
