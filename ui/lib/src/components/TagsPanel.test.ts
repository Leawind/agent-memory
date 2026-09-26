// TagsPanel 挂载冒烟测试：标签表格从 API 渲染
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TagsPanel from './TagsPanel.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('TagsPanel', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() =>
        Promise.resolve(
          jsonResponse({
            total_tags: 2,
            tags: [
              { name: 'rust', description: '语言', memory_count: 3, last_used_at: 100, created_at: 50 },
              { name: 'infra', description: '', memory_count: 0, last_used_at: null, created_at: 60 },
            ],
          }),
        ),
      ),
    )
  })

  it('mounts and renders tag rows', async () => {
    const wrapper = mount(TagsPanel)
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('rust')
    expect(html).toContain('语言')
    expect(html).toContain('新建标签')
    wrapper.unmount()
  })

  it('refetches with the regex filter query param', async () => {
    const wrapper = mount(TagsPanel)
    await flushPromises()
    await flushPromises()
    const input = wrapper.find('.am-tag-filter input')
    expect(input.exists()).toBe(true)
    await input.setValue('^proj/')
    // 防抖 300ms 后应带 filter 参数重新请求
    await new Promise((r) => setTimeout(r, 350))
    await flushPromises()
    const calls = (fetch as ReturnType<typeof vi.fn>).mock.calls.map((c) => String(c[0]))
    expect(calls.some((u) => u.includes('/api/tags?filter=%5Eproj%2F'))).toBe(true)
    wrapper.unmount()
  })
})
