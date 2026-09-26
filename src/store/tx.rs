//! 事务纪律的入口：所有数据库访问都经 `with_db_in` 在单事务内执行。

use std::path::Path;

use super::Store;

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
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

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
}
