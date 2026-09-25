// App 壳挂载冒烟测试：顶栏导航渲染 + 主题应用
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '@agent-memory/ui'
import App from './App.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

// whoami 走 res.json() 解析，需要带 json() 的响应
function jsonWithJson(body: unknown) {
  return {
    ok: true,
    status: 200,
    text: () => Promise.resolve(JSON.stringify(body)),
    json: () => Promise.resolve(body),
  }
}

const adminCaps = { read: true, create: true, update: true, delete: true, tag_manage: true, admin: true }

describe('App shell', () => {
  beforeEach(() => {
    setMemoryUILocale('zh')
    localStorage.removeItem('agent-memory-theme')
    // 模拟持久化的中文语言偏好（happy-dom 的 navigator.language 是 en-US）
    localStorage.setItem('agent-memory-locale', 'zh')
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

  it('mounts the top-nav shell and renders the memories panel', async () => {
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('agent-memory')
    expect(html).toContain('记忆管理')
    expect(html).toContain('新建记忆')
    expect(html).toContain('标签管理')
    expect(html).toContain('运维')
    wrapper.unmount()
  })

  it('applies the stored dark theme on mount', async () => {
    localStorage.setItem('agent-memory-theme', 'dark')
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    expect(document.documentElement.classList.contains('dark')).toBe(true)
    wrapper.unmount()
  })

  it('renders English UI when the stored language preference is en', async () => {
    localStorage.setItem('agent-memory-locale', 'en')
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('Memories')
    expect(html).toContain('Tags')
    expect(html).not.toContain('记忆管理')
    wrapper.unmount()
  })

  it('prompts for the access token when the server answers 401', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        if (String(url).includes('/api/whoami')) {
          return Promise.resolve({ ok: false, status: 401, text: () => Promise.resolve('') })
        }
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    expect(wrapper.html()).toContain('访问令牌')
    wrapper.unmount()
  })

  it('loads identity data once the admin identity resolves after mount', async () => {
    // whoami 挂起，模拟真实时序：挂载早于身份解析完成
    let resolveWhoAmI!: (res: unknown) => void
    const identityCalls: string[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.includes('/api/whoami')) {
          return new Promise((resolve) => {
            resolveWhoAmI = resolve
          })
        }
        if (u.includes('/api/identities')) {
          identityCalls.push(u)
          return Promise.resolve(
            jsonResponse({
              identities: [{ name: 'admin', token: 'aaaa', permissions: adminCaps, created_at: 1 }],
            }),
          )
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(jsonResponse({ instructions: null, conventions: null, default_instructions: '' }))
        }
        if (u.includes('/api/tags')) return Promise.resolve(jsonResponse({ total_tags: 0, tags: [] }))
        if (u.includes('/api/stats')) {
          return Promise.resolve(
            jsonResponse({ path: '/tmp/m.db', memories: 0, tags: 0, next_id: 'm1', file_size: 0, newest_update: null }),
          )
        }
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    // 身份未就绪时不应预取管理数据
    expect(identityCalls.length).toBe(0)
    resolveWhoAmI(jsonWithJson({ name: 'admin', mode: 'token', permissions: adminCaps }))
    await flushPromises()
    await flushPromises()
    // 回归：admin 就绪后必须补加载（此前 onMounted 只判断一次，面板永远停在空态）
    expect(identityCalls.length).toBeGreaterThan(0)
    wrapper.unmount()
  })

  it('refreshes the target panel data when switching tabs', async () => {
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    const countTagsCalls = () => fetchMock.mock.calls.filter((c) => String(c[0]).includes('/api/tags')).length
    const before = countTagsCalls()
    expect(before).toBeGreaterThan(0)
    const navButtons = wrapper.findAll('nav button')
    await navButtons[1].trigger('click')
    await flushPromises()
    await flushPromises()
    // 回归：面板 v-show 常驻不会重新挂载，切回标签管理必须显式刷新（此前一直显示陈旧数据）
    expect(countTagsCalls()).toBeGreaterThan(before)
    wrapper.unmount()
  })
})
