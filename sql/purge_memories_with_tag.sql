DELETE FROM memories
WHERE id IN (SELECT mt.memory_id FROM memory_tags mt
             JOIN tags t ON t.id = mt.tag_id
             WHERE t.name = ?1)
