-- 标签带作用域：值是标签名的一部分（`scope::value`），scope 成为独立实体。
--
-- 旧模型把值升格为词条，一条边是（源词条, 目标词条）两行引用；同一记忆对同一源词条只有一行
-- （主键是 memory_id + tag_id），所以一个源端在一条记忆上本来就只有一个取值。新模型里一条
-- 记忆的标签就是一个名字：裸标签 `rust`，带作用域 `system::pc1sw0`，同一 scope 在一条记忆上
-- 至多一个。于是“这条记忆在某个维度上有没有取值”由名字本身回答，不再依赖另一个词条的在场。
--
-- 迁移规则（确定性，不丢数据）：
--   1. 被用作边源端、或被标为必填的名字成为 scope，并带走该词条的描述与必填生效时间；
--   2. 每条边成为带作用域的标签 `<源>::<目标>`，标签描述沿用旧目标词条的描述；
--   3. 旧词条若不再被任何记忆使用（它只是旧模型里“值”或“源端”的载体），随迁移删除；
--      仍被以裸标签使用的名字保留为新模型里的普通标签。
--
-- 源词条在新模型里是 scope 名，它可能仍有一条裸标签（旧模型允许同一个名字在不同记忆上扮演
-- 不同角色）。这种标签保留：scope 与标签是两个命名空间，同一名字可以同时出现在两边。

CREATE TABLE IF NOT EXISTS tag_scopes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    required_since INTEGER
);

INSERT OR IGNORE INTO tag_scopes (name, description, created_at, required_since)
SELECT s.name, s.description, s.created_at, s.required_since
FROM tags s
WHERE s.required_since IS NOT NULL
   OR EXISTS (
        SELECT 1 FROM memory_tags mt
        WHERE mt.tag_id = s.id AND mt.target_tag_id IS NOT NULL
   );

CREATE TABLE tags_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    scope_id INTEGER REFERENCES tag_scopes(id) ON DELETE CASCADE,
    description TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL
);

INSERT INTO tags_new (id, name, scope_id, description, created_at)
SELECT id, name, NULL, description, created_at FROM tags;

INSERT OR IGNORE INTO tags_new (name, scope_id, description, created_at)
SELECT s.name || '::' || t.name,
       sc.id,
       t.description,
       CAST(strftime('%s', 'now') AS INTEGER)
FROM memory_tags mt
JOIN tags s ON s.id = mt.tag_id
JOIN tags t ON t.id = mt.target_tag_id
JOIN tag_scopes sc ON sc.name = s.name
WHERE mt.target_tag_id IS NOT NULL;

CREATE TABLE memory_tags_new (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags_new(id) ON DELETE CASCADE,
    PRIMARY KEY (memory_id, tag_id)
);

INSERT OR IGNORE INTO memory_tags_new (memory_id, tag_id)
SELECT memory_id, tag_id FROM memory_tags WHERE target_tag_id IS NULL;

INSERT OR IGNORE INTO memory_tags_new (memory_id, tag_id)
SELECT mt.memory_id, lbl.id
FROM memory_tags mt
JOIN tags s ON s.id = mt.tag_id
JOIN tags t ON t.id = mt.target_tag_id
JOIN tags_new lbl ON lbl.name = s.name || '::' || t.name
WHERE mt.target_tag_id IS NOT NULL;

-- Names that only carried a role in the old model (an edge source or a value) and that nothing
-- carries as a bare tag any more stop being vocabulary entries.
DELETE FROM tags_new
WHERE NOT EXISTS (
    SELECT 1 FROM memory_tags_new mt WHERE mt.tag_id = tags_new.id
);

DROP TABLE memory_tags;
DROP TABLE tags;
ALTER TABLE tags_new RENAME TO tags;
ALTER TABLE memory_tags_new RENAME TO memory_tags;
