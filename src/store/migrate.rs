//! schema 迁移运行器：`migrations/` 目录经 build.rs 生成 `MIGRATIONS`，
//! 这里按 `PRAGMA user_version` 恰好应用一次。

use rusqlite::Connection;

use super::MIGRATIONS;

/// 迁移运行器：`PRAGMA user_version` 记录已应用的迁移数量，每个待应用迁移
/// 在独立事务中执行并推进 user_version，保证恰好应用一次。数据库比已知
/// 迁移更新（来自更新版本的程序）时拒绝打开，绝不带着未知的 schema 写数据。
pub(super) fn run_migrations(conn: &Connection) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;
    use crate::store::Store;

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
}
