INSERT INTO memory_embeddings(memory_id, model, chunk, dim, vec, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)
ON CONFLICT(memory_id, model, chunk) DO UPDATE SET
    dim = excluded.dim,
    vec = excluded.vec,
    updated_at = excluded.updated_at
