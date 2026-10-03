//! Central registry of SQL statements: all business SQL lives in the repo-root `sql/` directory (one
//! file per statement, file names matching constants one-to-one) and is embedded at build time via `include_str!` —
//! no SQL text ever appears in Rust code. Parameters always use bound placeholders (?N); string concatenation is off-limits.
//!
//! To add or change a statement: edit the file under `sql/` and update this module's constants in step; the
//! one-to-one correspondence is guarded by the `sql_files_are_all_registered` test.

pub const TAG_EXISTS: &str = include_str!("../sql/tag_exists.sql");
pub const TAG_CREATE: &str = include_str!("../sql/tag_create.sql");
pub const TAG_AUTOCREATE: &str = include_str!("../sql/tag_autocreate.sql");
pub const TAG_ALL_NAMES: &str = include_str!("../sql/tag_all_names.sql");
pub const TAG_BY_NAME: &str = include_str!("../sql/tag_by_name.sql");
pub const TAG_VIEW: &str = include_str!("../sql/tag_view.sql");
pub const TAG_VIEW_ALL: &str = include_str!("../sql/tag_view_all.sql");
pub const TAG_SET_NAME: &str = include_str!("../sql/tag_set_name.sql");
pub const TAG_SET_DESCRIPTION: &str = include_str!("../sql/tag_set_description.sql");
pub const TAG_DELETE: &str = include_str!("../sql/tag_delete.sql");
pub const TAG_MEMORY_COUNT: &str = include_str!("../sql/tag_memory_count.sql");
pub const TAG_EXPORT_ALL: &str = include_str!("../sql/tag_export_all.sql");

pub const MEMORY_IDS_WITH_TAG: &str = include_str!("../sql/memory_ids_with_tag.sql");
pub const PURGE_MEMORIES_WITH_TAG: &str = include_str!("../sql/purge_memories_with_tag.sql");
pub const MEMORY_INSERT: &str = include_str!("../sql/memory_insert.sql");
pub const MEMORY_LINK_TAG: &str = include_str!("../sql/memory_link_tag.sql");
pub const MEMORY_UNLINK_TAG: &str = include_str!("../sql/memory_unlink_tag.sql");
pub const MEMORY_BY_ID: &str = include_str!("../sql/memory_by_id.sql");
pub const MEMORY_EXISTS: &str = include_str!("../sql/memory_exists.sql");
pub const MEMORY_TAGS_OF: &str = include_str!("../sql/memory_tags_of.sql");
pub const MEMORY_ALL: &str = include_str!("../sql/memory_all.sql");
pub const MEMORY_TAG_PAIRS: &str = include_str!("../sql/memory_tag_pairs.sql");
pub const MEMORY_TAG_ID_PAIRS: &str = include_str!("../sql/memory_tag_id_pairs.sql");
pub const MEMORY_LIST_PAGE: &str = include_str!("../sql/memory_list_page.sql");
pub const MEMORY_LIST_COUNT: &str = include_str!("../sql/memory_list_count.sql");
pub const MEMORY_UPDATE_FIELDS: &str = include_str!("../sql/memory_update_fields.sql");
pub const MEMORY_DELETE: &str = include_str!("../sql/memory_delete.sql");
pub const MEMORY_SUMMARIES: &str = include_str!("../sql/memory_summaries.sql");

pub const EMBEDDING_PUT: &str = include_str!("../sql/embedding_put.sql");
pub const EMBEDDING_DELETE: &str = include_str!("../sql/embedding_delete.sql");
pub const EMBEDDING_ACTIVE_ALL: &str = include_str!("../sql/embedding_active_all.sql");
pub const EMBEDDING_PENDING_BATCH: &str = include_str!("../sql/embedding_pending_batch.sql");
pub const EMBEDDING_PENDING_COUNT: &str = include_str!("../sql/embedding_pending_count.sql");
pub const EMBEDDING_EMBEDDED_COUNT: &str = include_str!("../sql/embedding_embedded_count.sql");

pub const HYGIENE_ORPHANS: &str = include_str!("../sql/hygiene_orphans.sql");
pub const HYGIENE_MEMORIES: &str = include_str!("../sql/hygiene_memories.sql");

pub const IDENTITY_INSERT: &str = include_str!("../sql/identity_insert.sql");
pub const IDENTITY_BY_TOKEN: &str = include_str!("../sql/identity_by_token.sql");
pub const IDENTITY_ALL: &str = include_str!("../sql/identity_all.sql");
pub const IDENTITY_DELETE: &str = include_str!("../sql/identity_delete.sql");
pub const IDENTITY_UPDATE_PERMISSIONS: &str =
    include_str!("../sql/identity_update_permissions.sql");
pub const IDENTITY_UPDATE_TOKEN: &str = include_str!("../sql/identity_update_token.sql");
pub const IDENTITY_ANY_ADMIN: &str = include_str!("../sql/identity_any_admin.sql");
pub const SETTINGS_GET: &str = include_str!("../sql/settings_get.sql");
pub const SETTINGS_PUT: &str = include_str!("../sql/settings_put.sql");
pub const SETTINGS_DELETE: &str = include_str!("../sql/settings_delete.sql");
pub const TOKEN_GENERATE: &str = include_str!("../sql/token_generate.sql");

pub const STATS_MEMORY_COUNT: &str = include_str!("../sql/stats_memory_count.sql");
pub const STATS_TAG_COUNT: &str = include_str!("../sql/stats_tag_count.sql");
pub const STATS_HAS_SEQUENCE: &str = include_str!("../sql/stats_has_sequence.sql");
pub const STATS_NEXT_ID: &str = include_str!("../sql/stats_next_id.sql");
pub const STATS_MAX_ID: &str = include_str!("../sql/stats_max_id.sql");
pub const STATS_NEWEST: &str = include_str!("../sql/stats_newest.sql");

#[cfg(test)]
mod tests {
    use super::*;

    /// Sync guard between the sql/ directory and this module's constants: every .sql file in the directory must map,
    /// by naming convention (tag_exists.sql ↔ TAG_EXISTS), to exactly one registered constant;
    /// extra registrations, missing ones, or misspelled file names all fail this test.
    #[test]
    fn sql_files_are_all_registered() {
        let sql_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("sql");
        let mut on_disk: Vec<String> = std::fs::read_dir(&sql_dir)
            .expect("sql/ directory must exist")
            .map(|e| {
                e.expect("readable entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter(|name| name.ends_with(".sql"))
            .collect();
        on_disk.sort();

        // Convention: constant name (lowercased) + ".sql" is the file name, and the content really comes from that file.
        // Note: the sorted comparison must derive both sides the same way (uppercasing changes relative order
        // because of where '_' sits in ASCII), so constants are derived into file names before sorting against the disk listing.
        let mut expected_files: Vec<String> = vec![
            ("TAG_EXISTS", TAG_EXISTS),
            ("TAG_CREATE", TAG_CREATE),
            ("TAG_AUTOCREATE", TAG_AUTOCREATE),
            ("TAG_ALL_NAMES", TAG_ALL_NAMES),
            ("TAG_BY_NAME", TAG_BY_NAME),
            ("TAG_VIEW", TAG_VIEW),
            ("TAG_VIEW_ALL", TAG_VIEW_ALL),
            ("TAG_SET_NAME", TAG_SET_NAME),
            ("TAG_SET_DESCRIPTION", TAG_SET_DESCRIPTION),
            ("TAG_DELETE", TAG_DELETE),
            ("TAG_MEMORY_COUNT", TAG_MEMORY_COUNT),
            ("TAG_EXPORT_ALL", TAG_EXPORT_ALL),
            ("MEMORY_IDS_WITH_TAG", MEMORY_IDS_WITH_TAG),
            ("PURGE_MEMORIES_WITH_TAG", PURGE_MEMORIES_WITH_TAG),
            ("MEMORY_INSERT", MEMORY_INSERT),
            ("MEMORY_LINK_TAG", MEMORY_LINK_TAG),
            ("MEMORY_UNLINK_TAG", MEMORY_UNLINK_TAG),
            ("MEMORY_BY_ID", MEMORY_BY_ID),
            ("MEMORY_EXISTS", MEMORY_EXISTS),
            ("MEMORY_TAGS_OF", MEMORY_TAGS_OF),
            ("MEMORY_ALL", MEMORY_ALL),
            ("MEMORY_TAG_PAIRS", MEMORY_TAG_PAIRS),
            ("MEMORY_TAG_ID_PAIRS", MEMORY_TAG_ID_PAIRS),
            ("MEMORY_LIST_PAGE", MEMORY_LIST_PAGE),
            ("MEMORY_LIST_COUNT", MEMORY_LIST_COUNT),
            ("MEMORY_UPDATE_FIELDS", MEMORY_UPDATE_FIELDS),
            ("MEMORY_DELETE", MEMORY_DELETE),
            ("MEMORY_SUMMARIES", MEMORY_SUMMARIES),
            ("EMBEDDING_PUT", EMBEDDING_PUT),
            ("EMBEDDING_DELETE", EMBEDDING_DELETE),
            ("EMBEDDING_ACTIVE_ALL", EMBEDDING_ACTIVE_ALL),
            ("EMBEDDING_PENDING_BATCH", EMBEDDING_PENDING_BATCH),
            ("EMBEDDING_PENDING_COUNT", EMBEDDING_PENDING_COUNT),
            ("EMBEDDING_EMBEDDED_COUNT", EMBEDDING_EMBEDDED_COUNT),
            ("HYGIENE_ORPHANS", HYGIENE_ORPHANS),
            ("HYGIENE_MEMORIES", HYGIENE_MEMORIES),
            ("STATS_MEMORY_COUNT", STATS_MEMORY_COUNT),
            ("STATS_TAG_COUNT", STATS_TAG_COUNT),
            ("STATS_HAS_SEQUENCE", STATS_HAS_SEQUENCE),
            ("STATS_NEXT_ID", STATS_NEXT_ID),
            ("STATS_MAX_ID", STATS_MAX_ID),
            ("STATS_NEWEST", STATS_NEWEST),
            ("IDENTITY_INSERT", IDENTITY_INSERT),
            ("IDENTITY_BY_TOKEN", IDENTITY_BY_TOKEN),
            ("IDENTITY_ALL", IDENTITY_ALL),
            ("IDENTITY_DELETE", IDENTITY_DELETE),
            ("IDENTITY_UPDATE_PERMISSIONS", IDENTITY_UPDATE_PERMISSIONS),
            ("IDENTITY_UPDATE_TOKEN", IDENTITY_UPDATE_TOKEN),
            ("IDENTITY_ANY_ADMIN", IDENTITY_ANY_ADMIN),
            ("SETTINGS_GET", SETTINGS_GET),
            ("SETTINGS_PUT", SETTINGS_PUT),
            ("SETTINGS_DELETE", SETTINGS_DELETE),
            ("TOKEN_GENERATE", TOKEN_GENERATE),
        ]
        .iter()
        .map(|(name, content)| {
            let file = format!("{}.sql", name.to_ascii_lowercase());
            let expected = std::fs::read_to_string(sql_dir.join(&file))
                .unwrap_or_else(|e| panic!("sql/{file} should exist and be readable: {e}"));
            assert_eq!(
                *content, expected,
                "constant {name} does not match sql/{file} content"
            );
            file
        })
        .collect();
        expected_files.sort();

        assert_eq!(
            expected_files, on_disk,
            "sql/ directory files and src/sql.rs constants are out of sync"
        );
    }
}
