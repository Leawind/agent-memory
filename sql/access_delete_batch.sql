DELETE FROM memory_access_events WHERE id IN (SELECT value FROM json_each(?1))
