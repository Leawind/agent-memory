// App shell mount smoke test: top-bar navigation rendering, theme application, identity dropdown
// and admin tab permissions
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '@agent-memory/ui'
import App from './App.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

// whoami parses via res.json(), so the response needs a json() method
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

/** Assert the label set of the nav buttons */
function navLabels(wrapper: ReturnType<typeof mount>): string[] {
  return wrapper.findAll('nav button').map((b) => b.text())
}

/** The token dialog's open state (App.vue's tokenDialog). el-dialog's close transition never
 * truly finishes in happy-dom, so DOM assertions of the collapsed state are unreliable; read the
 * component state driving it instead */
function dialogOpen(wrapper: ReturnType<typeof mount>): boolean {
  return (wrapper.vm.$ as unknown as { setupState: { tokenDialog: boolean } }).setupState.tokenDialog
}

describe('App shell', () => {
  beforeEach(() => {
    setMemoryUILocale('zh')
    localStorage.clear()
    // el-dropdown menus teleport to body; leftovers from the previous case may intercept querySelector:
    // clear body so each case starts from a clean DOM
    document.body.innerHTML = ''
    // Simulate a persisted Chinese language preference (happy-dom's navigator.language is en-US)
    localStorage.setItem('agent-memory-locale', 'zh')
    document.documentElement.classList.remove('dark')
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.includes('/api/tags')) {
          return Promise.resolve(jsonResponse({ tags: [] }))
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
    expect(html).toContain('记忆')
    expect(html).toContain('新建记忆')
    expect(html).toContain('标签')
    // The ops tab was removed: the overview now opens as a dialog by clicking the top-bar title
    expect(html).not.toContain('运维')
    wrapper.unmount()
  })

  it('shows the overview dialog with stats when the brand title is clicked', async () => {
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    const countStatsCalls = () =>
      vi.mocked(globalThis.fetch).mock.calls.filter((c) => String(c[0]).includes('/api/stats')).length
    // No stats requests while the dialog is unopened
    expect(countStatsCalls()).toBe(0)

    await wrapper.find('.brand-btn').trigger('click')
    await flushPromises()
    await flushPromises()
    const dialog = wrapper.find('.el-dialog')
    expect(dialog.isVisible()).toBe(true)
    expect(countStatsCalls()).toBe(1)
    expect(dialog.text()).toContain('/tmp/m.db')
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
    expect(html).not.toContain('记忆')
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

  it('works as the anonymous identity when the server grants anonymous capabilities', async () => {
    // Auth is on but anonymous read-only is configured: browsing works without a token, no token
    // dialog pops, the identity area shows the anonymous label and the dropdown stays usable
    // (making it easy to add an identity and upgrade)
    const anonWho = { name: 'anonymous', mode: 'anonymous' as const, permissions: viewerCaps }
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        if (String(url).includes('/api/whoami')) return Promise.resolve(jsonWithJson(anonWho))
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    expect(dialogOpen(wrapper)).toBe(false)
    expect(wrapper.find('.identity-btn').exists()).toBe(true)
    expect(wrapper.find('.identity-btn').text()).toContain('匿名访问')
    // Read-only anonymous: content panels render as usual, the admin tab is hidden (no admin capability)
    expect(navLabels(wrapper)).toEqual(['记忆', '标签'])
    expect(wrapper.html()).toContain('新建记忆')
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
          return Promise.resolve(jsonResponse({ instructions: null, auth_required: false }))
        }
        if (u.includes('/api/tags')) return Promise.resolve(jsonResponse({ tags: [] }))
        return Promise.resolve(jsonResponse({ total: 0, memories: [] }))
      }),
    )
    const wrapper = mount(App, { global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    // Admin identity: all three tabs present, the admin panel is mounted (identity table visible)
    expect(navLabels(wrapper)).toEqual(['记忆', '标签', '管理'])
    expect(wrapper.html()).toContain('Token 鉴权')
    wrapper.unmount()

    // Read-only identity: the admin tab is hidden and the admin panel is not mounted (not even in the DOM)
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
    expect(navLabels(viewer)).toEqual(['记忆', '标签'])
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

    // Open the identity dropdown and switch to viewer: the admin tab disappears with it (dropdown menus teleport to body)
    await wrapper.find('.identity-btn').trigger('click')
    await flushPromises()
    const viewerItem = [...document.querySelectorAll('.el-dropdown-menu__item')].find((el) =>
      el.textContent?.includes('viewer'),
    )
    expect(viewerItem).toBeTruthy()
    // Each identity row carries a token suffix hint (same style as the admin UI: ellipsis + last 4 chars)
    expect(viewerItem!.textContent).toContain('…ewer')
    viewerItem!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flushPromises()
    await flushPromises()
    expect(navLabels(wrapper)).toEqual(['记忆', '标签'])
    expect(wrapper.find('.identity-btn').text()).toContain('viewer')
    wrapper.unmount()
  })

  it('keeps the identity dropdown usable when the current token turns invalid', async () => {
    // broken's token has been rejected by the server; viewer is still valid, and switching must restore the page
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

    // Regression: when the current identity fails, only it is removed and the token dialog pops;
    // the dropdown must remain (it previously vanished along with who, leaving no way to switch and recover)
    expect(wrapper.find('.identity-btn').exists()).toBe(true)
    expect(wrapper.find('.identity-btn').text()).toContain('未连接')
    expect(wrapper.find('.el-dialog').isVisible()).toBe(true)
    // happy-dom cannot do real hit-testing: assert that the dialog render container has the
    // penetrable class (container pointer-events:none, dialog body auto), guaranteeing that in a
    // real browser the top-bar dropdown is not blocked by the full-screen container while the
    // dialog is open (previously blocked even with modal=false)
    expect(wrapper.find('.el-modal-dialog.is-penetrable').exists()).toBe(true)
    expect(JSON.parse(localStorage.getItem('agent-memory-identities')!)).toEqual({ viewer: 'tok-viewer' })

    // The dropdown still lists the other identities; after a successful switch the token dialog collapses and the page recovers under the new identity
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
    // el-dialog's close transition never truly finishes in happy-dom (display:none never lands),
    // so assert the component state driving the dialog instead
    expect(dialogOpen(wrapper)).toBe(false)
    expect(navLabels(wrapper)).toEqual(['记忆', '标签'])
    wrapper.unmount()
  })

  it('recovers through the unauthorized event when the active token is revoked mid-session', async () => {
    // admin is valid at mount; revoked by the server mid-session: from then on requests carrying tok-admin always 401
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

    // A panel request gets 401 -> authFetch broadcasts UNAUTHORIZED_EVENT -> the invalid identity is removed and the token dialog pops
    revoked = true
    await wrapper.findAll('nav button')[1].trigger('click')
    await flushPromises()
    await flushPromises()

    expect(JSON.parse(localStorage.getItem('agent-memory-identities')!)).toEqual({ viewer: 'tok-viewer' })
    expect(wrapper.find('.identity-btn').exists()).toBe(true)
    expect(dialogOpen(wrapper)).toBe(true)

    // The non-modal dialog does not lock the top bar: the identity dropdown stays operable, switching to viewer recovers
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

    // Open the dropdown and delete the current identity admin: auto-switch to the remaining viewer, which is then the only identity left
    await wrapper.find('.identity-btn').trigger('click')
    await flushPromises()
    const adminRemove = [...document.querySelectorAll('.el-dropdown-menu__item .id-remove')][0]
    expect(adminRemove).toBeTruthy()
    adminRemove!.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    await flushPromises()
    await flushPromises()

    expect(JSON.parse(localStorage.getItem('agent-memory-identities')!)).toEqual({ viewer: 'tok-viewer' })
    expect(wrapper.find('.identity-btn').text()).toContain('viewer')
    expect(navLabels(wrapper)).toEqual(['记忆', '标签'])
    wrapper.unmount()
  })

  it('loads identity data once the admin identity resolves after mount', async () => {
    // whoami hangs, simulating real timing: mounting happens before identity resolution completes
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
          return Promise.resolve(jsonResponse({ instructions: null, default_instructions: '' }))
        }
        if (u.includes('/api/tags')) return Promise.resolve(jsonResponse({ tags: [] }))
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
    // No prefetching of admin data while the identity is not ready
    expect(identityCalls.length).toBe(0)
    resolveWhoAmI(jsonWithJson({ name: 'admin', mode: 'token', permissions: adminCaps }))
    await flushPromises()
    await flushPromises()
    // Regression: loading must happen once admin is ready (onMounted previously checked only once, leaving the panel stuck empty forever)
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
    // Regression: v-show panels stay mounted and are never remounted, so switching back to a tab
    // must refresh explicitly (previously stale data was shown indefinitely)
    expect(countTagsCalls()).toBeGreaterThan(before)
    wrapper.unmount()
  })
})
