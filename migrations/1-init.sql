-- 基线 schema（迁移 1）。
--
-- 发布前允许破坏性更改：直接改写本文件即可，旧数据库删掉重建（项目未发布，
-- 不保留旧数据）。发布后任何 schema 变更都必须新增 `2-xxx.sql`、`3-xxx.sql`
-- 等新迁移文件，绝不改写已发布的迁移——迁移运行器按 `PRAGMA user_version`
-- 记录已应用数量，逐个事务执行，保证每个迁移恰好应用一次。

CREATE TABLE IF NOT EXISTS tags (
    name TEXT PRIMARY KEY,
    description TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS memories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    summary TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS memory_tags (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    tag_name TEXT NOT NULL REFERENCES tags(name) ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (memory_id, tag_name)
);

CREATE INDEX IF NOT EXISTS idx_memory_tags_tag_name ON memory_tags(tag_name);
CREATE INDEX IF NOT EXISTS idx_memories_updated_at ON memories(updated_at);
