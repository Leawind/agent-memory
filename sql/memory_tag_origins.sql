SELECT mt.tag_id, s.name, mt.target_tag_id, g.name FROM memory_tags mt
JOIN tags s ON s.id = mt.tag_id
LEFT JOIN tags g ON g.id = mt.target_tag_id
WHERE mt.memory_id = ?1
ORDER BY s.name
