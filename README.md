# agent-memory

自托管 MCP 记忆服务器：多个 AI agent 通过 HTTP 共享同一个记忆库，附带 Web 管理界面。

- **一个库，多个 agent**：ZCode / Claude Desktop / Cursor 等指向同一 URL 即共享记忆，跨平台 `.db` 文件直接复制迁移
- **单二进制，单文件存储**：SQLite（WAL 模式）并发安全，管理界面嵌入二进制
- **可选鉴权**：token 多身份 + 逐项能力开关；不开启则是本机开放模式
- **可选语义搜索**：配置 OpenAI 兼容 embedding 服务即升级混合检索，服务不可用自动回退关键词

## 快速开始

```bash
cargo install --path .   # 界面产物已入库，克隆后可直接安装
agent-memory serve       # 默认 127.0.0.1:8899，数据库为当前目录下 memory.db
```

- **管理界面**：浏览器打开 `http://127.0.0.1:8899/`
- **agent 接入**：MCP 配置指向 `http://127.0.0.1:8899/mcp`

```json
"agent-memory": {
  "type": "http",
  "url": "http://127.0.0.1:8899/mcp",
  "headers": { "Authorization": "Bearer <token>" }
}
```

所有客户端指向同一 URL 即共享同一记忆库，用标签区分项目。数据就是单个 SQLite 文件，服务器停止后复制 `.db` 即可迁移到任何机器。

### 常用参数与子命令

无配置文件，全部走命令行参数：

| 参数 / 命令 | 默认 | 说明 |
|---|---|---|
| `--host` | `127.0.0.1` | 监听地址；跨机器共享用 `0.0.0.0`（仅限可信网络） |
| `--port` | `8899` | 监听端口 |
| `--db` | `./memory.db` | 数据库文件路径 |
| `--verbose` | 关 | 记录全部请求、耗时与调用者身份 |
| `stats` / `doctor` | | 统计信息 / 体检（发现问题时退出码 1） |
| `export <file>` / `import <file>` | | 导出 JSON 备份 / 恢复到空库 |
| `embed-backfill` | | 补跑语义搜索向量 |
| `token reset [name]` | | 重置身份 token；省略名字时重置最早的 admin（token 丢失的兜底） |

## 鉴权

- **默认开放**：鉴权开关关闭时任何能连上端口的程序都可读写，只建议本机使用（内置 Origin 校验防 DNS rebinding）
- **团队共享**：先在管理界面「身份与访问」创建管理员身份并保存 token，再打开鉴权开关（要求库里已有 admin，防止锁死）；之后所有请求都要带 token，为每个成员建身份、逐项勾选能力（读取 / 新建 / 修改 / 删除 / 标签管理 / 管理员）
- **token 即身份**：无注册登录，明文只在创建/重置时显示一次，服务端只存 SHA-256；丢失随时重置吊销

## 语义搜索（可选）

在管理界面「身份与访问 → 语义搜索」配置一个 OpenAI 兼容 `/embeddings` 服务（云服务与本地 Ollama 同一套配置）：

| 部署 | base_url | model | API Key |
|---|---|---|---|
| SiliconFlow（国内直连） | `https://api.siliconflow.cn/v1` | `BAAI/bge-m3` | 必填 |
| 本地 Ollama（数据不出机） | `http://127.0.0.1:11434/v1` | `bge-m3` | 留空 |
| 阿里云百炼 / 智谱等 | 各家 OpenAI 兼容端点 | 对应 embedding 模型 | 必填 |

开启后 `memory_search` 自动融合关键词与语义两路召回（RRF），换说法、跨语言的查询也能命中。**回退是承诺**：embedding 服务不可用时搜索自动降级回纯关键词（响应带 `semantic_fallback` 标记），写入永不因此阻塞，缺向量随时一键补跑。

## MCP 工具（10 个）

| 工具 | 说明 |
|---|---|
| `memory_create` / `memory_update` / `memory_delete` | 写入与删除；未知标签自动创建，重复摘要会提示 |
| `memory_list` / `memory_search` | 分页浏览 / 搜索，只返回摘要与片段，节省上下文 |
| `memory_get(ids)` | 取完整正文（1–50 条） |
| `tag_create` / `tag_list` / `tag_rename` / `tag_delete` | 标签体系：改名级联同步；删除可选只摘引用或连带删记忆 |

搜索规则：多词 AND 命中，引号短语逐字相邻，中文子串直接匹配；排序权重 标签 > 摘要 > 正文。

## 从源码开发

```bash
pnpm install && pnpm build   # 仅改了 ui/ 时需要（构建产物不入库；未构建时二进制内嵌占位页）
cargo install --path .       # 安装到 ~/.cargo/bin
cargo test                   # Rust 与前端全部测试（e2e 真实起服务器）
cargo clippy --all-targets   # 提交前应零告警
```

更多见 [AGENTS.md](AGENTS.md)（模块地图与架构不变量）和 [ui/README.md](ui/README.md)（前端结构）。

## 许可

MIT（[LICENSE](LICENSE)）
