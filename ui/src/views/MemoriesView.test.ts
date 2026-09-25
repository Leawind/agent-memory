// MemoriesView 挂载冒烟测试：验证列表/搜索两种模式的数据接线
// （fetch 按路由 mock，Element Plus 真实渲染；happy-dom 环境）
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import MemoriesView from './MemoriesView.vue'

const listPayload = {
  total: 1,
  offset: 0,
  limit: 20,
  memories: [{ id: 'm1', summary: '列表模式的记忆', tags: ['t1'], created_at: 1, updated_at: 2 }],
}
const searchPayload = {
  total_matches: 1,
  results: [{ id: 'm1', summary: '搜索命中的记忆', tags: ['t1'], score: 50, snippet: '…命中片段…', updated_at: 2 }],
}
const tagsPayload = { total_tags: 1, tags: [{ name: 't1', description: '', memory_count: 1 }] }

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

function mountView() {
  return mount(MemoriesView, { global: { plugins: [ElementPlus] } })
}

describe('MemoriesView', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => Promise.resolve(mockFetch(url))),
    )
  })

  it('mounts and renders list rows from the API', async () => {
    const wrapper = mountView()
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('列表模式的记忆')
    expect(html).toContain('m1')
    wrapper.unmount()
  })

  it('switches to search results when a query is entered', async () => {
    const wrapper = mountView()
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
})
