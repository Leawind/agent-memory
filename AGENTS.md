# AGENTS.md — agent-memory 项目指南

自托管 MCP 记忆服务器（纯 HTTP）：多 agent 共享一个 SQLite 记忆库，内嵌 Vue3 管理界面。
Rust 依赖 serde / serde_json / tiny_http / rusqlite(bundled) / clap / rust-embed。

**本文件只承载方向、约束与不变量，不复述实现细节——行为以代码与测试为唯一事实来源。**

## 工作约定

- **不要 kill 用户启动的 agent-memory 进程**（需要新二进制生效时，提示用户自行重启）。
- **记得适时提交**：每完成一个独立、可验证的改动（源码 + 测试）就 commit，
  不要把多件不相干的事攒在一个工作区里。
- **提交信息只用英文**：主题与正文一律纯英文，不夹中文；需要指称中文 UI 文案时用英文转述。
- **没有正式版 = 没有兼容包袱**：项目未发布，允许破坏性更改——改契约、改行为、删旧功能
  都直接改，不写兼容层 / 弃用路径 / 渐进迁移逻辑（数据库迁移机制除外，见架构不变量 3）。
- **文档只定方向**：文档（含本文件）只写约束、边界与约定；实现细节进了文档就会过时，
  改代码时不必回改文档复述行为，只有方向或边界变了才改文档。

## 常用命令

```bash
cargo test                      # 全部测试（单元 + 端到端，e2e 真实启动 HTTP 服务器与 CLI 子命令）
cargo clippy --all-targets      # 提交前应零告警（Cargo.toml [lints.clippy] 已提升默认严格度）
cargo fmt --check               # 提交前格式必须通过
cargo install --path . --force  # 更新本机安装（改完 Rust 代码后跑，否则运行中的是旧二进制）
pnpm build                      # 仅当改了 ui/ 时需要；app 产物 ui/dist 由 rust-embed 编译期嵌入
pnpm test / typecheck / format  # JS 侧命令在根目录可用（pnpm workspaces，根目录一次 pnpm install）
```

- 若配置了共享/非默认输出目录（编译产物不在 ./target），e2e 等需要定位二进制处用
  `cargo metadata` 取真实输出目录。
- 改前端后 `pnpm format && pnpm build` 再 `cargo build`：build.rs 跟踪 ui/dist，dist 变化
  自动重编译重嵌入，无需 cargo clean。

## 模块地图

```
src/
  main.rs        clap CLI：serve（默认）+ stats/doctor/export/import/embed-backfill/token reset，
                 全部命令行参数，无配置文件
  http.rs        HTTP 传输：路由 /mcp·/api·静态 UI·/health、多 worker、Origin 防护、body 上限、
                 Bearer 鉴权拦截（fail-closed）
  auth.rs        身份与能力模型：Cap 能力登记表（唯一权威）、Permissions 校验、IdentityCtx
  api.rs         管理后端 /api/*：复用工具 handler，只做路由与状态码映射
  protocol.rs    MCP 协议层：双时代共用一张方法表，时代只作用于信封
  resources.rs   memory:// 资源面
  notify.rs      订阅与变更通知（请求级 SSE）
  tools/         工具入口与权限守卫（mod）、契约 defs.rs、文本渲染 render、参数解析 params、
                 业务处理器 tag_ops / memory_ops
  store/         SQLite 持久化：Store 与事务入口 tx、迁移运行器 migrate、各域数据操作
                 （tags/memories/identities/settings/embeddings）、体检/统计/导出导入 ops
  sql.rs + sql/  SQL 登记表与语句文件（每条一个文件）
  search.rs      关键词搜索
  tag_expr.rs    标签集合表达式（布尔运算 + /…/ 正则原子）
  embed.rs       语义搜索与向量（全项目唯一出站 HTTP）
  model.rs       纯数据模型与 API 限制常量
migrations/      schema 迁移脚本（build.rs 编译期生成 MIGRATIONS）
ui/lib           @agent-memory/ui —— 可嵌入 Vue3 组件库（集成指南见 ui/README.md）
ui/app           @agent-memory/app —— 独立站点薄壳，产物输出 ui/dist
scripts/         开发脚本（TypeScript）
```

## 架构不变量（改代码前必读）

每条都是红线；具体机制以代码为准。

1. **stdout 干净**：服务器模式日志只写 stderr；CLI 子命令结果走 stdout；HTTP 数据只走 body。
2. **事务纪律**：读走只读事务，写走 IMMEDIATE 写事务（防"先读后升级"死锁）。只读与否由
   defs.rs 的 readOnlyHint / REST 的 HTTP 方法派生；成功才 COMMIT，失败靠连接关闭回滚。
3. **SQL 与迁移外置**：Rust 代码不出现 SQL 文本——业务语句在 `sql/`，schema 在
   `migrations/`（目录即唯一事实源）。未发布前改 schema 直接改写基线，旧库删掉重建；
   **发布后**只能新增迁移文件，绝不改写已发布的迁移。open 时做 schema 指纹校验，
   不符即拒绝打开（防降级写坏数据）。
4. **跨平台数据**：schema 不存平台相关状态（绝对路径、换行风格等）。
5. **defs.rs 是契约**：新增/改工具先改 defs.rs 的 schema（含描述）与能力映射，参数校验从
   schema 派生。渐进式披露：列表/搜索永不返回正文，只有单条读取返回；资源面同源一致
   （目录型资源不带正文）。REST 复用同一批 handler，不得另写校验逻辑。错误分类用
   ToolError，禁止按错误文本匹配。
6. **协议双时代**：现代与 legacy 两代共用一张方法表与同一批处理器，时代只作用于信封校验，
   业务行为永不分叉。新增协议能力默认只进现代面；往 legacy 面加东西前先论证旧客户端
   确实需要。
7. **保留标签 convention 与 id 边界**：常驻约定记忆挂在保留标签 `convention` 下（open 时
   播种、恒存在）；不可改名/删除，挂/摘需 Admin；标签名用单数形式。记忆 id 是不稳定标识
   （删除/合并即失效、导出导入重编号），跨记忆引用用标签/关键词互链，不用 id；id 格式
   严格 `m{n}`。
8. **时间戳边界**：模型层 u64 秒，SQL 绑定 i64。
9. **构建产物不入库**：target/、node_modules/、ui/dist、ui/lib/dist 全部 gitignore；
   ui/dist 缺失时 build.rs 生成占位页兜底，全新 clone 不装 node 也能 cargo build。
10. **前端格式化**：Prettier 统一（配置在根 .prettierrc.json），提交前先 `pnpm format`。
11. **渲染安全**：记忆正文是多 agent 共写的外部输入——Markdown 渲染必须过 DOMPurify
    （ui/lib/src/markdown.ts 是唯一合法 v-html 入口）；服务端纯文本数据一律文本插值，
    新增渲染入口不得绕过。
12. **鉴权边界**：无账号体系，token 即身份，服务端只存哈希；token 生成只用 SQLite
    randomblob，不引随机数依赖。能力登记表唯一权威在 auth.rs::Cap，工具能力要求登记在
    defs.rs，执行入口集中把守，处理器内不重复校验；ctx 自上而下贯穿，不得绕过。
    身份/设置管理走 admin 守卫的 REST 端点，不走 MCP 工具面。鉴权解析 fail-closed；
    401 不携带 WWW-Authenticate；带无效 token 是认证失败、不回退匿名，匿名能力由
    settings 的 anonymous_permissions 决定。静态 UI 与 /health 永远免鉴权。
13. **语义搜索回退**：embedding 服务不可用只降级不失败——搜索回退关键词并显式标记，
    写入静默留待补跑。embedding 调用一律在数据库事务之外（embed.rs 是全项目唯一出站
    HTTP）。向量是派生数据：随记忆级联删除、不入导出、模型指纹不符视为缺失。混合排序
    用 RRF 融合，禁止不同量纲直接加权比较。渐进式披露对语义召回同样生效。

## 测试约定

- 端到端在 `tests/e2e/`：真实 spawn 二进制，HTTP/SSE 客户端手写（不引 dev 依赖），服务器
  启动用全局锁串行化避免端口竞争；请求行/URL 里的非 ASCII 必须 percent-encode。
- 单元测试分布在各模块；数据文件一律指到临时目录，绝不碰用户真实数据。
- 新增工具至少覆盖：正常流、参数错误、渐进式披露边界（正文不泄露）。
- 前端 vitest 默认 happy-dom；涉及 DOMPurify 的测试必须标
  `// @vitest-environment jsdom`。

## 开发脚本（scripts/）

- `sdk-compat-check.ts`：手写双时代客户端的兼容性回归，发布前必跑。
- `soak.ts`：双进程混合负载浸泡，能区分客户端过载与服务端真实错误。
- 两个脚本的类型检查：`pnpm typecheck`。
