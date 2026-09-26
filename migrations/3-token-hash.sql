-- 迁移 3：token 改为哈希存储。
--
-- 明文 token 是能力凭据（未来可用），与记忆数据是不同的泄露面：库文件被
-- 备份/误传时，明文 token 让攻击者还能继续冒用线上库。改为只存 SHA-256
-- （token 为 256 位随机数，熵足够，无需慢哈希），token 明文仅在创建/重置
-- 响应中出现一次；列表只留尾缀提示（token_hint，供辨认身份）。
--
-- SQLite 无法计算 SHA-256，旧库的明文迁移分两步：
--   1. 本迁移：明文暂存到 identities_legacy_token，identities 只留空哈希；
--   2. 打开数据库时（src/store.rs）把暂存明文逐一哈希回填，随后清空暂存表
--      （表保留但恒为空，避免每次打开都做 DDL）。
-- 新建库第 2 步是空转。token_hint 由 SQL 侧从明文截尾缀，无需暂存。

CREATE TABLE identities_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    token_hash TEXT NOT NULL,
    token_hint TEXT NOT NULL,
    permissions TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

-- 未迁移行的明文暂存（升级完成后恒为空）
CREATE TABLE identities_legacy_token (
    id INTEGER PRIMARY KEY,
    token TEXT NOT NULL
);

INSERT INTO identities_legacy_token SELECT id, token FROM identities;
INSERT INTO identities_new
    SELECT id, name, '', substr(token, -4), permissions, created_at FROM identities;

DROP TABLE identities;
ALTER TABLE identities_new RENAME TO identities;

-- 哈希唯一性用部分索引表达：空串（待回填）不参与唯一约束
CREATE UNIQUE INDEX identities_token_hash ON identities(token_hash) WHERE token_hash <> '';
