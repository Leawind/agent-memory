//! 聚合查询与运维入口：体检（hygiene）、统计（stats）、导出导入。

use crate::sql;
use serde_json::{json, Map, Value};
use std::collections::{HashMap, HashSet};

use super::Store;

impl Store {
    /// 体检：报告数据中的隐患（只读）。覆盖外键被关闭时可能混入的脏数据。
    pub fn hygiene_issues(&self) -> Result<Vec<String>, String> {
        let mut issues = Vec::new();
        // 孤儿引用：关联行指向不存在的标签 id（外键被关闭时可能混入）
        let orphan_ids: Vec<i64> = self
            .conn
            .prepare(sql::HYGIENE_ORPHANS)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if !orphan_ids.is_empty() {
            issues.push(format!(
                "memory_tags rows reference missing tags (schema corruption): tag ids {}",
                orphan_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        // 反向孤儿：关联行指向不存在的记忆（外键被关闭时可能混入）
        let reverse_ids: Vec<i64> = self
            .conn
            .prepare(
                "SELECT DISTINCT memory_id FROM memory_tags \
                 WHERE memory_id NOT IN (SELECT id FROM memories) ORDER BY memory_id",
            )
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if !reverse_ids.is_empty() {
            issues.push(format!(
                "database contains join rows pointing to missing memories (schema corruption): {}",
                reverse_ids
                    .iter()
                    .map(|id| Self::format_id(*id))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        // 仅大小写不同的标签组
        let names: Vec<String> = self
            .conn
            .prepare(sql::TAG_ALL_NAMES)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut groups: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        for n in &names {
            groups.entry(n.to_lowercase()).or_default().push(n.clone());
        }
        for group in groups.values() {
            if group.len() > 1 {
                issues.push(format!(
                    "case-conflicting tag group: {} (keep one and merge the rest with tag_update)",
                    group.join(" / ")
                ));
            }
        }
        // 空摘要 / 空正文
        let rows = self
            .conn
            .prepare(sql::HYGIENE_MEMORIES)
            .map_err(|e| e.to_string())?
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for (id, summary, content) in rows {
            if summary.trim().is_empty() {
                issues.push(format!(
                    "memory {} has an empty summary",
                    Self::format_id(id)
                ));
            }
            if content.trim().is_empty() {
                issues.push(format!("memory {} has empty content", Self::format_id(id)));
            }
        }
        // 语义搜索覆盖：开启且有记忆缺当前模型向量时提示补跑。
        // 这是派生数据问题而非数据损坏，但放进来让 doctor 成为唯一的体检入口。
        if let Some(cfg) = self.embedding_config()? {
            let pending = self.embedding_pending_count(&cfg.model)?;
            if pending > 0 {
                issues.push(format!(
                    "{pending} memories lack up-to-date embeddings (model '{}'); \
                     run `agent-memory embed-backfill` or use the admin UI",
                    cfg.model
                ));
            }
        }
        Ok(issues)
    }

    /// 数据概况（JSON 形态，CLI 与 API 共用）。
    pub fn stats(&self) -> Result<Value, String> {
        let memories: i64 = self
            .conn
            .query_row(sql::STATS_MEMORY_COUNT, [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let tags: i64 = self
            .conn
            .query_row(sql::STATS_TAG_COUNT, [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        // 真实的下一 id 由 AUTOINCREMENT 的 sqlite_sequence 权威记录：删除最大 id
        // 也不会回退、不会复用。该内部表在 memories 首次插入后才由 SQLite 创建，
        // 全空库下不存在——此时退回 MAX(id)+1（同样得到 1）。
        let has_sequence: i64 = self
            .conn
            .query_row(sql::STATS_HAS_SEQUENCE, [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let next_id: i64 = if has_sequence != 0 {
            self.conn
                .query_row(sql::STATS_NEXT_ID, [], |r| r.get(0))
                .map_err(|e| e.to_string())?
        } else {
            self.conn
                .query_row(sql::STATS_MAX_ID, [], |r| r.get::<_, i64>(0))
                .map_err(|e| e.to_string())?
                + 1
        };
        let newest = self
            .conn
            .query_row(sql::STATS_NEWEST, [], |r| {
                Ok(json!({
                    "id": Self::format_id(r.get::<_, i64>(0)?),
                    "updated_at": r.get::<_, i64>(1)?,
                }))
            })
            .map(Some)
            .or_else(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Ok(None),
                other => Err(other.to_string()),
            })?;
        let file_size = std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0);
        let schema_version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let embedding = match self.embedding_config()? {
            Some(cfg) => {
                let embedded = self.embedding_embedded_count(&cfg.model)?;
                let pending = self.embedding_pending_count(&cfg.model)?;
                json!({
                    "enabled": true,
                    "model": cfg.model,
                    "embedded": embedded,
                    "pending": pending,
                })
            }
            None => json!({ "enabled": false }),
        };
        Ok(json!({
            "path": self.path.display().to_string(),
            "memories": memories,
            "tags": tags,
            "next_id": Self::format_id(next_id),
            "file_size": file_size,
            "newest_update": newest.unwrap_or(Value::Null),
            "schema_version": schema_version,
            "embedding": embedding,
        }))
    }

    /// 导出内容：完整记忆 + 标签表，独立于存储内部模式（跨平台迁移也可走这里）。
    ///
    /// 形状：`tags` / `memories` 都是以 id 为键的对象——tag 键是内部自增 id 的
    /// 十进制字符串，记忆键是 `m<N>`（与 `format_id` 一致）；记忆的 `tags` 数组
    /// 按 tag id 字符串引用。只含主数据（名称/描述/正文/时间戳），不含
    /// memory_count、last_used_at、条目总数等派生信息（均可由本文件推出）。
    /// 导入会重新编号，id 键只在文件内充当引用记号。
    pub fn export_dump(&self) -> Result<Value, String> {
        let mut tags = Map::new();
        let mut st = self
            .conn
            .prepare(sql::TAG_EXPORT_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, name, description, created_at) = row.map_err(|e| e.to_string())?;
            tags.insert(
                id.to_string(),
                json!({
                    "name": name,
                    "description": description,
                    "created_at": created_at,
                }),
            );
        }

        // 记忆 ↔ 标签关联按内部 id 取对（memory_id 升序、tag_id 升序）。
        let mut refs: HashMap<i64, Vec<String>> = HashMap::new();
        let mut pairs = self
            .conn
            .prepare(sql::MEMORY_TAG_ID_PAIRS)
            .map_err(|e| e.to_string())?;
        let pair_rows = pairs
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        for pair in pair_rows {
            let (memory_id, tag_id) = pair.map_err(|e| e.to_string())?;
            refs.entry(memory_id).or_default().push(tag_id.to_string());
        }

        let mut memories = Map::new();
        let mut st = self
            .conn
            .prepare(sql::MEMORY_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, i64>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            let (id, summary, content, created_at, updated_at) = row.map_err(|e| e.to_string())?;
            let tags = refs.remove(&id).unwrap_or_default();
            memories.insert(
                Self::format_id(id),
                json!({
                    "summary": summary,
                    "content": content,
                    "tags": tags,
                    "created_at": created_at,
                    "updated_at": updated_at,
                }),
            );
        }
        Ok(json!({
            "exported_at": crate::model::now(),
            "tags": Value::Object(tags),
            "memories": Value::Object(memories),
        }))
    }

    /// 从 `export_dump` 产生的 JSON 恢复数据。要求目标库为空——导入是"恢复/
    /// 迁移"而非合并，避免与既有数据的 id、标签描述产生歧义。
    /// 记忆的 created_at / updated_at 按导出值保留；id 不保留（重新编号），
    /// 导出文件里的 id 键只用来表达记忆对标签的引用——引用了导出中不存在的
    /// tag id 直接报错。旧版数组形状的导出一律拒绝，不写兼容。
    /// 返回 (导入的记忆数, 导入的标签数)。
    pub fn import_dump(&self, dump: &Value) -> Result<(usize, usize), String> {
        let empty = self.stats()?;
        if empty["memories"].as_u64().unwrap_or(0) != 0 || empty["tags"].as_u64().unwrap_or(0) != 0
        {
            return Err(
                "target database is not empty; import refuses to merge -- point --db at a fresh database".into(),
            );
        }
        let tags = dump
            .get("tags")
            .and_then(Value::as_object)
            .ok_or("invalid export: 'tags' must be an object keyed by tag id")?;
        let memories = dump
            .get("memories")
            .and_then(Value::as_object)
            .ok_or("invalid export: 'memories' must be an object keyed by memory id")?;

        // id 键导入后重新编号，但仍是引用记号，先按导出格式从严校验：
        // tag 键为正整数十进制串，记忆键为 "m<N>"。
        for key in tags.keys() {
            let valid = key.parse::<u64>().map(|n| n > 0).unwrap_or(false);
            if !valid {
                return Err(format!(
                    "invalid export: tag key '{key}' is not a positive integer id"
                ));
            }
        }
        for key in memories.keys() {
            if Store::parse_id(key).is_none() {
                return Err(format!(
                    "invalid export: memory key '{key}' does not follow the \"m<N>\" id format"
                ));
            }
        }

        // 建标签（导出文件内重名同样拒绝——库里 name 唯一），
        // 导出 tag id → 库内新 id 的映射供记忆引用换算。
        let mut tag_id_by_key: HashMap<&str, i64> = HashMap::with_capacity(tags.len());
        let mut seen_names = HashSet::with_capacity(tags.len());
        for (key, t) in tags {
            let name = t
                .get("name")
                .and_then(Value::as_str)
                .ok_or("invalid export: tag without name")?;
            let description = t.get("description").and_then(Value::as_str).unwrap_or("");
            let name = crate::model::normalize_tag_name(name)?;
            if !seen_names.insert(name.clone()) {
                return Err(format!("invalid export: duplicate tag name '{name}'"));
            }
            let description = validate_max_len(
                description,
                "tag description",
                crate::model::MAX_TAG_DESC_CHARS,
            )?;
            self.tag_create(&name, &description)?;
            let id = self
                .tag_ids_for_names(std::slice::from_ref(&name))?
                .into_iter()
                .next()
                .ok_or_else(|| format!("tag '{name}' vanished right after creation"))?;
            tag_id_by_key.insert(key.as_str(), id);
        }

        for (key, m) in memories {
            let summary = m
                .get("summary")
                .and_then(Value::as_str)
                .ok_or("invalid export: memory without summary")?;
            let content = m
                .get("content")
                .and_then(Value::as_str)
                .ok_or("invalid export: memory without content")?;
            let summary =
                validate_nonempty_len(summary, "summary", crate::model::MAX_SUMMARY_CHARS)?;
            let content =
                validate_nonempty_len(content, "content", crate::model::MAX_CONTENT_CHARS)?;
            let created_at = m.get("created_at").and_then(Value::as_u64).unwrap_or(0);
            let updated_at = m
                .get("updated_at")
                .and_then(Value::as_u64)
                .unwrap_or(created_at);
            let empty_tags: Vec<Value> = Vec::new();
            let tag_refs = m
                .get("tags")
                .and_then(Value::as_array)
                .unwrap_or(&empty_tags);
            let mut tag_ids = Vec::with_capacity(tag_refs.len());
            for r in tag_refs {
                let s = r
                    .as_str()
                    .ok_or("invalid export: memory tags must be tag id strings")?;
                let id = tag_id_by_key.get(s).ok_or_else(|| {
                    format!("invalid export: memory {key} references unknown tag id '{s}'")
                })?;
                tag_ids.push(*id);
            }
            self.insert_memory(&summary, &content, &tag_ids, created_at, updated_at)?;
        }
        Ok((memories.len(), tags.len()))
    }
}

/// 非空 + 长度校验（导入侧的兜底；正常 API 路径由 tools::params 校验）。
fn validate_nonempty_len(s: &str, what: &str, max: usize) -> Result<String, String> {
    let t = s.trim();
    if t.is_empty() {
        return Err(format!("invalid export: {what} must not be empty"));
    }
    validate_max_len(t, what, max)
}

fn validate_max_len(s: &str, what: &str, max: usize) -> Result<String, String> {
    if s.chars().count() > max {
        return Err(format!(
            "invalid export: {what} is too long (max {max} characters)"
        ));
    }
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    #[test]
    fn next_id_never_reuses_deleted_max() {
        let path = temp_db("next-id");
        cleanup(&path);
        {
            let st = Store::open(&path).unwrap();
            // 全空库：sqlite_sequence 尚不存在，走 MAX+1 回退路径，下一 id 为 m1
            assert_eq!(st.stats().unwrap()["next_id"], "m1");

            let a = st.insert_memory("a", "ca", &[], 1, 1).unwrap();
            let b = st.insert_memory("b", "cb", &[], 2, 2).unwrap();
            let c = st.insert_memory("c", "cc", &[], 3, 3).unwrap();
            assert_eq!(st.stats().unwrap()["next_id"], "m4");

            // 删除当前最大 id：下一 id 不得回退（AUTOINCREMENT 序列只前进）
            st.delete_memories(&[c]).unwrap();
            assert_eq!(st.stats().unwrap()["next_id"], "m4");
            assert_eq!(st.insert_memory("d", "cd", &[], 4, 4).unwrap(), 4);

            // 删光所有记忆后同样不复用
            st.delete_memories(&[a, b]).unwrap();
            assert_eq!(st.stats().unwrap()["next_id"], "m5");
        }
        // 序列跨连接持久：重开库后仍不回退
        {
            let st = Store::open(&path).unwrap();
            assert_eq!(st.stats().unwrap()["next_id"], "m5");
            assert_eq!(st.insert_memory("e", "ce", &[], 5, 5).unwrap(), 5);
        }
        cleanup(&path);
    }

    #[test]
    fn hygiene_reports_real_problems_only() {
        let path = temp_db("hygiene");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        assert!(st.hygiene_issues().unwrap().is_empty());

        // 关闭外键注入孤儿引用、反向孤儿、大小写冲突、空摘要
        st.conn.execute("PRAGMA foreign_keys = OFF", []).unwrap();
        let rust_id = st.link_tags(&["rust".into()]).unwrap().ids[0];
        // 孤儿引用：关联行指向不存在的标签 id 999
        st.conn
            .execute(
                "INSERT INTO memory_tags(memory_id, tag_id) VALUES (1, 999)",
                [],
            )
            .unwrap();
        // 反向孤儿：关联行指向不存在的记忆 m42（rust 标签本身存在）
        st.conn
            .execute(
                "INSERT INTO memory_tags(memory_id, tag_id) VALUES (42, ?1)",
                [rust_id],
            )
            .unwrap();
        st.conn
            .execute("INSERT INTO tags(name, created_at) VALUES ('Rust', 1)", [])
            .unwrap();
        st.conn
            .execute(
                "INSERT INTO memories(id, summary, content, created_at, updated_at) \
                 VALUES (1, '   ', 'c', 1, 1)",
                [],
            )
            .unwrap();
        let issues = st.hygiene_issues().unwrap().join("\n");
        assert!(issues.contains("999"), "missing orphan: {issues}");
        assert!(
            issues.contains("pointing to missing memories"),
            "missing reverse orphan: {issues}"
        );
        assert!(
            issues.contains("case-conflicting"),
            "missing case: {issues}"
        );
        assert!(issues.contains("empty summary"), "missing empty: {issues}");
        cleanup(&path);
    }

    #[test]
    fn export_dump_keys_by_id_and_carries_no_derived_fields() {
        let path = temp_db("export");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("t", "desc").unwrap();
        let ids = st.link_tags(&["t".into()]).unwrap().ids;
        st.insert_memory("s", "body", &ids, 1, 1).unwrap();
        let dump = st.export_dump().unwrap();

        // 派生信息一律不进导出（都可由导出文件本身推出）
        assert!(dump.get("total_memories").is_none());
        assert!(dump.get("total_tags").is_none());
        // tags 以内部自增 id 为键，只含主数据
        let tag_key = ids[0].to_string();
        let tag = &dump["tags"][&tag_key];
        assert_eq!(tag["name"], "t");
        assert_eq!(tag["description"], "desc");
        assert!(tag.get("memory_count").is_none());
        assert!(tag.get("last_used_at").is_none());
        // memories 以 "m<N>" 为键，标签引用按 tag id 字符串
        let memory = &dump["memories"]["m1"];
        assert_eq!(memory["summary"], "s");
        assert_eq!(memory["content"], "body");
        assert_eq!(memory["tags"], json!([tag_key]));
        assert_eq!(memory["created_at"], 1);
        assert_eq!(memory["updated_at"], 1);
        assert!(dump["exported_at"].as_u64().is_some());
        // 无标签记忆导出为空数组
        st.insert_memory("s2", "body2", &[], 2, 2).unwrap();
        let dump = st.export_dump().unwrap();
        assert_eq!(dump["memories"]["m2"]["tags"], json!([]));
        cleanup(&path);
    }

    #[test]
    fn import_dump_restores_and_refuses_non_empty_target() {
        let src = temp_db("import-src");
        let dst = temp_db("import-dst");
        cleanup(&src);
        cleanup(&dst);
        let dump = {
            let st = Store::open(&src).unwrap();
            st.tag_create("t", "带描述的标签").unwrap();
            let ids = st.link_tags(&["t".into()]).unwrap().ids;
            st.insert_memory("s1", "body1", &ids, 100, 200).unwrap();
            st.insert_memory("s2", "body2", &[], 300, 400).unwrap();
            st.export_dump().unwrap()
        };

        // 恢复到空库：计数与时间戳都保留；tag id 重新编号后引用自动换算
        let (memories, tags) = Store::open(&dst).unwrap().import_dump(&dump).unwrap();
        assert_eq!(memories, 2);
        assert_eq!(tags, 1);
        let st = Store::open(&dst).unwrap();
        assert_eq!(st.stats().unwrap()["memories"], 2);
        let all = st.all_memories().unwrap();
        assert_eq!(all[0].created_at, 100);
        assert_eq!(all[0].updated_at, 200);
        assert_eq!(all[0].tags, vec!["t".to_string()]);
        assert_eq!(st.tag_view("t").unwrap()["description"], "带描述的标签");

        // 目标库非空 → 拒绝
        let err = Store::open(&dst).unwrap().import_dump(&dump).unwrap_err();
        assert!(err.contains("not empty"), "got: {err}");

        // 结构损坏的导出文件 → 报错
        let broken = Store::open(&temp_db("import-broken-dst"));
        drop(broken);
        let dst2 = temp_db("import-broken");
        cleanup(&dst2);
        let st2 = Store::open(&dst2).unwrap();
        let err = st2.import_dump(&json!({"memories": {}})).unwrap_err();
        assert!(err.contains("'tags' must be an object"), "got: {err}");
        cleanup(&src);
        cleanup(&dst);
        cleanup(&dst2);
    }

    /// 旧版数组形状的导出一律拒绝（无版本兼容），错误要说清期望的形状。
    #[test]
    fn import_dump_rejects_legacy_array_format() {
        let dst = temp_db("import-legacy");
        cleanup(&dst);
        let st = Store::open(&dst).unwrap();
        let legacy = json!({
            "exported_at": 1,
            "total_memories": 0,
            "total_tags": 0,
            "tags": [],
            "memories": [],
        });
        let err = st.import_dump(&legacy).unwrap_err();
        assert!(err.contains("must be an object"), "got: {err}");
        // 两个方向都各自点名
        let err = st
            .import_dump(&json!({"tags": {}, "memories": []}))
            .unwrap_err();
        assert!(err.contains("'memories' must be an object"), "got: {err}");
        assert_eq!(st.stats().unwrap()["memories"], 0);
        cleanup(&dst);
    }

    /// 导入与 API 契约一致：空白摘要、超长标签名、非字符串/未知 tag 引用、
    /// 非法 id 键、文件内重名标签都要被拒。每个用例独立空库（生产路径里
    /// import 在单写事务内执行，失败整体回滚——回滚本身由下面的专项测试把守）。
    #[test]
    fn import_dump_rejects_contract_violations() {
        let cases: Vec<(&str, Value, &str)> = vec![
            // 空白摘要
            (
                "blank-summary",
                json!({"tags": {}, "memories": {"m1": {"summary": "   ", "content": "c"}}}),
                "must not be empty",
            ),
            // 超长标签名（>100 字符）
            (
                "oversized-tag",
                json!({"tags": {"1": {"name": "x".repeat(101)}}, "memories": {}}),
                "too long",
            ),
            // 记忆的标签引用不是字符串（tag id 是字符串记号）
            (
                "non-string-ref",
                json!({"tags": {}, "memories": {"m1": {"summary": "s", "content": "c", "tags": [42]}}}),
                "must be tag id strings",
            ),
            // 引用了导出中不存在的 tag id → 报错（值从严错报）
            (
                "unknown-ref",
                json!({"tags": {"1": {"name": "t", "description": ""}}, "memories": {"m1": {"summary": "s", "content": "c", "tags": ["2"]}}}),
                "unknown tag id '2'",
            ),
            // 非法 id 键：tag 键非正整数、记忆键缺 m 前缀
            (
                "bad-tag-key",
                json!({"tags": {"t1": {"name": "t", "description": ""}}, "memories": {}}),
                "tag key 't1' is not a positive integer",
            ),
            (
                "bad-memory-key",
                json!({"tags": {}, "memories": {"1": {"summary": "s", "content": "c"}}}),
                "memory key '1' does not follow",
            ),
            // 导出文件内标签重名 → 报错（库里 name 唯一）
            (
                "duplicate-tag",
                json!({"tags": {"1": {"name": "t", "description": ""}, "2": {"name": "t", "description": "other"}}, "memories": {}}),
                "duplicate tag name 't'",
            ),
        ];
        for (slug, dump, expect) in &cases {
            let path = temp_db(&format!("import-invalid-{slug}"));
            cleanup(&path);
            let st = Store::open(&path).unwrap();
            let err = st.import_dump(dump).unwrap_err();
            assert!(err.contains(expect), "{slug}: got: {err}");
            // 拒绝不得落库任何记忆
            assert_eq!(st.stats().unwrap()["memories"], 0, "{slug}");
            cleanup(&path);
        }
    }

    /// 生产路径中 import 在单个写事务内执行：中途失败时已建出的标签也随
    /// 事务整体回滚，不残留半份数据。
    #[test]
    fn import_dump_failure_rolls_back_whole_transaction() {
        let path = temp_db("import-rollback");
        cleanup(&path);
        let dump = json!({
            "tags": {
                "1": {"name": "kept", "description": ""},
                "2": {"name": "other", "description": ""},
            },
            "memories": {"m1": {"summary": "s", "content": "c", "tags": ["3"]}},
        });
        let err = crate::store::with_db_in(&path, crate::store::TxMode::Write, |st: &Store| {
            st.import_dump(&dump).map(|_| ())
        })
        .unwrap_err();
        assert!(err.contains("unknown tag id '3'"), "got: {err}");
        let st = Store::open(&path).unwrap();
        assert_eq!(st.stats().unwrap()["tags"], 0, "tags must roll back");
        assert_eq!(st.stats().unwrap()["memories"], 0);
        cleanup(&path);
    }
}
