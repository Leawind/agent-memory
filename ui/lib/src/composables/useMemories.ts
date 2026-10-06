// Memory panel state machine: dual list/search modes, race protection, page-range fallback,
// tag diffing.
// Failed actions throw an Error; the panel layer shows the toast uniformly.
import { computed, onMounted, ref, watch } from 'vue'
import { useApiClient } from '../api/client'
import { buildMemoriesQuery, isSearchMode } from '../query'
import { useMemoryConfig } from '../config'
import { listTags } from '../api/tags'
import { t } from '../i18n'
import type { MemoryListResp, MemorySearchResp, MemorySummary, SearchResult, TagListResp } from '../types'

/** Shape of the editor form: id of null means creating new */
export interface MemoryDraft {
  id: string | null
  summary: string
  content: string
  tags: string[]
}

/** The tag expression survives reloads: it can be long enough that retyping is a real cost */
const TAG_EXPR_KEY = 'agent-memory-tag-expr'

export function useMemories() {
  const client = useApiClient()
  const { defaultPageSize } = useMemoryConfig()

  const query = ref('')
  /** Tag set algebra filter, e.g. "(a&b)|c" — applies to list and search alike */
  const tagExpr = ref(localStorage.getItem(TAG_EXPR_KEY) ?? '')
  watch(tagExpr, (v) => {
    try {
      if (v) localStorage.setItem(TAG_EXPR_KEY, v)
      else localStorage.removeItem(TAG_EXPR_KEY)
    } catch {
      /* storage unavailable (privacy mode etc.): filtering works, just not persisted */
    }
  })
  /** Search mode: auto (server decides hybrid/keyword per its config) / keyword / hybrid */
  const mode = ref<'auto' | 'keyword' | 'hybrid'>('auto')
  const sort = ref<'updated_at' | 'id'>('updated_at')
  const order = ref<'asc' | 'desc'>('desc')
  const page = ref(1)
  const pageSize = ref(defaultPageSize)
  const rows = ref<MemorySummary[]>([])
  const searchResults = ref<SearchResult[]>([])
  /**
   * Whether the view is currently settled on search results: false while a search intent has
   * not produced data yet (typing, or the in-flight request), so the card list keeps showing
   * the last settled view instead of flashing an empty list between keystrokes.
   */
  const showSearchResults = ref(false)
  /** Semantic fallback notice: non-empty when a hybrid request fell back to keyword search because the embedding service was unavailable (told explicitly, never silently) */
  const note = ref('')
  const total = ref(0)
  const loading = ref(false)
  const tagOptions = ref<string[]>([])

  const searching = computed(() => isSearchMode(query.value))

  // A change to the search term / tag expression is a new query intent: restart from page 1
  function onSearch() {
    page.value = 1
    return reload()
  }

  function buildQuery() {
    return buildMemoriesQuery({
      query: query.value,
      tagExpr: tagExpr.value,
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
        showSearchResults.value = true
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
        showSearchResults.value = false
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

  onMounted(() => {
    void reload().catch(() => {}) // first-load errors are toasted by the caller (wrapped at the panel layer)
    loadTagOptions()
  })

  return {
    query,
    tagExpr,
    mode,
    sort,
    order,
    page,
    pageSize,
    rows,
    searchResults,
    showSearchResults,
    note,
    total,
    loading,
    tagOptions,
    searching,
    onSearch,
    reload,
    loadTagOptions,
  }
}

export type MemoriesStore = ReturnType<typeof useMemories>
