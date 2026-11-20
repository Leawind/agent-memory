-- 重新嵌入一条记忆前清掉它现有的全部片：分片数可变，旧片不能残留成幽灵候选。
DELETE FROM memory_embeddings WHERE memory_id = ?1 AND model = ?2
