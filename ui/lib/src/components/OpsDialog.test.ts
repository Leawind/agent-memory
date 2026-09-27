// OpsDialog 挂载冒烟测试：打开后显示统计与数据库概况；每次打开重新拉取，未打开不请求
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import OpsDialog from './OpsDialog.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('OpsDialog', () => {
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

  function countStatsCalls(): number {
    return vi.mocked(globalThis.fetch).mock.calls.filter((c) => String(c[0]).includes('/api/stats')).length
  }

  it('fetches and renders stats only when opened, refetching on each open', async () => {
    const wrapper = mount(OpsDialog, { props: { visible: false } })
    await flushPromises()
    // 未打开不产生请求（弹窗常驻挂载，靠 visible 触发拉取）
    expect(countStatsCalls()).toBe(0)

    await wrapper.setProps({ visible: true })
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(countStatsCalls()).toBe(1)
    expect(html).toContain('/tmp/memory.db')
    expect(html).toContain('数据库版本')
    // 体检与备份属 admin 功能，已移入 AdminPanel
    expect(html).not.toContain('数据体检')
    expect(html).not.toContain('导出备份')
    // next_id / 最近更新不再展示
    expect(html).not.toContain('下一个记忆 ID')
    expect(html).not.toContain('最近更新')

    // 重新打开重拉：概况即点即新，无需手动刷新
    await wrapper.setProps({ visible: false })
    await flushPromises()
    await wrapper.setProps({ visible: true })
    await flushPromises()
    expect(countStatsCalls()).toBe(2)
    wrapper.unmount()
  })
})
