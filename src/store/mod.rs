//! SQLite persistence: a single database file (WAL mode).
//!
//! The database file format is platform-independent (SQLite handles byte order and alignment itself), so the same .db
//! file can be copied directly between Windows / Linux / macOS.
//!
//! **All SQL lives outside Rust**: business statements are in the repo-root `sql/` directory (see `crate::sql`),
//! and schema migrations in the `migrations/` directory (build.rs generates `MIGRATIONS` at compile time;
//! the directory is the single source of truth — adding a migration is just dropping in `NUM-NAME.sql` and recompiling).
//! The migration runner tracks progress via `PRAGMA user_version`, applying each migration in its own transaction
//! so each applies exactly once. Before release, breaking changes are allowed (rewriting the baseline, dropping and rebuilding the database);
//! after release, only new migration files may be added.
//!
//! This layer only stores and retrieves data plus a few aggregate queries — no argument validation; see `tools::params` for that,
//! and `model` for the pure data model. Every request executes inside one transaction (see `with_db`);
//! on panic or error the transaction rolls back and the data stays untouched.
//!
//! Split by domain: `tags` / `memories` / `identities` / `settings` /
//! `embeddings` each carry their data operations as `impl Store`; `ops` holds the aggregate queries (health check / statistics /
//! export/import), `tx` is the transaction entry point, and `migrate` is the migration runner.

mod embeddings;
mod identities;
mod lifecycle;
mod memories;
mod migrate;
mod ops;
mod settings;
mod tag_rules;
mod tags;
mod tx;

pub use memories::ListFilter;
pub use tx::{with_db_in, TxMode};

use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use crate::sql;

const BUSY_TIMEOUT_MS: u64 = 5000;

// Schema migrations embedded from the `migrations/` directory at build time
// (see `build.rs`), which is the single source of truth.
include!(concat!(env!("OUT_DIR"), "/migrations.rs"));

pub struct Store {
    pub path: PathBuf,
    pub conn: Connection,
}

/// Default location of the database file (the current working directory; the --db flag overrides it).
/// The intended usage is for the user to run maintenance from a fixed directory: the data sits in the startup directory, visible and under control.
/// All subcommands share this default — maintenance commands must run in the same directory as serve, or pass --db explicitly.
pub fn default_path() -> PathBuf {
    PathBuf::from("memory.db")
}

/// Relative paths are anchored to the current working directory, so stats / the ops panel always show an unambiguous path.
pub fn normalize_path(p: &Path) -> PathBuf {
    if p.is_absolute() {
        return p.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(p))
        .unwrap_or_else(|_| p.to_path_buf())
}

impl Store {
    /// Open the database: create directories, set WAL/busy_timeout/foreign keys, run migrations.
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
        // WAL: reads and writers do not exclude each other; writers queue via SQLite's own locking + busy_timeout.
        // When the last connection closes cleanly, SQLite checkpoints automatically, leaving a single portable .db file.
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| format!("cannot enable WAL mode: {e}"))?;
        // The recommended WAL level: an application crash loses nothing, only a power cut may lose the latest transaction (never corrupts the database),
        // in exchange for no forced fsync on every commit — a clear win for this service's one-write-per-request pattern.
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(|e| format!("cannot set synchronous mode: {e}"))?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(|e| format!("cannot enable foreign keys: {e}"))?;
        migrate::run_migrations(&conn)?;
        verify_schema(&conn)?;
        // The reserved tag is a permanent fixture of the store: seed it at open so existence
        // checks (tag_expr validation, did-you-mean, resource reads) never depend on whether
        // an admin has created it yet. INSERT OR IGNORE keeps admin-customized rows intact.
        conn.execute(
            sql::TAG_SEED_RESERVED,
            rusqlite::params![
                crate::model::RESERVED_TAG,
                crate::model::RESERVED_TAG_DESCRIPTION,
                crate::model::now() as i64
            ],
        )
        .map_err(|e| format!("cannot seed the reserved tag: {e}"))?;
        Ok(Store {
            path: normalize_path(path),
            conn,
        })
    }

    /// Boundary conversion between the integer id and the API string ("m3").
    pub fn format_id(id: i64) -> String {
        format!("m{id}")
    }

    /// Accepts only the "m{n}" format (n a positive integer); the m-prefix-less form is not tolerated and returns None.
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

/// Expected schema: parsed from the embedded migration texts (lowercased table name → list of lowercased column names),
/// cached for the process lifetime — the migration texts are fixed at compile time, so the parse result never changes.
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

/// Parse the table and column names of a CREATE TABLE (including the IF NOT EXISTS form) from a SQL snippet.
/// Only needs to cover the style used by this project's migrations: strip `--` line comments first; take the table name up to `(`;
/// take the first identifier of each top-level comma segment as a column definition, skipping constraint sections (PRIMARY KEY etc.).
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
        // Column definition body: match parentheses until the closing ')'
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

/// Split on commas at bracket depth 0 (degenerates to plain splitting when column definitions contain no nested brackets).
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

/// Schema fingerprint check: user_version only records "how many migrations were applied" — it does not prove the tables actually exist
/// (a database whose migrations were rewritten in place has both numbers right, yet tables may be missing or stale). On open, the tables and columns
/// declared by the migration texts are compared against what is actually there, and any mismatch refuses to open — turning the
/// "no such table" that would otherwise blow up deep in operation into a named startup failure, with a recovery path.
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

    /// Fingerprint check: a database whose user_version is right but tables are missing must refuse to open, with an error that names names
    /// (turning the "no such table" that would blow up deep in operation into an open-time failure).
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

    /// Fingerprint check: stale-shaped tables (an old schema linking by name, without an id column) are refused too,
    /// with the error listing the missing and extra column names.
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
