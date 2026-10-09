INSERT INTO memory_lifecycle(memory_id, expires_at, archived_at, pinned) VALUES (?1, ?2, ?3, ?4)
ON CONFLICT(memory_id) DO UPDATE SET expires_at=excluded.expires_at, archived_at=excluded.archived_at, pinned=excluded.pinned
