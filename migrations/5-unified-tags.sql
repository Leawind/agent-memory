-- 标签统一：值升格为词汇表词条，关系行从（键, 值文本）改为（源端词条, 目标端词条）。
--
-- memory_tags.value（自由文本）由 target_tag_id（对 tags 的引用）取代：NULL 即裸标签，
-- 非 NULL 即一条从 tag_id 指向 target_tag_id 的边。tags 表从此是统一命名空间，
-- 不再区分键与值两种角色——角色由各条记忆上的关系行决定，一个词条可以在
-- 不同记忆上扮演不同角色。
--
-- 模型不变式（应用层校验，本迁移把旧数据也归整到它上面）：同一条记忆上，
-- 被指向的词条不得拥有自身行（目标端与源端/裸状态互斥）。旧数据中三类冲突
-- 按确定性规则消解：
--   1. 值与既有标签同名 → 复用同一词条（tags.name 唯一，跨键强制合一）；
--   2. 同一记忆上词条既作目标端又作源端（嵌套）→ 删指向它的边，保留其源端行；
--   3. 同一记忆上词条既作目标端又作裸标签 → 删裸行，在场由边保证。

-- 1. 值升格：出现过的每个值都成为词条（已有同名词条时忽略，即复用）。
INSERT OR IGNORE INTO tags (name, description, created_at)
SELECT DISTINCT value, '', CAST(strftime('%s', 'now') AS INTEGER)
FROM memory_tags
WHERE value IS NOT NULL;

-- 2. 重建关系表：值文本换成对目标词条的引用。
CREATE TABLE memory_tags_new (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    target_tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (memory_id, tag_id)
);

INSERT INTO memory_tags_new (memory_id, tag_id, target_tag_id)
SELECT memory_id,
       tag_id,
       CASE WHEN value IS NULL THEN NULL
            ELSE (SELECT id FROM tags WHERE name = value) END
FROM memory_tags;

-- 3. 嵌套消解：指向"在同一条记忆上拥有源端行"的词条的边删除。
--    自环（值与键同名）没有独立的源端行可保留，同样在此删除。
--    受影响行先在 CTE 中物化，删除集合由单一快照决定——互指边对（a→b 且 b→a）
--    两条都在受害者集合内，结果确定（全部消解），不受行处理顺序影响。
DELETE FROM memory_tags_new
WHERE rowid IN (
    WITH sources AS (
        SELECT DISTINCT memory_id, tag_id FROM memory_tags_new
        WHERE target_tag_id IS NOT NULL
    )
    SELECT edge.rowid
    FROM memory_tags_new AS edge
    JOIN sources ON sources.memory_id = edge.memory_id
                AND sources.tag_id = edge.target_tag_id
    WHERE edge.target_tag_id IS NOT NULL
);

-- 4. 裸行消解：在同一条记忆上作目标端的词条的裸行删除（在场由边保证）。
DELETE FROM memory_tags_new AS bare
WHERE bare.target_tag_id IS NULL
  AND EXISTS (
      SELECT 1 FROM memory_tags_new AS edge
      WHERE edge.memory_id = bare.memory_id
        AND edge.target_tag_id = bare.tag_id
  );

DROP TABLE memory_tags;
ALTER TABLE memory_tags_new RENAME TO memory_tags;
CREATE INDEX idx_memory_tags_tag_id ON memory_tags(tag_id);
CREATE INDEX idx_memory_tags_target_tag_id ON memory_tags(target_tag_id);
