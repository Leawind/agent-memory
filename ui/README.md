# ui — agent-memory 管理界面

Vue3 + Element Plus + Vite + **TypeScript**（strict）。构建产物 `dist/` 提交入库，由 Rust 侧 rust-embed 编译期嵌入二进制（`debug-embed`，本地 `cargo test` 也用真实产物）。

## 常用命令

```bash
npm install       # 首次安装依赖（package-lock.json 已锁定版本）
npm run dev       # 开发模式：热更新，/api 与 /mcp 代理到 127.0.0.1:8899
npm test          # vitest 单元测试（查询串组装、API 封装约定）
npm run typecheck # vue-tsc 类型检查（strict）
npm run build     # 构建到 dist/（改完前端必须跑）
```

改完前端后的发布流程：

```bash
npm run build     # 产物入库
cd .. && ./build.sh   # 重新编译 Rust，嵌入新产物
```

## 约定

- **全部源码使用 TypeScript**：组件 `<script setup lang="ts">`，服务端响应类型集中在 `src/types.ts`；禁止再新建 .js 源文件
- **dist 必须提交**：cargo 构建不依赖 node，二进制是唯一交付物
- 界面数据全部走 `/api/*`（服务端复用 MCP 工具层的 handler，校验语义一致），不在前端做业务校验以外的假设
- 错误提示统一 `ElMessage.error`，删除等破坏性操作必须二次确认
- 组件测试放 `*.test.ts`（vitest + happy-dom；挂载测试需 stub `ResizeObserver`，见 `src/test-setup.ts`）
