SELECT t.name, t.description, COUNT(mt.memory_id), MAX(m.updated_at), t.created_at
FROM tags t
LEFT JOIN memory_tags mt ON mt.tag_id = t.id
LEFT JOIN memories m ON m.id = mt.memory_id
GROUP BY t.id
ORDER BY COUNT(mt.memory_id) DESC, t.name ASC
