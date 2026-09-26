-- 待补跑 = 没有当前模型向量的记忆（缺行或模型指纹过期），id 升序取一批。
SELECT m.id, m.summary, m.content
FROM memories m
LEFT JOIN memory_embeddings e ON e.memory_id = m.id AND e.model = ?1
WHERE e.memory_id IS NULL
ORDER BY m.id
LIMIT ?2
