SELECT t.name, t.description,
       (SELECT COUNT(DISTINCT memory_id) FROM memory_tags
        WHERE tag_id = t.id OR target_tag_id = t.id)
FROM tags t
WHERE t.name = ?1
