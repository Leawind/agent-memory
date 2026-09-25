// TagsView 挂载冒烟测试：标签表格从 API 渲染
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import TagsView from './TagsView.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('TagsView', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url) =>
        Promise.resolve(
          jsonResponse({
            total_tags: 2,
            tags: [
              { name: 'rust', description: '语言', memory_count: 3, last_used_at: 100 },
              { name: 'infra', description: '', memory_count: 0, last_used_at: null },
            ],
          })
        )
      )
    )
  })

  it('mounts and renders tag rows', async () => {
    const wrapper = mount(TagsView, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('rust')
    expect(html).toContain('语言')
    expect(html).toContain('新建标签')
    wrapper.unmount()
  })
})
