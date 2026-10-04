// Memory panel state machine: dual list/search modes, race protection, page-range fallback,
// tag diffing.
// Failed actions throw an Error; the panel layer shows the toast uniformly.
import { computed, onMounted, ref } from 'vue'
import { useApiClient } from '../api/client'
import { buildMemoriesQuery, isSearchMode } from '../query'
import { useMemoryConfig } from '../config'
import { createMemory, deleteMemory, getMemory, updateMemory } from '../api/memories'
import { listTags } from '../api/tags'
import { t } from '../i18n'
import type { MemoryFull, MemoryListResp, MemorySearchResp, MemorySummary, SearchResult, TagListResp } from '../types'

/** Shape of the editor form: id of null means creating new */
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
  /** Search mode: auto (server decides hybrid/keyword per its config) / keyword / hybrid */
  const mode = ref<'auto' | 'keyword' | 'hybrid'>('auto')
  const sort = ref<'updated_at' | 'id'>('updated_at')
  const order = ref<'asc' | 'desc'>('desc')
  const page = ref(1)
  const pageSize = ref(defaultPageSize)
  const rows = ref<MemorySummary[]>([])
  const searchResults = ref<SearchResult[]>([])
  /** Semantic fallback notice: non-empty when a hybrid request fell back to keyword search because the embedding service was unavailable (told explicitly, never silently) */
  const note = ref('')
  const total = ref(0)
  const loading = ref(false)
  const tagOptions = ref<string[]>([])

  const searching = computed(() => isSearchMode(query.value))

  // A change to the search term / tag filter is a new query intent: restart from page 1
  function onSearch() {
    page.value = 1
    return reload()
  }

  function buildQuery() {
    return buildMemoriesQuery({
      query: query.value,
      tagFilter: tagFilter.value,
      mode: mode.value,
      sort: sort.value,
      order: order.value,
      page: page.value,
      pageSize: pageSize.value,
    })
  }

  // Request sequence number: when queries fire in quick succession (Enter in the input /
  // clearing / paging), late stale responses are discarded so a slow old result never
  // overwrites a newer one
  let requestSeq = 0

  async function reload() {
    const seq = ++requestSeq
    loading.value = true
    try {
      const qs = buildQuery()
      if (isSearchMode(query.value)) {
        const data = (await client.get<MemorySearchResp>(`/api/memories?${qs}`)) as MemorySearchResp
        if (seq !== requestSeq) return
        // Snippets are plain text from the server and are rendered via text interpolation — never v-html
        searchResults.value = data.results ?? []
        total.value = data.total_matches ?? 0
        // When a hybrid request fell back to keyword search because the embedding service was
        // unavailable, say so explicitly (never silently)
        note.value = data.semantic_fallback ? t('memories.semanticFallback') : ''
      } else {
        const data = (await client.get<MemoryListResp>(`/api/memories?${qs}`)) as MemoryListResp
        if (seq !== requestSeq) return
        // After deletion/filtering the current page may fall out of range: fall back to the
        // last page and refetch
        const totalPages = Math.max(1, Math.ceil(data.total / pageSize.value))
        if (data.memories?.length === 0 && data.total > 0 && page.value > totalPages) {
          page.value = totalPages
          loading.value = false
          return reload()
        }
        rows.value = data.memories ?? []
        total.value = data.total ?? 0
        note.value = ''
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
      /* Silent: a failed tag dropdown must not block the main list */
    }
  }

  /** Create or edit (tags are the target set, internally converted to add_tags / remove_tags). Refreshes the list on success. */
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
    void reload().catch(() => {}) // first-load errors are toasted by the caller (wrapped at the panel layer)
    loadTagOptions()
  })

  return {
    query,
    tagFilter,
    mode,
    sort,
    order,
    page,
    pageSize,
    rows,
    searchResults,
    note,
    total,
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
