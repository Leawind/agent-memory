-- 指定记忆的待补跑行（没有当前模型的向量），连标题与正文一起取回，供创建时的语义查重提示
-- 精确补嵌这一条，而不必依赖后台回填的进度。
SELECT m.id, m.summary, m.content
FROM memories m
WHERE m.id = ?2
  AND NOT EXISTS (
    SELECT 1 FROM memory_embeddings e WHERE e.memory_id = m.id AND e.model = ?1
  )
