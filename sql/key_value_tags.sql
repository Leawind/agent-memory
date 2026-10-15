SELECT name FROM tags
WHERE substr(name, 1, length(?1) + 1) = ?1 || ':'
ORDER BY name
