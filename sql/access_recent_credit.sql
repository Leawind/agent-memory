SELECT EXISTS(SELECT 1 FROM memory_access_events WHERE memory_id=?1 AND actor=?2 AND kind=?3 AND occurred_at>=?4 AND weight>0)
