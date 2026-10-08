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
  original_tags: ['t1'],
  derived_tags: [],
  lifecycle: { kind: 'fact', expires_at: null, archived_at: null, pinned: false, state: 'active' },
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
const tagsPayload = { tags: [{ name: 't1', count: 1, description: '测试标签' }] }
const statsPayload = { embedding: { enabled: true, model: 'bge-m3', embedded: 1, pending: 0 } }

function mockFetch(url: string | URL) {
  const u = String(url)
  if (u.startsWith('/api/tags')) return jsonResponse(tagsPayload)
  if (u.includes('/api/stats')) return jsonResponse(statsPayload)
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
    localStorage.clear()
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
    // Semantic search is enabled server-side (stats): the placeholder says so
    const input = wrapper.find('input[placeholder*="搜索"]')
    expect(input.exists()).toBe(true)
    expect((input.element as HTMLInputElement).placeholder).toContain('语义')
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
    // Toolbar order: the expression box is an input (not a select), so the selects are
    // [sort, mode].
    const selects = wrapper.findAllComponents({ name: 'ElSelect' })
    expect(selects.length).toBeGreaterThanOrEqual(2)
    selects[0].vm.$emit('update:modelValue', 'id:asc')
    await flushPromises()
    await flushPromises()
    const urls = fetchMock.mock.calls.map((c) => String(c[0]))
    const listCall = urls.filter((u) => u.includes('/api/memories?')).at(-1)
    expect(listCall).toContain('sort=id')
    expect(listCall).toContain('order=asc')
    wrapper.unmount()
  })

  it('sends the tag expression as tag_expr and persists it', async () => {
    // A persisted expression survives remounts: it rides along on the very first load
    localStorage.setItem('agent-memory-tag-expr', '(rust&web)|notes')
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    const initialUrls = fetchMock.mock.calls.map((c) => String(c[0]))
    expect(initialUrls.some((u) => u.includes(`tag_expr=${encodeURIComponent('(rust&web)|notes')}`))).toBe(true)
    const input = wrapper.find('.tag-filter input')
    expect((input.element as HTMLInputElement).value).toBe('(rust&web)|notes')

    // Editing applies on Enter (immediate) and re-saves storage
    fetchMock.mockClear()
    await input.setValue('!rust')
    await input.trigger('keydown.enter')
    await flushPromises()
    await flushPromises()
    expect(localStorage.getItem('agent-memory-tag-expr')).toBe('!rust')
    const urls = fetchMock.mock.calls.map((c) => String(c[0]))
    expect(urls.at(-1)).toContain(`tag_expr=${encodeURIComponent('!rust')}`)
    wrapper.unmount()
  })

  it('shows the tag description in a hover tooltip on the card chips', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    await flushPromises()
    const tooltip = wrapper.find('.card-tags').findComponent({ name: 'ElTooltip' })
    expect(tooltip.exists()).toBe(true)
    // The chip's tooltip carries the description from the tag list
    expect(tooltip.props('content')).toBe('测试标签')
    expect(tooltip.props('disabled')).toBe(false)
    wrapper.unmount()
  })

  it('toggleTagFilter sets the expression to the tag and re-click clears it', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    fetchMock.mockClear()
    const vm = wrapper.vm as unknown as { toggleTagFilter: (name: string) => void }

    vm.toggleTagFilter('t1')
    await flushPromises()
    await flushPromises()
    expect(fetchMock.mock.calls.some((c) => String(c[0]).includes(`tag_expr=${encodeURIComponent('t1')}`))).toBe(true)
    expect((wrapper.find('.tag-filter input').element as HTMLInputElement).value).toBe('t1')
    expect(localStorage.getItem('agent-memory-tag-expr')).toBe('t1')

    // Clicking the already-active tag clears the filter entirely
    fetchMock.mockClear()
    vm.toggleTagFilter('t1')
    await flushPromises()
    await flushPromises()
    const urls = fetchMock.mock.calls.map((c) => String(c[0]))
    expect(urls.some((u) => u.startsWith('/api/memories'))).toBe(true)
    expect(urls.some((u) => u.includes('tag_expr='))).toBe(false)
    expect((wrapper.find('.tag-filter input').element as HTMLInputElement).value).toBe('')
    expect(localStorage.getItem('agent-memory-tag-expr')).toBeNull()
    wrapper.unmount()
  })
})
