# Changelog

## Unreleased

- **工具**：`memory_search` hybrid 模式先按余弦截断语义通道（约 2×limit、下限 20）再做 RRF 融合，`total_matches` 不再恒等于库总量；hybrid 时响应新增 `keyword_matches` 行，标明多少命中来自字面关键词、多少仅由语义通道带入
- **schema（破坏性）**：标签改用内部自增 id 关联记忆；基线迁移改写，旧库不兼容，打开时做 schema 指纹校验、不符即拒绝（按提示 export/import 迁移）
- **工具（破坏性）**：`tag_rename` 更名 `tag_update`；响应全面去噪——空字段整个省略、恒真布尔删除；摘要不再携带 `created_at`
- **工具**：对模型暴露的时间戳改为本地墙钟的紧凑人读格式（存储与导出仍是 epoch 秒）；标签视图字段瘦身
- **export / import（破坏性）**：导出重构为紧凑对象结构、只保留主数据；`import` 只接受新格式
- **鉴权**：身份 token 改为 `sk_` 前缀的新格式，已签发旧 token 重置后更换

## 0.1.0 (2026-09-25)

首个版本：多 agent 经 HTTP 共享单个 SQLite 记忆库的 MCP 服务器，内置 Vue3 管理界面；可选 token 鉴权与语义搜索，单二进制交付。详见 [README](README.md)。
