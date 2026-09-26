//! 聚合查询与运维入口：体检（hygiene）、统计（stats）、导出导入。

use crate::sql;
use serde_json::{json, Value};

use super::Store;

impl Store {
    /// 体检：报告数据中的隐患（只读）。覆盖外键被关闭时可能混入的脏数据。
    pub fn hygiene_issues(&self) -> Result<Vec<String>, String> {
        let mut issues = Vec::new();
        // 孤儿引用：关联表指向不存在的标签
        let orphans: Vec<String> = self
            .conn
            .prepare(sql::HYGIENE_ORPHANS)
            .map_err(|e| e.to_string())?
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if !orphans.is_empty() {
            issues.push(format!(
                "memories reference tags missing from the tag table: {} (fix with tag_create, or remove the references)",
                orphans.join(", ")
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
                    "case-conflicting tag group: {} (keep one and merge the rest with tag_rename)",
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
    pub fn export_dump(&self) -> Result<Value, String> {
        let tags = self.tag_views()?;
        let memories: Vec<Value> = self.all_memories()?.iter().map(|m| m.full_view()).collect();
        Ok(json!({
            "exported_at": crate::model::now(),
            "total_memories": memories.len(),
            "total_tags": tags.len(),
            "tags": tags,
            "memories": memories,
        }))
    }

    /// 从 `export_dump` 产生的 JSON 恢复数据。要求目标库为空——导入是"恢复/
    /// 迁移"而非合并，避免与既有数据的 id、标签描述产生歧义。
    /// 记忆的 created_at / updated_at 按导出值保留；id 不保留（重新编号）。
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
            .and_then(Value::as_array)
            .ok_or("invalid export: missing 'tags' array")?;
        let memories = dump
            .get("memories")
            .and_then(Value::as_array)
            .ok_or("invalid export: missing 'memories' array")?;

        for t in tags {
            let name = t
                .get("name")
                .and_then(Value::as_str)
                .ok_or("invalid export: tag without name")?;
            let description = t.get("description").and_then(Value::as_str).unwrap_or("");
            let name = crate::model::normalize_tag_name(name)?;
            let description = validate_max_len(
                description,
                "tag description",
                crate::model::MAX_TAG_DESC_CHARS,
            )?;
            self.tag_create(&name, &description)?;
        }
        for m in memories {
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
            let tags: Vec<String> = m
                .get("tags")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .map(|t| {
                            let s = t
                                .as_str()
                                .ok_or("invalid export: memory tags must be strings")?;
                            crate::model::normalize_tag_name(s)
                        })
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()?
                .unwrap_or_default();
            self.ensure_tags_exist(&tags)?;
            self.insert_memory(&summary, &content, &tags, created_at, updated_at)?;
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
        st.conn
            .execute(
                "INSERT INTO memory_tags(memory_id, tag_name) VALUES (1, 'ghost')",
                [],
            )
            .unwrap();
        // 反向孤儿：关联行指向不存在的记忆 m42
        st.conn
            .execute(
                "INSERT INTO memory_tags(memory_id, tag_name) VALUES (42, 'rust')",
                [],
            )
            .unwrap();
        st.conn
            .execute("INSERT INTO tags(name, created_at) VALUES ('Rust', 1)", [])
            .unwrap();
        st.conn
            .execute("INSERT INTO tags(name, created_at) VALUES ('rust', 1)", [])
            .unwrap();
        st.conn
            .execute(
                "INSERT INTO memories(id, summary, content, created_at, updated_at) \
                 VALUES (1, '   ', 'c', 1, 1)",
                [],
            )
            .unwrap();
        let issues = st.hygiene_issues().unwrap().join("\n");
        assert!(issues.contains("ghost"), "missing orphan: {issues}");
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
    fn export_dump_contains_full_data() {
        let path = temp_db("export");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("t", "desc").unwrap();
        st.ensure_tags_exist(&["t".into()]).unwrap();
        st.insert_memory("s", "body", &["t".into()], 1, 1).unwrap();
        let dump = st.export_dump().unwrap();
        assert_eq!(dump["total_memories"], 1);
        assert_eq!(dump["tags"][0]["name"], "t");
        assert_eq!(dump["memories"][0]["content"], "body");
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
            st.ensure_tags_exist(&["t".into()]).unwrap();
            st.insert_memory("s1", "body1", &["t".into()], 100, 200)
                .unwrap();
            st.insert_memory("s2", "body2", &[], 300, 400).unwrap();
            st.export_dump().unwrap()
        };

        // 恢复到空库：计数与时间戳都保留
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
        let err = st2.import_dump(&json!({"memories": []})).unwrap_err();
        assert!(err.contains("missing 'tags'"), "got: {err}");
        cleanup(&src);
        cleanup(&dst);
        cleanup(&dst2);
    }

    /// 导入与 API 契约一致：空白摘要、超长标签名、非字符串标签都要被拒。
    #[test]
    fn import_dump_rejects_contract_violations() {
        let dst = temp_db("import-invalid");
        cleanup(&dst);
        let st = Store::open(&dst).unwrap();

        // 空白摘要
        let bad_summary = json!({
            "tags": [],
            "memories": [{"summary": "   ", "content": "c"}]
        });
        let err = st.import_dump(&bad_summary).unwrap_err();
        assert!(err.contains("must not be empty"), "got: {err}");

        // 超长标签名（>100 字符）
        let bad_tag = json!({
            "tags": [{"name": "x".repeat(101)}],
            "memories": []
        });
        let err = st.import_dump(&bad_tag).unwrap_err();
        assert!(err.contains("too long"), "got: {err}");

        // 记忆的标签不是字符串
        let bad_tags = json!({
            "tags": [],
            "memories": [{"summary": "s", "content": "c", "tags": [42]}]
        });
        let err = st.import_dump(&bad_tags).unwrap_err();
        assert!(err.contains("must be strings"), "got: {err}");

        // 以上任何失败都不能落库
        assert_eq!(st.stats().unwrap()["memories"], 0);
        cleanup(&dst);
    }
}
