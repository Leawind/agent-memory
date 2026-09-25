# agent-memory

自托管 MCP 记忆服务器：**一个记忆库服务所有 agent**，多个 AI agent（ZCode / Claude Desktop / Cursor 等）通过网络访问同一份数据，并内置 Vue3 Web 管理界面供人直接浏览与管理。

- **传输**：纯 HTTP——agent 走 MCP Streamable HTTP（`/mcp`），人走浏览器（`/`）
- **存储**：单文件 SQLite（WAL 模式），多 agent / 多进程并发安全
- **跨平台**：SQLite 文件格式平台无关，同一份 `.db` 可在 Windows / Linux / macOS 间直接复制
- **单二进制**：管理界面构建产物嵌入二进制，`cargo build` 即得完整交付物

## 快速开始

```bash
# 构建并部署到 bin/（Windows 为 agent-memory.exe，其他平台为 agent-memory）
./build.sh

# 启动（默认 127.0.0.1:8899，数据库在 ~/.agent-memory/memory.db）
./bin/agent-memory.exe        # Linux/macOS: ./bin/agent-memory
```

打开 `http://127.0.0.1:8899/` 即是管理界面；让 agent 的 MCP 配置指向 `http://127.0.0.1:8899/mcp` 即接入同一记忆库。

常用命令行参数（无配置文件，全部走参数）：

| 参数 | 默认值 | 说明 |
|---|---|---|
| `--host` | `127.0.0.1` | 监听地址；跨机器共享用 `0.0.0.0`（仅限可信网络） |
| `--port` | `8899` | 监听端口 |
| `--db` | `~/.agent-memory/memory.db` | SQLite 数据库文件路径 |

子命令：`serve`（默认）/ `stats` / `doctor`（体检，有问题退出码 1）/ `export <file>`（导出 JSON 备份，拒绝覆盖已有文件）/ `import <file>`（从备份恢复，要求目标库为空）。

## 接入 MCP 客户端（多 agent 共享）

```json
"agent-memory": { "type": "http", "url": "http://127.0.0.1:8899/mcp" }
```

所有客户端指向同一 URL 即共享同一记忆库，用标签体系区分项目。并发安全由 SQLite WAL + busy_timeout 保证，多进程、多连接同时读写不会互相覆盖。

> ⚠️ 服务器无鉴权：任何能连上该端口的程序都能读写记忆库。跨机器共享时（`--host 0.0.0.0`）务必只暴露给可信网段。内置 Origin 校验防浏览器 DNS rebinding。

## 功能

10 个 MCP 工具（与管理界面共用同一套业务逻辑）：

- **标签管理**：`tag_create` / `tag_list` / `tag_rename` / `tag_delete`——标签有唯一名称 + 可选描述（≤500 字符），分类体系由 agent 自主维护；改名级联同步所有引用；删除分 `detach`（摘引用）与 `purge`（连带删记忆）
- **记忆管理**：`memory_create` / `memory_update` / `memory_delete`
- **渐进式披露**：`memory_list` / `memory_search` 只返回 id + 标签 + 摘要 + 片段；`memory_get` 才取回完整正文，最大限度节省上下文
- **Markdown 正文**：记忆正文推荐用 Markdown 书写（工具描述中已向 agent 声明），管理界面编辑时可预览渲染结果，详情抽屉默认渲染、可切回源码；渲染前经 DOMPurify 消毒，多 agent 共写的恶意内容不会在管理界面执行
- **关键词搜索**：空格分词、全部词命中（AND），引号短语要求逐字相邻；大小写不敏感子串匹配，中文无需分词；加权排序——标签 > 摘要 > 正文，命中次数（封顶）与 ASCII 整词命中加权，片段窗口自动选覆盖词最多的位置；片段经 HTML 转义
- **防呆提示**：重复摘要 → `duplicate_of`；仅大小写不同的标签 → `similar_existing`；拼错参数名 → 立即报错并列出合法参数

## 管理界面

- **记忆管理**：搜索、标签过滤、排序、分页、新建/编辑（标签回车即建、正文 Markdown 编辑/预览切换）、全文抽屉（渲染/源码切换）、删除确认
- **标签管理**：表格 + 新建/编辑（名称唯一、描述限长）+ 删除时选择 detach/purge
- **运维**：统计卡片、doctor 体检、一键导出 JSON 备份

![记忆管理](docs/ui-memories.png)

![标签管理](docs/ui-tags.png)

![运维](docs/ui-ops.png)

界面源码在 `ui/`（Vue3 + Element Plus + Vite + TypeScript），构建产物 `ui/dist` 提交入库并由 rust-embed 编译期嵌入。JS 侧（`ui/` 与 `scripts/`）由根 package.json 的 npm workspaces 统一管理，根目录一次安装：

```bash
npm install          # 首次（或依赖变更后）
npm run build        # 修改前端后重新构建（等价 npm run build -w ui）
./build.sh           # 再重新编译嵌入
```

## 性能包络

本机（Windows，2026-09 实测，keep-alive 连接、纯服务端往返）300 条记忆时：

- 单次创建 ~23 ms（含每请求开库 + `BEGIN IMMEDIATE` 事务提交）
- 关键词搜索 ~10 ms、分页浏览 ~11 ms（search/list 需载入全量记忆到内存评分）

写入成本主要来自每请求的连接建立与事务提交，随条目数近似线性（搜索/列表全量加载）——数千条以内完全无感；上万条仍可用，届时优先做标签清理与归档，而不是改存储引擎。

## 数据与跨平台

- 数据保存在单个 SQLite 文件（默认 `~/.agent-memory/memory.db`），表结构：`tags` / `memories` / `memory_tags`（双外键级联）
- **Schema 迁移**：迁移脚本在 `migrations/` 目录（`NUM-NAME.sql`），构建时嵌入二进制；运行器按 `PRAGMA user_version` 逐个事务应用、恰好一次。发布前允许破坏性更改（改写基线脚本、删库重建）；发布后只新增迁移文件，旧数据库自动平滑升级
- **跨平台迁移**：SQLite 文件格式平台无关。服务器停止后（WAL 会自动合并回主文件）直接复制 `.db` 即可换机使用；运行中迁移或需要可读格式时，用 `export` + `import` 完成"导出 → 恢复"闭环
- **数据安全**：每个请求在单个 `BEGIN IMMEDIATE` 事务内执行，错误或 panic 时连接关闭自动回滚，磁盘数据保持原样；主文件损坏由 SQLite 自身页校验防护
- **SQL 管理**：全部业务 SQL 外置在 `sql/` 目录（每条语句一个文件），构建期嵌入——Rust 代码中不出现 SQL 文本，参数一律绑定占位符

## 架构

```
src/
  main.rs        clap CLI（serve/stats/doctor/export/import）
  http.rs        HTTP 层：/mcp + /api/* + 静态 UI + /health，多 worker 线程
  protocol.rs    MCP 协议层（JSON-RPC：initialize / ping / tools/*，批量与通知）
  tools/
    mod.rs       工具入口：单事务"执行→提交"，未知参数校验
    defs.rs      工具清单 + JSON Schema（对 agent 的契约，参数校验的权威来源）
    params.rs    参数解析与校验
    tag_ops.rs   标签增删查改
    memory_ops.rs 记忆增删改查、浏览、搜索
  store.rs       SQLite 持久化（WAL / 迁移运行器 / 级联 / 体检 / 统计 / 导出）
  sql.rs         SQL 登记表（include_str! 嵌入 sql/ 目录）
  search.rs      关键词搜索与评分（内存内，语义与存储无关）
  model.rs       纯数据模型（Memory / 归一化 / API 限制常量）
migrations/      Schema 迁移脚本（build.rs 编译期生成 MIGRATIONS 数组）
sql/             业务 SQL（每条语句一个文件）
ui/              Vue3 + Element Plus + Vite + TypeScript 管理界面（dist 入库嵌入）
```

## 测试

```bash
cargo test    # 56 个单元测试 + 6 个端到端测试（真实进程走 HTTP，覆盖 MCP / REST / 静态 UI / 双进程并发 / CLI）
npm test      # 管理界面测试（19 个：组件挂载冒烟、查询串组装、API 封装、Markdown 渲染与消毒）
npm run typecheck   # TypeScript 类型检查（覆盖 ui/ 与 scripts/ 两个 workspace）
```

另有与官方 MCP TypeScript SDK 的兼容性联调脚本（开发用，需 node）：

```bash
./bin/agent-memory.exe &                # 先起服务器
node scripts/sdk-compat-check.ts
```

## 工具一览

| 工具 | 只读 | 说明 |
|---|---|---|
| `tag_create(name, description?)` | | 新建标签，重名报错；存在仅大小写不同的标签时在 `similar_existing` 里提示（非阻塞） |
| `tag_list()` | ✓ | 全部标签 + 描述 + 记忆计数 + 最近使用时间 |
| `tag_rename(old_name, new_name?, description?)` | | 重命名（级联同步所有引用）/ 改描述 |
| `tag_delete(name, mode?)` | | `detach`（默认，只摘引用）/ `purge`（连带删除记忆） |
| `memory_create(summary, content, tags?)` | | 新建记忆；未知标签自动创建并在 `tags_autocreated` 汇报；摘要重复时提示 `duplicate_of` |
| `memory_list(tag?, sort?, order?, offset?, limit?)` | ✓ | 分页浏览，只返回摘要 |
| `memory_search(query, tags?, offset?, limit?)` | ✓ | 关键词搜索，返回摘要 + 正文片段，支持翻页 |
| `memory_get(ids)` | ✓ | 取 1–50 条完整正文（渐进式披露第二层） |
| `memory_update(id, summary?, content?, add_tags?, remove_tags?)` | | 增量更新，无需先取当前标签列表 |
| `memory_delete(ids)` | | 永久删除 1–50 条 |

## 推荐的 agent 使用方式

1. **存**：`memory_create`，摘要写得精确自洽（未来扫描全靠它），标签自选自管
2. **查**：先 `memory_search` / `memory_list`（便宜），再对值得读的 id 调 `memory_get`（贵）
3. **管**：定期 `tag_list` 检查分类体系，用 `tag_rename` 纠错，过期记忆用 `memory_delete` 清理
