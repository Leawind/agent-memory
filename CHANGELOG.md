# Changelog

## Unreleased

- **工具**：标签视图瘦身（`tag_list`、`tag_create`/`tag_update` 回显的 `tag`、`memory://tags/{tag}` 目录元数据）——字段改为 `name` / `count`（原 `memory_count`），空 `description` 整个省略，`reserved` 仅在保留标签上以 `true` 出现；删除行级 `created_at` / `last_used_at`（时间戳占 token、不直观且参考价值低）与响应级 `total_tags`（与数组长度重复）/ `total_memories`（总量归 stats）。破坏性更改：管理界面标签表去掉两列时间列，mock API 与 sdk-compat-check 同步新形状；记忆摘要时间戳不动（面板按其排序）
- **export**：导出 JSON 精简重构——`tags` / `memories` 改为以 id 为键的对象（tag 键 = 内部自增 id 十进制串，记忆键 = `m<N>`），记忆对标签的引用改为按 tag id 数组；去掉 `total_memories` / `total_tags` / `memory_count` / `last_used_at` 等派生信息（均可由导出文件本身推出），只保留主数据；CLI 与 REST 导出一律紧凑 JSON（单行）。破坏性更改：`import` 只接受新格式，旧版数组形状直接拒绝；引用导出中不存在的 tag id、非法 id 键、文件内重名标签均报错；目标库必须为空、id 重新编号、记忆时间戳按导出值保留的语义不变
- **鉴权**：身份 token 改为 `sk_` 前缀 + 62 位小写十六进制（共 65 字符；SQLite `hex()` 输出大写，生成语句用 `lower()` 收敛），随机性仍全部来自 SQLite `randomblob`；库内按哈希比对，已签发的旧 token 继续有效，重置后获得新格式
- **schema**：标签改用自增内部 id 关联记忆（`memory_tags(memory_id, tag_id)`），标签名唯一非空仍是对外唯一标识；id 是系统内部属性，对 MCP 使用者不可见，改名只动标签行、引用自动跟随。破坏性更改：基线迁移直接改写，旧库不兼容（schema 指纹校验会拒绝打开并给出 export/import 恢复指引）
- **修复**：`Store::open` 新增 schema 指纹校验——`user_version` 只证明应用过几个迁移，不证明表真的存在/形状正确（就地改写基线的历史库曾以 `no such table: memory_embeddings` 的形式在 `memory_update` 深处爆炸）；现在打开时即把迁移文本声明的表与列和实测比对，不符则指名道姓地拒绝
- **工具**：`tag_rename` 更名 `tag_update`（name + 可选 new_name/description，至少其一），响应新增 `renamed` / `description_updated` 布尔；`memory_create` / `memory_update` 回传 `tags_autocreated` / `tags_reused` / `tags_missing_description` 三分类；裸数字 id 进 `invalid_ids` 不再与"不存在"混淆（`memory_delete` 此前会把不可解析 id 同时报进 `deleted` 与 `missing`），`memory_update` 对非法 id 回 400 并写明 `m{n}` 格式，schema 加 `pattern` 约束；时间戳单位（epoch 秒 UTC）写进契约与服务端 instructions

## 0.1.0 (2026-09-25)

首个发布版本。

- **定位**：自托管 MCP 记忆服务器，多个 agent（不同项目/会话/机器）通过同一 URL 共享一个记忆库，内置 Vue3 Web 管理界面
- **传输**：纯 HTTP——agent 走 MCP Streamable HTTP（`POST /mcp`，无状态 JSON 模式，协议版本 2024-11-05 / 2025-03-26 / 2025-06-18；非 JSON Content-Type 按规范回 415），人走浏览器（`/`）；`GET /health` 探活；Origin 校验防 DNS rebinding；已与官方 TypeScript SDK（`@modelcontextprotocol/sdk`）联调通过，兼容性脚本见 `scripts/`
- **存储**：单文件 SQLite（rusqlite bundled，WAL + busy_timeout + synchronous=NORMAL），`tags` / `memories` / `memory_tags` 三表外键级联（改名同步引用、删标签摘引用/连带删记忆）；事务读写分离——只读请求走 DEFERRED 快照（由工具契约的 readOnlyHint 派生），写请求走 IMMEDIATE 写锁，多 agent 边写边读互不阻塞；每个请求单事务，错误或 panic 自动回滚
- **SQL 外置与迁移机制**：业务 SQL 全部在 `sql/` 目录（每条语句一个文件，构建期嵌入，Rust 代码零 SQL）；schema 迁移脚本在 `migrations/` 目录（build.rs 编译期生成清单），运行器按 `PRAGMA user_version` 逐个事务应用恰好一次；发布前允许破坏性更改（改写基线、删库重建），发布后只新增迁移文件即可平滑升级旧库
- **跨平台**：数据库文件格式平台无关，同一份 .db 可在 Windows / Linux / macOS 间复制（停服后）；`export` 子命令提供 JSON 备份
- **CLI**：clap 实现，`serve`（默认，`--host`/`--port`/`--db` 全部命令行参数，无配置文件）+ 只读子命令 `stats` / `doctor`（问题退出码 1）/ `export`（拒绝覆盖已有文件）/ `import`（从备份恢复，要求目标库为空）
- **工具**：10 个 MCP 工具——标签 CRUD（唯一名称 + 可选描述 ≤500 字符、仅大小写冲突提示、detach/purge 两种删除）、记忆 CRUD、渐进式披露（list/search 不泄露正文）、关键词 AND 搜索（中文子串命中、引号短语、命中次数与整词加权排序、HTML 转义片段）、重复摘要与拼写错误防呆提示；REST API 与 MCP 工具共用同一套 handler；工具响应上下文友好——文本通道紧凑 JSON、structuredContent 按协商协议版本（MCP-Protocol-Version 头）裁剪、search 不回显 query、翻页省略 hint、零结果给放宽建议
- **管理界面**：Vue3 + Element Plus + Vite + TypeScript（中文），记忆搜索/过滤/分页/编辑、标签管理与删除模式选择、统计/体检/导出/导入；记忆正文推荐 Markdown，编辑对话框与详情抽屉均支持渲染/源码切换，渲染前经 DOMPurify 消毒；构建产物入库并由 rust-embed 嵌入，单二进制交付
- **质量**：56 个 Rust 单元测试（含 util/迁移/体检） + 6 个端到端测试（MCP 全流程、REST CRUD、静态 UI、双进程并发零丢失、export→import 往返、CLI 子命令）+ 19 个前端测试（组件挂载冒烟、查询串组装、API 封装、Markdown 渲染与消毒），clippy 零告警，rustfmt 统一格式；MSRV 1.85（依赖树实际要求，CI 有专用 job 校验）；搜索片段的大小写折叠偏移问题已修复（逐字符折叠并映射回原文字节偏移）
