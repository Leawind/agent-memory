SELECT mt.memory_id, s.name, g.name FROM memory_tags mt
JOIN tags s ON s.id = mt.tag_id
LEFT JOIN tags g ON g.id = mt.target_tag_id
ORDER BY mt.memory_id, s.name
