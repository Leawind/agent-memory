-- 标签统一为键值对：关系表携带值列（NULL 即裸标签），键词汇与值分离。
-- 旧存储把整个键值对拼在 tags.name 一列（'key:value'），本迁移做原地拆分；
-- original / derived 两列与 CHECK 约束是已移除的标签派生系统的遗物（恒为 1/0），
-- 一并删除；tag_keys 键注册表已无读写方，随之删除。

CREATE TABLE memory_tags_new (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    value TEXT,
    PRIMARY KEY (memory_id, tag_id)
);

-- 键词汇：组合名的键半段提升为独立词汇行（值不进词汇表）。
INSERT INTO tags (name, description, created_at)
SELECT substr(name, 1, instr(name, ':') - 1), '', created_at
FROM tags
WHERE instr(name, ':') > 0
  AND substr(name, 1, instr(name, ':') - 1) NOT IN (SELECT name FROM tags);

-- 关系行：组合名关系改写为（键引用，值）。
INSERT INTO memory_tags_new (memory_id, tag_id, value)
SELECT mt.memory_id,
       (SELECT k.id FROM tags k
        WHERE k.name = substr(t.name, 1, instr(t.name, ':') - 1)),
       substr(t.name, instr(t.name, ':') + 1)
FROM memory_tags mt
JOIN tags t ON t.id = mt.tag_id
WHERE instr(t.name, ':') > 0;

-- 裸标签关系原样保留；同键已有值行时跳过（一键一值）。
INSERT INTO memory_tags_new (memory_id, tag_id, value)
SELECT mt.memory_id, mt.tag_id, NULL
FROM memory_tags mt
JOIN tags t ON t.id = mt.tag_id
WHERE instr(t.name, ':') = 0
  AND NOT EXISTS (
      SELECT 1 FROM memory_tags_new n
      WHERE n.memory_id = mt.memory_id AND n.tag_id = mt.tag_id
  );

DROP TABLE memory_tags;
ALTER TABLE memory_tags_new RENAME TO memory_tags;
CREATE INDEX idx_memory_tags_tag_id ON memory_tags(tag_id);

-- 组合名行已拆分完毕，从词汇表移除。
DELETE FROM tags WHERE instr(name, ':') > 0;

DROP TABLE IF EXISTS tag_keys;
