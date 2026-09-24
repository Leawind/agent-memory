# agent-memory

自托管、低占用的 **MCP 记忆服务器**，供 AI agent（ZCode / Claude Desktop / Cursor 等）长期保存和检索知识。

纯 Rust 实现，仅依赖 `serde` / `serde_json`，无运行时、无数据库、无网络：

| 指标 | 数值 |
|---|---|
| 二进制体积 | ~424 KB |
| 常驻内存 RSS | ~3.8 MB |
| 私有内存 | ~600 KB |
| 冷启动 | 毫秒级 |

## 功能

- **标签管理**（tag 由 agent 自己维护分类体系）：`tag_create` / `tag_list` / `tag_rename` / `tag_delete`
- **记忆管理**：`memory_create` / `memory_update` / `memory_delete`
- **渐进式披露**：`memory_list` / `memory_search` 只返回 **id + 标签 + 摘要 + 片段**（便宜的第一层），agent 挑出值得读的 id 后用 `memory_get` 才取回**完整正文**（昂贵的第二层），最大限度节省上下文窗口
- **关键词搜索**：`memory_search`，空格分词、全部词命中（AND）、加权排序（标签精确 40 / 标签子串 25 > 摘要 10 > 正文 3）；匹配为大小写不敏感的子串查找，**中文无需分词直接命中**；支持 `offset`/`limit` 翻页
- **参数契约校验**：拼错的参数名立刻报错并列出合法参数（合法名直接从 JSON Schema 派生），避免调用方以为参数生效了
- **重复检测**：`memory_create` 的摘要与既有记忆相同（大小写不敏感）时，响应中列出 `duplicate_of`，引导改用 `memory_update` 而非重复存储

每条记忆的属性：`id`、`tags`（标签，agent 自管理）、`summary`（摘要）、`content`（正文）、`created_at`、`updated_at`。

## 数据存储与安全

- 数据保存在**单个 JSON 文件**（默认 `~/.agent-memory/memory.json`，可用环境变量 `AGENT_MEMORY_PATH` 或 `--data <path>` 覆盖）
- **原子写入**：先写 `.tmp` 再 rename 替换，任何时刻文件都是完整内容；替换前旧文件拷为 `.bak`
- **损坏恢复**：主文件损坏时自动回退加载 `.bak`；两者都坏则拒绝启动，绝不覆盖数据
- **跨进程文件锁**（`<file>.lock`）：多个客户端进程（多个 agent 会话）同时使用同一数据文件时，每次请求在锁内"重读磁盘 → 修改 → 原子写回"，不会互相覆盖

## 架构

```
src/
  main.rs        CLI（--data/--version）+ stdio 事件循环
  protocol.rs    MCP 协议层（initialize / ping / tools/list / tools/call）
  tools/
    mod.rs       工具入口：锁内"重读→执行→写回"，分发，未知参数校验
    defs.rs      工具清单 + JSON Schema（对 agent 的契约，参数校验的权威来源）
    params.rs    参数解析与校验
    tag_ops.rs   标签增删查改
    memory_ops.rs 记忆增删改查、浏览、搜索
  model.rs       纯数据模型（Tag / Memory / 归一化 / API 限制常量）
  store.rs       持久化（原子写 + 备份 + 跨进程锁）
  search.rs      关键词搜索与评分
```

架构不变量与开发约定见 [AGENTS.md](AGENTS.md)。

## 构建

```bash
./build.sh     # cargo build --release 并部署到 bin/agent-memory.exe
```

⚠️ ZCode MCP 配置指向 `bin/agent-memory.exe`，改完代码后必须重新运行 `./build.sh`，否则注册的是旧二进制。

测试：

```bash
cargo test   # 36 个单元测试 + 6 个端到端测试（真实进程走 stdio，含多进程并发与竞态写入）
```

## 运维子命令

```bash
./bin/agent-memory.exe stats                    # 数据概况：条数、标签数、文件大小、最近更新
./bin/agent-memory.exe doctor                   # 数据体检：孤儿引用/大小写冲突标签组/空字段（有问题时退出码 1）
./bin/agent-memory.exe export backup.json       # 导出可读 JSON 备份（文件已存在则拒绝覆盖）
```

两者均为只读操作：存储采用原子替换写入，读取无需加锁，与运行中的服务器并发安全。

## 性能包络

本机（Windows，2026-09 实测）1000 条记忆（文件 ~420KB）时：

- 单次创建均摊 ~17ms（含每次请求的全量"重读 + 重写"）
- 进程内搜索毫秒级（跨进程测量时进程启动占大头）

设计上每次请求都会全量重写数据文件（简单性与多进程正确性的取舍，见 AGENTS.md），因此写入成本随条目数线性增长：**数千条以内完全无感**；上万条仍可用，但写入会明显变慢——真到那个规模时优先做标签清理与归档，而不是改存储引擎。

## 接入 MCP 客户端

本服务器使用 **stdio 传输**（换行分隔的 JSON-RPC 2.0），协议版本支持 2024-11-05 / 2025-03-26 / 2025-06-18。

### ZCode（本机已注册）

已写入用户级配置 `~/.zcode/cli/config.json` 的 `mcp.servers`：

```json
"agent-memory": {
  "type": "stdio",
  "command": "D:\\Workspace\\FromGithub\\Leawind\\agent-memory\\bin\\agent-memory.exe",
  "args": [],
  "env": {}
}
```

重启 ZCode 后自动连接，即可使用 `mcp__agent-memory__memory_search` 等工具。

### Claude Desktop / Cursor 等通用写法

```json
{
  "mcpServers": {
    "agent-memory": {
      "command": "D:\\Workspace\\FromGithub\\Leawind\\agent-memory\\bin\\agent-memory.exe"
    }
  }
}
```

### 配置项

| 环境变量 | 说明 |
|---|---|
| `AGENT_MEMORY_PATH` | 数据文件路径，默认 `~/.agent-memory/memory.json` |
| `AGENT_MEMORY_LOCK_WAIT_MS` | 存储锁等待上限（毫秒），默认 5000，最大 60000；设小可快速失败 |

命令行：`--data <path>`（优先级最高）、`--version`、`--help`。

> 想按项目隔离记忆：在该项目的 MCP 配置里给 `env` 加 `AGENT_MEMORY_PATH` 指向项目内路径即可；全局共享则用默认路径，用标签区分项目。

## 工具一览

| 工具 | 只读 | 说明 |
|---|---|---|
| `tag_create(name, description?)` | | 新建标签，重名报错；存在仅大小写不同的标签时在 `similar_existing` 里提示（非阻塞） |
| `tag_list()` | ✓ | 全部标签 + 记忆计数 + 最后使用时间 |
| `tag_rename(old_name, new_name?, description?)` | | 重命名（同步所有引用）/ 改描述 |
| `tag_delete(name, mode?)` | | `detach`（默认，只摘标签）/ `purge`（连带删除记忆） |
| `memory_create(summary, content, tags?)` | | 新建记忆；未知标签自动创建并在 `tags_autocreated` 里汇报；摘要重复时在 `duplicate_of` 里提示 |
| `memory_list(tag?, sort?, order?, offset?, limit?)` | ✓ | 分页浏览，**只返回摘要** |
| `memory_search(query, tags?, offset?, limit?)` | ✓ | 关键词搜索，返回摘要 + 正文片段，支持翻页 |
| `memory_get(ids)` | ✓ | 取 1–50 条**完整正文**（渐进式披露第二层） |
| `memory_update(id, summary?, content?, add_tags?, remove_tags?)` | | 增量更新，无需先取当前标签列表 |
| `memory_delete(ids)` | | 永久删除 1–50 条 |

## 推荐的 agent 使用方式

1. **存**：`memory_create`，摘要写得精确自洽（未来扫描全靠它），标签自选自管
2. **查**：先 `memory_search` / `memory_list`（便宜），再对值得读的 id 调 `memory_get`（贵）——不要一上来取全文
3. **管**：定期 `tag_list` 检查分类体系，用 `tag_rename` 纠错（会发现并合并拼写偏差），过期记忆用 `memory_delete` 清理
