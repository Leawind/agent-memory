SELECT m.id, m.summary, m.content, m.created_at, m.updated_at
FROM memories m
WHERE (?1 IS NULL OR EXISTS (SELECT 1 FROM memory_tags mt
              JOIN tags t ON t.id = mt.tag_id
              WHERE mt.memory_id = m.id AND t.name = ?1)
                  OR EXISTS (SELECT 1 FROM memory_tags mt
              JOIN tags t ON t.id = mt.target_tag_id
              WHERE mt.memory_id = m.id AND t.name = ?1))
  AND (?2 IS NULL OR m.id IN (SELECT value FROM json_each(?2)))
ORDER BY CASE ?3 WHEN 'id' THEN m.id ELSE m.updated_at END * ?4,
         m.id * ?4
LIMIT ?5 OFFSET ?6
