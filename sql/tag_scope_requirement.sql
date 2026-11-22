-- The scope a tag belongs to and whether that scope is required on every write (no row: bare tag).
SELECT s.name AS scope, (s.required_since IS NOT NULL)
FROM tags t
JOIN tag_scopes s ON s.id = t.scope_id
WHERE t.name = ?1
