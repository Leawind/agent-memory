//! 记忆数据操作：插入 / 查询 / 分页浏览 / 更新 / 删除。

use crate::model::Memory;
use crate::sql;
use rusqlite::params;
use std::collections::HashMap;

use super::Store;

impl Store {
    /// 摘要与既有记忆归一化相同的条目 id（Rust 侧比较，Unicode 语义一致）。
    pub fn find_duplicates_by_summary(&self, summary: &str) -> Result<Vec<String>, String> {
        let mut st = self
            .conn
            .prepare(sql::MEMORY_SUMMARIES)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?;
        let norm = summary.to_lowercase();
        let mut out = Vec::new();
        for r in rows {
            let (id, s) = r.map_err(|e| e.to_string())?;
            if s.to_lowercase() == norm {
                out.push(Self::format_id(id));
            }
        }
        Ok(out)
    }

    /// 插入记忆并按内部标签 id 关联（id 由 handler 先经 `link_tags` 解析）。
    pub fn insert_memory(
        &self,
        summary: &str,
        content: &str,
        tag_ids: &[i64],
        created_at: u64,
        updated_at: u64,
    ) -> Result<i64, String> {
        self.conn
            .execute(
                sql::MEMORY_INSERT,
                params![summary, content, created_at as i64, updated_at as i64],
            )
            .map_err(|e| e.to_string())?;
        let id = self.conn.last_insert_rowid();
        for tag_id in tag_ids {
            self.conn
                .execute(sql::MEMORY_LINK_TAG, params![id, tag_id])
                .map_err(|e| e.to_string())?;
        }
        Ok(id)
    }

    fn memory_by_id(&self, id: i64) -> Result<Option<Memory>, String> {
        let row = self.conn.query_row(sql::MEMORY_BY_ID, [id], |r| {
            let created: i64 = r.get(3)?;
            let updated: i64 = r.get(4)?;
            Ok(Memory {
                id: Self::format_id(r.get(0)?),
                summary: r.get(1)?,
                content: r.get(2)?,
                tags: Vec::new(),
                created_at: created as u64,
                updated_at: updated as u64,
            })
        });
        let mut m = match row {
            Ok(m) => m,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.to_string()),
        };
        m.tags = self.tags_of(id)?;
        Ok(Some(m))
    }

    fn tags_of(&self, id: i64) -> Result<Vec<String>, String> {
        let mut st = self
            .conn
            .prepare(sql::MEMORY_TAGS_OF)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([id], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// 全量记忆（含标签，id 升序），供内存搜索器使用。
    pub fn all_memories(&self) -> Result<Vec<Memory>, String> {
        let mut st = self
            .conn
            .prepare(sql::MEMORY_ALL)
            .map_err(|e| e.to_string())?;
        let mut out: Vec<Memory> = st
            .query_map([], |r| {
                let created: i64 = r.get(3)?;
                let updated: i64 = r.get(4)?;
                Ok(Memory {
                    id: Self::format_id(r.get(0)?),
                    summary: r.get(1)?,
                    content: r.get(2)?,
                    tags: Vec::new(),
                    created_at: created as u64,
                    updated_at: updated as u64,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut tag_stmt = self
            .conn
            .prepare(sql::MEMORY_TAG_PAIRS)
            .map_err(|e| e.to_string())?;
        let pairs = tag_stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let mut by_memory: HashMap<i64, Vec<String>> = HashMap::new();
        for (mid, name) in pairs {
            by_memory.entry(mid).or_default().push(name);
        }
        for m in &mut out {
            let id = Self::parse_id(&m.id).unwrap_or(0);
            if let Some(tags) = by_memory.get(&id) {
                m.tags = tags.clone();
            }
        }
        Ok(out)
    }

    /// 分页浏览（可选标签过滤：`tag` 精确单标签；`tag_set` 为标签内部 id
    /// 集合的 JSON 数组文本，命中任一即可——名字 → id 的解析与正则过滤
    /// 由调用方先在标签全集上完成），
    /// 返回 (总数, 当前页)。
    ///
    /// 全静态 SQL（见 sql/memory_list_page.sql）：`?1`/`?2` 为 NULL 时不过滤；
    /// 集合过滤用 json_each 展开（SQLite 内建 JSON1）；排序列用 CASE 在
    /// `?3` 间选择；`?4` 传 ±1 实现正/倒序（列均为整数）。
    /// `sort` 只接受 handler 白名单化后的取值。
    pub fn list_memories(
        &self,
        tag: Option<&str>,
        tag_set: Option<&str>,
        sort: &str,
        asc: bool,
        offset: u64,
        limit: u64,
    ) -> Result<(u64, Vec<Memory>), String> {
        let dir: i64 = if asc { 1 } else { -1 };
        let total: u64 = self
            .conn
            .query_row(sql::MEMORY_LIST_COUNT, params![tag, tag_set], |r| {
                r.get::<_, i64>(0)
            })
            .map(|n| n as u64)
            .map_err(|e| e.to_string())?;
        let mut st = self
            .conn
            .prepare(sql::MEMORY_LIST_PAGE)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map(
                params![tag, tag_set, sort, dir, limit as i64, offset as i64],
                |r| {
                    let created: i64 = r.get(3)?;
                    let updated: i64 = r.get(4)?;
                    Ok(Memory {
                        id: Self::format_id(r.get(0)?),
                        summary: r.get(1)?,
                        content: r.get(2)?,
                        tags: Vec::new(),
                        created_at: created as u64,
                        updated_at: updated as u64,
                    })
                },
            )
            .map_err(|e| e.to_string())?;
        let mut page: Vec<Memory> = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        for m in &mut page {
            let id = Self::parse_id(&m.id).unwrap_or(0);
            m.tags = self.tags_of(id)?;
        }
        Ok((total, page))
    }

    /// 按 id 批量取完整记忆，返回 (找到的, 缺失的)。
    pub fn get_memories(&self, ids: &[i64]) -> Result<(Vec<Memory>, Vec<String>), String> {
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for id in ids {
            match self.memory_by_id(*id)? {
                Some(m) => found.push(m),
                None => missing.push(Self::format_id(*id)),
            }
        }
        Ok((found, missing))
    }

    /// 更新记忆字段与标签；返回是否发生变更（决定是否刷新 updated_at）。
    /// 标签以内部 id 增删（add 由 handler 先 `link_tags` 解析，remove 由
    /// `tag_ids_for_names` 解析——不存在的名字静默跳过）。
    pub fn update_memory(
        &self,
        id: i64,
        summary: Option<&str>,
        content: Option<&str>,
        add_tag_ids: &[i64],
        remove_tag_ids: &[i64],
    ) -> Result<bool, String> {
        if !self.memory_exists(id)? {
            return Err(format!(
                "memory '{}' not found (use memory_list or memory_search first)",
                Self::format_id(id)
            ));
        }
        let mut changed = false;
        if summary.is_some() || content.is_some() {
            self.conn
                .execute(
                    sql::MEMORY_UPDATE_FIELDS,
                    params![summary, content, crate::model::now() as i64, id],
                )
                .map_err(|e| e.to_string())?;
            // 字段变了向量即过期：删掉让它落回补跑队列（写入挂接会立刻重嵌；
            // embedding 服务不可用时就地留空，不阻塞更新本身）
            self.embedding_delete(id)?;
            changed = true;
        }
        // 契约（defs.rs）：add_tags 先于 remove_tags 执行，两个列表都含同一
        // 标签时最终结果是移除（显式 remove 的意图优先）。
        for tag_id in add_tag_ids {
            let n = self
                .conn
                .execute(sql::MEMORY_LINK_TAG, params![id, tag_id])
                .map_err(|e| e.to_string())?;
            changed = changed || n > 0;
        }
        for tag_id in remove_tag_ids {
            let n = self
                .conn
                .execute(sql::MEMORY_UNLINK_TAG, params![id, tag_id])
                .map_err(|e| e.to_string())?;
            changed = changed || n > 0;
        }
        if changed {
            self.conn
                .execute(sql::MEMORY_TOUCH, params![crate::model::now() as i64, id])
                .map_err(|e| e.to_string())?;
        }
        Ok(changed)
    }

    pub fn memory_exists(&self, id: i64) -> Result<bool, String> {
        match self.conn.query_row(sql::MEMORY_EXISTS, [id], |_| Ok(())) {
            Ok(()) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 删除记忆，返回 (已删 id, 缺失 id)。
    pub fn delete_memories(&self, ids: &[i64]) -> Result<(Vec<String>, Vec<String>), String> {
        let mut deleted = Vec::new();
        let mut missing = Vec::new();
        for id in ids {
            let n = self
                .conn
                .execute(sql::MEMORY_DELETE, [id])
                .map_err(|e| e.to_string())?;
            if n > 0 {
                deleted.push(Self::format_id(*id));
            } else {
                missing.push(Self::format_id(*id));
            }
        }
        Ok((deleted, missing))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;
    use serde_json::json;

    /// 测试辅助：按标签名建链并插入记忆（生产路径由 handler 解析 id）。
    fn insert_with_tags(st: &Store, summary: &str, content: &str, tags: &[&str], at: u64) -> i64 {
        let names: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        let ids = st.link_tags(&names).unwrap().ids;
        st.insert_memory(summary, content, &ids, at, at).unwrap()
    }

    #[test]
    fn memory_lifecycle_with_tags() {
        let path = temp_db("lifecycle");
        cleanup(&path);
        let st = Store::open(&path).unwrap();

        let dup = st.find_duplicates_by_summary("Rust notes").unwrap();
        assert!(dup.is_empty());
        let id = insert_with_tags(&st, "Rust notes", "borrow checker", &["rust", "notes"], 10);
        assert_eq!(Store::format_id(id), "m1");

        // 重复检测（大小写不敏感）
        let dup = st.find_duplicates_by_summary("rust NOTES").unwrap();
        assert_eq!(dup, vec!["m1".to_string()]);

        // 全量读取带标签
        let all = st.all_memories().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].tags, vec!["notes".to_string(), "rust".to_string()]);

        // 更新：改字段 + 增删标签（名字 → id 的解析走与 handler 相同的路径）
        let add = st.link_tags(&["study".into()]).unwrap();
        let remove = st.tag_ids_for_names(&["notes".into()]).unwrap();
        let changed = st
            .update_memory(id, Some("new summary"), None, &add.ids, &remove)
            .unwrap();
        assert!(changed);
        let (found, missing) = st.get_memories(&[id]).unwrap();
        assert!(missing.is_empty());
        assert_eq!(found[0].summary, "new summary");
        assert_eq!(found[0].tags, vec!["rust".to_string(), "study".to_string()]);

        // 不存在的记忆报错
        assert!(st.update_memory(999, Some("x"), None, &[], &[]).is_err());

        // 删除
        let (deleted, missing) = st.delete_memories(&[id, 999]).unwrap();
        assert_eq!(deleted, vec!["m1".to_string()]);
        assert_eq!(missing, vec!["m999".to_string()]);
        assert_eq!(st.stats().unwrap()["memories"], 0);
        cleanup(&path);
    }

    #[test]
    fn list_paginates_and_filters() {
        let path = temp_db("list");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        for i in 0..5 {
            insert_with_tags(&st, &format!("s{i}"), "c", &["t1"], i);
        }
        let (total, page) = st
            .list_memories(None, None, "updated_at", false, 0, 3)
            .unwrap();
        assert_eq!(total, 5);
        assert_eq!(page.len(), 3);
        assert_eq!(page[0].summary, "s4");
        let (total2, page2) = st
            .list_memories(None, None, "updated_at", false, 3, 3)
            .unwrap();
        assert_eq!(total2, 5);
        assert_eq!(page2.len(), 2);
        let (total3, page3) = st
            .list_memories(Some("t1"), None, "updated_at", true, 0, 200)
            .unwrap();
        assert_eq!(total3, 5);
        assert_eq!(page3[0].summary, "s0");
        let (total4, _) = st
            .list_memories(Some("t1"), None, "created_at", false, 0, 200)
            .unwrap();
        assert_eq!(total4, 5);
        // 标签集合过滤（tag_set 为 id JSON 数组文本，json_each 展开；空集 = 无结果）
        insert_with_tags(&st, "s-other", "c", &["t2"], 9);
        let id_set = |names: &[&str]| {
            let owned: Vec<String> = names.iter().map(|t| t.to_string()).collect();
            json!(st.tag_ids_for_names(&owned).unwrap()).to_string()
        };
        let (total5, page5) = st
            .list_memories(None, Some(&id_set(&["t1"])), "updated_at", false, 0, 200)
            .unwrap();
        assert_eq!(total5, 5);
        assert!(page5.iter().all(|m| m.summary != "s-other"));
        let (total6, _) = st
            .list_memories(
                None,
                Some(&id_set(&["t1", "t2"])),
                "updated_at",
                false,
                0,
                200,
            )
            .unwrap();
        assert_eq!(total6, 6);
        let (total7, _) = st
            .list_memories(None, Some("[]"), "updated_at", false, 0, 200)
            .unwrap();
        assert_eq!(total7, 0);
        // 精确 tag 与集合同时使用 = AND
        let (total8, _) = st
            .list_memories(
                Some("t2"),
                Some(&id_set(&["t1"])),
                "updated_at",
                false,
                0,
                200,
            )
            .unwrap();
        assert_eq!(total8, 0);
        cleanup(&path);
    }

    /// 契约（defs.rs）：add_tags 先于 remove_tags 执行，
    /// 同一标签同时出现在两个列表时最终结果是移除。
    #[test]
    fn update_memory_add_before_remove_ends_removed() {
        let path = temp_db("add-remove");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let id = insert_with_tags(&st, "s", "c", &["a"], 1);
        let add = st.link_tags(&["a".into(), "b".into()]).unwrap();
        let remove = st.tag_ids_for_names(&["a".into()]).unwrap();
        st.update_memory(id, None, None, &add.ids, &remove).unwrap();
        let (found, _) = st.get_memories(&[id]).unwrap();
        assert_eq!(found[0].tags, vec!["b".to_string()]);
        cleanup(&path);
    }
}
