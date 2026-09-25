//! MCP 工具层：参数解析、工具定义与业务处理的入口。
//!
//! 模块划分：
//! - `defs`    工具清单与 JSON Schema（对 agent 暴露的契约，参数校验的唯一权威来源）
//! - `params`  参数解析与校验辅助
//! - `tag_ops` 标签增删查改
//! - `memory_ops` 记忆增删改查、浏览与搜索
//!
//! 数据访问约定：`execute_with_db` 在单个 SQLite 事务内"打开 → 执行 → 提交"，
//! panic 或错误时整个事务回滚，多进程并发由 SQLite WAL + busy_timeout 保证。

mod defs;
mod memory_ops;
mod params;
mod tag_ops;

use crate::store::{self, Store};
use serde_json::{Map, Value};
use std::path::Path;

pub use defs::{tool_definitions, TOOL_NAMES};

pub const INSTRUCTIONS: &str = "Persistent long-term memory store. Each memory has: tags (a taxonomy YOU curate), a one-line summary, and full content. Progressive disclosure: memory_search / memory_list return only ids, tags and summaries; call memory_get on just the ids worth reading to reveal full content. Save durable knowledge (decisions, facts, preferences, project context) with memory_create; write precise, self-contained summaries so future scans stay cheap; prefer memory_update over re-storing near-duplicates; keep tags tidy with the tag_* tools.";

/// 工具层错误：按类别而非文本分类，REST 层据此映射 HTTP 状态码
/// （NotFound → 404，Invalid → 400），MCP 层一律以 isError 结果回显消息。
#[derive(Debug, Clone)]
pub enum ToolError {
    /// 资源不存在（REST → 404）
    NotFound(String),
    /// 参数校验失败或业务冲突（REST → 400）
    Invalid(String),
}

impl ToolError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        ToolError::NotFound(msg.into())
    }

    pub fn invalid(msg: impl Into<String>) -> Self {
        ToolError::Invalid(msg.into())
    }

    /// 面向 agent / 管理界面的可读消息。
    pub fn message(&self) -> &str {
        match self {
            ToolError::NotFound(m) | ToolError::Invalid(m) => m,
        }
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl From<String> for ToolError {
    /// 数据层/参数层的 String 错误默认归入 Invalid；
    /// "不存在"类别由 handler 显式构造 NotFound。
    fn from(message: String) -> Self {
        ToolError::Invalid(message)
    }
}

/// 在单个事务内完成"打开数据库 → 执行 → 提交"。
/// 事务模式由工具契约决定：readOnlyHint 为真的工具走 DEFERRED 快照，
/// 其余走 IMMEDIATE 写锁（见 store::TxMode）。
pub fn execute_with_db(db_path: &Path, name: &str, args: &Value) -> Result<Value, ToolError> {
    let mode = if defs::is_read_only(name) {
        store::TxMode::ReadOnly
    } else {
        store::TxMode::Write
    };
    store::with_db_in(db_path, mode, |st| execute(st, name, args))
}

pub fn execute(st: &Store, name: &str, args: &Value) -> Result<Value, ToolError> {
    let map = args
        .as_object()
        .ok_or_else(|| ToolError::invalid("arguments must be a JSON object"))?;
    check_known_args(name, map)?;

    match name {
        "tag_create" => tag_ops::tag_create(st, map),
        "tag_list" => tag_ops::tag_list(st),
        "tag_rename" => tag_ops::tag_rename(st, map),
        "tag_delete" => tag_ops::tag_delete(st, map),
        "memory_create" => memory_ops::memory_create(st, map),
        "memory_list" => memory_ops::memory_list(st, map),
        "memory_search" => memory_ops::memory_search(st, map),
        "memory_get" => memory_ops::memory_get(st, map),
        "memory_update" => memory_ops::memory_update(st, map),
        "memory_delete" => memory_ops::memory_delete(st, map),
        _ => Err(ToolError::invalid(format!("unknown tool '{name}'"))),
    }
}

/// 拒绝未知参数：合法参数名从 inputSchema.properties 派生，与对 agent 声明的契约天然同步。
/// 这能尽早暴露调用方的拼写错误，否则参数会被静默忽略，引发更难排查的行为。
fn check_known_args(name: &str, args: &Map<String, Value>) -> Result<(), String> {
    if !TOOL_NAMES.contains(&name) {
        return Ok(()); // 未知工具的错误信息在下面的 match 兜底分支里给出
    }
    let known = defs::known_args(name).expect("every listed tool has a definition");
    for key in args.keys() {
        if !known.iter().any(|k| k == key) {
            let valid = if known.is_empty() {
                "(none)".to_string()
            } else {
                known.join(", ")
            };
            return Err(format!(
                "unknown argument '{key}' for tool '{name}'; valid arguments: {valid}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEQ: AtomicU32 = AtomicU32::new(0);

    fn temp_db(tag: &str) -> PathBuf {
        let n = SEQ.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!(
            "agent-memory-tools-{}-{}-{}.db",
            std::process::id(),
            tag,
            n
        ))
    }

    fn call(path: &Path, name: &str, args: Value) -> Result<Value, ToolError> {
        execute_with_db(path, name, &args)
    }

    fn cleanup(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(PathBuf::from(format!("{}{}", path.display(), suffix)));
        }
    }

    #[test]
    fn full_memory_lifecycle() {
        let path = temp_db("lifecycle");
        let created = call(
            &path,
            "memory_create",
            json!({
                "summary": "Rust borrow checker notes",
                "content": "The borrow checker forbids simultaneous aliasing and mutation.",
                "tags": ["rust", "notes"]
            }),
        )
        .unwrap();
        let id = created["memory"]["id"].as_str().unwrap().to_string();
        assert_eq!(created["tags_autocreated"].as_array().unwrap().len(), 2);

        // 搜索命中，且不泄露正文（渐进式披露第一层）
        let found = call(&path, "memory_search", json!({"query": "borrow"})).unwrap();
        let results = found["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["id"], id.as_str());
        assert!(results[0].get("content").is_none());
        assert!(results[0]["snippet"].as_str().unwrap().contains("borrow"));

        // 列表只有摘要
        let listing = call(&path, "memory_list", json!({})).unwrap();
        let items = listing["memories"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert!(items[0].get("content").is_none());

        // 取全文（第二层）
        let got = call(&path, "memory_get", json!({"ids": [id]})).unwrap();
        assert_eq!(
            got["memories"][0]["content"],
            "The borrow checker forbids simultaneous aliasing and mutation."
        );

        // 更新：增量增删标签
        let upd = call(
            &path,
            "memory_update",
            json!({"id": id, "add_tags": ["study"], "remove_tags": ["notes"]}),
        )
        .unwrap();
        assert_eq!(upd["memory"]["tags"].as_array().unwrap().len(), 2);

        // 删除
        let del = call(&path, "memory_delete", json!({"ids": [id]})).unwrap();
        assert_eq!(del["deleted"].as_array().unwrap().len(), 1);
        let after = call(&path, "memory_list", json!({})).unwrap();
        assert_eq!(after["total"], 0);

        cleanup(&path);
    }

    #[test]
    fn tag_management_flows() {
        let path = temp_db("tags");
        call(
            &path,
            "tag_create",
            json!({"name": "rust", "description": "Rust language"}),
        )
        .unwrap();
        assert!(call(&path, "tag_create", json!({"name": "rust"})).is_err());

        call(
            &path,
            "memory_create",
            json!({
                "summary": "s", "content": "c", "tags": ["rust"]
            }),
        )
        .unwrap();

        // 重命名会同步所有记忆
        call(
            &path,
            "tag_rename",
            json!({"old_name": "rust", "new_name": "lang", "description": "Programming languages"}),
        )
        .unwrap();
        let tl = call(&path, "tag_list", json!({})).unwrap();
        let tags = tl["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0]["name"], "lang");
        assert_eq!(tags[0]["memory_count"], 1);
        assert_eq!(tags[0]["description"], "Programming languages");

        // detach 只摘标签
        call(
            &path,
            "tag_delete",
            json!({"name": "lang", "mode": "detach"}),
        )
        .unwrap();
        let after = call(&path, "memory_list", json!({})).unwrap();
        assert_eq!(after["total"], 1);
        assert_eq!(after["memories"][0]["tags"].as_array().unwrap().len(), 0);

        cleanup(&path);
    }

    #[test]
    fn purge_deletes_memories() {
        let path = temp_db("purge");
        call(
            &path,
            "memory_create",
            json!({"summary": "a", "content": "ca", "tags": ["x"]}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "b", "content": "cb", "tags": ["x", "keep"]}),
        )
        .unwrap();
        call(
            &path,
            "memory_create",
            json!({"summary": "c", "content": "cc", "tags": ["keep"]}),
        )
        .unwrap();

        let r = call(&path, "tag_delete", json!({"name": "x", "mode": "purge"})).unwrap();
        assert_eq!(r["memories_deleted"].as_array().unwrap().len(), 2);
        let after = call(&path, "memory_list", json!({})).unwrap();
        assert_eq!(after["total"], 1);

        cleanup(&path);
    }

    #[test]
    fn validation_and_missing_errors() {
        let path = temp_db("errors");
        // 必填参数缺失
        assert!(call(&path, "memory_create", json!({"summary": "s"})).is_err());
        // 空摘要
        assert!(call(
            &path,
            "memory_create",
            json!({"summary": "   ", "content": "c"})
        )
        .is_err());
        // 不存在的记忆
        assert!(call(&path, "memory_update", json!({"id": "m99", "summary": "x"})).is_err());
        // 不存在的标签
        assert!(call(&path, "tag_delete", json!({"name": "nope"})).is_err());
        // memory_get 对缺失 id 返回 missing 而不是报错，并附引导提示
        let got = call(&path, "memory_get", json!({"ids": ["99"]})).unwrap();
        assert_eq!(got["missing"].as_array().unwrap().len(), 1);
        assert!(got["note"].as_str().unwrap().contains("memory_search"));

        cleanup(&path);
    }

    #[test]
    fn unknown_arguments_are_rejected() {
        let path = temp_db("unknown-args");
        // 拼错的参数名立刻报错并列出合法参数，而不是被静默忽略
        let err = call(
            &path,
            "memory_create",
            json!({"summray": "typo", "content": "c"}),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("'summray'") && err.to_string().contains("summary"),
            "got: {err}"
        );
        // 无参数工具传了参数也要报错
        let err = call(&path, "tag_list", json!({"filter": "x"})).unwrap_err();
        assert!(err.to_string().contains("'filter'"), "got: {err}");
        // 正常参数不受影响
        call(
            &path,
            "memory_create",
            json!({"summary": "ok", "content": "c"}),
        )
        .unwrap();

        cleanup(&path);
    }

    #[test]
    fn search_offset_paginates() {
        let path = temp_db("search-offset");
        for i in 0..4 {
            call(
                &path,
                "memory_create",
                json!({"summary": format!("item {}", i), "content": "shared keyword", "tags": []}),
            )
            .unwrap();
        }
        let first = call(
            &path,
            "memory_search",
            json!({"query": "shared", "limit": 2}),
        )
        .unwrap();
        assert_eq!(first["total_matches"], 4);
        assert_eq!(first["returned"], 2);
        let page2 = call(
            &path,
            "memory_search",
            json!({"query": "shared", "limit": 2, "offset": 2}),
        )
        .unwrap();
        assert_eq!(page2["returned"], 2);
        let ids1: Vec<&str> = first["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        let ids2: Vec<&str> = page2["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert!(
            ids1.iter().all(|id| !ids2.contains(id)),
            "pages must not overlap"
        );

        cleanup(&path);
    }

    #[test]
    fn memory_list_reports_tag_state() {
        let path = temp_db("list-note");
        call(&path, "tag_create", json!({"name": "empty-tag"})).unwrap();
        // 标签不存在
        let missing = call(&path, "memory_list", json!({"tag": "nope"})).unwrap();
        assert!(missing["note"].as_str().unwrap().contains("does not exist"));
        // 标签存在但为空
        let empty = call(&path, "memory_list", json!({"tag": "empty-tag"})).unwrap();
        assert!(empty["note"].as_str().unwrap().contains("no memories"));

        cleanup(&path);
    }

    #[test]
    fn tag_create_warns_on_case_collision() {
        let path = temp_db("case-collision");
        call(&path, "tag_create", json!({"name": "rust"})).unwrap();
        let second = call(&path, "tag_create", json!({"name": "Rust"})).unwrap();
        assert_eq!(second["similar_existing"], "rust");
        assert!(second["note"].as_str().unwrap().contains("tag_rename"));
        // 非阻塞：两个标签都创建成功
        let tl = call(&path, "tag_list", json!({})).unwrap();
        assert_eq!(tl["tags"].as_array().unwrap().len(), 2);

        cleanup(&path);
    }

    #[test]
    fn tag_rename_supports_case_only_rename() {
        let path = temp_db("case-rename");
        call(
            &path,
            "memory_create",
            json!({"summary": "s", "content": "c", "tags": ["rust"]}),
        )
        .unwrap();

        // 仅大小写改名是 similar_existing 提示的指定修复路径，必须可用
        let r = call(
            &path,
            "tag_rename",
            json!({"old_name": "rust", "new_name": "Rust"}),
        )
        .unwrap();
        assert_eq!(r["memories_updated"], 1);
        let got = call(&path, "memory_get", json!({"ids": ["m1"]})).unwrap();
        assert_eq!(got["memories"][0]["tags"][0], "Rust");

        // 改名后再建任何其他大小写拼写的变体：提示都应触发（这正是防碎片化的场景）
        let back = call(&path, "tag_create", json!({"name": "rust"})).unwrap();
        assert_eq!(back["similar_existing"], "Rust");
        let hint = call(&path, "tag_create", json!({"name": "RUST"})).unwrap();
        assert_eq!(hint["similar_existing"], "Rust");
        // 而精确重名仍然报错
        assert!(call(&path, "tag_create", json!({"name": "Rust"})).is_err());

        cleanup(&path);
    }

    #[test]
    fn every_listed_tool_has_a_dispatch_arm() {
        // 同步防线：TOOL_NAMES 与 execute 的 match 分支必须一一对应。
        // 漏掉任何一端时（加工具忘改清单/清单加了没实现），此测试都会失败。
        let path = temp_db("dispatch");
        for name in TOOL_NAMES {
            let r = execute_with_db(&path, name, &json!({}));
            if let Err(e) = r {
                assert!(
                    !e.to_string().contains("unknown tool"),
                    "tool '{name}' is listed but has no dispatch arm: {e}"
                );
            }
        }
        assert!(execute_with_db(&path, "nonexistent", &json!({}))
            .unwrap_err()
            .to_string()
            .contains("unknown tool"));
        cleanup(&path);
    }

    #[test]
    fn duplicate_summary_is_flagged() {
        let path = temp_db("dup");
        call(
            &path,
            "memory_create",
            json!({"summary": "Rust notes", "content": "v1", "tags": []}),
        )
        .unwrap();
        // 归一化（大小写不敏感）后相同摘要 → 提示已有条目
        let second = call(
            &path,
            "memory_create",
            json!({"summary": "rust NOTES", "content": "v2", "tags": []}),
        )
        .unwrap();
        let dups = second["duplicate_of"].as_array().unwrap();
        assert_eq!(dups.len(), 1);
        assert_eq!(dups[0], "m1");
        // 不重复时为空数组
        let third = call(
            &path,
            "memory_create",
            json!({"summary": "totally different", "content": "v3"}),
        )
        .unwrap();
        assert_eq!(third["duplicate_of"].as_array().unwrap().len(), 0);
        cleanup(&path);
    }

    #[test]
    fn persistence_across_calls() {
        let path = temp_db("persist");
        let c = call(
            &path,
            "memory_create",
            json!({"summary": "first", "content": "one", "tags": []}),
        )
        .unwrap();
        let id = c["memory"]["id"].as_str().unwrap().to_string();
        // 每次调用都重新打开数据库，id 计数必须延续
        let c2 = call(
            &path,
            "memory_create",
            json!({"summary": "second", "content": "two", "tags": []}),
        )
        .unwrap();
        assert_ne!(c2["memory"]["id"].as_str().unwrap(), id);
        cleanup(&path);
    }
}
