-- 迁移 6：向量缓存按记忆分片。长正文按结构切成多片、各自嵌入，检索时按记忆取最佳片；
-- 主键加入 chunk 序号，使同一记忆的多片共存。片序号从 0 连续分配，重新嵌入时整条替换。
--
-- 向量是派生数据：重建表即清空全部缓存，正文无损，随后由后台回填或
-- `embed-backfill` 按新结构重新生成。

DROP TABLE IF EXISTS memory_embeddings;

CREATE TABLE memory_embeddings (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    model TEXT NOT NULL,
    chunk INTEGER NOT NULL,
    dim INTEGER NOT NULL,
    vec BLOB NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (memory_id, model, chunk)
);

CREATE INDEX IF NOT EXISTS idx_memory_embeddings_model ON memory_embeddings(model);
