-- 已缓存向量的记忆条数（一条记忆多片只算一条），与待补跑计数同一量纲。
SELECT COUNT(DISTINCT memory_id) FROM memory_embeddings WHERE model = ?1
