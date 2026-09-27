SELECT m.id, m.summary, m.content, m.created_at, m.updated_at
FROM memories m
WHERE (?1 IS NULL OR EXISTS (SELECT 1 FROM memory_tags mt
              WHERE mt.memory_id = m.id AND mt.tag_id = (SELECT id FROM tags WHERE name = ?1)))
  AND (?2 IS NULL OR EXISTS (SELECT 1 FROM memory_tags mt
              WHERE mt.memory_id = m.id AND mt.tag_id IN (SELECT value FROM json_each(?2))))
ORDER BY CASE ?3 WHEN 'created_at' THEN m.created_at WHEN 'id' THEN m.id ELSE m.updated_at END * ?4,
         m.id * ?4
LIMIT ?5 OFFSET ?6
