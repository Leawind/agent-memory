-- 迁移 2：memory_embeddings 改为 (memory_id, model) 复合主键，多个嵌入模型的
-- 向量缓存得以共存（多模型配置 + 按模型补跑/删除的前提）。
--
-- 向量是派生数据：重建表即清空全部缓存，正文毫发无损，随后由后台回填或
-- `embed-backfill` 按新 schema 重新生成。旧库（user_version = 1）在此被原位
-- 升级，避免为一次派生数据表的结构调整要求操作者导出重建；基线（迁移 1）
-- 已直接写为新形状，本迁移对全新数据库是等价重建（空表，无实际损失）。

DROP TABLE IF EXISTS memory_embeddings;

CREATE TABLE memory_embeddings (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    model TEXT NOT NULL,
    dim INTEGER NOT NULL,
    vec BLOB NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (memory_id, model)
);

CREATE INDEX IF NOT EXISTS idx_memory_embeddings_model ON memory_embeddings(model);
