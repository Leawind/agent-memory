SELECT DISTINCT memory_id FROM memory_tags
WHERE memory_id NOT IN (SELECT id FROM memories)
ORDER BY memory_id
