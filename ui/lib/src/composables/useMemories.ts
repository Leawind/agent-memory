// Memory panel state machine: dual list/search modes, race protection, page-range fallback,
// tag diffing.
// Failed actions throw an Error; the panel layer shows the toast uniformly.
import { computed, onMounted, ref, watch } from 'vue'
import { ApiError, useApiClient } from '../api/client'
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
  const state = ref<'active' | 'archived' | 'expired' | 'all'>('active')
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
  /** Hybrid only: how many of the settled search's matches are literal keyword hits
   * (undefined = not reported — keyword mode, or the response predates the field) */
  const keywordMatches = ref<number | undefined>(undefined)
  const loading = ref(false)
  const tagOptions = ref<string[]>([])
  /** Tag name -> description, for hover tooltips on the memory cards' tag chips */
  const tagDescriptions = ref<Record<string, string>>({})

  const searching = computed(() => isSearchMode(query.value))

  /** Whether semantic recall is available server-side (shapes the search box's placeholder and
   * what "search" means here). Probed once via /api/stats; failures leave keyword-only wording. */
  const semanticReady = ref(false)
  onMounted(() => {
    client
      .get<{ embedding?: { enabled?: boolean } }>('/api/stats')
      .then((stats) => {
        semanticReady.value = stats?.embedding?.enabled === true
      })
      .catch(() => {})
  })

  // A change to the search term / tag expression is a new query intent: restart from page 1
  function onSearch() {
    page.value = 1
    return reload()
  }

  let searchCursor = ''
  let cursorIntent = ''
  const intent = () => JSON.stringify([query.value.trim(), tagExpr.value.trim(), mode.value, state.value])

  function buildQuery(cursor?: string) {
    return buildMemoriesQuery({
      query: query.value,
      tagExpr: tagExpr.value,
      mode: mode.value,
      sort: sort.value,
      order: order.value,
      page: page.value,
      pageSize: pageSize.value,
      state: state.value,
      cursor,
    })
  }

  // Request sequence number: when queries fire in quick succession (Enter in the input /
  // clearing / paging), late stale responses are discarded so a slow old result never
  // overwrites a newer one
  let requestSeq = 0

  function reload(): Promise<void> {
    searchCursor = ''
    cursorIntent = ''
    return load(false)
  }

  function reloadPage(): Promise<void> {
    return load(true)
  }

  async function load(preserveSnapshot: boolean): Promise<void> {
    const seq = ++requestSeq
    loading.value = true
    try {
      const currentIntent = intent()
      if (cursorIntent !== currentIntent) searchCursor = ''
      const cursor = preserveSnapshot ? searchCursor : ''
      const qs = buildQuery(cursor)
      if (isSearchMode(query.value)) {
        let data: MemorySearchResp
        try {
          data = await client.get<MemorySearchResp>(`/api/memories?${qs}`)
        } catch (error) {
          if (seq !== requestSeq) return
          if (!cursor || !(error instanceof ApiError) || error.status !== 400 || error.code !== 'stale_search_cursor')
            throw error
          searchCursor = ''
          cursorIntent = ''
          page.value = 1
          data = await client.get<MemorySearchResp>(`/api/memories?${buildQuery()}`)
        }
        if (seq !== requestSeq) return
        searchCursor = data.cursor ?? ''
        cursorIntent = currentIntent
        // Snippets are plain text from the server and are rendered via text interpolation — never v-html
        searchResults.value = data.results ?? []
        total.value = data.total_matches ?? 0
        keywordMatches.value = data.keyword_matches
        // When a hybrid request fell back to keyword search because the embedding service was
        // unavailable, say so explicitly (never silently)
        note.value = data.semantic_fallback ? t('memories.semanticFallback') : ''
        showSearchResults.value = true
      } else {
        searchCursor = ''
        cursorIntent = ''
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
        keywordMatches.value = undefined
        showSearchResults.value = false
      }
    } finally {
      if (seq === requestSeq) loading.value = false
    }
  }

  async function loadTagOptions() {
    try {
      const data = (await listTags(client)) as TagListResp
      const descs: Record<string, string> = {}
      for (const tg of data.tags ?? []) if (tg.description) descs[tg.name] = tg.description
      tagDescriptions.value = descs
      tagOptions.value = (data.tags ?? []).map((tg) => tg.name)
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
    state,
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
    keywordMatches,
    loading,
    tagOptions,
    tagDescriptions,
    searching,
    semanticReady,
    onSearch,
    reload,
    reloadPage,
    loadTagOptions,
  }
}

export type MemoriesStore = ReturnType<typeof useMemories>
