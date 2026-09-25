# Changelog

## 0.1.0 (2026-09-25)

首个发布版本。

- **定位**：自托管 MCP 记忆服务器，多个 agent（不同项目/会话/机器）通过同一 URL 共享一个记忆库，内置 Vue3 Web 管理界面
- **传输**：纯 HTTP——agent 走 MCP Streamable HTTP（`POST /mcp`，无状态 JSON 模式，协议版本 2024-11-05 / 2025-03-26 / 2025-06-18；非 JSON Content-Type 按规范回 415），人走浏览器（`/`）；`GET /health` 探活；Origin 校验防 DNS rebinding；已与官方 TypeScript SDK（`@modelcontextprotocol/sdk`）联调通过，兼容性脚本见 `scripts/`
- **存储**：单文件 SQLite（rusqlite bundled，WAL + busy_timeout + synchronous=NORMAL），`tags` / `memories` / `memory_tags` 三表外键级联（改名同步引用、删标签摘引用/连带删记忆）；事务读写分离——只读请求走 DEFERRED 快照（由工具契约的 readOnlyHint 派生），写请求走 IMMEDIATE 写锁，多 agent 边写边读互不阻塞；每个请求单事务，错误或 panic 自动回滚
- **SQL 外置与迁移机制**：业务 SQL 全部在 `sql/` 目录（每条语句一个文件，构建期嵌入，Rust 代码零 SQL）；schema 迁移脚本在 `migrations/` 目录（build.rs 编译期生成清单），运行器按 `PRAGMA user_version` 逐个事务应用恰好一次；发布前允许破坏性更改（改写基线、删库重建），发布后只新增迁移文件即可平滑升级旧库
- **跨平台**：数据库文件格式平台无关，同一份 .db 可在 Windows / Linux / macOS 间复制（停服后）；`export` 子命令提供 JSON 备份
- **CLI**：clap 实现，`serve`（默认，`--host`/`--port`/`--db` 全部命令行参数，无配置文件）+ 只读子命令 `stats` / `doctor`（问题退出码 1）/ `export`（拒绝覆盖已有文件）/ `import`（从备份恢复，要求目标库为空）
- **工具**：10 个 MCP 工具——标签 CRUD（唯一名称 + 可选描述 ≤500 字符、仅大小写冲突提示、detach/purge 两种删除）、记忆 CRUD、渐进式披露（list/search 不泄露正文）、关键词 AND 搜索（中文子串命中、加权排序）、重复摘要与拼写错误防呆提示；REST API 与 MCP 工具共用同一套 handler
- **管理界面**：Vue3 + Element Plus + Vite + TypeScript（中文），记忆搜索/过滤/分页/编辑、标签管理与删除模式选择、统计/体检/导出/导入；记忆正文推荐 Markdown，编辑对话框与详情抽屉均支持渲染/源码切换，渲染前经 DOMPurify 消毒；构建产物入库并由 rust-embed 嵌入，单二进制交付
- **质量**：56 个 Rust 单元测试（含 util/迁移/体检） + 6 个端到端测试（MCP 全流程、REST CRUD、静态 UI、双进程并发零丢失、export→import 往返、CLI 子命令）+ 19 个前端测试（组件挂载冒烟、查询串组装、API 封装、Markdown 渲染与消毒），clippy 零告警，rustfmt 统一格式；MSRV 1.85（依赖树实际要求，CI 有专用 job 校验）；搜索片段的大小写折叠偏移问题已修复（逐字符折叠并映射回原文字节偏移）
