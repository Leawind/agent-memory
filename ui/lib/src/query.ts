// Query string assembly for memory list/search (pure functions, kept separate from
// components for easier testing).
// Agreement with the server: a query goes to memory_search (multi-select tag filter),
// otherwise it goes to memory_list (single tag + sorting).

export interface MemoriesQueryState {
  query: string
  tagFilter: string
  /** Search mode: auto (server decides per its config) / keyword / hybrid; auto sends no parameter */
  mode: string
  sort: string
  order: string
  page: number
  pageSize: number
}

export function buildMemoriesQuery(s: MemoriesQueryState): string {
  const searching = isSearchMode(s.query)
  const p = new URLSearchParams()
  if (searching) {
    p.set('query', s.query.trim())
    if (s.tagFilter) p.set('tags', s.tagFilter)
    if (s.mode && s.mode !== 'auto') p.set('mode', s.mode)
  } else {
    if (s.tagFilter) p.set('tag', s.tagFilter)
    p.set('sort', s.sort)
    p.set('order', s.order)
  }
  p.set('offset', String((s.page - 1) * s.pageSize))
  p.set('limit', String(s.pageSize))
  return p.toString()
}

export function isSearchMode(query: string): boolean {
  return query.trim().length > 0
}
