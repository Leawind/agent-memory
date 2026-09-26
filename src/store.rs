//! SQLite 持久化：单个数据库文件（WAL 模式）。
//!
//! 数据库文件格式是平台无关的（SQLite 自身处理字节序与对齐），同一份 .db
//! 可以在 Windows / Linux / macOS 之间直接复制使用。
//!
//! **SQL 全部外置**：业务语句在仓库根 `sql/` 目录（见 `crate::sql`），
//! schema 迁移在 `migrations/` 目录（build.rs 编译期生成 `MIGRATIONS`，
//! 目录即唯一事实源——加迁移只需丢入 `NUM-NAME.sql` 并重新编译）。
//! 迁移运行器按 `PRAGMA user_version` 记录进度，每个迁移独立事务，
//! 保证恰好应用一次；发布前允许破坏性更改（改写基线、删库重建），
//! 发布后只能新增迁移文件。
//!
//! 本层只做数据存取与少量聚合查询，不做参数校验；校验见 `tools::params`，
//! 纯数据模型见 `model`。所有请求在一个事务内执行（见 `with_db`），
//! panic 或错误时事务回滚，数据保持原样。

use crate::model::Memory;
use crate::sql;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

const BUSY_TIMEOUT_MS: u64 = 5000;

// Schema migrations embedded from the `migrations/` directory at build time
// (see `build.rs`), which is the single source of truth.
include!(concat!(env!("OUT_DIR"), "/migrations.rs"));

pub struct Store {
    pub path: PathBuf,
    pub conn: Connection,
}

/// 数据库文件路径的默认位置（当前工作目录下；--db 参数可覆盖）。
/// 设计用法是用户自己在固定目录运行维护：数据就在启动目录里，可见可控。
/// 所有子命令共享这一默认——维护命令须与 serve 在同一目录执行，或显式 --db。
pub fn default_path() -> PathBuf {
    PathBuf::from("memory.db")
}

/// 相对路径锚定到当前工作目录，保证 stats / 运维面板展示的始终是明确路径。
pub fn normalize_path(p: &Path) -> PathBuf {
    if p.is_absolute() {
        return p.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(p))
        .unwrap_or_else(|_| p.to_path_buf())
}

impl Store {
    /// 打开数据库：建目录、设 WAL/busy_timeout/外键、跑迁移。
    pub fn open(path: &Path) -> Result<Store, String> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("cannot create data directory: {e}"))?;
            }
        }
        let conn = Connection::open(path)
            .map_err(|e| format!("cannot open database at {}: {}", path.display(), e))?;
        conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))
            .map_err(|e| format!("cannot set busy timeout: {e}"))?;
        // WAL：读写不互斥，写者之间靠 SQLite 自己的锁 + busy_timeout 排队。
        // 最后一个连接正常关闭时 SQLite 会自动 checkpoint，此时 .db 单文件即可带走。
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| format!("cannot enable WAL mode: {e}"))?;
        // WAL 下的推荐档位：应用崩溃不丢数据，仅断电可能丢最近事务（不会损坏库），
        // 换取每次提交不再强制 fsync——对本服务"每请求一写"的模式收益明显。
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| format!("cannot set synchronous mode: {e}"))?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| format!("cannot enable foreign keys: {e}"))?;
        run_migrations(&conn)?;
        Ok(Store {
            path: normalize_path(path),
            conn,
        })
    }

    /// id 整数 ↔ API 字符串（"m3"）的边界换算。
    pub fn format_id(id: i64) -> String {
        format!("m{id}")
    }

    /// 仅接受 "m{n}" 格式（n 为正整数）；省略 m 前缀的写法不受容忍，返回 None。
    pub fn parse_id(raw: &str) -> Option<i64> {
        raw.strip_prefix('m')?
            .parse::<i64>()
            .ok()
            .filter(|n| *n > 0)
    }

    // ---------------------------------------------------------------- 标签

    pub fn tag_exists(&self, name: &str) -> Result<bool, String> {
        match self.conn.query_row(sql::TAG_EXISTS, [name], |_| Ok(())) {
            Ok(()) => Ok(true),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false),
            Err(e) => Err(e.to_string()),
        }
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

    // ---------------------------------------------------------------- 记忆

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

    /// 插入记忆并关联标签（标签须已存在——handler 先 ensure_tags_exist）。
    pub fn insert_memory(
        &self,
        summary: &str,
        content: &str,
        tags: &[String],
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
        for tag in tags {
            self.conn
                .execute(sql::MEMORY_LINK_TAG, params![id, tag])
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

    /// 分页浏览（可选标签过滤），返回 (总数, 当前页)。
    ///
    /// 全静态 SQL（见 sql/memory_list_page.sql）：`?1` 为 NULL 时不过滤标签；
    /// 排序列用 CASE 在 `?2` 间选择；`?3` 传 ±1 实现正/倒序（列均为整数）。
    /// `sort` 只接受 handler 白名单化后的取值。
    pub fn list_memories(
        &self,
        tag: Option<&str>,
        sort: &str,
        asc: bool,
        offset: u64,
        limit: u64,
    ) -> Result<(u64, Vec<Memory>), String> {
        let dir: i64 = if asc { 1 } else { -1 };
        let total: u64 = self
            .conn
            .query_row(sql::MEMORY_LIST_COUNT, [tag], |r| r.get::<_, i64>(0))
            .map(|n| n as u64)
            .map_err(|e| e.to_string())?;
        let mut st = self
            .conn
            .prepare(sql::MEMORY_LIST_PAGE)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map(params![tag, sort, dir, limit as i64, offset as i64], |r| {
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
    pub fn update_memory(
        &self,
        id: i64,
        summary: Option<&str>,
        content: Option<&str>,
        add_tags: &[String],
        remove_tags: &[String],
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
            changed = true;
        }
        // 契约（defs.rs）：add_tags 先于 remove_tags 执行，两个列表都含同一
        // 标签时最终结果是移除（显式 remove 的意图优先）。
        if !add_tags.is_empty() {
            self.ensure_tags_exist(add_tags)?;
            for t in add_tags {
                let n = self
                    .conn
                    .execute(sql::MEMORY_LINK_TAG, params![id, t])
                    .map_err(|e| e.to_string())?;
                changed = changed || n > 0;
            }
        }
        for tag in remove_tags {
            let n = self
                .conn
                .execute(sql::MEMORY_UNLINK_TAG, params![id, tag])
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

    // ---------------------------------------------------------------- 身份与设置

    /// 生成随机 token（64 位十六进制，SQLite PRNG 由系统熵播种）。
    pub fn generate_token(&self) -> Result<String, String> {
        self.conn
            .query_row(sql::TOKEN_GENERATE, [], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())
    }

    pub fn identity_count(&self) -> Result<u64, String> {
        self.conn
            .query_row(sql::IDENTITY_COUNT, [], |r| r.get::<_, i64>(0))
            .map(|n| n as u64)
            .map_err(|e| e.to_string())
    }

    /// 新建身份：生成随机 token，返回 (token, 管理视图)。重名由唯一约束显式化。
    /// token 明文只在本返回值出现一次，库内仅存哈希与尾缀提示。
    pub fn identity_create(
        &self,
        name: &str,
        permissions: &crate::auth::Permissions,
    ) -> Result<(String, Value), String> {
        let token = self.generate_token()?;
        let (hash, hint) = token_hash_and_hint(&token);
        let stored = permissions.to_stored_string();
        self.conn
            .execute(
                sql::IDENTITY_INSERT,
                params![name, hash, hint, stored, crate::model::now() as i64],
            )
            .map(|_| ())
            .map_err(|e| {
                if is_unique_violation(&e) {
                    format!("identity '{name}' already exists")
                } else {
                    e.to_string()
                }
            })?;
        let view = self.identity_view(name)?.ok_or_else(|| {
            format!("identity '{name}' vanished right after creation (concurrent modification)")
        })?;
        Ok((token, view))
    }

    /// Bearer token → 请求身份；未知 token 返回 None（调用方决定 401）。
    /// 入参哈希后与库存哈希比对，库内无明文。
    pub fn identity_ctx_by_token(
        &self,
        token: &str,
    ) -> Result<Option<crate::auth::IdentityCtx>, String> {
        let hash = crate::util::sha256_hex(token.as_bytes());
        let row = self.conn.query_row(sql::IDENTITY_BY_TOKEN, [hash], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        });
        match row {
            Ok((_id, name, perms_json)) => {
                let perms = parse_stored_permissions(&perms_json)?;
                Ok(Some(crate::auth::IdentityCtx::new(&name, perms)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 管理视图列表。token 只存哈希，这里只给尾缀提示（供辨认，不可复原）。
    pub fn identity_list(&self) -> Result<Vec<Value>, String> {
        let mut st = self
            .conn
            .prepare(sql::IDENTITY_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.map(|row| {
            let (name, token, perms_json, created_at) = row.map_err(|e| e.to_string())?;
            let perms = parse_stored_permissions(&perms_json)?;
            Ok(json!({
                "name": name,
                "token_hint": token,
                "permissions": perms.to_json(),
                "created_at": created_at,
            }))
        })
        .collect()
    }

    /// 单个身份的管理视图（含 token）。
    pub fn identity_view(&self, name: &str) -> Result<Option<Value>, String> {
        self.identity_list()
            .map(|all| all.into_iter().find(|v| v["name"].as_str() == Some(name)))
    }

    pub fn identity_set_permissions(
        &self,
        name: &str,
        permissions: &crate::auth::Permissions,
    ) -> Result<bool, String> {
        let stored = permissions.to_stored_string();
        let n = self
            .conn
            .execute(sql::IDENTITY_UPDATE_PERMISSIONS, params![stored, name])
            .map_err(|e| e.to_string())?;
        Ok(n > 0)
    }

    /// 重置 token（旧 token 立即失效），返回新 token 明文（仅此一次）；
    /// 身份不存在返回 None。
    pub fn identity_reset_token(&self, name: &str) -> Result<Option<String>, String> {
        let token = self.generate_token()?;
        let (hash, hint) = token_hash_and_hint(&token);
        let n = self
            .conn
            .execute(sql::IDENTITY_UPDATE_TOKEN, params![hash, hint, name])
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(None);
        }
        Ok(Some(token))
    }

    pub fn identity_delete(&self, name: &str) -> Result<bool, String> {
        let n = self
            .conn
            .execute(sql::IDENTITY_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(n > 0)
    }

    /// 读取服务器设置；键不存在返回 None。
    pub fn settings_get(&self, key: &str) -> Result<Option<String>, String> {
        match self
            .conn
            .query_row(sql::SETTINGS_GET, [key], |r| r.get::<_, String>(0))
        {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn settings_put(&self, key: &str, value: &str) -> Result<(), String> {
        self.conn
            .execute(sql::SETTINGS_PUT, params![key, value])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// token 鉴权开关（settings 键，显式且持久化）：开启与否由操作者决定，
    /// 与库里是否存在身份无关。缺省视为关闭（全新库 = 开放模式零配置）。
    pub const SETTING_AUTH_REQUIRED: &'static str = "auth_required";

    pub fn auth_required(&self) -> Result<bool, String> {
        Ok(self.settings_get(Self::SETTING_AUTH_REQUIRED)?.as_deref() == Some("true"))
    }

    pub fn set_auth_required(&self, on: bool) -> Result<(), String> {
        self.settings_put(
            Self::SETTING_AUTH_REQUIRED,
            if on { "true" } else { "false" },
        )
    }

    // ---------------------------------------------------------------- 运维

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
        Ok(json!({
            "path": self.path.display().to_string(),
            "memories": memories,
            "tags": tags,
            "next_id": Self::format_id(next_id),
            "file_size": file_size,
            "newest_update": newest.unwrap_or(Value::Null),
            "schema_version": schema_version,
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

fn row_to_tag_view(r: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    let count: i64 = r.get(2)?;
    let last: Option<i64> = r.get(3)?;
    Ok(json!({
        "name": r.get::<_, String>(0)?,
        "description": r.get::<_, String>(1)?,
        "memory_count": count,
        "last_used_at": last.map(|t| json!(t)).unwrap_or(Value::Null),
    }))
}

fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(e, rusqlite::Error::SqliteFailure(ee, _) if ee.code == rusqlite::ErrorCode::ConstraintViolation)
}

/// 解析 identities.permissions 列。写入路径已严格校验，这里再防线一次：
/// 损坏的行会让读取报错（身份解析 fail-closed），而不是静默放大权限。
fn parse_stored_permissions(json_text: &str) -> Result<crate::auth::Permissions, String> {
    let v: Value =
        serde_json::from_str(json_text).map_err(|e| format!("corrupt permissions JSON: {e}"))?;
    crate::auth::Permissions::from_json(&v)
}

/// 迁移运行器：`PRAGMA user_version` 记录已应用的迁移数量，每个待应用迁移
/// 在独立事务中执行并推进 user_version，保证恰好应用一次。数据库比已知
/// 迁移更新（来自更新版本的程序）时拒绝打开，绝不带着未知的 schema 写数据。
fn run_migrations(conn: &Connection) -> Result<(), String> {
    let applied: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    let from = applied.max(0) as usize;
    let target = MIGRATIONS.len();
    if from > target {
        return Err(format!(
            "database schema v{from} is newer than supported v{target}; upgrade agent-memory or restore a matching database file"
        ));
    }
    for (index, source) in MIGRATIONS.iter().enumerate().skip(from) {
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| format!("migration {}: cannot begin: {}", index + 1, e))?;
        tx.execute_batch(source)
            .map_err(|e| format!("migration {} failed: {}", index + 1, e))?;
        tx.pragma_update(None, "user_version", (index + 1) as i64)
            .map_err(|e| format!("migration {}: cannot record version: {}", index + 1, e))?;
        tx.commit()
            .map_err(|e| format!("migration {}: cannot commit: {}", index + 1, e))?;
    }
    Ok(())
}

/// token → (存储哈希, 尾缀提示)。token 为 64 位 hex，提示取末 4 字符。
fn token_hash_and_hint(token: &str) -> (String, String) {
    let hint: String = token
        .chars()
        .skip(token.chars().count().saturating_sub(4))
        .collect();
    (crate::util::sha256_hex(token.as_bytes()), hint)
}

/// 事务模式：只读请求用 DEFERRED（WAL 下获得一致性快照且不抢写锁，
/// 读与读、读与写互不阻塞），写请求用 IMMEDIATE（一开始就取写锁，
/// 配合 busy_timeout 让并发写者排队，避免 DEFERRED 读后升级写锁的死锁）。
pub enum TxMode {
    ReadOnly,
    Write,
}

/// 在单个事务内使用数据库：panic 或错误时回滚，成功才提交。
///
/// 事务模式见 `TxMode`。错误类型泛型化（`E: From<String>`），调用方
/// 可选择自己的错误类型（如工具层的 `ToolError`）；基础设施错误
/// （打开/建事务/提交）从 String 转换而来。错误或 panic 时函数提前
/// 返回/展开，连接关闭自动回滚——依赖"成功才 COMMIT"的顺序。
pub fn with_db_in<T, E>(
    path: &Path,
    mode: TxMode,
    f: impl FnOnce(&Store) -> Result<T, E>,
) -> Result<T, E>
where
    E: From<String>,
{
    let store = Store::open(path).map_err(E::from)?;
    let begin = match mode {
        TxMode::ReadOnly => "BEGIN DEFERRED",
        TxMode::Write => "BEGIN IMMEDIATE",
    };
    store
        .conn
        .execute_batch(begin)
        .map_err(|e| E::from(format!("cannot begin transaction: {e}")))?;
    let out = f(&store)?;
    store
        .conn
        .execute_batch("COMMIT")
        .map_err(|e| E::from(format!("cannot commit: {e}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agent-memory-store-{}-{}.db",
            std::process::id(),
            tag
        ))
    }

    fn cleanup(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(PathBuf::from(format!("{}{}", path.display(), suffix)));
        }
    }

    #[test]
    fn open_creates_schema_and_is_idempotent() {
        let path = temp_db("schema");
        cleanup(&path);
        {
            let st = Store::open(&path).unwrap();
            assert_eq!(st.stats().unwrap()["memories"], 0);
        }
        {
            let st = Store::open(&path).unwrap();
            assert_eq!(st.stats().unwrap()["memories"], 0);
        }
        cleanup(&path);
    }

    #[test]
    fn migrations_record_user_version() {
        let path = temp_db("version");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let version: i64 = st
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version as usize, MIGRATIONS.len());
        cleanup(&path);
    }

    #[test]
    fn future_schema_version_is_rejected() {
        let path = temp_db("future");
        cleanup(&path);
        {
            let st = Store::open(&path).unwrap();
            st.conn.execute_batch("PRAGMA user_version = 999").unwrap();
        }
        let Err(err) = Store::open(&path) else {
            panic!("expected open to fail on newer schema")
        };
        assert!(err.contains("newer than supported"), "got: {err}");
        cleanup(&path);
    }

    #[test]
    fn memory_lifecycle_with_tags() {
        let path = temp_db("lifecycle");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let tags = vec!["rust".to_string(), "notes".to_string()];
        st.ensure_tags_exist(&tags).unwrap();

        let dup = st.find_duplicates_by_summary("Rust notes").unwrap();
        assert!(dup.is_empty());
        let id = st
            .insert_memory("Rust notes", "borrow checker", &tags, 10, 10)
            .unwrap();
        assert_eq!(Store::format_id(id), "m1");

        // 重复检测（大小写不敏感）
        let dup = st.find_duplicates_by_summary("rust NOTES").unwrap();
        assert_eq!(dup, vec!["m1".to_string()]);

        // 全量读取带标签
        let all = st.all_memories().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].tags, vec!["notes".to_string(), "rust".to_string()]);

        // 更新：改字段 + 增删标签
        let changed = st
            .update_memory(
                id,
                Some("new summary"),
                None,
                &["study".into()],
                &["notes".into()],
            )
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

    #[test]
    fn list_paginates_and_filters() {
        let path = temp_db("list");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.ensure_tags_exist(&["t1".into()]).unwrap();
        for i in 0..5 {
            st.insert_memory(&format!("s{i}"), "c", &["t1".into()], i, i)
                .unwrap();
        }
        let (total, page) = st.list_memories(None, "updated_at", false, 0, 3).unwrap();
        assert_eq!(total, 5);
        assert_eq!(page.len(), 3);
        assert_eq!(page[0].summary, "s4");
        let (total2, page2) = st.list_memories(None, "updated_at", false, 3, 3).unwrap();
        assert_eq!(total2, 5);
        assert_eq!(page2.len(), 2);
        let (total3, page3) = st
            .list_memories(Some("t1"), "updated_at", true, 0, 200)
            .unwrap();
        assert_eq!(total3, 5);
        assert_eq!(page3[0].summary, "s0");
        let (total4, _) = st
            .list_memories(Some("t1"), "created_at", false, 0, 200)
            .unwrap();
        assert_eq!(total4, 5);
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
    fn id_roundtrip_and_strict_parse() {
        assert_eq!(Store::parse_id("m12"), Some(12));
        assert_eq!(Store::parse_id("12"), None);
        assert_eq!(Store::parse_id("m0"), None);
        assert_eq!(Store::parse_id("abc"), None);
        assert_eq!(Store::format_id(12), "m12");
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

    #[test]
    fn with_db_rolls_back_on_error() {
        let path = temp_db("tx");
        cleanup(&path);
        let r: Result<(), String> = with_db_in(&path, TxMode::Write, |st| {
            st.ensure_tags_exist(&["t".into()])?;
            st.insert_memory("s", "c", &["t".into()], 1, 1)?;
            Err("boom".into())
        });
        assert!(r.is_err());
        let st = Store::open(&path).unwrap();
        assert_eq!(
            st.stats().unwrap()["memories"],
            0,
            "rollback must remove the memory"
        );
        assert_eq!(
            st.stats().unwrap()["tags"],
            0,
            "rollback must remove the tag"
        );
        cleanup(&path);
    }

    #[test]
    fn identity_lifecycle_token_lookup_and_reset() {
        let path = temp_db("identity");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        assert_eq!(st.identity_count().unwrap(), 0);

        let (token, view) = st
            .identity_create("alice", &crate::auth::Permissions::all())
            .unwrap();
        assert_eq!(token.len(), 64, "token = hex(randomblob(32))");
        assert_eq!(view["name"], "alice");
        assert_eq!(view["permissions"]["admin"], true);
        // 库内无明文：视图只带尾缀提示，且哈希不可逆推出 token
        assert_eq!(view["token_hint"], &token[token.len() - 4..]);
        assert!(serde_json::to_string(&view).unwrap().find(&token).is_none());
        // 重名报错
        assert!(st
            .identity_create("alice", &crate::auth::Permissions::default())
            .is_err());

        // token → 身份上下文
        let ctx = st.identity_ctx_by_token(&token).unwrap().unwrap();
        assert_eq!(ctx.name, "alice");
        assert!(ctx.can(crate::auth::Cap::Admin));
        assert!(!ctx.open_mode);
        // 未知 token → None
        assert!(st.identity_ctx_by_token("nope").unwrap().is_none());

        // 收窄权限后，同一 token 的能力同步收窄
        st.identity_set_permissions("alice", &crate::auth::Permissions::default())
            .unwrap();
        let ctx = st.identity_ctx_by_token(&token).unwrap().unwrap();
        assert!(!ctx.can(crate::auth::Cap::Read));

        // 重置 token：旧 token 立即失效
        let new_token = st.identity_reset_token("alice").unwrap().unwrap();
        assert_ne!(new_token, token);
        assert!(st.identity_ctx_by_token(&token).unwrap().is_none());
        assert!(st.identity_ctx_by_token(&new_token).unwrap().is_some());
        // 不存在的身份重置 → None
        assert!(st.identity_reset_token("nope").unwrap().is_none());

        // 删除后计数归零；再删返回 false
        assert!(st.identity_delete("alice").unwrap());
        assert!(!st.identity_delete("alice").unwrap());
        assert_eq!(st.identity_count().unwrap(), 0);
        cleanup(&path);
    }

    #[test]
    fn settings_roundtrip_and_upsert() {
        let path = temp_db("settings");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        assert_eq!(st.settings_get("instructions").unwrap(), None);
        st.settings_put("instructions", "v1").unwrap();
        st.settings_put("instructions", "v2").unwrap();
        assert_eq!(
            st.settings_get("instructions").unwrap().as_deref(),
            Some("v2")
        );
        // 其他键互不影响
        st.settings_put("other", "x").unwrap();
        assert_eq!(
            st.settings_get("instructions").unwrap().as_deref(),
            Some("v2")
        );
        cleanup(&path);
    }

    /// 契约（defs.rs）：add_tags 先于 remove_tags 执行，
    /// 同一标签同时出现在两个列表时最终结果是移除。
    #[test]
    fn update_memory_add_before_remove_ends_removed() {
        let path = temp_db("add-remove");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.ensure_tags_exist(&["a".into(), "b".into()]).unwrap();
        let id = st.insert_memory("s", "c", &["a".into()], 1, 1).unwrap();
        st.update_memory(id, None, None, &["a".into(), "b".into()], &["a".into()])
            .unwrap();
        let (found, _) = st.get_memories(&[id]).unwrap();
        assert_eq!(found[0].tags, vec!["b".to_string()]);
        cleanup(&path);
    }
}
