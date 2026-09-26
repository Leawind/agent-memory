SELECT COUNT(*) FROM memories m
WHERE (?1 IS NULL OR EXISTS (SELECT 1 FROM memory_tags mt
              WHERE mt.memory_id = m.id AND mt.tag_name = ?1))
  AND (?2 IS NULL OR EXISTS (SELECT 1 FROM memory_tags mt
              WHERE mt.memory_id = m.id AND mt.tag_name IN (SELECT value FROM json_each(?2))))
