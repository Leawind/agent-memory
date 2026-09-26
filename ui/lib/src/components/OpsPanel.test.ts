// OpsPanel 挂载冒烟测试：统计卡片与数据库概况（只读；体检/备份在 AdminPanel）
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import OpsPanel from './OpsPanel.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('OpsPanel', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.includes('/api/stats')) {
          return Promise.resolve(
            jsonResponse({
              path: '/tmp/memory.db',
              memories: 7,
              tags: 3,
              file_size: 4096,
              schema_version: 2,
            }),
          )
        }
        if (u.includes('/health')) {
          return Promise.resolve(jsonResponse({ status: 'ok', version: '0.1.0' }))
        }
        return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve('') })
      }),
    )
  })

  it('renders stats cards from the API', async () => {
    const wrapper = mount(OpsPanel)
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('/tmp/memory.db')
    expect(html).toContain('数据库版本')
    // 体检与备份属 admin 功能，已移入 AdminPanel
    expect(html).not.toContain('数据体检')
    expect(html).not.toContain('导出备份')
    // next_id / 最近更新不再展示
    expect(html).not.toContain('下一个记忆 ID')
    expect(html).not.toContain('最近更新')
    wrapper.unmount()
  })
})
