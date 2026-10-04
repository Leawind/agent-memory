SELECT t.name, t.description, COUNT(mt.memory_id)
FROM tags t
LEFT JOIN memory_tags mt ON mt.tag_id = t.id
GROUP BY t.id
ORDER BY COUNT(mt.memory_id) DESC, t.name ASC
