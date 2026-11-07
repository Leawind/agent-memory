SELECT id, summary, content, created_at, updated_at FROM memories
WHERE id IN (SELECT value FROM json_each(?1)) ORDER BY id
