SELECT memory_id FROM memory_tags
WHERE tag_id = (SELECT id FROM tags WHERE name = ?1)
