-- (tag name, scope name) of the given tag ids: the per-scope uniqueness check of one write.
-- Name-ordered so a rejection always reports the same pair.
SELECT t.name AS tag, s.name AS scope
FROM tags t
JOIN tag_scopes s ON s.id = t.scope_id
WHERE t.id IN (SELECT value FROM json_each(?1))
ORDER BY t.name
