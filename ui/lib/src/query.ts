// Query string assembly for memory list/search (pure functions, kept separate from
// components for easier testing).
// Agreement with the server: a query goes to memory_search, otherwise it goes to memory_list.
// The tag expression (tag set algebra, e.g. "(a&b)|c") filters both modes. Values are encoded
// by hand with encodeURIComponent rather than through URLSearchParams: form-style serialization
// turns spaces into '+', which the server's percent-decode reads literally, corrupting tag
// names that contain spaces.

export interface MemoriesQueryState {
  query: string
  /** Tag set algebra expression, e.g. "(a&b)|c"; empty string = no expression filter */
  tagExpr: string
  /** Search mode: auto (server decides per its config) / keyword / hybrid; auto sends no parameter */
  mode: string
  sort: string
  order: string
  page: number
  pageSize: number
  state?: string
}

function pair(k: string, v: string): string {
  return `${k}=${encodeURIComponent(v)}`
}

export function buildMemoriesQuery(s: MemoriesQueryState): string {
  const searching = isSearchMode(s.query)
  const p: string[] = []
  if (searching) {
    p.push(pair('query', s.query.trim()))
    if (s.mode && s.mode !== 'auto') p.push(pair('mode', s.mode))
  } else {
    p.push(pair('sort', s.sort))
    p.push(pair('order', s.order))
  }
  const expr = s.tagExpr.trim()
  if (expr) p.push(pair('tag_expr', expr))
  if (s.state && s.state !== 'active') p.push(pair('state', s.state))
  p.push(pair('offset', String((s.page - 1) * s.pageSize)))
  p.push(pair('limit', String(s.pageSize)))
  return p.join('&')
}

export function isSearchMode(query: string): boolean {
  return query.trim().length > 0
}
