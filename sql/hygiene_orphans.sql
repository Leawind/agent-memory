SELECT DISTINCT tag_name FROM memory_tags
WHERE tag_name NOT IN (SELECT name FROM tags)
ORDER BY tag_name
