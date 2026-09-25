SELECT t.name, t.description, COUNT(mt.memory_id), MAX(m.updated_at)
FROM tags t
LEFT JOIN memory_tags mt ON mt.tag_name = t.name
LEFT JOIN memories m ON m.id = mt.memory_id
WHERE t.name = ?1
GROUP BY t.name
