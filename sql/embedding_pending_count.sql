SELECT COUNT(*) FROM memories m
LEFT JOIN memory_embeddings e ON e.memory_id = m.id AND e.model = ?1
WHERE e.memory_id IS NULL
