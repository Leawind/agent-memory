-- 删除一个向量身份的全部缓存行（按模型管理缓存），返回受影响行数。
DELETE FROM memory_embeddings WHERE model = ?1
