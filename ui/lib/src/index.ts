// Public entry point of @agent-memory/ui
// CSS (design tokens + Markdown typography) is extracted into dist/style.css during the
// lib build; the host must import it itself: import '@agent-memory/ui/style.css'

// Config injection
export { MemoryUIConfigKey, provideMemoryUI, useMemoryConfig } from './config'
export type { MemoryUIConfig, ResolvedMemoryUIConfig } from './config'

// i18n (standalone-scoped vue-i18n instance; the host can switch languages programmatically)
export { memoryUIi18n, setMemoryUILocale, applyMemoryUILocalePreference, currentMemoryUILocale, t } from './i18n'
export type { MemoryUILocale, MemoryUILocaleOption } from './i18n'

// Components
export { default as MemoryAdmin } from './components/MemoryAdmin.vue'
export { default as MemoriesPanel } from './components/MemoriesPanel.vue'
export { default as TagsPanel } from './components/TagsPanel.vue'
export { default as OpsDialog } from './components/OpsDialog.vue'
export { default as AdminPanel } from './components/AdminPanel.vue'
export { default as MemoryEditorDialog } from './components/MemoryEditorDialog.vue'
export { default as FormDialog } from './components/FormDialog.vue'
export { default as MarkdownView } from './components/MarkdownView.vue'

// API and utilities
export { createApiClient, useApiClient } from './api/client'
export type { ApiClient } from './api/client'
export { renderMarkdown } from './markdown'
export { formatTime, formatSize } from './format'
export { buildMemoriesQuery, isSearchMode } from './query'
export { useMemories } from './composables/useMemories'
export { useTags } from './composables/useTags'
export { useOps } from './composables/useOps'
export { useAdmin } from './composables/useAdmin'
export { toastSuccess, toastError } from './toast'

// Server types
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
  WhoAmI,
} from './types'

// Let the vite lib build extract this library's styles
import './styles/tokens.css'
import './styles/markdown.css'
