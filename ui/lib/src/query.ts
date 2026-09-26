// 记忆列表/搜索的查询串组装（纯函数，独立于组件便于测试）。
// 与服务端约定：有 query 走 memory_search（tags 多选过滤），
// 否则走 memory_list（tag 单选 + 排序）。

export interface MemoriesQueryState {
  query: string
  tagFilter: string
  /** 搜索模式：auto（服务端按配置决定）/ keyword / hybrid；auto 不传参数 */
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
