// @agent-memory/ui 公共出口
// CSS（设计令牌 + Markdown 排版）在 lib 构建时抽取为 dist/style.css，
// 宿主需自行引入：import '@agent-memory/ui/style.css'

// 配置注入
export { MemoryUIConfigKey, provideMemoryUI, useMemoryConfig } from './config'
export type { MemoryUIConfig, ResolvedMemoryUIConfig } from './config'

// 组件
export { default as MemoryAdmin } from './components/MemoryAdmin.vue'
export { default as MemoriesPanel } from './components/MemoriesPanel.vue'
export { default as TagsPanel } from './components/TagsPanel.vue'
export { default as OpsPanel } from './components/OpsPanel.vue'
export { default as MemoryEditorDialog } from './components/MemoryEditorDialog.vue'
export { default as MemoryDetailDrawer } from './components/MemoryDetailDrawer.vue'
export { default as MarkdownView } from './components/MarkdownView.vue'

// API 与工具
export { createApiClient, useApiClient } from './api/client'
export type { ApiClient } from './api/client'
export { renderMarkdown, sanitizeHtml } from './markdown'
export { formatTime, formatSize } from './format'
export { buildMemoriesQuery, isSearchMode } from './query'
export { useMemories } from './composables/useMemories'
export { useTags } from './composables/useTags'
export { useOps } from './composables/useOps'

// 服务端类型
export type {
  DoctorResp,
  HealthInfo,
  ImportResp,
  MemoryFull,
  MemoryListResp,
  MemorySearchResp,
  MemorySummary,
  SearchResult,
  StatsInfo,
  TagListResp,
  TagView,
} from './types'

// 让 vite lib 构建抽取本库样式
import './styles/tokens.css'
import './styles/markdown.css'
