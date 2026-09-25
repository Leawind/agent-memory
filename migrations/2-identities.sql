-- 迁移 2：多身份 token 鉴权 + 服务器设置。
--
-- identities：访问身份（token 即身份，无账号/注册/登录——由操作者经 Web UI
-- 或 CLI 签发）。permissions 为能力清单 JSON（如 {"read":true,"admin":false,...}），
-- 键集合由 src/auth.rs 的登记表校验，未知键在写入前被拒绝。token 明文存储：
-- 数据库泄露即记忆全泄露，哈希不增值，换来管理界面可随时查看复制。
--
-- settings：键值设置表。key = 'instructions' 存自定义 initialize 提示词。
--
-- 语义约定：identities 表为空 = 无鉴权开放模式（个人本地部署零配置）；
-- 存在任何身份后，/mcp 与 /api 请求都必须携带 Authorization: Bearer <token>。

CREATE TABLE IF NOT EXISTS identities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    token TEXT NOT NULL UNIQUE,
    permissions TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
