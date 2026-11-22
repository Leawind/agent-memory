SELECT t.name, t.description,
       (SELECT COUNT(DISTINCT memory_id) FROM memory_tags WHERE tag_id = t.id),
       s.name AS scope
FROM tags t
LEFT JOIN tag_scopes s ON s.id = t.scope_id
WHERE t.name = ?1
