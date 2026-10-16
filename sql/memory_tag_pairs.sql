SELECT mt.memory_id, t.name, mt.value FROM memory_tags mt
JOIN tags t ON t.id = mt.tag_id
ORDER BY mt.memory_id, t.name
