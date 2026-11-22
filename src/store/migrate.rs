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
                    "ALTER TABLE identities DROP COLUMN tag;
                     DROP TABLE memory_tags;
                     DROP TABLE tags;
                     DROP TABLE tag_scopes;
                     CREATE TABLE tags (
                         id INTEGER PRIMARY KEY AUTOINCREMENT,
                         name TEXT NOT NULL UNIQUE,
                         description TEXT NOT NULL DEFAULT '',
                         created_at INTEGER NOT NULL
                     );
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
        assert_eq!(version, super::MIGRATIONS.len() as i64);
        // After the scoped-tags migration a surviving edge is one tag name, and the promoted
        // value word is no longer a role of its own.
        let names = |memory: i64| -> Vec<String> {
            let mut names: Vec<String> = st
                .tag_rows_for(memory)
                .unwrap()
                .into_iter()
                .map(|r| r.name)
                .collect();
            names.sort();
            names
        };
        assert_eq!(
            names(1),
            vec!["project::shared".to_string(), "topic::shared".to_string()]
        );
        assert_eq!(names(2), vec!["topic::shared".to_string()]);
        assert!(names(3).is_empty());
        assert!(names(4).is_empty());
        assert!(st.scope_exists("project").unwrap());
        assert!(st.scope_exists("topic").unwrap());
        cleanup(&path);
    }

    /// Migration 8 (scoped tags) end to end: a database in the pre-scope shape gets a scope for
    /// every name used as an edge source and for every required name, each edge becomes one
    /// scoped tag carrying the former value's description, bare tags survive, and vocabulary
    /// entries that only carried a role in the old model disappear.
    #[test]
    fn migration_8_turns_edges_into_scoped_tags() {
        let path = temp_db("migration-8");
        cleanup(&path);
        {
            let st = Store::open(&path).unwrap();
            st.conn
                .execute_batch(
                    "ALTER TABLE identities DROP COLUMN tag;
                     DROP TABLE memory_tags;
                     DROP TABLE tags;
                     DROP TABLE tag_scopes;
                     CREATE TABLE tags (
                         id INTEGER PRIMARY KEY AUTOINCREMENT,
                         name TEXT NOT NULL UNIQUE,
                         description TEXT NOT NULL DEFAULT '',
                         created_at INTEGER NOT NULL,
                         required_since INTEGER
                     );
                     CREATE TABLE memory_tags (
                         memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
                         tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                         target_tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
                         PRIMARY KEY (memory_id, tag_id)
                     );
                     INSERT INTO tags(name, description, created_at, required_since) VALUES
                         ('system', 'the box a fact is about', 1, 10),
                         ('rust', 'language', 1, NULL),
                         ('pc1', 'workstation', 1, NULL),
                         ('lp2', '', 1, NULL),
                         ('country', 'where', 1, 20),
                         ('project', 'work', 1, NULL),
                         ('nmmt', 'the mod', 1, NULL);
                     INSERT INTO memories(summary, content, created_at, updated_at) VALUES
                         ('m1', '', 1, 1), ('m2', '', 1, 1);
                     INSERT INTO memory_tags VALUES
                         (1, (SELECT id FROM tags WHERE name='system'), (SELECT id FROM tags WHERE name='pc1')),
                         (1, (SELECT id FROM tags WHERE name='rust'), NULL),
                         (1, (SELECT id FROM tags WHERE name='project'), (SELECT id FROM tags WHERE name='nmmt')),
                         (2, (SELECT id FROM tags WHERE name='system'), (SELECT id FROM tags WHERE name='lp2')),
                         (2, (SELECT id FROM tags WHERE name='nmmt'), NULL);
                     PRAGMA user_version = 7;",
                )
                .unwrap();
        }
        let st = Store::open(&path).unwrap();
        // Required names and edge sources both became scopes, each carrying the old cut-off.
        assert_eq!(
            st.required_scopes().unwrap(),
            vec!["country".to_string(), "system".to_string()]
        );
        let names = |memory: i64| -> Vec<String> {
            let mut names: Vec<String> = st
                .tag_rows_for(memory)
                .unwrap()
                .into_iter()
                .map(|r| r.name)
                .collect();
            names.sort();
            names
        };
        assert_eq!(
            names(1),
            vec![
                "project::nmmt".to_string(),
                "rust".to_string(),
                "system::pc1".to_string()
            ]
        );
        assert_eq!(
            names(2),
            vec!["nmmt".to_string(), "system::lp2".to_string()]
        );
        // The value word keeps the tag it described, so the scoped tag inherits it.
        let view = st.tag_view("system::pc1").unwrap();
        assert_eq!(view["description"], "workstation");
        // A name nothing carries any more stops being a tag; the scope keeps its description.
        assert!(!st.tag_exists("pc1").unwrap());
        assert!(!st.tag_exists("country").unwrap());
        assert!(st.tag_exists("nmmt").unwrap(), "a bare use keeps the tag");
        cleanup(&path);
    }
}
