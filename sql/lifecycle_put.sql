INSERT INTO memory_lifecycle(memory_id, kind, expires_at, archived_at, pinned) VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(memory_id) DO UPDATE SET kind=excluded.kind, expires_at=excluded.expires_at,
archived_at=excluded.archived_at, pinned=excluded.pinned
