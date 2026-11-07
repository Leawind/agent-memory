SELECT mt.memory_id, s.name, g.name FROM memory_tags mt
JOIN tags s ON s.id = mt.tag_id
LEFT JOIN tags g ON g.id = mt.target_tag_id
WHERE (?1 IS NULL OR mt.memory_id IN (SELECT value FROM json_each(?1)))
ORDER BY mt.memory_id, s.name
