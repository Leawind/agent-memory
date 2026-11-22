SELECT name FROM tag_scopes
WHERE required_since IS NOT NULL
ORDER BY name
