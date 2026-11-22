-- One row per scope: memories carrying any of its tags, and how many tags it holds.
SELECT s.name, s.description,
       (SELECT COUNT(DISTINCT mt.memory_id) FROM memory_tags mt
        JOIN tags t ON t.id = mt.tag_id
        WHERE t.scope_id = s.id),
       (SELECT COUNT(*) FROM tags t WHERE t.scope_id = s.id),
       (s.required_since IS NOT NULL)
FROM tag_scopes s
ORDER BY 3 DESC, s.name ASC
