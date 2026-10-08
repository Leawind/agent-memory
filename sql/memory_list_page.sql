SELECT m.id, m.summary, m.content, m.created_at, m.updated_at
FROM memories m LEFT JOIN memory_lifecycle l ON l.memory_id = m.id
WHERE (?1 IS NULL OR EXISTS (SELECT 1 FROM memory_tags mt
              WHERE mt.memory_id = m.id AND mt.tag_id = (SELECT id FROM tags WHERE name = ?1)))
  AND (?2 IS NULL OR m.id IN (SELECT value FROM json_each(?2)))
  AND CASE ?7
    WHEN 'all' THEN 1
    WHEN 'archived' THEN l.archived_at IS NOT NULL
    WHEN 'expired' THEN l.archived_at IS NULL AND l.expires_at IS NOT NULL AND l.expires_at <= ?8
    ELSE l.archived_at IS NULL AND (l.expires_at IS NULL OR l.expires_at > ?8)
  END
ORDER BY CASE ?3 WHEN 'id' THEN m.id ELSE m.updated_at END * ?4,
         m.id * ?4
LIMIT ?5 OFFSET ?6
