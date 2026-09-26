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
//!
//! 按领域拆分：`tags` / `memories` / `identities` / `settings` /
//! `embeddings` 各以 `impl Store` 承载数据操作，`ops` 是体检 / 统计 /
//! 导出导入等聚合查询，`tx` 是事务入口，`migrate` 是迁移运行器。

mod embeddings;
mod identities;
mod memories;
mod migrate;
mod ops;
mod settings;
mod tags;
mod tx;

pub use tx::{with_db_in, TxMode};

use rusqlite::Connection;
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
        migrate::run_migrations(&conn)?;
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
}

fn is_unique_violation(e: &rusqlite::Error) -> bool {
    matches!(e, rusqlite::Error::SqliteFailure(ee, _) if ee.code == rusqlite::ErrorCode::ConstraintViolation)
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::path::{Path, PathBuf};

    pub(crate) fn temp_db(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "agent-memory-store-{}-{}.db",
            std::process::id(),
            tag
        ))
    }

    pub(crate) fn cleanup(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(PathBuf::from(format!("{}{}", path.display(), suffix)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{cleanup, temp_db};
    use super::*;

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
    fn id_roundtrip_and_strict_parse() {
        assert_eq!(Store::parse_id("m12"), Some(12));
        assert_eq!(Store::parse_id("12"), None);
        assert_eq!(Store::parse_id("m0"), None);
        assert_eq!(Store::parse_id("abc"), None);
        assert_eq!(Store::format_id(12), "m12");
    }
}
