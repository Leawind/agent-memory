SELECT id, memory_id, kind, occurred_at, weight FROM memory_access_events WHERE occurred_at<?1 ORDER BY occurred_at, id LIMIT 10000
