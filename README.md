# agent-memory

自托管 MCP 记忆服务器：多个 AI agent 通过 HTTP 共享同一个记忆库，附带 Web 管理界面。定位是记忆库而非知识库——由 agent 维护，存时效性、场景化的信息，而非长期通用知识。

- **一个库，多个 agent**：ZCode / Claude Desktop / Cursor 等指向同一 URL 即共享记忆，跨平台 `.db` 文件直接复制迁移
- **单二进制，单文件存储**：SQLite 并发安全，管理界面嵌入二进制
- **可选鉴权**：token 多身份 + 逐项能力开关；不开启则是本机开放模式
- **可选语义搜索**：配置 OpenAI 兼容 embedding 服务即升级混合检索，服务不可用自动回退关键词

## 快速开始

```bash
cargo install --path .   # 克隆后可直接安装
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

无配置文件，全部走命令行参数（`agent-memory --help` 查看全部）；子命令：`stats` / `doctor` / `export` / `import` / `embed-backfill` / `token reset`。

## 鉴权

- **默认开放**：鉴权关闭时任何能连上端口的程序都可读写，仅限本机使用
- **团队共享**：先在管理界面「身份与访问」创建管理员身份并保存 token，再打开鉴权开关；之后为每个成员建身份、逐项勾选能力
- **匿名访问（可选）**：可为无 token 的请求单独配置一组能力（比如只读）；未设置时无 token 一律 401，无效 token 是认证失败、不降级成匿名
- **token 即身份**：明文只在创建/重置时显示一次，丢失随时重置吊销

## 语义搜索（可选）

在管理界面「身份与访问 → 语义搜索」配置一个 OpenAI 兼容 `/embeddings` 服务（云服务与本地 Ollama 同一套配置）：

| 部署                      | base_url                        | model               | API Key |
| ------------------------- | ------------------------------- | ------------------- | ------- |
| SiliconFlow（国内直连）   | `https://api.siliconflow.cn/v1` | `BAAI/bge-m3`       | 必填    |
| 本地 Ollama（数据不出机） | `http://127.0.0.1:11434/v1`     | `bge-m3`            | 留空    |
| 阿里云百炼 / 智谱等       | 各家 OpenAI 兼容端点            | 对应 embedding 模型 | 必填    |

开启后搜索自动融合内容关键词、标签与语义三个通道，换说法、跨语言的查询也能命中。标签条件先限制召回范围，摘要和正文独立于标签参与语义检索。**回退是承诺**：embedding 服务不可用时自动降级回关键词与标签检索并显式标记，写入永不因此阻塞，缺向量随时补跑（`embed-backfill`）。

## MCP 工具

覆盖记忆与标签的增删改查；列表 / 搜索只返回摘要与片段、不返回正文，完整正文用单条读取获取。完整契约以运行时 `tools/list` 为准。

## 从源码开发

```bash
pnpm install && pnpm build   # 仅改了 ui/ 时需要
cargo install --path .       # 安装到 ~/.cargo/bin
cargo test                   # 全部测试（含端到端）
cargo clippy --all-targets   # 提交前应零告警
```

更多见 [AGENTS.md](AGENTS.md)（架构不变量）和 [ui/README.md](ui/README.md)（组件库集成指南）。

## 许可

MIT（[LICENSE](LICENSE)）
