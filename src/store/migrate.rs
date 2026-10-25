//! Schema migration runner: the `migrations/` directory is compiled into `MIGRATIONS` by build.rs,
//! and each migration is applied exactly once based on `PRAGMA user_version`.

use rusqlite::Connection;

use super::MIGRATIONS;

/// Migration runner: `PRAGMA user_version` records how many migrations have been applied; each pending migration
/// runs in its own transaction and advances user_version, so each is applied exactly once. When the database
/// is newer than the known migrations (written by a newer program), opening is refused — never write with an unknown schema.
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

    /// Migration 5 (value promotion) end to end: a database in the pre-unification shape gets
    /// its values promoted to vocabulary entries, its relations rewritten to edges, and its
    /// role conflicts resolved exactly like the migration file documents.
    #[test]
    fn migration_5_promotes_values_and_resolves_role_conflicts() {
        let path = temp_db("migration-5");
        cleanup(&path);
        {
            let st = Store::open(&path).unwrap();
            // Reshape memory_tags back to the pre-unification shape (value text column) and
            // seed old-style rows, covering every documented conflict class:
            //   m1: cross-key same value → both edges share the promoted tag
            //   m2: bare + target → the bare row dies, presence survives via the edge
            //   m3: mutual nesting (each side is both source and target) → both edges die,
            //       the rule applied to its letter leaves no annotation
            //   m4: self-edge (value equals its own key) → the row dies
            st.conn
                .execute_batch(
                    "DROP TABLE memory_tags;
                     CREATE TABLE memory_tags (
                         memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
                         tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                         value TEXT,
                         PRIMARY KEY (memory_id, tag_id)
                     );
                     INSERT INTO tags(name, description, created_at) VALUES
                         ('project', '', 1), ('topic', '', 1), ('shared', '', 1);
                     INSERT INTO memories(summary, content, created_at, updated_at) VALUES
                         ('m1', '', 1, 1), ('m2', '', 1, 1), ('m3', '', 1, 1), ('m4', '', 1, 1);
                     INSERT INTO memory_tags VALUES
                         (1, (SELECT id FROM tags WHERE name='project'), 'shared'),
                         (1, (SELECT id FROM tags WHERE name='topic'), 'shared'),
                         (2, (SELECT id FROM tags WHERE name='shared'), NULL),
                         (2, (SELECT id FROM tags WHERE name='topic'), 'shared'),
                         (3, (SELECT id FROM tags WHERE name='project'), 'shared'),
                         (3, (SELECT id FROM tags WHERE name='shared'), 'project'),
                         (4, (SELECT id FROM tags WHERE name='shared'), 'shared');
                     PRAGMA user_version = 4;",
                )
                .unwrap();
        }
        let st = Store::open(&path).unwrap();
        let version: i64 = st
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 5);
        // Every former value is a vocabulary entry now, sharing its row with the same-named key
        let id_of = |name: &str| -> i64 {
            st.tag_ids_by_names(&[name.to_string()])
                .unwrap()
                .into_iter()
                .next()
                .unwrap()
        };
        assert_eq!(id_of("shared"), id_of("shared"));
        let shared = id_of("shared");
        let rows = |memory: i64| -> Vec<(i64, Option<i64>)> {
            st.tag_rows_for(memory)
                .unwrap()
                .into_iter()
                .map(|r| (r.tag_id, r.target_id))
                .collect()
        };
        let project = id_of("project");
        let topic = id_of("topic");
        // m1: both edges point at the same promoted tag
        assert_eq!(
            rows(1),
            vec![(project, Some(shared)), (topic, Some(shared))]
        );
        // m2: the bare row of the targeted tag died, the edge survived
        assert_eq!(rows(2), vec![(topic, Some(shared))]);
        // m3: both edges of the mutual nesting pair died (each targets a source)
        assert_eq!(rows(3), Vec::<(i64, Option<i64>)>::new());
        // m4: the self-edge died entirely
        assert_eq!(rows(4), Vec::<(i64, Option<i64>)>::new());
        cleanup(&path);
    }
}
