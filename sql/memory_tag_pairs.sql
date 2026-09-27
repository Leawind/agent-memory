SELECT mt.memory_id, t.name FROM memory_tags mt
JOIN tags t ON t.id = mt.tag_id
ORDER BY mt.memory_id, t.name
