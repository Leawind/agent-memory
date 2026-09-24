# Changelog

## 0.3.0 (2026-09-25)

- 新增：`doctor` CLI 子命令——只读数据体检（孤儿标签引用、大小写冲突标签组、空摘要/正文），发现问题时退出码为 1，便于脚本化巡检
- 新增：`tag_create` 在存在仅大小写不同的既有标签时返回 `similar_existing` 与 `note`（非阻塞），引导 agent 保持分类体系整洁
- 新增：`Store::hygiene_issues()` 体检逻辑与单元测试
- CI：新增 MSRV job，在 Rust 1.82 上执行 `cargo check`，校验 `rust-version` 声明真实可用
- 新增：真并发多进程写入竞态端到端测试（两个进程同时争抢文件锁，断言零丢失）；性能包络实测并写入 README（千条记忆时单次创建 ~17ms）

## 0.2.0 (2026-09-25)

- 新增：`memory_create` 响应携带 `duplicate_of` —— 摘要归一化后与既有记忆相同时列出其 id，引导改用 `memory_update` 而非重复存储
- 新增：CLI 只读运维子命令 `stats`（数据概况）与 `export <file>`（可读 JSON 备份，拒绝覆盖已有文件）
- 新增：`AGENT_MEMORY_LOCK_WAIT_MS` 环境变量，可配置存储锁等待上限（默认 5000ms，最大 60000ms）
- 新增：`memory_search` 支持 `offset` 翻页；`memory_list` 对"标签存在但无记忆"给出提示
- 改进：未知参数名立刻报错并列出合法参数（合法名从 JSON Schema 的 properties 派生，契约与校验不会失同步）
- 修复：带 id 但缺 `method` 的请求现在返回 JSON-RPC -32600（原先被静默丢弃，客户端会挂起等待）；`params: null` 归一化为空对象；批量消息中的非对象成员逐个回 -32600
- 改进：`memory_get` 存在缺失 id 时附带引导提示（指向 memory_list / memory_search）
- 重构：模块拆分——`model.rs`（纯数据模型）+ `store.rs`（持久化）+ `tools/{defs,params,tag_ops,memory_ops}`（工具层），降低单文件复杂度
- 质量：36 个测试（31 单元 + 5 端到端，协议层新增进程内单元测试含对抗性输入）、clippy 零告警、rustfmt 统一格式

## 0.1.0 (2026-09-24)

- 首发：10 个 MCP 工具（标签增删查改、记忆增删改查、渐进式披露访问、关键词搜索）
- 单文件 JSON 存储：原子写入 + `.bak` 备份 + 损坏自动回退 + 跨进程文件锁
- stdio MCP 传输，协议版本支持 2024-11-05 / 2025-03-26 / 2025-06-18
