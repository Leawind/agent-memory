-- 已落库的向量身份清单（缓存管理页用）：每个身份一行 + 该身份的缓存行数。
-- 配置里存在但从未落库的身份不在其中，由调用方与配置条目取并集。
SELECT model, COUNT(*) FROM memory_embeddings GROUP BY model ORDER BY model
