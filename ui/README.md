# ui — agent-memory 前端

前端拆成两个 pnpm workspace 包（lockfile 在仓库根目录）：

- **`ui/lib` → `@agent-memory/ui`**：可嵌入的 Vue3 组件库——工作台 `MemoryWorkspace`
  （标签侧栏 + 记忆列表 + 编辑弹窗，两侧数据联动）、管理面板 `AdminPanel`、概况弹窗
  `OpsDialog`、便捷壳 `MemoryAdmin` 及配套弹层，供其他 Vue3 系统作为组件集成。
- **`ui/app` → `@agent-memory/app`**：独立站点薄壳（顶栏 + 主题/语言切换 + 多身份令牌
  管理），页面主体即工作台。构建产物输出 `ui/dist/`（不入库），由 rust-embed 编译期
  嵌入二进制；dist 缺失时 build.rs 落占位页兜底。

组件行为细节以代码与测试为准，本文件只写集成契约与约定。

## 组件库集成指南

### 安装

组件库不打包 Vue / Element Plus，它们是 peer dependencies——宿主系统自带时零重复、主题一致：

```
peerDependencies: vue ^3.5 · element-plus >=2.9 <3 · @element-plus/icons-vue ^2.3
```

宿主需自行引入 Element Plus 样式（全量或按需，按需时别漏 `ElMessage`/`ElMessageBox`）与组件库样式：

```ts
import 'element-plus/dist/index.css'
import '@agent-memory/ui/style.css'
```

模板里的 `el-*` 组件已在构建期注入具名导入（unplugin-vue-components），宿主**无需** `app.use(ElementPlus)`。

### 用法

```ts
import { provideMemoryUI } from '@agent-memory/ui'
```

```vue
<script setup lang="ts">
import { MemoryWorkspace, provideMemoryUI } from '@agent-memory/ui'

// 任意祖先组件注入一次即可；全部字段可省略（默认同源根路径部署）
provideMemoryUI({
  baseUrl: 'http://127.0.0.1:8899', // API 前缀，默认 ''
  fetch: myAuthedFetch,             // 可选：注入带鉴权头/拦截器的 fetch，默认 globalThis.fetch
  defaultPageSize: 20,
})
</script>

<template>
  <!-- 整台工作台：标签侧栏 + 记忆列表，自带编辑弹窗与两侧联动刷新 -->
  <MemoryWorkspace />
  <!-- AdminPanel / OpsDialog / 各弹窗组件按宿主需要挂入弹窗或页面 -->
</template>
```

- 工作台在窄容器（<720px，ResizeObserver 实测）自动切 compact：侧栏折叠为列表上方的限高块、
  拖拽条隐藏、工具栏换行、弹层收窄，可放进宿主任意尺寸的卡片/抽屉。
- 面板 props：`show-header: false` 隐藏标题/副标题区（宿主页面已有标题时只要工具栏+列表）；
  `title` / `subtitle` 覆盖默认文案（默认文案随界面语言）。
- `MemoryAdmin` 高度默认撑满父容器，嵌入时可用 CSS 变量 `--memory-admin-height` 覆盖（如 `480px`）。
- **多语言**：内置 vue-i18n（`zh` / `en`），默认 `auto` 跟随浏览器语言；`provideMemoryUI({ locale: 'en' })`
  固定语言，或运行时调用 `setMemoryUILocale('en')` 即全库生效（时间格式同步切换）。
  Element Plus 自身文案（分页等）由宿主的 `<el-config-provider :locale>` 控制。
- 主题跟随 Element Plus 的 `--el-*` CSS 变量：宿主引入 `element-plus/theme-chalk/dark/css-vars.css`
  并在 `<html>` 上加 `dark` 类即可暗色化，组件库自身不定义颜色常量。

## 常用命令

根目录一次 `pnpm install`；以下经根脚本转发到两个 ui workspace：

```bash
pnpm build        # 先 lib 后 app；app 产物输出 ui/dist（改完前端必须跑）
pnpm test        # vitest：lib（配置注入/API 封装/查询串/Markdown 消毒/工作台与弹窗挂载）+ app 壳
pnpm typecheck    # vue-tsc 两包（strict）
pnpm format       # Prettier（配置在根 .prettierrc.json：无分号、单引号、120 列）
pnpm dev          # 仅 app：热更新。默认启用内置 mock API（ui/app/mock/，内存假数据，
                  # 无需 Rust 后端；重启复位），终端横幅会打印预置身份 token
pnpm dev:live     # 仅 app：连真实后端，/api、/mcp、/health 代理到 127.0.0.1:8899
```

### 前端独立开发（mock 模式）

`pnpm dev` 默认加载 `ui/app/mock/api.ts`（Vite dev 中间件）：内存假数据 + 固定 admin 身份，
三个面板（含管理页）与概况弹窗直接完整渲染，改代码即时热更新，**不依赖任何后端**。
增删改会真的改内存数组（保存后列表可见变化），重启 dev server 复位。
鉴权/搜索回退等服务端语义的联调用 `pnpm dev:live` 连真实后端验证。

## 约定

- **全部源码 TypeScript**（strict）：服务端响应类型集中在 `ui/lib/src/types.ts`；禁止新建 .js 源文件
- **构建产物不入库**：`ui/dist`、`ui/lib/dist` 均在 .gitignore；dist 缺失时 build.rs
  生成占位 index.html，保证 cargo 构建不依赖 node（全新 clone 可直接编译）
- build.rs 以 `rerun-if-changed=ui/dist` 跟踪目录：改完前端 `pnpm build` 后直接
  `cargo build` 即可重新嵌入，无需 cargo clean
- 界面数据全部走 `/api/*`（服务端复用 MCP 工具层的 handler，校验语义一致）
- **所有 v-html 入口必须消毒**：Markdown 走 `renderMarkdown()`，服务端 HTML 片段（搜索 snippet）走
  `sanitizeHtml()`，统一出口 `ui/lib/src/markdown.ts`，不得绕过
- 错误提示统一经 `toast.ts` 出口（`toastSuccess`/`toastError`：可点击关闭、位置让开顶栏），
  删除等破坏性操作必须二次确认；数据操作在 composables，动作失败抛错、面板层统一 toast
- 组件测试放 `*.test.ts`（vitest + happy-dom；挂载测试 stub `ResizeObserver`，见各包 `src/test-setup.ts`；
  DOMPurify 相关测试标 `// @vitest-environment jsdom`）
