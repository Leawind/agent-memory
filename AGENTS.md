# AGENTS.md — agent-memory 项目指南

自托管、低占用的 MCP 记忆服务器（stdio），供 AI agent 长期保存/检索知识。纯 Rust，仅依赖 serde/serde_json。

## 常用命令

```bash
cargo test        # 全部测试（单元 + 端到端，端到端会真实启动服务器进程）
cargo clippy --all-targets   # 提交前应零告警
./build.sh        # 构建 release 并部署到 bin/agent-memory.exe
                  # ⚠️ 改完代码必须跑，否则 bin/ 里是旧二进制（MCP 配置指向它）
```

注意：本机配置了全局共享的 CARGO_TARGET_DIR（编译产物不在 ./target），`build.sh` 会从 cargo 元数据定位真实输出目录。

运维子命令 `stats` / `export` 是只读的（读不加锁，靠存储层原子替换保证并发安全）；改存储层时不要破坏这个性质。

## 模块结构

```
src/
  main.rs        CLI（--data/--version）+ stdio 事件循环（stdin 读 JSON-RPC 行，stdout 写响应）
  protocol.rs    MCP 协议层：initialize / ping / tools/list / tools/call，通知不回包
  tools/mod.rs   工具入口：锁内"重读→执行→写回"，分发，未知参数校验
  tools/defs.rs  工具清单 + JSON Schema（对 agent 的契约，唯一权威来源）
  tools/params.rs 参数解析/校验（值从严错报、写法从宽：单字符串可当数组）
  tools/tag_ops.rs / memory_ops.rs  业务处理器
  model.rs       纯数据模型：Tag / Memory / id·标签名归一化 / API 限制常量
  store.rs       持久化：原子写(.tmp+rename) + .bak 备份 + 跨进程锁(<file>.lock)
  search.rs      关键词搜索：AND 语义、加权（标签>摘要>正文）、中文子串匹配
```

## 架构不变量（改代码前必读）

1. **stdout 是协议通道**：任何日志/诊断只能写 stderr，否则破坏 MCP stdio 传输。
2. **写盘纪律**：所有变更必须经 `execute_with_store`（锁内重读磁盘→修改→原子写回）。
   每个请求都从磁盘重载是多进程安全的关键，不要引入长驻内存状态做写路径。
   注意：读操作也会全量重写（always-save，简化正确性推理）。这是有意取舍——
   若要改为"只读跳过写盘"，必须先补一个"只读分类错误会被测试捕获"的护栏，
   否则把变更处理器误标为只读会造成静默丢数据。
3. **数据安全优先**：主文件损坏且无备份时必须报错退出，绝不能在坏数据上继续写；
   save() 必须保持原子性（先写 tmp 再 rename）。
4. **defs.rs 是契约**：新增/修改工具先改 defs.rs 的 schema（含描述），参数校验自动从
   properties 派生；渐进式披露约定不变——list/search 永不返回 content，只有 memory_get 返回。
5. **兼容性**：数据文件格式走 FORMAT_VERSION，只升不破；协议版本支持
   2024-11-05 / 2025-03-26 / 2025-06-18。
6. **panic 不落盘**：处理逻辑 panic 时（catch_unwind 捕获）磁盘数据保持原样，依赖
   "成功才写盘"这一顺序，调整 execute_with_store 时不要破坏它。

## 测试约定

- 端到端测试在 `tests/e2e.rs`，直接 spawn 二进制走真实 stdio；数据文件一律用
  `AGENT_MEMORY_PATH` 指到临时目录，绝不碰用户真实数据（~/.agent-memory）。
- 新增工具时至少覆盖：正常流、参数错误、渐进式披露边界（正文不泄露）。
