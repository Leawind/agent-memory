// App 壳挂载冒烟测试：顶栏导航渲染、主题应用、身份下拉与管理标签页权限
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
const viewerCaps = { read: true, create: false, update: false, delete: false, tag_manage: false, admin: false }

/** 断言 nav 按钮的标签集合 */
function navLabels(wrapper: ReturnType<typeof mount>): string[] {
  return wrapper.findAll('nav button').map((b) => b.text())
}

/** 令牌弹窗的开关状态（App.vue 的 tokenDialog）。el-dialog 的关闭过渡在 happy-dom
 * 里不会真正走完，DOM 断言收起状态不可靠，只能看驱动它的组件状态 */
function dialogOpen(wrapper: ReturnType<typeof mount>): boolean {
  return (wrapper.vm.$ as unknown as { setupState: { tokenDialog: boolean } }).setupState.tokenDialog
}

describe('App shell', () => {
  beforeEach(() => {
    setMemoryUILocale('zh')
    localStorage.clear()
    // el-dropdown 菜单 teleport 到 body，上一个用例的弹层可能残留并截走 querySelector：
    // 清空 body，保证每个用例从干净 DOM 开始
    document.body.innerHTML = ''
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
    expect(html).toContain('Agent Memory')
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

  it('shows the admin tab with its management panel only for admin identities', async () => {
    const adminWho = { name: 'admin', mode: 'token' as const, permissions: adminCaps }
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.includes('/api/whoami')) return Promise.resolve(jsonWithJson(adminWho))
        if (u.includes('/api/identities')) {
          return Promise.resolve(
            jsonResponse({
              identities: [{ name: 'admin', token_hint: 'aaaa', permissions: adminCaps, created_at: 1 }],
            }),
          )
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(jsonResponse({ instructions: null, conventions: null, auth_required: false }))
        }
        if (u.includes('/api/tags')) return Promise.resolve(jsonResponse({ total_tags: 0, tags: [] }))
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    // admin 身份：四个标签页齐全，管理面板已挂载（身份表可见）
    expect(navLabels(wrapper)).toEqual(['记忆管理', '标签管理', '运维', '管理'])
    expect(wrapper.html()).toContain('Token 鉴权')
    wrapper.unmount()

    // 只读身份：管理标签页隐藏，管理面板不挂载（连 DOM 都没有）
    const viewerWho = { name: 'viewer', mode: 'token' as const, permissions: viewerCaps }
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        if (String(url).includes('/api/whoami')) return Promise.resolve(jsonWithJson(viewerWho))
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    const viewer = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    expect(navLabels(viewer)).toEqual(['记忆管理', '标签管理', '运维'])
    expect(viewer.html()).not.toContain('Token 鉴权')
    viewer.unmount()
  })

  it('drops the admin tab when switching to a read-only stored identity', async () => {
    const identities = [
      { name: 'admin', token: 'tok-admin', permissions: adminCaps },
      { name: 'viewer', token: 'tok-viewer', permissions: viewerCaps },
    ]
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        const auth = String(new Headers(init?.headers).get('Authorization') ?? '')
        if (u.includes('/api/whoami')) {
          const name = auth.includes('tok-admin') ? 'admin' : auth.includes('tok-viewer') ? 'viewer' : ''
          if (!name) return Promise.resolve({ ok: false, status: 401, text: () => Promise.resolve('') })
          const identity = identities.find((i) => i.name === name)!
          return Promise.resolve(jsonWithJson({ name, mode: 'token', permissions: identity.permissions }))
        }
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    localStorage.setItem('agent-memory-identities', JSON.stringify({ admin: 'tok-admin', viewer: 'tok-viewer' }))
    localStorage.setItem('agent-memory-identity', 'admin')
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    expect(navLabels(wrapper)).toContain('管理')

    // 打开身份下拉，切到 viewer：管理标签页随之消失（下拉菜单 teleport 到 body）
    await wrapper.find('.identity-btn').trigger('click')
    await flushPromises()
    const viewerItem = [...document.querySelectorAll('.el-dropdown-menu__item')].find((el) =>
      el.textContent?.includes('viewer'),
    )
    expect(viewerItem).toBeTruthy()
    // 每个身份行带 token 尾缀提示（与管理界面同风格：省略号 + 末 4 位）
    expect(viewerItem!.textContent).toContain('…ewer')
    viewerItem!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flushPromises()
    await flushPromises()
    expect(navLabels(wrapper)).toEqual(['记忆管理', '标签管理', '运维'])
    expect(wrapper.find('.identity-btn').text()).toContain('viewer')
    wrapper.unmount()
  })

  it('keeps the identity dropdown usable when the current token turns invalid', async () => {
    // broken 的 token 已被服务端拒绝；viewer 仍有效，切换后必须能恢复页面
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        const auth = String(new Headers(init?.headers).get('Authorization') ?? '')
        if (u.includes('/api/whoami')) {
          if (auth.includes('tok-viewer')) {
            return Promise.resolve(jsonWithJson({ name: 'viewer', mode: 'token', permissions: viewerCaps }))
          }
          return Promise.resolve({ ok: false, status: 401, text: () => Promise.resolve('') })
        }
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    localStorage.setItem('agent-memory-identities', JSON.stringify({ broken: 'tok-broken', viewer: 'tok-viewer' }))
    localStorage.setItem('agent-memory-identity', 'broken')

    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()

    // 回归：当前身份失效只移除它自己并弹令牌框，下拉必须仍在（此前随 who 一起消失，无法切换自救）
    expect(wrapper.find('.identity-btn').exists()).toBe(true)
    expect(wrapper.find('.identity-btn').text()).toContain('未连接')
    expect(wrapper.find('.el-dialog').isVisible()).toBe(true)
    // happy-dom 做不了真实命中检测：断言弹窗渲染容器带 penetrable 类（容器 pointer-events:none，
    // 弹窗本体 auto），保证真实浏览器里弹窗开着时顶栏下拉不被全屏容器挡住（此前 modal=false 仍被拦）
    expect(wrapper.find('.el-modal-dialog.is-penetrable').exists()).toBe(true)
    expect(JSON.parse(localStorage.getItem('agent-memory-identities')!)).toEqual({ viewer: 'tok-viewer' })

    // 下拉仍列出其余身份，切换成功后令牌框收起、页面以新身份恢复
    await wrapper.find('.identity-btn').trigger('click')
    await flushPromises()
    const viewerItem = [...document.querySelectorAll('.el-dropdown-menu__item')].find((el) =>
      el.textContent?.includes('viewer'),
    )
    expect(viewerItem).toBeTruthy()
    viewerItem!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flushPromises()
    await flushPromises()

    expect(wrapper.find('.identity-btn').text()).toContain('viewer')
    // el-dialog 的关闭过渡在 happy-dom 里不会真正走完（display:none 不落地），
    // 改断言驱动弹窗的组件状态本身
    expect(dialogOpen(wrapper)).toBe(false)
    expect(navLabels(wrapper)).toEqual(['记忆管理', '标签管理', '运维'])
    wrapper.unmount()
  })

  it('recovers through the unauthorized event when the active token is revoked mid-session', async () => {
    // 挂载时 admin 有效；中途被服务端重置：此后带 tok-admin 的请求一律 401
    let revoked = false
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        const auth = String(new Headers(init?.headers).get('Authorization') ?? '')
        if (auth.includes('tok-admin') && revoked) {
          return Promise.resolve({ ok: false, status: 401, text: () => Promise.resolve('') })
        }
        if (u.includes('/api/whoami')) {
          if (auth.includes('tok-admin')) {
            return Promise.resolve(jsonWithJson({ name: 'admin', mode: 'token', permissions: adminCaps }))
          }
          if (auth.includes('tok-viewer')) {
            return Promise.resolve(jsonWithJson({ name: 'viewer', mode: 'token', permissions: viewerCaps }))
          }
          return Promise.resolve({ ok: false, status: 401, text: () => Promise.resolve('') })
        }
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    localStorage.setItem('agent-memory-identities', JSON.stringify({ admin: 'tok-admin', viewer: 'tok-viewer' }))
    localStorage.setItem('agent-memory-identity', 'admin')
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    expect(wrapper.find('.identity-btn').text()).toContain('admin')

    // 面板请求收到 401 → authFetch 广播 UNAUTHORIZED_EVENT → 移除失效身份并弹令牌框
    revoked = true
    await wrapper.findAll('nav button')[1].trigger('click')
    await flushPromises()
    await flushPromises()

    expect(JSON.parse(localStorage.getItem('agent-memory-identities')!)).toEqual({ viewer: 'tok-viewer' })
    expect(wrapper.find('.identity-btn').exists()).toBe(true)
    expect(dialogOpen(wrapper)).toBe(true)

    // 非模态弹窗不锁顶栏：身份下拉仍可操作，切到 viewer 即恢复
    await wrapper.find('.identity-btn').trigger('click')
    await flushPromises()
    const viewerItem = [...document.querySelectorAll('.el-dropdown-menu__item')].find((el) =>
      el.textContent?.includes('viewer'),
    )
    expect(viewerItem).toBeTruthy()
    viewerItem!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flushPromises()
    await flushPromises()

    expect(wrapper.find('.identity-btn').text()).toContain('viewer')
    expect(dialogOpen(wrapper)).toBe(false)
    wrapper.unmount()
  })

  it('falls back to the next stored identity when removing the current one', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        const auth = String(new Headers(init?.headers).get('Authorization') ?? '')
        if (u.includes('/api/whoami')) {
          if (auth.includes('tok-viewer')) {
            return Promise.resolve(jsonWithJson({ name: 'viewer', mode: 'token', permissions: viewerCaps }))
          }
          return Promise.resolve(jsonWithJson({ name: 'admin', mode: 'token', permissions: adminCaps }))
        }
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    localStorage.setItem('agent-memory-identities', JSON.stringify({ admin: 'tok-admin', viewer: 'tok-viewer' }))
    localStorage.setItem('agent-memory-identity', 'admin')
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()

    // 打开下拉，删除当前身份 admin：自动切到剩余的 viewer，身份表只剩它
    await wrapper.find('.identity-btn').trigger('click')
    await flushPromises()
    const adminRemove = [...document.querySelectorAll('.el-dropdown-menu__item .id-remove')][0]
    expect(adminRemove).toBeTruthy()
    adminRemove!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flushPromises()
    await flushPromises()

    expect(JSON.parse(localStorage.getItem('agent-memory-identities')!)).toEqual({ viewer: 'tok-viewer' })
    expect(wrapper.find('.identity-btn').text()).toContain('viewer')
    expect(navLabels(wrapper)).toEqual(['记忆管理', '标签管理', '运维'])
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
