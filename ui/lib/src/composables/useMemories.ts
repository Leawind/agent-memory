// 记忆面板的状态机：列表/搜索双模式、竞态防护、翻页回退、标签 diff。
// 动作失败时抛出 Error，由面板层统一 toast。
import { computed, onMounted, ref } from 'vue'
import { useApiClient } from '../api/client'
import { buildMemoriesQuery, isSearchMode } from '../query'
import { sanitizeHtml } from '../markdown'
import { useMemoryConfig } from '../config'
import { createMemory, deleteMemory, getMemory, updateMemory } from '../api/memories'
import { listTags } from '../api/tags'
import type { MemoryFull, MemoryListResp, MemorySearchResp, MemorySummary, SearchResult, TagListResp } from '../types'

/** 编辑器表单的形状：id 为 null 表示新建 */
export interface MemoryDraft {
  id: string | null
  summary: string
  content: string
  tags: string[]
}

export function useMemories() {
  const client = useApiClient()
  const { defaultPageSize } = useMemoryConfig()

  const query = ref('')
  const tagFilter = ref('')
  const sort = ref<'updated_at' | 'created_at' | 'id'>('updated_at')
  const order = ref<'asc' | 'desc'>('desc')
  const page = ref(1)
  const pageSize = ref(defaultPageSize)
  const rows = ref<MemorySummary[]>([])
  const searchResults = ref<SearchResult[]>([])
  const total = ref(0)
  const note = ref('')
  const loading = ref(false)
  const tagOptions = ref<string[]>([])

  const searching = computed(() => isSearchMode(query.value))

  // 搜索词/过滤标签变化属于新的查询意图：从第一页重新开始
  function onSearch() {
    page.value = 1
    return reload()
  }

  function buildQuery() {
    return buildMemoriesQuery({
      query: query.value,
      tagFilter: tagFilter.value,
      sort: sort.value,
      order: order.value,
      page: page.value,
      pageSize: pageSize.value,
    })
  }

  // 请求序号：连续触发查询时（输入回车/清空/翻页）丢弃迟到的过期响应，
  // 避免慢的旧结果覆盖新结果
  let requestSeq = 0

  async function reload() {
    const seq = ++requestSeq
    loading.value = true
    try {
      const qs = buildQuery()
      if (isSearchMode(query.value)) {
        const data = (await client.get<MemorySearchResp>(`/api/memories?${qs}`)) as MemorySearchResp
        if (seq !== requestSeq) return
        // 服务端片段是 HTML（<mark> 高亮），经 DOMPurify 消毒后再进 v-html
        searchResults.value = (data.results ?? []).map((r) => ({ ...r, snippet: sanitizeHtml(r.snippet) }))
        total.value = data.total_matches ?? 0
        note.value = ''
      } else {
        const data = (await client.get<MemoryListResp>(`/api/memories?${qs}`)) as MemoryListResp
        if (seq !== requestSeq) return
        // 删除/过滤后当前页可能超出范围：回退到最后一页重新拉取
        const totalPages = Math.max(1, Math.ceil(data.total / pageSize.value))
        if (data.memories?.length === 0 && data.total > 0 && page.value > totalPages) {
          page.value = totalPages
          loading.value = false
          return reload()
        }
        rows.value = data.memories ?? []
        total.value = data.total ?? 0
        note.value = data.note ?? ''
      }
    } finally {
      if (seq === requestSeq) loading.value = false
    }
  }

  async function loadTagOptions() {
    try {
      const data = (await listTags(client)) as TagListResp
      tagOptions.value = (data.tags ?? []).map((t) => t.name)
    } catch {
      /* 静默：标签下拉失败不阻塞主列表 */
    }
  }

  /** 新建或编辑（标签为目标集合，内部换算成 add_tags / remove_tags）。成功后刷新列表。 */
  async function saveMemory(draft: MemoryDraft) {
    if (draft.id) {
      const current: MemoryFull = await getMemory(client, draft.id)
      const before = new Set<string>(current.tags)
      const after = new Set<string>(draft.tags)
      await updateMemory(client, draft.id, {
        summary: draft.summary,
        content: draft.content,
        add_tags: [...after].filter((t) => !before.has(t)),
        remove_tags: [...before].filter((t) => !after.has(t)),
      })
    } else {
      await createMemory(client, { summary: draft.summary, content: draft.content, tags: draft.tags })
    }
    await Promise.all([reload(), loadTagOptions()])
  }

  async function removeMemory(id: string) {
    await deleteMemory(client, id)
    await reload()
  }

  onMounted(() => {
    void reload().catch(() => {}) // 首屏错误由调用方 toast（面板层包装）
    loadTagOptions()
  })

  return {
    query,
    tagFilter,
    sort,
    order,
    page,
    pageSize,
    rows,
    searchResults,
    total,
    note,
    loading,
    tagOptions,
    searching,
    onSearch,
    reload,
    loadTagOptions,
    saveMemory,
    removeMemory,
  }
}

export type MemoriesStore = ReturnType<typeof useMemories>
