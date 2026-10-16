INSERT INTO memory_lifecycle(memory_id, expires_at, pinned) VALUES (?1, ?2, ?3)
ON CONFLICT(memory_id) DO UPDATE SET expires_at=excluded.expires_at, pinned=excluded.pinned
