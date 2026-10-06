//! Entry point of transaction discipline: all database access runs inside a single transaction via `with_db_in`.

use std::path::Path;

use super::Store;

/// Transaction modes: read-only requests use DEFERRED (a consistent snapshot under WAL without grabbing
/// the write lock; reads never block reads or writes), write requests use IMMEDIATE (the write lock is taken
/// up front, and busy_timeout queues concurrent writers — avoiding the DEFERRED read-then-upgrade deadlock).
pub enum TxMode {
    ReadOnly,
    Write,
}

/// Use the database inside a single transaction: rollback on panic or error, commit only on success.
///
/// The transaction mode is `TxMode`. The error type is generic (`E: From<String>`) so callers can pick their
/// own error type (e.g. the tool layer's `ToolError`); infrastructure errors
/// (open/begin/commit) are converted from String. On error or panic the function returns early
/// or unwinds, and closing the connection rolls back automatically — relying on the "commit only on success" ordering.
///
/// Every call opens a fresh connection, which also re-runs migrations and the schema fingerprint
/// check. That is a deliberate tradeoff, not an oversight: at this service's scale (one writer at
/// a time, personal-server request rates) the per-request open cost is noise, and in exchange
/// every request is self-healing (a schema drifted out from under a running process is refused
/// immediately instead of blowing up mid-operation). Revisit with a connection pool only if
/// profiling ever demands it.
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
            let ids = st.link_tags(&["t".into()], true)?.ids;
            st.insert_memory("s", "c", &ids, 1, 1)?;
            Err("boom".into())
        });
        assert!(r.is_err());
        let st = Store::open(&path).unwrap();
        assert_eq!(
            st.stats().unwrap()["memories"],
            0,
            "rollback must remove the memory"
        );
        // Only the seeded reserved tag remains: the transaction's own tag rolled back
        assert_eq!(
            st.stats().unwrap()["tags"],
            1,
            "rollback must remove the tag"
        );
        assert_eq!(
            st.tag_count_unreserved().unwrap(),
            0,
            "rollback must remove the tag"
        );
        cleanup(&path);
    }
}
