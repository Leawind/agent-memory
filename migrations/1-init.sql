-- 基线 schema（迁移 1）。
--
-- 发布前允许破坏性更改：直接改写本文件即可，旧数据库删掉重建（项目未发布，
-- 不保留旧数据）。发布后任何 schema 变更都必须新增 `2-xxx.sql`、`3-xxx.sql`
-- 等新迁移文件，绝不改写已发布的迁移——迁移运行器按 `PRAGMA user_version`
-- 记录已应用数量，逐个事务执行，保证每个迁移恰好应用一次。

-- tags：标签自带代理主键 id（系统内部属性，对 MCP 使用者不可见）。
-- 标签名唯一非空，是对外的唯一标识；改名只动 name，id 恒定，
-- 记忆与标签的关联因此不受改名影响。

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
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

-- memory_tags 只存 tag id，不冗余标签名（改名零成本、不留陈旧名）。

CREATE TABLE IF NOT EXISTS memory_tags (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    original INTEGER NOT NULL DEFAULT 1 CHECK (original IN (0, 1)),
    derived INTEGER NOT NULL DEFAULT 0 CHECK (derived IN (0, 1)),
    CHECK (original = 1 OR derived = 1),
    PRIMARY KEY (memory_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_memory_tags_tag_id ON memory_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_memories_updated_at ON memories(updated_at);

CREATE TABLE IF NOT EXISTS memory_lifecycle (
    memory_id INTEGER PRIMARY KEY REFERENCES memories(id) ON DELETE CASCADE,
    expires_at INTEGER CHECK (expires_at >= 0),
    archived_at INTEGER CHECK (archived_at >= 0),
    pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1))
);

-- Access scoring version 1 has a fixed 30-day half-life. Hourly compaction keeps
-- sufficient statistics for this scoring version; original timestamps beyond retention
-- cannot be replayed under an arbitrary new decay formula.
CREATE TABLE IF NOT EXISTS memory_access_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    actor TEXT NOT NULL,
    event_key TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('read', 'use')),
    occurred_at INTEGER NOT NULL CHECK (occurred_at >= 0),
    weight REAL NOT NULL CHECK (weight >= 0),
    UNIQUE (memory_id, actor, event_key)
);
CREATE INDEX IF NOT EXISTS idx_access_events_time ON memory_access_events(occurred_at, id);
CREATE INDEX IF NOT EXISTS idx_access_events_credit ON memory_access_events(memory_id, actor, kind, occurred_at);

CREATE TABLE IF NOT EXISTS memory_access_projection (
    memory_id INTEGER PRIMARY KEY REFERENCES memories(id) ON DELETE CASCADE,
    score REAL NOT NULL CHECK (score >= 0),
    as_of INTEGER NOT NULL CHECK (as_of >= 0),
    reads INTEGER NOT NULL CHECK (reads >= 0),
    uses INTEGER NOT NULL CHECK (uses >= 0)
);

CREATE TABLE IF NOT EXISTS memory_access_buckets (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    bucket_end INTEGER NOT NULL,
    score REAL NOT NULL CHECK (score >= 0),
    reads INTEGER NOT NULL CHECK (reads >= 0),
    uses INTEGER NOT NULL CHECK (uses >= 0),
    PRIMARY KEY (memory_id, bucket_end)
);

-- identities：访问身份（token 即身份，无账号/注册/登录——由操作者经 Web UI
-- 或 CLI 签发）。permissions 为能力清单 JSON（如 {"read":true,"admin":false,...}），
-- 键集合由 src/auth.rs 的登记表校验，未知键在写入前被拒绝。token 只存 SHA-256
-- 哈希与尾缀提示（token 是能力凭据，与记忆数据是不同的泄露面——库文件被
-- 备份/误传时凭据不外泄），明文仅在创建/重置响应中出现一次。

CREATE TABLE IF NOT EXISTS identities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    token_hash TEXT NOT NULL,
    token_hint TEXT NOT NULL,
    permissions TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_identities_token_hash ON identities(token_hash);

CREATE TABLE IF NOT EXISTS tag_predicates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    namespace TEXT NOT NULL,
    owner_id INTEGER REFERENCES identities(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    predicate TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    CHECK ((owner_id IS NULL AND namespace IN ('global','local','anonymous'))
        OR (owner_id IS NOT NULL AND namespace = 'identity:' || owner_id)),
    UNIQUE (namespace, name)
);

-- memory_embeddings：语义搜索的向量缓存（派生数据，可随时用补跑重建）。
-- 每条记忆 × 每个模型 ID 一行：model 列存用户指定的稳定 ID，
-- 指纹由 settings 中的 embedding_cache:<ID> 记录。不同 ID 的缓存互不覆盖，由运行时
-- 选中的身份决定使用哪一份。正文变更时按 memory_id 整体失效（所有身份的
-- 缓存同时作废，各自补跑）。vec 为 f32 小端字节序列。
-- 外键级联：删除记忆时该记忆的全部向量随之消失。

CREATE TABLE IF NOT EXISTS memory_embeddings (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    model TEXT NOT NULL,
    dim INTEGER NOT NULL,
    vec BLOB NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (memory_id, model)
);

CREATE INDEX IF NOT EXISTS idx_memory_embeddings_model ON memory_embeddings(model);

-- settings：键值设置表。key = 'instructions' 存自定义 initialize 提示词；
-- key = 'auth_required' 存 token 鉴权开关（"true"/"false"，缺省关闭——
-- 鉴权边界由显式开关决定，与是否已创建身份无关）。
-- key = 'embedding_models' / 'rerank_models' 存语义搜索与重排的模型配置
-- （有序 JSON 数组，每项含 enabled 开关与 OpenAI 兼容端点的 base_url / model /
-- api_key；api_key 为服务端秘密，明文存放于本表）。

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
