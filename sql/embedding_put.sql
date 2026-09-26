INSERT INTO memory_embeddings(memory_id, model, dim, vec, updated_at)
VALUES (?1, ?2, ?3, ?4, ?5)
ON CONFLICT(memory_id) DO UPDATE SET
    model = excluded.model,
    dim = excluded.dim,
    vec = excluded.vec,
    updated_at = excluded.updated_at
