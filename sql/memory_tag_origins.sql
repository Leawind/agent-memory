SELECT t.id, t.name, mt.original, mt.derived FROM memory_tags mt
JOIN tags t ON t.id = mt.tag_id WHERE mt.memory_id = ?1 ORDER BY t.name
