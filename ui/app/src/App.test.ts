// App 壳挂载冒烟测试：MemoryAdmin 渲染 + 主题应用
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import App from './App.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('App shell', () => {
  beforeEach(() => {
    localStorage.removeItem('agent-memory-theme')
    document.documentElement.classList.remove('dark')
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.includes('/api/tags')) {
          return Promise.resolve(jsonResponse({ total_tags: 0, tags: [] }))
        }
        if (u.includes('/api/stats')) {
          return Promise.resolve(
            jsonResponse({ path: '/tmp/m.db', memories: 0, tags: 0, next_id: 'm1', file_size: 0, newest_update: null }),
          )
        }
        if (u.includes('/health')) return Promise.resolve(jsonResponse({ status: 'ok', version: '0.0.0' }))
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
  })

  it('mounts the admin shell and renders the memories panel', async () => {
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('agent-memory')
    expect(html).toContain('记忆管理')
    expect(html).toContain('新建记忆')
    wrapper.unmount()
  })

  it('applies the stored dark theme on mount', async () => {
    localStorage.setItem('agent-memory-theme', 'dark')
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    expect(document.documentElement.classList.contains('dark')).toBe(true)
    wrapper.unmount()
  })
})
