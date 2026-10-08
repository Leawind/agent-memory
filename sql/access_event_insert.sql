INSERT OR IGNORE INTO memory_access_events(memory_id, actor, event_key, kind, occurred_at, weight)
VALUES (?1, ?2, COALESCE(?3, lower(hex(randomblob(16)))), ?4, ?5, ?6)
