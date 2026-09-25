DELETE FROM memories WHERE id IN (SELECT memory_id FROM memory_tags WHERE tag_name = ?1)
