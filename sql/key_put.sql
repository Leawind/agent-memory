INSERT INTO tag_keys(name, key_type, multi, description, created_at)
VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(name) DO UPDATE SET
    key_type = excluded.key_type,
    multi = excluded.multi,
    description = excluded.description
