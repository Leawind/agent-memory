# ui — agent-memory 前端

前端拆成两个 npm workspace 包（lockfile 在仓库根目录）：

- **`ui/lib` → `@agent-memory/ui`**：可嵌入的 Vue3 组件库。导出四个自包含可复用面板
  `MemoriesPanel` / `TagsPanel` / `OpsPanel`（只读概况）/ `AdminPanel`（admin 专属：身份、鉴权
  开关、自定义提示词、备份、体检，经 `who` prop 门控），均支持 `show-header` /
  `title` / `subtitle` props 裁剪；另附便捷壳 `MemoryAdmin`（sidebar/tabs 双布局）与
  弹层组件（编辑对话框、详情抽屉、Markdown 视图），供其他 Vue3 系统作为组件集成。
- **`ui/app` → `@agent-memory/app`**：独立管理站点薄壳（Modrinth 风格顶部导航栏 + 主题/语言
  切换 + 多身份令牌下拉），直接组装四个面板（「管理」页仅对 admin 能力身份显示）。
  构建产物输出到 `ui/dist/` 并提交入库，由 Rust 侧 rust-embed
  编译期嵌入二进制。

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
import { MemoriesPanel, provideMemoryUI } from '@agent-memory/ui'

// 任意祖先组件注入一次即可；全部字段可省略（默认同源根路径部署）
provideMemoryUI({
  baseUrl: 'http://127.0.0.1:8899', // API 前缀，默认 ''
  fetch: myAuthedFetch,             // 可选：注入带鉴权头/拦截器的 fetch，默认 globalThis.fetch
  defaultPageSize: 20,
})
</script>

<template>
  <!-- 整台管理台；嵌入宿主页面时用 tabs 布局更省空间 -->
  <MemoryAdmin layout="tabs" />
  <!-- 或只嵌入单个面板（自包含：面板头 + 工具栏 + 表格） -->
  <!-- <MemoriesPanel /> -->
</template>
```

- 面板在窄容器（<720px，ResizeObserver 实测）自动切 compact：工具栏换行、隐藏低优先级列、弹层收窄，
  可放进宿主任意尺寸的卡片/抽屉。
- 面板 props：`show-header: false` 隐藏标题/副标题区（宿主页面已有标题时只要工具栏+表格）；
  `title` / `subtitle` 覆盖默认文案（默认文案随界面语言）。
- `MemoryAdmin` 高度默认撑满父容器，嵌入时可用 CSS 变量 `--memory-admin-height` 覆盖（如 `480px`）。
- **多语言**：内置 vue-i18n（`zh` / `en`），默认 `auto` 跟随浏览器语言；`provideMemoryUI({ locale: 'en' })`
  固定语言，或运行时调用 `setMemoryUILocale('en')` 即全库生效（时间格式同步切换）。
  Element Plus 自身文案（分页等）由宿主的 `<el-config-provider :locale>` 控制。
- 主题跟随 Element Plus 的 `--el-*` CSS 变量：宿主引入 `element-plus/theme-chalk/dark/css-vars.css`
  并在 `<html>` 上加 `dark` 类即可暗色化，组件库自身不定义颜色常量。

## 常用命令

根目录一次 `npm install`；以下经根脚本转发到两个 ui workspace：

```bash
npm run build     # 先 lib 后 app；app 产物输出 ui/dist（改完前端必须跑）
npm test          # vitest：lib（配置注入/API 封装/查询串/Markdown 消毒/三面板挂载）+ app 壳
npm run typecheck # vue-tsc 两包（strict）
npm run format    # Prettier（配置在根 .prettierrc.json：无分号、单引号、120 列）
npm run dev       # 仅 app：热更新。默认启用内置 mock API（ui/app/mock/，内存假数据，
                  # 无需 Rust 后端；重启复位），终端横幅会打印预置身份 token
npm run dev:live  # 仅 app：连真实后端，/api、/mcp、/health 代理到 127.0.0.1:8899
```

### 前端独立开发（mock 模式）

`npm run dev` 默认加载 `ui/app/mock/api.ts`（Vite dev 中间件）：内存假数据 + 固定 admin 身份，
四个面板（含管理页）直接完整渲染，改代码即时热更新，**不依赖任何后端**。
增删改会真的改内存数组（保存后列表可见变化），重启 dev server 复位。
鉴权/搜索回退等服务端语义的联调用 `npm run dev:live` 连真实后端验证。

## 约定

- **全部源码 TypeScript**（strict）：服务端响应类型集中在 `ui/lib/src/types.ts`；禁止新建 .js 源文件
- **dist 必须提交**：cargo 构建不依赖 node，二进制是唯一交付物
- ⚠️ **rust-embed 陷阱**：仅在编译期已存在的文件会被跟踪，**新增**的 dist 产物文件不会触发 Rust 重编译——
  改完前端后 `cargo build` 前先 `cargo clean -p agent-memory`，否则二进制里可能还是旧 UI
- 界面数据全部走 `/api/*`（服务端复用 MCP 工具层的 handler，校验语义一致）
- **所有 v-html 入口必须消毒**：Markdown 走 `renderMarkdown()`，服务端 HTML 片段（搜索 snippet）走
  `sanitizeHtml()`，统一出口 `ui/lib/src/markdown.ts`，不得绕过
- 错误提示统一经 `toast.ts` 出口（`toastSuccess`/`toastError`：可点击关闭、位置让开顶栏），
  删除等破坏性操作必须二次确认；数据操作在 composables，动作失败抛错、面板层统一 toast
- 组件测试放 `*.test.ts`（vitest + happy-dom；挂载测试 stub `ResizeObserver`，见各包 `src/test-setup.ts`；
  DOMPurify 相关测试标 `// @vitest-environment jsdom`）
