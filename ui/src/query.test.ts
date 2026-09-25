// 查询串组装的单元测试（与 server 端 tools::params 行为对齐的约定）
import { describe, expect, it } from 'vitest'
import { buildMemoriesQuery, isSearchMode } from './query.js'

describe('buildMemoriesQuery', () => {
  it('list mode: tag filter + sort + pagination', () => {
    const qs = buildMemoriesQuery({
      query: '',
      tagFilter: '项目',
      sort: 'created_at',
      order: 'asc',
      page: 2,
      pageSize: 50,
    })
    const p = new URLSearchParams(qs)
    expect(p.get('tag')).toBe('项目')
    expect(p.get('sort')).toBe('created_at')
    expect(p.get('order')).toBe('asc')
    expect(p.get('offset')).toBe('50')
    expect(p.get('limit')).toBe('50')
    expect(p.has('query')).toBe(false)
  })

  it('search mode: query + tags filter', () => {
    const qs = buildMemoriesQuery({
      query: '  rust  ',
      tagFilter: '项目',
      sort: 'updated_at',
      order: 'desc',
      page: 1,
      pageSize: 20,
    })
    const p = new URLSearchParams(qs)
    expect(p.get('query')).toBe('rust')
    expect(p.get('tags')).toBe('项目')
    expect(p.has('tag')).toBe(false)
    expect(p.has('sort')).toBe(false)
  })

  it('offset is zero-based from page number', () => {
    const qs = buildMemoriesQuery({
      query: '',
      tagFilter: '',
      sort: 'updated_at',
      order: 'desc',
      page: 3,
      pageSize: 20,
    })
    expect(new URLSearchParams(qs).get('offset')).toBe('40')
  })

  it('isSearchMode treats whitespace-only as list mode', () => {
    expect(isSearchMode('rust')).toBe(true)
    expect(isSearchMode('   ')).toBe(false)
    expect(isSearchMode('')).toBe(false)
  })
})
