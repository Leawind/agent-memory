SELECT DISTINCT tag_id FROM memory_tags
WHERE tag_id NOT IN (SELECT id FROM tags)
ORDER BY tag_id
