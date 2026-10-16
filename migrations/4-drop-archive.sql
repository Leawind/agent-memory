-- 归档机制移除：生命周期只剩到期时间与置顶，记忆状态只有 active / expired 两种。
-- archived_at 列挂着 CHECK 约束，SQLite 拒绝直接 DROP COLUMN，按重建表方式删除；
-- 已归档记忆随之恢复为 active（另有到期时间的照常按到期判定）。

CREATE TABLE memory_lifecycle_new (
    memory_id INTEGER PRIMARY KEY REFERENCES memories(id) ON DELETE CASCADE,
    expires_at INTEGER CHECK (expires_at >= 0),
    pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1))
);

INSERT INTO memory_lifecycle_new (memory_id, expires_at, pinned)
    SELECT memory_id, expires_at, pinned FROM memory_lifecycle;

DROP TABLE memory_lifecycle;
ALTER TABLE memory_lifecycle_new RENAME TO memory_lifecycle;
