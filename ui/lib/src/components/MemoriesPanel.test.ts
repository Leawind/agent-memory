// MemoriesPanel mount smoke test: verifies data wiring in both list/search modes
// (fetch is mocked per route, Element Plus components render for real via unplugin-vue-components)
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import MemoriesPanel from './MemoriesPanel.vue'

const listPayload = {
  total: 1,
  offset: 0,
  limit: 20,
  memories: [{ id: 'm1', summary: '列表模式的记忆', tags: ['t1'], updated: '2026-10-04 08:00' }],
}
const searchPayload = {
  total_matches: 1,
  results: [
    {
      id: 'm1',
      summary: '搜索命中的记忆',
      tags: ['t1'],
      score: 50,
      snippet: '…命中片段…',
      updated: '2026-10-04 08:00',
    },
  ],
}
const tagsPayload = { tags: [{ name: 't1', count: 1 }] }

function mockFetch(url: string | URL) {
  const u = String(url)
  if (u.startsWith('/api/tags')) return jsonResponse(tagsPayload)
  if (u.includes('query=')) return jsonResponse(searchPayload)
  return jsonResponse(listPayload)
}

function jsonResponse(body: unknown) {
  return {
    ok: true,
    status: 200,
    text: () => Promise.resolve(JSON.stringify(body)),
  }
}

describe('MemoriesPanel', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => Promise.resolve(mockFetch(url))),
    )
  })

  it('mounts and renders list rows from the API', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('列表模式的记忆')
    expect(html).toContain('m1')
    wrapper.unmount()
  })

  it('switches to search results when a query is entered', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    const input = wrapper.find('input[placeholder*="关键词"]')
    expect(input.exists()).toBe(true)
    await input.setValue('命中')
    await input.trigger('keyup.enter')
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('搜索命中的记忆')
    expect(html).toContain('命中片段')
    wrapper.unmount()
  })

  it('refetches with the sorted params when the table emits sort-change', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    fetchMock.mockClear()
    const table = wrapper.findComponent({ name: 'ElTable' })
    expect(table.exists()).toBe(true)
    table.vm.$emit('sort-change', { prop: 'id', order: 'ascending' })
    await flushPromises()
    await flushPromises()
    const urls = fetchMock.mock.calls.map((c) => String(c[0]))
    const listCall = urls.filter((u) => u.includes('/api/memories?')).at(-1)
    // Sorting must actually reach the request parameters (regression: onSortChange once threw
    // due to parameter shadowing, so reload never ran)
    expect(listCall).toContain('sort=id')
    expect(listCall).toContain('order=asc')
    wrapper.unmount()
  })
})
