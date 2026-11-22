SELECT DISTINCT tag_id AS id FROM memory_tags
WHERE tag_id NOT IN (SELECT id FROM tags)
ORDER BY id
