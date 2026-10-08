SELECT actor, event_key, kind, occurred_at, weight FROM memory_access_events
WHERE memory_id = ?1 ORDER BY occurred_at, id;
