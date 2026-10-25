SELECT COUNT(*) FROM memories m
WHERE (?1 IS NULL OR EXISTS (SELECT 1 FROM memory_tags mt
              JOIN tags t ON t.id = mt.tag_id
              WHERE mt.memory_id = m.id AND t.name = ?1)
                  OR EXISTS (SELECT 1 FROM memory_tags mt
              JOIN tags t ON t.id = mt.target_tag_id
              WHERE mt.memory_id = m.id AND t.name = ?1))
  AND (?2 IS NULL OR m.id IN (SELECT value FROM json_each(?2)))
