SELECT m.id, m.summary, m.content, m.created_at, m.updated_at
FROM memories m
WHERE ?1 IS NULL
   OR EXISTS (SELECT 1 FROM memory_tags mt
              WHERE mt.memory_id = m.id AND mt.tag_name = ?1)
ORDER BY CASE ?2 WHEN 'created_at' THEN m.created_at ELSE m.updated_at END * ?3,
         m.id * ?3
LIMIT ?4 OFFSET ?5
