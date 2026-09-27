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
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
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
        verify_schema(&conn)?;
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

/// 期望 schema：从嵌入的迁移文本解析（表名小写 → 列名小写列表），
/// 进程内缓存一份——迁移文本编译期固定，解析结果不会变。
fn expected_schema() -> &'static HashMap<String, Vec<String>> {
    static EXPECTED: OnceLock<HashMap<String, Vec<String>>> = OnceLock::new();
    EXPECTED.get_or_init(|| {
        let mut tables = HashMap::new();
        for source in MIGRATIONS {
            parse_create_tables(source, &mut tables);
        }
        tables
    })
}

/// 从一段 SQL 里解析 CREATE TABLE（含 IF NOT EXISTS 形式）的表名与列名。
/// 只需覆盖本项目迁移的写法：先去掉 `--` 行注释；表名取到 `(` 为止；
/// 列定义取每个顶层逗号段的首个标识符，约束段（PRIMARY KEY 等）跳过。
fn parse_create_tables(sql_text: &str, out: &mut HashMap<String, Vec<String>>) {
    let mut cleaned = String::with_capacity(sql_text.len());
    for line in sql_text.lines() {
        match line.find("--") {
            Some(pos) => cleaned.push_str(&line[..pos]),
            None => cleaned.push_str(line),
        }
        cleaned.push('\n');
    }
    let upper = cleaned.to_ascii_uppercase();
    let mut cursor = 0usize;
    while let Some(found) = upper[cursor..].find("CREATE TABLE") {
        let start = cursor + found;
        let mut name_start = start + "CREATE TABLE".len();
        if upper[name_start..].starts_with(" IF NOT EXISTS") {
            name_start += " IF NOT EXISTS".len();
        }
        let Some(paren) = cleaned[name_start..].find('(') else {
            break;
        };
        let paren = name_start + paren;
        let table_name = cleaned[name_start..paren]
            .trim()
            .trim_matches('"')
            .to_ascii_lowercase();
        // 列定义体：括号配对到收口的 ')'
        let bytes = cleaned.as_bytes();
        let mut depth = 1usize;
        let mut end = paren + 1;
        while end < bytes.len() && depth > 0 {
            match bytes[end] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        let body = &cleaned[paren + 1..end.saturating_sub(1)];
        let mut columns = Vec::new();
        for part in split_top_level(body) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let first = part.split_whitespace().next().unwrap_or_default();
            if matches!(
                first.to_ascii_uppercase().as_str(),
                "CONSTRAINT" | "PRIMARY" | "UNIQUE" | "CHECK" | "FOREIGN"
            ) {
                continue;
            }
            columns.push(first.trim_matches('"').to_ascii_lowercase());
        }
        out.insert(table_name, columns);
        cursor = end;
    }
}

/// 按括号深度 0 的逗号切分（列定义里不含嵌套括号时退化为普通切分）。
fn split_top_level(body: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    for ch in body.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut current));
            }
            _ => current.push(ch),
        }
    }
    parts.push(current);
    parts
}

/// schema 指纹校验：user_version 只记录"应用了几个迁移"，不证明表真的存在
/// （就地改写过迁移的库两数值都对，表却可能缺失或形状陈旧）。打开时把迁移
/// 文本声明的表与列和实测比对，不符即拒绝打开——把"操作深处才爆的
/// no such table"提前变成指名道姓的启动失败，并给出恢复路径。
fn verify_schema(conn: &Connection) -> Result<(), String> {
    const GUIDANCE: &str = "the file was created by an incompatible version; export your data with the matching older build, then import into a fresh database";
    let expected = expected_schema();
    let actual_tables: HashSet<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok())
            .filter(|name| !name.starts_with("sqlite_"))
            .collect()
    };
    for (table, columns) in expected.iter() {
        if !actual_tables.contains(table) {
            return Err(format!(
                "database schema mismatch: expected table '{table}' is missing ({GUIDANCE})"
            ));
        }
        let actual_columns: HashSet<String> = {
            let mut stmt = conn
                .prepare(&format!("PRAGMA table_info({table})"))
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(1))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|r| r.ok()).collect()
        };
        let want: HashSet<String> = columns.iter().cloned().collect();
        let missing: Vec<_> = want.difference(&actual_columns).collect();
        let unexpected: Vec<_> = actual_columns.difference(&want).collect();
        if !missing.is_empty() || !unexpected.is_empty() {
            return Err(format!(
                "database schema mismatch: table '{table}' columns differ from the current schema \
                 (missing: {missing:?}, unexpected: {unexpected:?}) ({GUIDANCE})"
            ));
        }
    }
    Ok(())
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

    /// 指纹校验：user_version 对但表缺失的库必须拒绝打开，且错误指名道姓
    /// （把"操作深处才爆的 no such table"提前成打开失败）。
    #[test]
    fn schema_fingerprint_rejects_missing_table() {
        let path = temp_db("fingerprint");
        cleanup(&path);
        {
            let st = Store::open(&path).unwrap();
            st.conn
                .execute_batch("DROP TABLE memory_embeddings")
                .unwrap();
        }
        let Err(err) = Store::open(&path) else {
            panic!("opening a database with a missing table must fail");
        };
        assert!(
            err.contains("memory_embeddings") && err.contains("missing"),
            "got: {err}"
        );
        cleanup(&path);
    }

    /// 指纹校验：形状陈旧的表（旧 schema 按名关联、无 id 列）同样拒绝，
    /// 错误列出缺失与多余的列名。
    #[test]
    fn schema_fingerprint_rejects_stale_table_shape() {
        let path = temp_db("fingerprint-stale");
        cleanup(&path);
        {
            let st = Store::open(&path).unwrap();
            st.conn
                .execute_batch(
                    "DROP TABLE memory_tags;
                     CREATE TABLE memory_tags (
                         memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
                         tag_name TEXT NOT NULL,
                         PRIMARY KEY (memory_id, tag_name)
                     );",
                )
                .unwrap();
        }
        let Err(err) = Store::open(&path) else {
            panic!("opening a database with a stale table shape must fail");
        };
        assert!(
            err.contains("memory_tags") && err.contains("tag_id") && err.contains("tag_name"),
            "got: {err}"
        );
        cleanup(&path);
    }
}
