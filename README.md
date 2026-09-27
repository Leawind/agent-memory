# agent-memory

自托管 MCP 记忆服务器：**一个记忆库服务所有 agent**，多个 AI agent（ZCode / Claude Desktop / Cursor 等）通过网络访问同一份数据，并内置 Vue3 Web 管理界面供人直接浏览与管理。

- **传输**：纯 HTTP——agent 走 MCP Streamable HTTP（`/mcp`），人走浏览器（`/`）
- **存储**：单文件 SQLite（WAL 模式），多 agent / 多进程并发安全
- **鉴权**：可选的 token 多身份访问控制（无账号体系，token 即身份），与 MCP 规范的 `Authorization: Bearer` 载体兼容
- **跨平台**：SQLite 文件格式平台无关，同一份 `.db` 可在 Windows / Linux / macOS 间直接复制
- **单二进制**：管理界面构建产物嵌入二进制，`cargo build` 即得完整交付物

## 快速开始

```bash
# 从源码安装（产物进入 ~/.cargo/bin，需先 pnpm install && pnpm build 以准备嵌入的界面产物）
cargo install --path .

# 启动（默认 127.0.0.1:8899，数据库在当前工作目录 memory.db）
agent-memory serve
```

打开 `http://127.0.0.1:8899/` 即是管理界面；让 agent 的 MCP 配置指向 `http://127.0.0.1:8899/mcp` 即接入同一记忆库。

常用命令行参数（无配置文件，全部走参数）：

| 参数 | 默认值 | 说明 |
|---|---|---|
| `--host` | `127.0.0.1` | 监听地址；跨机器共享用 `0.0.0.0`（仅限可信网络） |
| `--port` | `8899` | 监听端口 |
| `--db` | `./memory.db`（当前工作目录） | SQLite 数据库文件路径 |
| `--verbose` | 关 | 详细日志：记录全部请求并附带耗时、请求者身份与 MCP 调用摘要。默认只记 4xx/5xx 请求与启动/异常事件 |

子命令：`serve`（默认）/ `stats` / `doctor`（体检，有问题退出码 1）/ `export <file>`（导出 JSON 备份，拒绝覆盖已有文件）/ `import <file>`（从备份恢复，要求目标库为空）/ `embed-backfill`（补跑语义搜索向量）/ `token reset [name]`（重置某身份的 token，省略名字时重置最早创建的管理员——token 丢失的兜底手段）。

## 两种部署形态

- **个人本地库（默认，零配置）**：鉴权开关关闭时服务器为无鉴权开放模式（与是否已创建身份无关），行为与单机工具一致。想要私有记忆，就在本地跑一个实例，MCP 配置只给自己用；可在管理界面「身份与访问 → 自定义提示词」里写明“这是我的私有记忆库”。
- **团队中央库**：先创建一个管理员身份并妥善保存 token，再在管理界面「身份与访问」页打开鉴权开关（开关开启要求库里已存在管理员身份，防锁死），服务器即刻要求所有 `/mcp` 与 `/api` 请求携带 token。操作者把管理员 token 分发给自己，在同一页为每个成员创建身份、逐项勾选能力开关（读取 / 新建 / 修改 / 删除 / 标签管理 / 管理员），把各自的 token 分发下去。

## 身份与鉴权

- **无账号体系**：没有注册、登录、密码——身份 = 一个名字 + 一枚随机 token（Web UI 或 CLI 签发，可随时重置吊销）。token 只存 SHA-256（库文件被备份/误传时凭据不外泄），明文仅在创建/重置响应中出现一次；列表只留尾缀提示，丢失即重置。
- **能力粒度**：能力按身份逐项独立开关，权限不足的请求得到 403（与 MCP 规范 "403 = insufficient permissions" 语义一致）；`initialize` 响应会把调用者身份与能力告知 agent，无需试错。
- **传输载体**：`Authorization: Bearer <token>`，与 MCP 2025-06-18 规范一致（规范定义的 OAuth 2.1 获取流程属于账号体系，本项目不实现；静态 token 走同一请求头，wire 兼容）。
- **Web UI**：每个浏览器输入一次 token（存 localStorage，之后自动携带）；退出登录即清除。

## 接入 MCP 客户端（多 agent 共享）

```json
"agent-memory": {
  "type": "http",
  "url": "http://127.0.0.1:8899/mcp",
  "headers": { "Authorization": "Bearer <token>" }
}
```

所有客户端指向同一 URL 即共享同一记忆库，用标签体系区分项目。并发安全由 SQLite WAL + busy_timeout 保证，多进程、多连接同时读写不会互相覆盖。启用鉴权后，不支持自定义 headers 的客户端可用 [mcp-remote](https://www.npmjs.com/package/mcp-remote) 桥接：`npx mcp-remote http://host:8899/mcp --header "Authorization: Bearer <token>"`。

> ⚠️ 鉴权开关关闭时服务器为无鉴权开放模式：任何能连上该端口的程序都能读写记忆库。跨机器共享时（`--host 0.0.0.0`）务必只暴露给可信网段，或尽早启用 token 鉴权。内置 Origin 校验防浏览器 DNS rebinding。

## 功能

10 个 MCP 工具（与管理界面共用同一套业务逻辑）：

- **标签管理**：`tag_create` / `tag_list` / `tag_rename` / `tag_delete`——标签有唯一名称 + 可选描述（≤500 字符），分类体系由 agent 自主维护；改名级联同步所有引用；删除分 `detach`（摘引用）与 `purge`（连带删记忆）
- **记忆管理**：`memory_create` / `memory_update` / `memory_delete`
- **渐进式披露**：`memory_list` / `memory_search` 只返回 id + 标签 + 摘要 + 片段；`memory_get` 才取回完整正文，最大限度节省上下文
- **Markdown 正文**：记忆正文推荐用 Markdown 书写（工具描述中已向 agent 声明），管理界面编辑时可预览渲染结果，详情抽屉默认渲染、可切回源码；渲染前经 DOMPurify 消毒，多 agent 共写的恶意内容不会在管理界面执行
- **关键词搜索**：空格分词、全部词命中（AND），引号短语要求逐字相邻；大小写不敏感子串匹配，中文无需分词；加权排序——标签 > 摘要 > 正文，命中次数（封顶）与 ASCII 整词命中加权，片段窗口自动选覆盖词最多的位置；片段经 HTML 转义
- **语义搜索（可选）**：配置 OpenAI 兼容 embedding 服务后，`memory_search` 自动升级为混合检索——关键词路与向量语义路（余弦相似度）经 RRF 融合排序，换说法、跨语言、模糊回忆的查询也能召回关键词零命中的记忆。**回退是承诺**：embedding 服务不可用时搜索自动降级回纯关键词（响应携带 `semantic_fallback` 标记），写入永不因此阻塞，缺向量的记忆随时可补跑
- **防呆提示**：重复摘要 → `duplicate_of`；仅大小写不同的标签 → `similar_existing`；拼错参数名 → 立即报错并列出合法参数

## 管理界面

- **记忆管理**：搜索、标签过滤、搜索模式选择（自动/仅关键词/关键词+语义）、排序、分页、新建/编辑（标签回车即建、正文 Markdown 编辑/预览切换）、全文抽屉（渲染/源码切换）、删除确认
- **标签管理**：表格 + 新建/编辑（名称唯一、描述限长）+ 删除时选择 detach/purge
- **运维**：统计卡片、语义搜索向量覆盖率与一键补跑、doctor 体检、一键导出 JSON 备份（doctor / 补跑 / 导出 / 导入需 admin 能力）
- **身份与访问**（admin）：成员列表、新建/编辑/吊销身份（能力复选框 + 预设模板）、token 查看复制、语义搜索配置（开关 + 服务地址/模型/API Key + 连接测试）、自定义提示词（基础提示词覆盖内置默认 + 附加规范追加，均可一键恢复默认）

![记忆管理](docs/ui-memories.png)

![标签管理](docs/ui-tags.png)

![运维](docs/ui-ops.png)

## 语义搜索（可选）

在管理界面「身份与访问 → 语义搜索」配置一个 OpenAI 兼容的 `/embeddings` 服务即可开启，云服务与本地部署同一套配置：

| 部署 | base_url | model | API Key |
|---|---|---|---|
| SiliconFlow（国内直连，bge-m3 免费档） | `https://api.siliconflow.cn/v1` | `BAAI/bge-m3` | 必填 |
| 本地 Ollama（数据不出机） | `http://127.0.0.1:11434/v1` | `bge-m3` | 留空 |
| 阿里云百炼 / 智谱等 | 各家 OpenAI 兼容端点 | `text-embedding-v3` / `embedding-3` | 必填 |

行为约定：

- **开启即混合**：`memory_search` 默认（`mode: auto`）自动融合关键词与语义两路召回，agent 无需感知；显式 `mode: keyword` 可随时退回纯关键词
- **服务不可用 ≠ 失败**：搜索回退关键词并标记 `semantic_fallback`；新建/更新的记忆照常保存，向量留待补跑（运维页「补跑向量化」按钮或 CLI `embed-backfill`）
- **向量是派生数据**：存 SQLite `memory_embeddings` 表，随记忆删除级联清理；`export`/`import` 不携带向量，恢复备份后补跑一次即可；更换模型后旧向量自动作废重嵌

## 性能

本机实测（Windows，2026-09，300 条记忆）：单次创建 ~23 ms，搜索/分页 ~10 ms。写入成本随条目数近似线性，数千条以内完全无感，上万条仍可用。

## 数据

- 数据保存在单个 SQLite 文件（默认当前工作目录下 `memory.db`；启动时 stderr 会打印实际路径）
- **跨机器迁移**：SQLite 文件格式平台无关，服务器停止后直接复制 `.db` 即可换机使用（Windows / Linux / macOS 通用）；需要可读格式时用 `export` + `import` 完成"导出 → 恢复"
- **数据安全**：每个请求在单个事务内执行，错误时自动回滚，磁盘数据保持原样

## MCP 工具

| 工具 | 只读 | 所需能力 | 说明 |
|---|---|---|---|---|
| `tag_create(name, description?)` | | `tag_manage` | 新建标签，重名报错；存在仅大小写不同的标签时在 `similar_existing` 里提示（非阻塞） |
| `tag_list()` | ✓ | `read` | 全部标签 + 描述 + 记忆计数 + 最近使用时间 |
| `tag_rename(old_name, new_name?, description?)` | | `tag_manage` | 重命名（级联同步所有引用）/ 改描述 |
| `tag_delete(name, mode?)` | | `tag_manage` | `detach`（默认，只摘引用）/ `purge`（连带删除记忆） |
| `memory_create(summary, content, tags?)` | | `create` | 新建记忆；未知标签自动创建并在 `tags_autocreated` 汇报；摘要重复时提示 `duplicate_of` |
| `memory_list(tag?, sort?, order?, offset?, limit?)` | ✓ | `read` | 分页浏览，只返回摘要 |
| `memory_search(query, tags?, mode?, offset?, limit?)` | ✓ | `read` | 搜索，返回摘要 + 正文片段，支持翻页；`mode` 缺省 `auto`（已配置语义搜索即为混合检索），`keyword` 强制纯关键词，`hybrid` 显式要求语义（未配置报错）；回退时响应携带 `semantic_fallback: true` |
| `memory_get(ids)` | ✓ | `read` | 取 1–50 条完整正文（渐进式披露第二层） |
| `memory_update(id, summary?, content?, add_tags?, remove_tags?)` | | `update` | 增量更新，无需先取当前标签列表 |
| `memory_delete(ids)` | | `delete` | 永久删除 1–50 条 |

## 推荐的 agent 使用方式

1. **存**：`memory_create`，摘要写得精确自洽（未来扫描全靠它），标签自选自管
2. **查**：先 `memory_search` / `memory_list`（便宜），再对值得读的 id 调 `memory_get`（贵）
3. **管**：定期 `tag_list` 检查分类体系，用 `tag_rename` 纠错，过期记忆用 `memory_delete` 清理

## 许可

MIT（见 [LICENSE](LICENSE)）。
