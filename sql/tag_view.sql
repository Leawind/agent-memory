SELECT t.name, t.description, COUNT(mt.memory_id)
FROM tags t
LEFT JOIN memory_tags mt ON mt.tag_id = t.id
WHERE t.name = ?1
GROUP BY t.id
