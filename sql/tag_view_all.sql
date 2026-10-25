SELECT t.name, t.description,
       (SELECT COUNT(DISTINCT memory_id) FROM memory_tags
        WHERE tag_id = t.id OR target_tag_id = t.id),
       (EXISTS (SELECT 1 FROM memory_tags WHERE target_tag_id = t.id)
        AND NOT EXISTS (SELECT 1 FROM memory_tags WHERE tag_id = t.id))
FROM tags t
ORDER BY 3 DESC, t.name ASC
