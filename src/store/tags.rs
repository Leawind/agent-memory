//! 标签数据操作：创建 / 视图 / 改名（外键级联）/ 删除（detach 与 purge）。

use crate::sql;
use rusqlite::params;
use serde_json::{json, Value};

use super::{is_unique_violation, Store};

/// 一批标签名的关联结果（见 `link_tags`）。
pub struct TagLinkage {
    /// 与输入名字逐一对齐的内部标签 id（对外不可见）。
    pub ids: Vec<i64>,
    /// 本次调用新建的标签名。
    pub autocreated: Vec<String>,
    /// 已存在而被复用的标签名。
    pub reused: Vec<String>,
    /// 复用标签中描述为空的（提示补写）。
    pub missing_description: Vec<String>,
}

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

    /// 改名/改描述。标签以内部 id 关联记忆，改名只动 name，引用自动跟随；
    /// 目标名已存在时报错（与"精确重名建标签报错"同一语义）。
    /// 返回引用该标签（随之改显新名）的记忆条数。
    pub fn tag_rename(
        &self,
        old: &str,
        new_name: Option<&str>,
        description: Option<&str>,
    ) -> Result<u64, String> {
        if !self.tag_exists(old)? {
            return Err(format!("tag '{old}' not found (see tag_list)"));
        }
        let final_name = new_name.unwrap_or(old);
        let mut memories_updated = 0u64;
        if final_name != old {
            if self.tag_exists(final_name)? {
                return Err(format!("tag '{final_name}' already exists"));
            }
            memories_updated = self.tag_memory_count(old)?;
            self.conn
                .execute(sql::TAG_RENAME, params![final_name, old])
                .map_err(|e| e.to_string())?;
        }
        if let Some(d) = description {
            self.conn
                .execute(sql::TAG_SET_DESCRIPTION, params![d, final_name])
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

    /// 确保标签存在（自动创建，空描述）并解析出内部 id，返回逐名对齐的
    /// 关联信息：`ids` 与输入名字一一对应；`autocreated` 是本次新建的标签，
    /// `reused` 是已存在的标签，`missing_description` 是复用标签中描述为空
    /// 的（多半由早前自动创建留下——提交给调用方决定是否提醒补写）。
    pub fn link_tags(&self, names: &[String]) -> Result<TagLinkage, String> {
        let mut out = TagLinkage {
            ids: Vec::with_capacity(names.len()),
            autocreated: Vec::new(),
            reused: Vec::new(),
            missing_description: Vec::new(),
        };
        for n in names {
            let inserted = self
                .conn
                .execute(sql::TAG_AUTOCREATE, params![n, crate::model::now() as i64])
                .map_err(|e| e.to_string())?;
            let (id, description): (i64, String) = self
                .conn
                .query_row(sql::TAG_BY_NAME, [n], |r| Ok((r.get(0)?, r.get(1)?)))
                .map_err(|e| e.to_string())?;
            out.ids.push(id);
            if inserted > 0 {
                out.autocreated.push(n.clone());
            } else {
                out.reused.push(n.clone());
                if description.is_empty() {
                    out.missing_description.push(n.clone());
                }
            }
        }
        Ok(out)
    }

    /// 解析既有标签名为内部 id；不存在的名字直接跳过（与旧的按名 unlink
    /// 静默无操作语义一致）。
    pub fn tag_ids_for_names(&self, names: &[String]) -> Result<Vec<i64>, String> {
        let mut ids = Vec::with_capacity(names.len());
        for n in names {
            let id = match self
                .conn
                .query_row(sql::TAG_BY_NAME, [n], |r| r.get::<_, i64>(0))
            {
                Ok(id) => Some(id),
                Err(rusqlite::Error::QueryReturnedNoRows) => None,
                Err(e) => return Err(e.to_string()),
            };
            if let Some(id) = id {
                ids.push(id);
            }
        }
        Ok(ids)
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

    /// 测试辅助：按标签名建链并插入记忆（生产路径由 handler 解析 id）。
    fn insert_with_tags(st: &Store, summary: &str, content: &str, tags: &[&str], at: u64) -> i64 {
        let names: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        let ids = st.link_tags(&names).unwrap().ids;
        st.insert_memory(summary, content, &ids, at, at).unwrap()
    }

    /// link_tags 的三分类：新建 / 复用 / 复用且缺描述；ids 与输入逐名对齐。
    #[test]
    fn link_tags_classifies_autocreated_reused_and_missing_description() {
        let path = temp_db("link-tags");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("described", "has description").unwrap();

        let first = st.link_tags(&["described".into(), "fresh".into()]).unwrap();
        assert_eq!(first.autocreated, vec!["fresh".to_string()]);
        assert_eq!(first.reused, vec!["described".to_string()]);
        assert!(first.missing_description.is_empty());
        assert_eq!(first.ids.len(), 2);

        // 第二次全部复用；此前自动创建的 fresh 描述为空 → 进 missing_description
        let second = st.link_tags(&["described".into(), "fresh".into()]).unwrap();
        assert!(second.autocreated.is_empty());
        assert_eq!(
            second.reused,
            vec!["described".to_string(), "fresh".to_string()]
        );
        assert_eq!(second.missing_description, vec!["fresh".to_string()]);
        // 同名 → 同 id（标签 id 恒定，改名不改 id）
        assert_eq!(first.ids, second.ids);
        cleanup(&path);
    }

    #[test]
    fn tag_rename_cascades_and_rejects_existing_target() {
        let path = temp_db("rename");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.tag_create("rust", "language").unwrap();
        st.tag_create("lang", "").unwrap();
        let a = insert_with_tags(&st, "a", "ca", &["rust"], 1);
        let _b = insert_with_tags(&st, "b", "cb", &["rust"], 1);

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
        insert_with_tags(&st, "a", "ca", &["x"], 1);
        insert_with_tags(&st, "b", "cb", &["x", "keep"], 1);
        insert_with_tags(&st, "c", "cc", &["keep"], 1);

        let updated = st.tag_delete_detach("x").unwrap();
        assert_eq!(updated, 2);
        assert!(!st.tag_exists("x").unwrap());
        assert_eq!(st.all_memories().unwrap().len(), 3);

        // 重建 x 并 purge：连带删除记忆
        st.tag_create("x", "").unwrap();
        let all = st.all_memories().unwrap();
        let b_id = Store::parse_id(&all[1].id).unwrap();
        let x_id = st.tag_ids_for_names(&["x".into()]).unwrap();
        st.update_memory(b_id, None, None, &x_id, &[]).unwrap();
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
