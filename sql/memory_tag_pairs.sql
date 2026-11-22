SELECT mt.memory_id, t.name FROM memory_tags mt
JOIN tags t ON t.id = mt.tag_id
WHERE (?1 IS NULL OR mt.memory_id IN (SELECT value FROM json_each(?1)))
ORDER BY mt.memory_id, t.name
