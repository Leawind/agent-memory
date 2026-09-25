// OpsView 挂载冒烟测试：统计卡片与体检区渲染
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import OpsView from './OpsView.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('OpsView', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn((url: string | URL) => {
      const u = String(url)
      if (u.includes('/api/stats')) {
        return Promise.resolve(
          jsonResponse({
            path: '/tmp/memory.db',
            memories: 7,
            tags: 3,
            next_id: 'm8',
            file_size: 4096,
            newest_update: { id: 'm7', updated_at: 100 },
          })
        )
      }
      if (u.includes('/health')) {
        return Promise.resolve(jsonResponse({ status: 'ok', version: '0.1.0' }))
      }
      return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve('') })
    }))
  })

  it('renders stats cards from the API', async () => {
    const wrapper = mount(OpsView, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('/tmp/memory.db')
    expect(html).toContain('导出备份')
    expect(html).toContain('数据体检')
    wrapper.unmount()
  })
})
