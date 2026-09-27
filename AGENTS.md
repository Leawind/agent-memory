# AGENTS.md — agent-memory 项目指南

自托管 MCP 记忆服务器（纯 HTTP）：多 agent 共享一个 SQLite 记忆库，内嵌 Vue3 管理界面。Rust 依赖 serde / serde_json / tiny_http / rusqlite(bundled) / clap / rust-embed。

## 工作约定

- **记得适时提交**：每完成一个独立、可验证的改动（源码 + 构建产物 + 测试）就 commit，
  不要把多件不相干的事攒在一个工作区里。
- **没有正式版 = 没有兼容包袱**：项目未发布，允许破坏性更改——改契约、改行为、删旧功能
  都直接改，不要迁就旧实现，不写兼容层 / 弃用路径 / 渐进迁移逻辑（数据库迁移机制除外，
  见架构不变量 3）。

## 常用命令

```bash
cargo test        # 全部测试（单元 + 端到端，e2e 真实启动 HTTP 服务器与 CLI 子命令）
cargo clippy --all-targets   # 提交前应零告警（Cargo.toml [lints.clippy] 已提升默认严格度）
cargo fmt --check            # 提交前格式必须通过
cargo install --path . --force   # 更新本机安装（改完 Rust 代码后跑，否则运行中的是旧二进制）
                  # 部署方式：源码 cargo install，MCP 客户端配置走 HTTP 地址，无构建脚本
pnpm build        # 仅当改了 ui/ 时需要；先 lib 后 app 两 workspace，app 产物 ui/dist 由 rust-embed 嵌入
                  # 构建产物不入库（见不变量 8）；build.rs 以 rerun-if-changed=ui/dist 跟踪目录，
                  # pnpm build 后直接 cargo build 即可重新嵌入，无需 cargo clean
```

注意：若通过 CARGO_TARGET_DIR 或 build.target-dir 配置了共享/非默认输出目录（编译产物不在
./target），e2e 测试等需要定位二进制时用 `cargo metadata` 从 cargo 元数据取真实输出目录。

JS 侧（ui/ 与 scripts/）由 pnpm workspaces 统一管理（pnpm-workspace.yaml，版本锁定在根
package.json 的 packageManager）：根目录一次 `pnpm install` 生成唯一 lockfile（pnpm-lock.yaml），
`pnpm test` / `typecheck` / `format` 等命令在根目录直接可用（内部转发到对应 workspace）。

## 模块结构

```
src/
  main.rs        clap CLI：serve(默认)/stats/doctor/export/import/embed-backfill/token reset；--host/--port/--db
                 全部命令行参数，无配置文件
  http.rs        HTTP 传输：路由 /mcp·/api·静态 UI·/health，多 worker，Origin 防护，panic 隔离，body 上限；
                 Bearer 鉴权拦截（resolve_identity，fail-closed，开关状态每次请求查库决定）
  auth.rs        身份与能力模型：Cap 能力登记表（唯一权威）、Permissions JSON 严格校验、
                 IdentityCtx（require/can/summary）；开放模式 = 全能力
  api.rs         管理后端 /api/*：复用 tools handler，percent 解码，404/400/403 映射（业务逻辑不在此层）；
                 例外：identities/settings 端点走专用 handler（agent 工具面不暴露权限管理）
  protocol.rs    MCP 协议层：initialize / ping / tools/list / tools/call，通知不回包，批量消息；
                 工具结果文本通道必须是紧凑 JSON，structuredContent 按协商版本（2025-06-18 起）附带；
                 initialize 回传自定义提示词（instructions 非空覆盖内置默认 + conventions 非空追加）+ 调用者身份行
  tools/mod.rs   工具入口：execute_with_db（单事务），分发，未知参数校验（从 schema 派生），
                 入口集中执行 ctx.require(defs::required_cap(name)) 权限守卫
  tools/defs.rs  工具清单 + JSON Schema（对 agent 的契约，唯一权威来源）+ 工具→能力映射 required_cap
  tools/params.rs 参数解析/校验（值从严错报、写法从宽：单字符串可当数组）
  tools/tag_ops.rs / memory_ops.rs  业务处理器（校验在此，数据操作下沉到 store）
  store/         SQLite 持久化目录：mod.rs 是 Store 结构体与 open/id 换算（open 时按迁移
                 文本做 schema 指纹校验：表 + 列与实测不符即拒绝打开），tx.rs 是
                 事务入口（TxMode/with_db_in），migrate.rs 是迁移运行器；
                 tags/memories/identities/settings/embeddings 各以 impl Store 承载数据操作，
                 ops.rs 是体检/统计/导出导入（identities token 即身份，只存哈希 + 尾缀提示）；
                 标签以自增内部 id 关联记忆（memory_tags.tag_id），标签名唯一非空是对外
                 唯一标识，id 对 MCP 使用者完全不可见，改名不动 id
  sql.rs         SQL 语句登记表：include_str! 嵌入 sql/ 目录，Rust 代码不出现 SQL 文本
  search.rs      关键词搜索：AND 语义（引号短语逐字相邻）、TF 封顶 + ASCII 整词加权、
                 中文子串匹配（内存内计算）、片段窗口优选；片段必须 HTML 转义（UI 以 v-html 渲染）
  embed.rs       语义搜索：OpenAI 兼容 /embeddings 客户端（ureq+rustls，全项目唯一出站 HTTP）、
                 向量 BLOB 编解码/余弦、RRF 混合排序、process_pending 有界批量补跑；
                 回退是硬性原则——服务不可用只降级不失败（搜索回退关键词 + semantic_fallback 标记，
                 写入静默留待补跑）；embedding 调用一律在数据库事务之外
  model.rs       纯数据模型：Memory / id·标签名·身份名归一化 / API 限制常量
migrations/      Schema 迁移脚本（NUM-NAME.sql），build.rs 编译期生成 MIGRATIONS 数组
sql/             业务 SQL（每条语句一个文件，文件名 ↔ sql.rs 常量，同步测试把守）
ui/              前端分两个 workspace 包（详见 ui/README.md）：
  ui/lib         @agent-memory/ui —— 可嵌入 Vue3 组件库（Element Plus 作 peerDependency，lib mode 构建）
                 核心是三个自包含可复用面板 MemoriesPanel/TagsPanel/AdminPanel（admin 专属：
                 身份、鉴权开关、自定义提示词、备份、体检；who prop 门控）（show-header/title/
                 subtitle props 裁剪），概况弹窗 OpsDialog 承载只读统计与版本信息（每次打开重拉，
                 无运维标签页），另附便捷壳 MemoryAdmin（sidebar/tabs；侧栏标题点击弹概况）
                 与弹层组件；多语言内置 vue-i18n（zh/en，独立作用域实例，auto 跟随浏览器，
                 setMemoryUILocale 运行时切换）；配置经 provideMemoryUI 注入（baseUrl/自定义
                 fetch/默认分页/locale）；数据操作在 composables，面板层只渲染与 toast
                 （toast.ts 统一出口：可点击关闭、起始位置让开顶栏）；样式全部引用 --el-* 变量跟随宿主主题
  ui/app         @agent-memory/app —— 独立站点薄壳（Modrinth 风格顶部导航栏 + 主题/语言切换），
                 直接组装三个面板（「管理」标签页按 admin 能力显示，非 admin 身份不产生管理请求），
                 点击顶栏标题弹出服务概况（OpsDialog），
                 产物输出 ui/dist；一个浏览器可存多个身份 token（auth.ts 的 localStorage 身份表，
                 标题处下拉切换/删除/添加，token 首尾提示本地计算），authFetch 为 lib 注入带 Authorization 的 fetch，
                 401 时移除失效身份并广播事件弹出令牌输入框；
                 dev 默认走内置 mock API（mock/api.ts，内存假数据 + 固定 admin 身份，
                 前端开发不依赖后端，重启复位），--mode live（pnpm run dev:live）代理真实服务器
                 记忆正文按 Markdown 渲染：ui/lib/src/markdown.ts（marked + DOMPurify）→ MarkdownView.vue
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
   `1-init.sql`，旧库删掉重建，不为旧库写兼容迁移（换库走 export/import，
   导出含标签描述与记忆时间戳，id 重新编号）；**发布后**任何 schema 变更只能
   新增 `2-xxx.sql`、`3-xxx.sql` 等新迁移文件，绝不改写已发布的迁移。
   运行器按 `PRAGMA user_version` 逐个事务应用，恰好一次；数据库比已知迁移
   更新时拒绝打开（防降级写坏数据）。**user_version 只证明应用过几个迁移，
   不证明表真的存在/形状正确**（就地改写迁移的历史库两数值都对）——所以
   `Store::open` 在迁移后做 schema 指纹校验：从迁移文本解析期望的表与列，
   与 sqlite_master/table_info 实测比对，不符即拒绝打开并给出 export/import
   恢复指引（migrate.rs/mod.rs 各有测试把守）。
4. **跨平台数据**：schema 内不得存平台相关状态（绝对路径、换行风格等）；SQLite 文件
   格式平台无关，同一份 .db 跨机复制可用（停服后复制，或用 export）。
5. **defs.rs 是契约**：新增/修改工具先改 defs.rs 的 schema（含描述），参数校验自动从
   properties 派生；渐进式披露约定不变——list/search 永不返回 content，只有 memory_get 返回。
   REST API（/api/*）必须复用同一批 handler，不得另写校验逻辑。
   错误分类用 `ToolError`（NotFound→404 / Invalid→400 / Forbidden→403），**禁止**再按错误文本匹配分类。
6. **协议兼容**：MCP 协议版本支持 2024-11-05 / 2025-03-26 / 2025-06-18；id 边界格式
   严格为 `"m{n}"`——normalize_id 只去空白，parse_id 拒绝省略 m 前缀的裸数字。
7. **时间戳边界**：模型层用 u64 秒；SQL 绑定用 i64（rusqlite 不支持 u64），读取后转回。
8. **仓库只有源码，构建产物一律不入库**：`ui/dist`、`ui/lib/dist`（连同 target/、
   node_modules/）全部 gitignore；rust-embed debug-embed 编译期嵌入 `ui/dist`，
   其缺失时由 build.rs 生成占位 index.html 兜底（全新 clone / CI 不装 node 也能
   cargo build，只是没有真实界面）。改前端后 `pnpm format && pnpm build` 再
   `cargo build`——build.rs 以 rerun-if-changed=ui/dist 跟踪目录，dist 变化自动
   触发重编译重嵌入，无需 cargo clean。
9. **前端格式化**：ui/ 源码用 Prettier 统一（无分号、单引号、120 列，配置在根
   .prettierrc.json）；CI 强制 `format:check`，提交前先 `pnpm format`。
10. **Markdown 渲染必须消毒**：记忆正文是多 agent 共写的外部输入，
    管理界面渲染前必须经 DOMPurify（`ui/lib/src/markdown.ts` 统一出口：
    Markdown 走 renderMarkdown，服务端 HTML 片段走 sanitizeHtml），
    新增渲染入口不得绕过它直接 `v-html`。
11. **鉴权边界**：无账号体系（不注册、不登录、不引 OAuth）——token 即身份，只存
    SHA-256 + 尾缀提示（哈希手写在 `util.rs`，token 明文仅在创建/重置响应出现一次，
    丢失即重置），token 生成只用 SQLite `randomblob`，不引随机数依赖。
    enforcement 条件 = settings 的 `auth_required` 开关（显式、持久化，每次请求查库决定；
    **开启**有服务端守卫：库里必须已存在 admin 能力的身份，防止翻上开关后无人持有
    token、管理面整体锁死——唯一开启入口是 REST/UI，开放模式下先建身份再翻开关；
    UI 在「管理」页切换）。能力登记表唯一权威在 `auth.rs::Cap`，
    permissions JSON 必须全键、未知键拒绝；工具能力要求登记在 `defs.rs::required_cap`，
    执行点在 `tools::execute` 入口集中把守，处理器内不得重复校验。鉴权解析在 HTTP 层
    一次完成（fail-closed：查库失败按 401 拒绝），ctx（`IdentityCtx`）自上而下贯穿
    tools/api/protocol，不得绕过；401 响应不携带 WWW-Authenticate（避免规范客户端
    走 OAuth 发现流程）。静态 UI 与 /health 永远免鉴权。身份/设置管理不走 MCP 工具面，
    走 admin 能力守卫的 REST 端点。
12. **语义搜索回退与事务纪律**：`embed.rs` 是全项目唯一的出站 HTTP 依赖
    （OpenAI 兼容 `/embeddings`，配置在 settings 四键 `embedding_*`，REST/UI 可改）；
    回退是硬性承诺——embedding 服务不可用只允许降级不允许失败：搜索回退纯关键词
    并携带 `semantic_fallback` 标记（显式 `mode: hybrid` 同样回退而非报错），写入
    静默留待补跑（向量缺失由 doctor/stats 呈现）。embedding 网络调用一律在数据库
    事务之外（写路径 = 工具事务提交后 `embed::after_write` 补跑，短事务存向量）；
    向量是派生数据（`memory_embeddings` 表随记忆级联删除，export/import 不携带，
    模型指纹不符视为缺失），混合排序用 RRF 融合（关键词分与余弦不同量纲，禁止
    直接加权比较）；渐进式披露不变量对语义召回同样生效（不返回正文）。

## 测试约定

- 端到端测试在 `tests/e2e/`（main.rs 只做 mod 声明，按场景分文件，共享基建在
  common.rs）：真实 spawn 二进制（serve --port 随机 + --db 临时目录），
  HTTP 客户端用 std::net 手写（不引 dev 依赖）；服务器启动用全局锁串行化避免端口竞争。
- 请求行/URL 里的非 ASCII 必须 percent-encode（tiny_http 不接受原始 UTF-8 请求行）。
- 单元测试分布在各模块（store 的级联/迁移/体检、tools 的契约校验、protocol 的对抗输入、
  http 的路由/Origin/解码）。数据文件一律指到临时目录，绝不碰用户真实数据。
- 新增工具时至少覆盖：正常流、参数错误、渐进式披露边界（正文不泄露）。
- 前端单元测试（ui/lib 与 ui/app，vitest）默认 happy-dom；涉及 DOMPurify 的测试必须标
  `// @vitest-environment jsdom`
  （DOMPurify 在 happy-dom 下会错剥常见块级标签，jsdom 是其官方支持的测试 DOM）。
  面板挂载测试 stub fetch + 真实渲染 Element Plus 组件（组件的 el-* 导入由
  unplugin-vue-components 在构建期注入，测试无需全局注册 EP）。

## 开发脚本（scripts/，TypeScript，需 node ≥24 原生类型剥离运行，不参与构建）

- `sdk-compat-check.ts`：用官方 MCP TypeScript SDK 走完整 agent 流程的兼容性
  回归（initialize/listTools/callTool/错误通道/无状态重连），发布前必跑。
  用法：根目录 `pnpm install` 后 `MCP_URL=http://127.0.0.1:8899/mcp node scripts/sdk-compat-check.ts`。
- `soak.ts`：双进程混合负载浸泡（写/搜/列表/标签轮转并发），失败分类能区分
  客户端过载（ECONNREFUSED 风暴）与服务端真实错误（5xx/BUSY）。
- 两个脚本的类型检查：`pnpm typecheck`（workspace 命令，CI ui-tests job 一并执行）。
