UPDATE memories
SET summary = COALESCE(?1, summary),
    content = COALESCE(?2, content),
    updated_at = ?3
WHERE id = ?4
