# AGENTS.md — agent-memory 项目指南

自托管 MCP 记忆服务器（纯 HTTP）：多 agent 共享一个 SQLite 记忆库，内嵌 Vue3 管理界面。Rust 依赖 serde / serde_json / tiny_http / rusqlite(bundled) / clap / rust-embed。

## 常用命令

```bash
cargo test        # 全部测试（单元 + 端到端，e2e 真实启动 HTTP 服务器与 CLI 子命令）
cargo clippy --all-targets   # 提交前应零告警（Cargo.toml [lints.clippy] 已提升默认严格度）
cargo fmt --check            # 提交前格式必须通过
./build.sh        # 构建 release 并部署到 bin/agent-memory.exe
                  # ⚠️ 改完 Rust 代码必须跑，否则 bin/ 里是旧二进制
npm run build     # 仅当改了 ui/src 时需要；等价 npm run build -w ui，产物 ui/dist 入库并由 rust-embed 嵌入
```

注意：本机配置了全局共享的 CARGO_TARGET_DIR（编译产物不在 ./target），`build.sh` 会从 cargo 元数据定位真实输出目录。

JS 侧（ui/ 与 scripts/）由根 package.json 的 npm workspaces 统一管理：根目录一次
`npm install` 生成唯一 lockfile（package-lock.json），`npm run test` / `typecheck` /
`format` 等命令在根目录直接可用（内部转发到对应 workspace）。

## 模块结构

```
src/
  main.rs        clap CLI：serve(默认)/stats/doctor/export；--host/--port/--db 全部命令行参数，无配置文件
  http.rs        HTTP 传输：路由 /mcp·/api·静态 UI·/health，多 worker，Origin 防护，panic 隔离，body 上限
  api.rs         管理后端 /api/*：复用 tools handler，percent 解码，404/400 映射（业务逻辑不在此层）
  protocol.rs    MCP 协议层：initialize / ping / tools/list / tools/call，通知不回包，批量消息；
                 工具结果文本通道必须是紧凑 JSON，structuredContent 按协商版本（2025-06-18 起）附带
  tools/mod.rs   工具入口：execute_with_db（单事务），分发，未知参数校验（从 schema 派生）
  tools/defs.rs  工具清单 + JSON Schema（对 agent 的契约，唯一权威来源）
  tools/params.rs 参数解析/校验（值从严错报、写法从宽：单字符串可当数组）
  tools/tag_ops.rs / memory_ops.rs  业务处理器（校验在此，数据操作下沉到 store）
  store.rs       SQLite 持久化：WAL / 迁移运行器 / 外键级联 / 体检 / 统计 / 导出
  sql.rs         SQL 语句登记表：include_str! 嵌入 sql/ 目录，Rust 代码不出现 SQL 文本
  search.rs      关键词搜索：AND 语义（引号短语逐字相邻）、TF 封顶 + ASCII 整词加权、
                 中文子串匹配（内存内计算）、片段窗口优选；片段必须 HTML 转义（UI 以 v-html 渲染）
  model.rs       纯数据模型：Memory / id·标签名归一化 / API 限制常量
migrations/      Schema 迁移脚本（NUM-NAME.sql），build.rs 编译期生成 MIGRATIONS 数组
sql/             业务 SQL（每条语句一个文件，文件名 ↔ sql.rs 常量，同步测试把守）
ui/              Vue3 + Element Plus + Vite 管理界面；dist 提交入库（rust-embed 嵌入，cargo 构建无需 node）
                 记忆正文按 Markdown 渲染：ui/src/markdown.ts（marked + DOMPurify）→ MarkdownView.vue
```

## 架构不变量（改代码前必读）

1. **stdout 干净**：服务器模式日志/诊断只写 stderr；CLI 子命令（stats/doctor/export）的结果走 stdout。HTTP 响应只走 body。
2. **事务纪律（读写分离）**：读请求走 `TxMode::ReadOnly`（`BEGIN DEFERRED`，
   WAL 下获得一致性快照且不抢写锁，读与读/写互不阻塞）；写请求走
   `TxMode::Write`（`BEGIN IMMEDIATE`，一开始就取写锁）。写必须用 IMMEDIATE：
   DEFERRED 下并发事务"先读后升级写锁"会立即 SQLITE_BUSY（死锁场景
   busy_timeout 不重试）。模式来源：MCP 路径由 defs.rs 的 readOnlyHint 注解
   派生（`is_read_only`），REST 路径由 HTTP 方法决定（GET=只读）。
   错误或 panic 时函数提前返回/展开，连接关闭自动回滚——依赖"成功才 COMMIT"的顺序。
3. **SQL 与迁移外置**：Rust 代码里不出现 SQL 文本——业务语句在 `sql/*.sql`
   （每条一个文件，`src/sql.rs` include_str! 登记，同步测试把守），schema 在
   `migrations/NUM-NAME.sql`（build.rs 编译期生成 `MIGRATIONS`，目录即唯一事实源）。
   **迁移策略**：项目未发布，允许破坏性更改——改 schema 直接改写基线
   `1-init.sql`，旧库删掉重建；**发布后**任何 schema 变更只能新增
   `2-xxx.sql`、`3-xxx.sql` 等新迁移文件，绝不改写已发布的迁移。
   运行器按 `PRAGMA user_version` 逐个事务应用，恰好一次；数据库比已知迁移
   更新时拒绝打开（防降级写坏数据）。
4. **跨平台数据**：schema 内不得存平台相关状态（绝对路径、换行风格等）；SQLite 文件
   格式平台无关，同一份 .db 跨机复制可用（停服后复制，或用 export）。
5. **defs.rs 是契约**：新增/修改工具先改 defs.rs 的 schema（含描述），参数校验自动从
   properties 派生；渐进式披露约定不变——list/search 永不返回 content，只有 memory_get 返回。
   REST API（/api/*）必须复用同一批 handler，不得另写校验逻辑。
   错误分类用 `ToolError`（NotFound→404 / Invalid→400），**禁止**再按错误文本匹配分类。
6. **协议兼容**：MCP 协议版本支持 2024-11-05 / 2025-03-26 / 2025-06-18；id 边界格式
   `"m{n}"`（normalize_id 容忍 "1"/"m1" 两种写法）。
7. **时间戳边界**：模型层用 u64 秒；SQL 绑定用 i64（rusqlite 不支持 u64），读取后转回。
8. **嵌入资产**：ui/dist 必须存在且被提交（rust-embed debug-embed 编译期嵌入）；
   改前端后先 `npm run format && npm run build` 再 `cargo build`。
9. **前端格式化**：ui/ 源码用 Prettier 统一（无分号、单引号、120 列，配置见
   ui/.prettierrc.json）；CI 强制 `format:check`，提交前先 `npm run format`。
10. **Markdown 渲染必须消毒**：记忆正文是多 agent 共写的外部输入，
    管理界面渲染前必须经 DOMPurify（`ui/src/markdown.ts` 统一出口），
    新增渲染入口不得绕过它直接 `v-html`。

## 测试约定

- 端到端测试在 `tests/e2e.rs`：真实 spawn 二进制（serve --port 随机 + --db 临时目录），
  HTTP 客户端用 std::net 手写（不引 dev 依赖）；服务器启动用全局锁串行化避免端口竞争。
- 请求行/URL 里的非 ASCII 必须 percent-encode（tiny_http 不接受原始 UTF-8 请求行）。
- 单元测试分布在各模块（store 的级联/迁移/体检、tools 的契约校验、protocol 的对抗输入、
  http 的路由/Origin/解码）。数据文件一律指到临时目录，绝不碰用户真实数据。
- 新增工具时至少覆盖：正常流、参数错误、渐进式披露边界（正文不泄露）。
- 前端单元测试默认 happy-dom；涉及 DOMPurify 的测试必须标 `// @vitest-environment jsdom`
  （DOMPurify 在 happy-dom 下会错剥常见块级标签，jsdom 是其官方支持的测试 DOM）。

## 开发脚本（scripts/，TypeScript，需 node ≥24 原生类型剥离运行，不参与构建）

- `sdk-compat-check.ts`：用官方 MCP TypeScript SDK 走完整 agent 流程的兼容性
  回归（initialize/listTools/callTool/错误通道/无状态重连），发布前必跑。
  用法：根目录 `npm install` 后 `MCP_URL=http://127.0.0.1:8899/mcp node scripts/sdk-compat-check.ts`。
- `soak.ts`：双进程混合负载浸泡（写/搜/列表/标签轮转并发），失败分类能区分
  客户端过载（ECONNREFUSED 风暴）与服务端真实错误（5xx/BUSY）。
- 两个脚本的类型检查：`npm run typecheck`（workspace 命令，CI ui-tests job 一并执行）。
