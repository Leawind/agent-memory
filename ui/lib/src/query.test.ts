// Unit tests for query string assembly (conventions aligned with the server's tools::params behavior)
import { describe, expect, it } from 'vitest'
import { buildMemoriesQuery, isSearchMode } from './query.js'

describe('buildMemoriesQuery', () => {
  it('includes the snapshot cursor only in search mode', () => {
    for (const query of ['', 'alpha']) {
      const qs = buildMemoriesQuery({
        query,
        tagExpr: '',
        mode: 'keyword',
        sort: 'id',
        order: 'asc',
        page: 2,
        pageSize: 20,
        cursor: 'snapshot',
      })
      expect(new URLSearchParams(qs).get('cursor')).toBe(query ? 'snapshot' : null)
    }
  })
  it('carries lifecycle filters through both list and search requests', () => {
    for (const query of ['', 'rust']) {
      const qs = buildMemoriesQuery({
        query,
        tagExpr: '',
        mode: 'auto',
        sort: 'id',
        order: 'asc',
        page: 1,
        pageSize: 20,
        state: 'archived',
      })
      expect(new URLSearchParams(qs).get('state')).toBe('archived')
    }
  })
  it('list mode: sort + pagination', () => {
    const qs = buildMemoriesQuery({
      query: '',
      tagExpr: '',
      mode: 'auto',
      sort: 'id',
      order: 'asc',
      page: 2,
      pageSize: 50,
    })
    const p = new URLSearchParams(qs)
    expect(p.get('sort')).toBe('id')
    expect(p.get('order')).toBe('asc')
    expect(p.get('offset')).toBe('50')
    expect(p.get('limit')).toBe('50')
    expect(p.has('query')).toBe(false)
    expect(p.has('tag_expr')).toBe(false)
  })

  it('search mode: query + mode', () => {
    const qs = buildMemoriesQuery({
      query: '  rust  ',
      tagExpr: '',
      mode: 'hybrid',
      sort: 'updated_at',
      order: 'desc',
      page: 1,
      pageSize: 20,
    })
    const p = new URLSearchParams(qs)
    expect(p.get('query')).toBe('rust')
    expect(p.get('mode')).toBe('hybrid')
    expect(p.has('tag')).toBe(false)
    expect(p.has('sort')).toBe(false)
  })

  it('tag expression rides along in both modes, percent-encoded by hand', () => {
    // encodeURIComponent keeps spaces as %20 (URLSearchParams would send '+', which the
    // server reads literally) and escapes the algebra characters
    const expr = '(rust & life)|项目'
    for (const query of ['', 'rust']) {
      const qs = buildMemoriesQuery({
        query,
        tagExpr: expr,
        mode: 'auto',
        sort: 'updated_at',
        order: 'desc',
        page: 1,
        pageSize: 20,
      })
      expect(qs).toContain(`tag_expr=${encodeURIComponent(expr)}`)
    }
    // The manual encoding round-trips: '+' stays a literal plus inside the value
    expect(
      buildMemoriesQuery({ query: '', tagExpr: 'a+b', mode: 'auto', sort: 'id', order: 'asc', page: 1, pageSize: 20 }),
    ).toContain('tag_expr=a%2Bb')
  })

  it('offset is zero-based from page number', () => {
    const qs = buildMemoriesQuery({
      query: '',
      tagExpr: '',
      mode: 'auto',
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
