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
// Full-memory shape returned by the detail endpoint the dialog fetches when a card is opened
const detailPayload = {
  id: 'm1',
  summary: '列表模式的记忆',
  content: '正文内容',
  tags: ['t1'],
  created: '2026-10-04 07:00',
  updated: '2026-10-04 08:00',
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
  if (/\/api\/memories\/m\d+/.test(u)) return jsonResponse(detailPayload)
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

  it('mounts and renders memory cards from the API', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('列表模式的记忆')
    expect(html).toContain('m1')
    // Card layout: the summary never shares a line with the tags
    expect(wrapper.find('.am-memory-card .card-summary').text()).toBe('列表模式的记忆')
    expect(wrapper.find('.am-memory-card .card-tags').text()).toContain('t1')
    wrapper.unmount()
  })

  it('opens the memory dialog when a card is clicked', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    await flushPromises()
    await wrapper.find('.am-memory-card').trigger('click')
    await flushPromises()
    const dialog = wrapper.findComponent({ name: 'ElDialog' })
    expect(dialog.exists()).toBe(true)
    expect(dialog.props('modelValue')).toBe(true)
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

  it('refetches with the sort params when the sort select changes', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    fetchMock.mockClear()
    // The select renders as an EP select; drive the ref-backed watcher through the component.
    // Toolbar order: tag filter first, sort control second.
    const selects = wrapper.findAllComponents({ name: 'ElSelect' })
    expect(selects.length).toBeGreaterThanOrEqual(2)
    selects[1].vm.$emit('update:modelValue', 'id:asc')
    await flushPromises()
    await flushPromises()
    const urls = fetchMock.mock.calls.map((c) => String(c[0]))
    const listCall = urls.filter((u) => u.includes('/api/memories?')).at(-1)
    expect(listCall).toContain('sort=id')
    expect(listCall).toContain('order=asc')
    wrapper.unmount()
  })
})
