# ui — agent-memory 前端

前端拆成两个 pnpm workspace 包（lockfile 在仓库根目录）：

- **`ui/lib` → `@agent-memory/ui`**：可嵌入的 Vue3 组件库——工作台 `MemoryWorkspace`、
  管理面板 `AdminPanel`、概况弹窗 `OpsDialog`、便捷壳 `MemoryAdmin` 及配套弹层，
  供其他 Vue3 系统作为组件集成。
- **`ui/app` → `@agent-memory/app`**：独立站点薄壳，页面主体即工作台；构建产物输出
  `ui/dist/`（不入库）。

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
  fetch: myAuthedFetch, // 可选：注入带鉴权头/拦截器的 fetch，默认 globalThis.fetch
  defaultPageSize: 20,
})
</script>

<template>
  <!-- 整台工作台：标签侧栏 + 记忆列表，自带编辑弹窗与两侧联动刷新 -->
  <MemoryWorkspace />
  <!-- AdminPanel / OpsDialog / 各弹窗组件按宿主需要挂入弹窗或页面 -->
</template>
```

- 工作台在窄容器自动切 compact 布局（侧栏折叠、弹层收窄），可放进宿主任意尺寸的卡片/抽屉。
- 面板 props：`show-header: false` 隐藏标题区（宿主页面已有标题时只要工具栏 + 列表）；
  `title` / `subtitle` 覆盖默认文案（默认文案随界面语言）。
- `MemoryAdmin` 高度默认撑满父容器，嵌入时可用 CSS 变量 `--memory-admin-height` 覆盖（如 `480px`）。
- **多语言**：内置 vue-i18n（`zh` / `en`），默认跟随浏览器语言；`provideMemoryUI({ locale: 'en' })`
  固定语言，或运行时调用 `setMemoryUILocale('en')` 全库生效。Element Plus 自身文案由宿主的
  `<el-config-provider :locale>` 控制。
- 主题跟随 Element Plus 的 `--el-*` CSS 变量：宿主引入 dark css-vars 并在 `<html>` 上加
  `dark` 类即可暗色化，组件库自身不定义颜色常量。

## 常用命令

根目录一次 `pnpm install`；以下命令在仓库根目录执行（`format` 覆盖包括本文件在内的全部非 Rust 源）：

```bash
pnpm build        # 先 lib 后 app（改完前端必须跑）
pnpm test
pnpm typecheck
pnpm format
pnpm dev          # 内置 mock API，无需后端即可完整开发调试（重启复位）
pnpm dev:live     # 连真实后端，验证鉴权等服务端语义
```

## 约定

- **全部源码 TypeScript**（strict）：禁止新建 .js 源文件
- 界面数据全部走 `/api/*`（服务端复用 MCP 工具层的 handler，校验语义一致）
- **所有 v-html 入口必须消毒**：统一出口 `ui/lib/src/markdown.ts`，不得绕过
- 删除等破坏性操作必须二次确认；错误提示统一经 `toast.ts` 出口
- 组件测试放 `*.test.ts`（vitest + happy-dom；挂载测试 stub `ResizeObserver`）；
  消毒相关测试必须标 `// @vitest-environment jsdom`
