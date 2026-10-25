SELECT DISTINCT id FROM (
    SELECT tag_id AS id FROM memory_tags
    UNION
    SELECT target_tag_id AS id FROM memory_tags WHERE target_tag_id IS NOT NULL
) WHERE id NOT IN (SELECT id FROM tags)
ORDER BY id
