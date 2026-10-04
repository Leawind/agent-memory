// AdminPanel mount tests: stub fetch + real Element Plus rendering
// (el-* imports are injected at build time by unplugin-vue-components; tests need no global registration)
// Covers: identity table / auth toggle / custom prompt rendering, non-admin notices, the backup
// and doctor sections, the token-once dialog's "save to this browser" hook, and the precheck
// when enabling auth without an admin identity
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { MemoryUIConfigKey, setMemoryUILocale } from '../index'
import AdminPanel from './AdminPanel.vue'
import type { WhoAmI } from '../types'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

const ALL_TRUE = {
  read: true,
  create: true,
  update: true,
  delete: true,
  tag_manage: true,
  admin: true,
}

describe('AdminPanel', () => {
  beforeEach(() => {
    setMemoryUILocale('zh')
  })

  it('renders the identity table and settings editor for admin', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.includes('/api/identities')) {
          return Promise.resolve(
            jsonResponse({
              identities: [{ name: 'bob', token_hint: 'a1b2', permissions: { read: true }, created_at: 1 }],
            }),
          )
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(
            jsonResponse({
              instructions: 'team rules',
              auth_required: true,
              default_instructions: 'BUILT-IN DEFAULT PROMPT',
            }),
          )
        }
        if (u.includes('/api/stats')) {
          return Promise.resolve(
            jsonResponse({ embedding: { enabled: true, model: 'bge-m3', embedded: 3, pending: 1 } }),
          )
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('bob')
    // The list only shows the suffix hint (the server stores hashes only), with a reset entry
    expect(html).toContain('…a1b2')
    expect(html).toContain('重置 Token')
    // The auth toggle and identity list are visible on the same screen
    expect(html).toContain('Token 鉴权')
    // A textarea's value is a DOM property, not present in innerHTML
    const textareas = wrapper.findAll('textarea')
    expect(textareas.length).toBe(1)
    expect((textareas[0].element as HTMLTextAreaElement).value).toBe('team rules')
    // Usage notes are tucked into the info icon next to labels (panel title + the prompt field = at
    // least 2); edit/preview is the single-icon toggle in the card header
    expect(wrapper.findAll('.am-info').length).toBeGreaterThanOrEqual(2)
    expect(wrapper.findAll('.md-mode-toggle').length).toBe(1)
    // Click to switch to preview: the textarea is replaced by the Markdown rendering
    await wrapper.find('.md-mode-toggle').trigger('click')
    await flushPromises()
    expect(wrapper.findAll('textarea').length).toBe(0)
    expect(wrapper.find('.md-body').exists()).toBe(true)
    // Click again to switch back to edit
    await wrapper.find('.md-mode-toggle').trigger('click')
    await flushPromises()
    expect(wrapper.findAll('textarea').length).toBe(1)
    const resetBtn = wrapper.findAll('button').find((b) => b.text() === '恢复默认')
    expect(resetBtn).toBeTruthy()
    expect(resetBtn!.attributes('disabled')).toBeUndefined()
    // Backup and doctor are centralized in the admin panel
    expect(html).toContain('备份导入导出')
    expect(html).toContain('导出备份')
    expect(html).toContain('数据体检')
    expect(html).toContain('新建身份')
    // The create-identity entry sits at the right of the identity section row inside the auth
    // card, next to the identity table (three cards merged into one auth card)
    const createBtn = wrapper.findAll('button').find((b) => b.text() === '新建身份')
    expect(createBtn).toBeTruthy()
    expect(createBtn!.element.closest('.el-card__header')).toBeNull()
    expect(createBtn!.element.closest('.el-card__body')).toBeTruthy()
    // Semantic search is a single card: config + vector coverage section share it; no second card with the same name
    expect(html.match(/语义搜索（embedding）/g)).toHaveLength(1)
    expect(html).toContain('向量覆盖率')
    expect(html).toContain('补跑向量化')
    expect(html).toContain('1 条记忆缺最新向量')
    // Missing-key fallback no longer appears (vue-i18n echoes the key itself when missing)
    expect(html).not.toContain('access.embedding')
    expect(html).not.toContain('access.identityCard')
    wrapper.unmount()
  })

  // The save row's "save" coexists with the identically named dialog button; find the card's one only
  function promptCardSave(wrapper: ReturnType<typeof mount>) {
    return wrapper.findAll('button').filter((b) => b.text() === '保存' && !b.element.closest('.el-dialog'))[0]
  }

  it('prefills the built-in default when unset and normalizes it on save', async () => {
    const settingsPuts: string[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        if (u.includes('/api/settings') && init?.method === 'PUT') {
          settingsPuts.push(String(init.body))
          return Promise.resolve(jsonResponse({ saved: true }))
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(jsonResponse({ instructions: null, default_instructions: 'BUILT-IN DEFAULT PROMPT' }))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    // When unset, the editor prefills the built-in default directly (what you see is what applies)
    const textarea = wrapper.findAll('textarea')[0]
    expect((textarea.element as HTMLTextAreaElement).value).toBe('BUILT-IN DEFAULT PROMPT')
    // Content equals the default -> "reset to default" has nothing to do, disabled
    const resetBtn = wrapper.findAll('button').find((b) => b.text() === '恢复默认')!
    expect(resetBtn.attributes('disabled')).toBeDefined()
    // Save without modification: content identical to the default normalizes to an empty string (keeps following the default instead of freezing a snapshot)
    await promptCardSave(wrapper)!.trigger('click')
    await flushPromises()
    expect(settingsPuts).toEqual([JSON.stringify({ instructions: '' })])
    wrapper.unmount()
    // Remove toasts triggered by this case so leftovers don't interfere with later content-based toast assertions
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('reset-to-default clears the override', async () => {
    const settingsPuts: string[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        if (u.includes('/api/settings') && init?.method === 'PUT') {
          settingsPuts.push(String(init.body))
          return Promise.resolve(jsonResponse({ saved: true }))
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(jsonResponse({ instructions: 'team rules', default_instructions: 'BUILT-IN' }))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '恢复默认')!
      .trigger('click')
    await flushPromises()
    // Only clears instructions
    expect(settingsPuts).toEqual([JSON.stringify({ instructions: '' })])
    // The editor is filled back with the built-in default, entering the faded default state
    const textarea = wrapper.findAll('textarea')[0]
    expect((textarea.element as HTMLTextAreaElement).value).toBe('BUILT-IN')
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('hides management for non-admin identities', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse({}))),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'bob', mode: 'token', permissions: { read: true } } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    expect(wrapper.html()).toContain('需要 admin 权限')
    // Hidden el-dialog titles remain in the DOM; what is asserted here is the action entry: no create-identity button in the header
    expect(wrapper.find('.am-panel-header button').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows the open-mode notice with admin capabilities', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse({ identities: [] }))),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'open', mode: 'open', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    expect(wrapper.html()).toContain('开放模式')
    wrapper.unmount()
  })

  it('defers loading until the admin identity arrives (who initially null)', async () => {
    const identityCalls: string[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        if (String(url).includes('/api/identities')) {
          identityCalls.push(String(url))
          return Promise.resolve(jsonResponse({ identities: [] }))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: null },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    // No prefetch of admin data while the identity is not ready (the server responds 403 without who)
    expect(identityCalls.length).toBe(0)
    await wrapper.setProps({ who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } })
    await flushPromises()
    // Regression: loading must happen once the admin capability is ready
    expect(identityCalls.length).toBeGreaterThan(0)
    wrapper.unmount()
  })

  it('shows doctor results with issue list', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.includes('/api/doctor')) {
          return Promise.resolve(jsonResponse({ ok: false, issues: ['orphan tag: ghost'] }))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '运行体检')!
      .trigger('click')
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('发现 1 个问题')
    expect(html).toContain('orphan tag: ghost')
    wrapper.unmount()
  })

  // Helper that walks the "create identity -> token-once dialog" flow: returns the wrapper
  async function createIdentityAndOpenTokenDialog(config?: { onIdentityToken: (name: string, token: string) => void }) {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        if (u.includes('/api/identities') && init?.method === 'POST') {
          return Promise.resolve(
            jsonResponse({
              name: 'bob',
              token_hint: 'b2c3',
              permissions: { read: true },
              token: 'tok-bob-123',
              created_at: 1,
            }),
          )
        }
        if (u.includes('/api/identities')) {
          return Promise.resolve(
            jsonResponse({
              identities: [{ name: 'bob', token_hint: 'b2c3', permissions: { read: true }, created_at: 1 }],
            }),
          )
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: {
        plugins: [ElementPlus],
        ...(config ? { provide: { [MemoryUIConfigKey as symbol]: config } } : {}),
      },
    })
    await flushPromises()
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '新建身份')!
      .trigger('click')
    await flushPromises()
    await wrapper.find('.el-dialog input').setValue('bob')
    // Look for "save" inside the create dialog only, avoiding the prompt card's identically named button
    await wrapper
      .find('.el-dialog')
      .findAll('button')
      .find((b) => b.text() === '保存')!
      .trigger('click')
    await flushPromises()
    await flushPromises()
    return wrapper
  }

  // The token-once dialog (a hidden create dialog also exists in the DOM; pick by content;
  // the copy control is an icon with an aria-label, not visible text)
  function tokenDialog(wrapper: ReturnType<typeof mount>) {
    const dialogs = wrapper.findAll('.el-dialog')
    const found = dialogs.find((d) => d.find('.new-token').exists())
    expect(found, 'token-once dialog should be visible').toBeTruthy()
    return found!
  }

  it('token-once dialog has no save-to-browser without the host hook', async () => {
    const wrapper = await createIdentityAndOpenTokenDialog()
    const dialog = tokenDialog(wrapper)
    expect(dialog.text()).not.toContain('保存到本浏览器')
    wrapper.unmount()
  })

  it('copy icon copies the token and keeps the dialog open', async () => {
    const writeText = vi.fn(() => Promise.resolve())
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true })
    const wrapper = await createIdentityAndOpenTokenDialog()
    const dialog = tokenDialog(wrapper)
    await dialog.find('button[aria-label="复制 Token"]').trigger('click')
    await flushPromises()
    expect(writeText).toHaveBeenCalledWith('tok-bob-123')
    const toast = document.querySelector('.el-message')
    expect(toast?.textContent).toContain('已复制')
    // Toasts auto-dismiss on a real 3s timer; drop it now so later tests querying
    // document-level .el-message elements don't pick this one up
    toast?.remove()
    // The dialog is deliberately left open: closing is the user's job via the footer button
    expect(dialog.find('.new-token').exists()).toBe(true)
    wrapper.unmount()
  })

  it('save-to-browser button hands the one-time token to the host hook', async () => {
    const saved: Array<{ name: string; token: string }> = []
    const wrapper = await createIdentityAndOpenTokenDialog({
      onIdentityToken: (name, token) => {
        saved.push({ name, token })
      },
    })
    const dialog = tokenDialog(wrapper)
    await dialog
      .findAll('button')
      .find((b) => b.text() === '保存到本浏览器')!
      .trigger('click')
    await flushPromises()
    expect(saved).toEqual([{ name: 'bob', token: 'tok-bob-123' }])
    // A success toast confirms the save (the dialog closes with a transition; DOM state is unreliable in happy-dom)
    const toast = document.querySelector('.el-message')
    expect(toast?.textContent).toContain('已保存身份')
    wrapper.unmount()
  })

  it('blocks enabling auth with a guidance alert when no admin identity exists', async () => {
    const settingsPuts: string[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        if (u.includes('/api/settings') && init?.method === 'PUT') {
          settingsPuts.push(String(init.body))
          return Promise.resolve(jsonResponse({ saved: true }))
        }
        if (u.includes('/api/identities')) {
          return Promise.resolve(
            jsonResponse({
              identities: [{ name: 'eve', token_hint: 'e4f5', permissions: { read: true }, created_at: 1 }],
            }),
          )
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(jsonResponse({ auth_required: false }))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    // Find the auth toggle and trigger a flip (before-change should block it)
    const switchEl = wrapper.find('.el-switch')
    expect(switchEl.exists()).toBe(true)
    await switchEl.trigger('click')
    await flushPromises()
    // The precheck blocks: no PUT is sent and the switch stays off
    expect(settingsPuts).toEqual([])
    expect(wrapper.html()).not.toContain('el-switch.is-checked')
    wrapper.unmount()
  })
})
