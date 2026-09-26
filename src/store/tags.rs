//! 标签数据操作：创建 / 视图 / 改名（外键级联）/ 删除（detach 与 purge）。

use crate::sql;
use rusqlite::params;
use serde_json::{json, Value};

use super::{is_unique_violation, Store};

impl Store {
    pub fn tag_exists(&self, name: &str) -> Result<bool, String> {
        match self.conn.query_row(sql::TAG_EXISTS, [name], |_| Ok(())) {
            Ok(()) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 名字匹配正则的既有标签名（按存储名返回，供 tag_set 过滤走 json_each）。
    pub fn tag_names_matching(&self, re: &regex::Regex) -> Result<Vec<String>, String> {
        let mut st = self
            .conn
            .prepare(sql::TAG_ALL_NAMES)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| {
            let n = match r {
                Ok(n) => n,
                Err(e) => return Some(Err(e.to_string())),
            };
            if re.is_match(&n) {
                Some(Ok(n))
            } else {
                None
            }
        })
        .collect()
    }

    /// 查找与给定名字仅大小写不同的既有标签（Rust 侧比较，Unicode 语义一致）。
    pub fn find_tag_case_insensitive(&self, name: &str) -> Result<Option<String>, String> {
        let mut st = self
            .conn
            .prepare(sql::TAG_ALL_NAMES)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let fold = name.to_lowercase();
        for r in rows {
            let n = r.map_err(|e| e.to_string())?;
            if n != name && n.to_lowercase() == fold {
                return Ok(Some(n));
            }
        }
        Ok(None)
    }

    /// 新建标签；已存在时报错（唯一约束的显式化）。
    pub fn tag_create(&self, name: &str, description: &str) -> Result<(), String> {
        self.conn
            .execute(
                sql::TAG_CREATE,
                params![name, description, crate::model::now() as i64],
            )
            .map(|_| ())
            .map_err(|e| {
                if is_unique_violation(&e) {
                    format!(
                        "tag '{name}' already exists (rename it with tag_rename, or see tag_list)"
                    )
                } else {
                    e.to_string()
                }
            })
    }

    /// 单个标签视图：描述、记忆计数、最近使用时间。
    pub fn tag_view(&self, name: &str) -> Result<Value, String> {
        match self.conn.query_row(sql::TAG_VIEW, [name], row_to_tag_view) {
            Ok(v) => Ok(v),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(format!("tag '{name}' not found (see tag_list)"))
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// 全部标签视图，按记忆数降序、名字升序（分类体系浏览的默认序）。
    pub fn tag_views(&self) -> Result<Vec<Value>, String> {
        let mut st = self
            .conn
            .prepare(sql::TAG_VIEW_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], row_to_tag_view)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// 改名/改描述。改名利用外键 ON UPDATE CASCADE 同步全部引用；
    /// 目标名已存在时报错（与"精确重名建标签报错"同一语义）。
    /// 返回受改名影响的记忆条数。
    pub fn tag_rename(
        &self,
        old: &str,
        new_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<u64, String> {
        if !self.tag_exists(old)? {
            return Err(format!("tag '{old}' not found (see tag_list)"));
        }
        let mut memories_updated = 0u64;
        let final_name = new_name.unwrap_or(old);
        if final_name != old {
            if self.tag_exists(final_name)? {
                return Err(format!("tag '{final_name}' already exists"));
            }
            memories_updated = self.tag_memory_count(old)?;
            self.conn
                .execute(sql::TAG_RENAME, params![final_name, description, old])
                .map_err(|e| e.to_string())?;
        } else if let Some(d) = description {
            self.conn
                .execute(sql::TAG_SET_DESCRIPTION, params![d, old])
                .map_err(|e| e.to_string())?;
        }
        Ok(memories_updated)
    }

    fn tag_memory_count(&self, name: &str) -> Result<u64, String> {
        self.conn
            .query_row(sql::TAG_MEMORY_COUNT, [name], |r| r.get::<_, i64>(0))
            .map(|n| n as u64)
            .map_err(|e| e.to_string())
    }

    /// detach：删标签（级联摘除全部引用），返回受影响记忆条数。
    pub fn tag_delete_detach(&self, name: &str) -> Result<u64, String> {
        if !self.tag_exists(name)? {
            return Err(format!("tag '{name}' not found (see tag_list)"));
        }
        let affected = self.tag_memory_count(name)?;
        self.conn
            .execute(sql::TAG_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(affected)
    }

    /// purge：连带删除所有带该标签的记忆，返回被删记忆 id 列表。
    pub fn tag_delete_purge(&self, name: &str) -> Result<Vec<String>, String> {
        if !self.tag_exists(name)? {
            return Err(format!("tag '{name}' not found (see tag_list)"));
        }
        let ids = self.ids_with_tag(name)?;
        let id_strs: Vec<String> = ids.iter().map(|i| Self::format_id(*i)).collect();
        self.conn
            .execute(sql::PURGE_MEMORIES_WITH_TAG, [name])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(sql::TAG_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(id_strs)
    }

    fn ids_with_tag(&self, name: &str) -> Result<Vec<i64>, String> {
        let mut st = self
            .conn
            .prepare(sql::MEMORY_IDS_WITH_TAG)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([name], |r| r.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// 确保标签存在（自动创建，空描述），返回新建的标签名列表。
    pub fn ensure_tags_exist(&self, names: &[String]) -> Result<Vec<String>, String> {
        let mut autocreated = Vec::new();
        for n in names {
            let inserted = self
                .conn
                .execute(sql::TAG_AUTOCREATE, params![n, crate::model::now() as i64])
                .map_err(|e| e.to_string())?;
            if inserted > 0 {
                autocreated.push(n.clone());
            }
        }
        Ok(autocreated)
    }
}

fn row_to_tag_view(r: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let count: i64 = r.get(2)?;
    let last: Option<i64> = r.get(3)?;
    let created: i64 = r.get(4)?;
    Ok(json!({
        "name": r.get::<_, String>(0)?,
        "description": r.get::<_, String>(1)?,
        "memory_count": count,
        "last_used_at": last.map(|t| json!(t)).unwrap_or(Value::Null),
        "created_at": created,
    }))
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    #[test]
    fn tag_rename_cascades_and_rejects_existing_target() {
        let path = temp_db("rename");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("rust", "language").unwrap();
        st.tag_create("lang", "").unwrap();
        st.ensure_tags_exist(&["rust".into()]).unwrap();
        let a = st.insert_memory("a", "ca", &["rust".into()], 1, 1).unwrap();
        let _b = st.insert_memory("b", "cb", &["rust".into()], 1, 1).unwrap();

        // 改名到已存在的标签：报错
        let err = st.tag_rename("rust", Some("lang"), None).unwrap_err();
        assert!(err.contains("already exists"), "got: {err}");

        // 正常改名：级联同步所有引用，返回受影响记忆数
        let updated = st.tag_rename("rust", Some("systems"), None).unwrap();
        assert_eq!(updated, 2);
        assert!(!st.tag_exists("rust").unwrap());
        assert!(st.tag_exists("systems").unwrap());
        let (found, _) = st.get_memories(&[a]).unwrap();
        assert_eq!(found[0].tags, vec!["systems".to_string()]);

        // 改描述
        st.tag_rename("systems", None, Some("programming")).unwrap();
        assert_eq!(
            st.tag_view("systems").unwrap()["description"],
            "programming"
        );

        // 不存在的标签报错
        assert!(st.tag_rename("nope", Some("x"), None).is_err());
        cleanup(&path);
    }

    #[test]
    fn tag_delete_detach_vs_purge() {
        let path = temp_db("delete");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("x", "").unwrap();
        st.tag_create("keep", "").unwrap();
        st.ensure_tags_exist(&["x".into(), "keep".into()]).unwrap();
        st.insert_memory("a", "ca", &["x".into()], 1, 1).unwrap();
        st.insert_memory("b", "cb", &["x".into(), "keep".into()], 1, 1)
            .unwrap();
        st.insert_memory("c", "cc", &["keep".into()], 1, 1).unwrap();

        let updated = st.tag_delete_detach("x").unwrap();
        assert_eq!(updated, 2);
        assert!(!st.tag_exists("x").unwrap());
        assert_eq!(st.all_memories().unwrap().len(), 3);

        // 重建 x 并 purge：连带删除记忆
        st.tag_create("x", "").unwrap();
        st.ensure_tags_exist(&["x".into()]).unwrap();
        let all = st.all_memories().unwrap();
        let b_id = Store::parse_id(&all[1].id).unwrap();
        st.update_memory(b_id, None, None, &["x".into()], &[])
            .unwrap();
        let deleted = st.tag_delete_purge("x").unwrap();
        assert_eq!(deleted.len(), 1);
        assert_eq!(st.all_memories().unwrap().len(), 2);
        assert!(!st.tag_exists("x").unwrap());
        cleanup(&path);
    }

    #[test]
    fn tag_create_unique_and_case_hint() {
        let path = temp_db("case");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("rust", "").unwrap();
        let err = st.tag_create("rust", "");
        assert!(err.unwrap_err().contains("already exists"));
        assert_eq!(
            st.find_tag_case_insensitive("Rust").unwrap(),
            Some("rust".to_string())
        );
        assert_eq!(st.find_tag_case_insensitive("rust").unwrap(), None);
        cleanup(&path);
    }
}
