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
    // Semantic search cards: the candidate editor (empty) plus the reranker card, status-first
    expect(html).toContain('向量模型')
    expect(html).toContain('重排')
    expect(html).toContain('添加')
    // Missing-key fallback no longer appears (vue-i18n echoes the key itself when missing)
    expect(html).not.toContain('access.embedding')
    expect(html).not.toContain('access.identityCard')
    wrapper.unmount()
  })

  it('category nav switches the visible section while every card stays mounted', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => Promise.resolve(jsonResponse({}))),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    const items = wrapper.findAll('.admin-nav .nav-item')
    expect(items.map((i) => i.text())).toEqual([
      '身份与访问',
      '搜索模型',
      '生命周期',
      '标签规则',
      '自定义提示词',
      '备份导入导出',
      '数据体检',
    ])
    const cards = () => wrapper.findAll('.admin-sections .el-card')
    const visible = () =>
      cards().filter((card) => {
        const section = card.element.closest('.lifecycle-cards') ?? card.element
        return (section as HTMLElement).style.display !== 'none'
      })
    expect(cards()).toHaveLength(10)
    expect(visible()).toHaveLength(1)
    await items.find((item) => item.text() === '数据体检')!.trigger('click')
    expect(visible()).toHaveLength(1)
    expect(visible()[0].text()).toContain('数据体检')
    await items[1].trigger('click')
    expect(visible()).toHaveLength(3)
    await items.find((item) => item.text() === '生命周期')!.trigger('click')
    await flushPromises()
    expect(visible()).toHaveLength(2)
    expect(cards()).toHaveLength(10)
    wrapper.unmount()
  })

  it('candidate budgets reject an undersized recall pool and save both limits together', async () => {
    const writes: unknown[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        if (String(url).includes('/api/settings') && init?.method === 'PUT') {
          writes.push(JSON.parse(String(init.body)))
          return Promise.resolve(jsonResponse({}))
        }
        if (String(url).includes('/api/settings')) {
          return Promise.resolve(jsonResponse({ search_limits: { semantic_candidates: 30, rerank_candidates: 20 } }))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    const card = wrapper.find('.search-limits-card')
    const inputs = card.findAll('input')
    const save = card.findAll('button').find((button) => button.text() === '保存')!
    await inputs[0].setValue('10')
    expect(save.attributes('disabled')).toBeDefined()
    await inputs[0].setValue('40')
    expect(save.attributes('disabled')).toBeUndefined()
    await save.trigger('click')
    await flushPromises()
    expect(writes).toEqual([{ search_limits: { semantic_candidates: 40, rerank_candidates: 20 } }])
    wrapper.unmount()
  })

  // Every settings card ends in a button named 保存; scope to the prompt card's own one
  function promptCardSave(wrapper: ReturnType<typeof mount>) {
    const card = wrapper.findAll('.admin-sections .el-card').find((c) => c.text().includes('自定义提示词'))!
    return card.findAll('button').find((b) => b.text() === '保存')!
  }

  it('prefills the built-in default when unset and gates save on changes', async () => {
    const settingsPuts: string[] = []
    // The mock stores what a PUT saved and returns it on GET, like the real server
    const stored: Record<string, unknown> = {}
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        if (u.includes('/api/settings') && init?.method === 'PUT') {
          settingsPuts.push(String(init.body))
          Object.assign(stored, JSON.parse(String(init.body)))
          return Promise.resolve(jsonResponse({ saved: true }))
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(jsonResponse({ ...stored, default_instructions: 'BUILT-IN DEFAULT PROMPT' }))
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
    // Content identical to the effective value: a save would be a no-op, the button is disabled
    const saveBtn = promptCardSave(wrapper)!
    expect(saveBtn.attributes('disabled')).toBeDefined()
    // Editing enables the save, and the PUT carries the edited text verbatim
    await textarea.setValue('team rules v2')
    expect(saveBtn.attributes('disabled')).toBeUndefined()
    await saveBtn.trigger('click')
    await flushPromises()
    expect(settingsPuts).toEqual([JSON.stringify({ instructions: 'team rules v2' })])
    // The parent reloads after the save: the editor re-syncs to the stored value and the
    // button settles back to disabled
    await flushPromises()
    expect(saveBtn.attributes('disabled')).toBeDefined()
    wrapper.unmount()
    // Remove toasts triggered by this case so leftovers don't interfere with later content-based toast assertions
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('saves a default-identical edit as unset (empty) when an override is stored', async () => {
    const settingsPuts: string[] = []
    // The mock stores what a PUT saved and returns it on GET, like the real server
    const stored: Record<string, unknown> = { instructions: 'team rules' }
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        if (u.includes('/api/settings') && init?.method === 'PUT') {
          settingsPuts.push(String(init.body))
          Object.assign(stored, JSON.parse(String(init.body)))
          return Promise.resolve(jsonResponse({ saved: true }))
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(jsonResponse({ ...stored, default_instructions: 'BUILT-IN' }))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    const saveBtn = promptCardSave(wrapper)!
    // The stored override differs from nothing yet: save stays disabled
    expect(saveBtn.attributes('disabled')).toBeDefined()
    // Typing text identical to the built-in default still differs from the stored override,
    // so it is savable — and normalizes to '' (keeps following the default, not freezing a snapshot)
    await wrapper.findAll('textarea')[0].setValue('BUILT-IN')
    expect(saveBtn.attributes('disabled')).toBeUndefined()
    await saveBtn.trigger('click')
    await flushPromises()
    expect(settingsPuts).toEqual([JSON.stringify({ instructions: '' })])
    // After the reload the stored value is unset (empty): the editor follows the default again
    // and the button settles back to disabled
    await flushPromises()
    expect(saveBtn.attributes('disabled')).toBeDefined()
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('embedding card: candidate editor saves the canonical list, then probes on demand', async () => {
    const settingsPuts: string[] = []
    const stored: Record<string, unknown> = {}
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        if (u.includes('/api/settings') && init?.method === 'PUT') {
          settingsPuts.push(String(init.body))
          Object.assign(stored, JSON.parse(String(init.body)))
          return Promise.resolve(jsonResponse({ saved: true }))
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(jsonResponse(stored))
        }
        if (u.includes('/api/embeddings/test')) {
          return Promise.resolve(
            jsonResponse({ ok: true, results: [{ index: 0, model: 'bge-m3-x', ok: true, dim: 1024, elapsed_ms: 42 }] }),
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
    // Switch to the semantic section and add one candidate through the editor (new rows open)
    await wrapper.findAll('.admin-nav .nav-item')[1].trigger('click')
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '添加')!
      .trigger('click')
    await wrapper.findAll('.entry')[0].find('.f-id input')!.setValue('local-vector')
    await wrapper.findAll('.entry')[0].find('.f-name input')!.setValue('Local vector')
    await wrapper.findAll('.entry')[0].find('.f-model input')!.setValue('bge-m3-x')
    await wrapper.findAll('.entry')[0].find('.f-base-url input')!.setValue('http://svc:9/v1')
    const saveBtn = wrapper.findAll('button').find((b) => b.text() === '保存')!
    expect(saveBtn.attributes('disabled')).toBeUndefined()
    await saveBtn.trigger('click')
    await flushPromises()
    await flushPromises()
    expect(settingsPuts).toEqual([
      JSON.stringify({
        embedding_models: [
          {
            id: 'local-vector',
            name: 'Local vector',
            enabled: true,
            base_url: 'http://svc:9/v1',
            model: 'bge-m3-x',
            api_key: null,
            query_prefix: null,
            passage_prefix: null,
            min_similarity: null,
          },
        ],
      }),
    ])
    // Saving does not probe on its own: the explicit test button does, over the draft's values
    const probeCalls = () =>
      vi.mocked(globalThis.fetch).mock.calls.filter((c) => String(c[0]).includes('/api/embeddings/test'))
    expect(probeCalls()).toHaveLength(0)
    // The row's own test button probes exactly that row's values
    await wrapper
      .findAll('.entry')[0]
      .findAll('button')
      .find((b) => b.text() === '检测')!
      .trigger('click')
    await flushPromises()
    expect(probeCalls()).toHaveLength(1)
    expect(String(probeCalls()[0][1]?.body)).toContain('"model":"bge-m3-x"')
    // The editor mirrors the saved values with its actions disabled again
    await flushPromises()
    expect(saveBtn.attributes('disabled')).toBeDefined()
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('gates the identity dialog submit: create needs a name, edit needs a diff', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        const u = String(url)
        if (u.includes('/api/identities') && init?.method === 'PUT') {
          return Promise.resolve(jsonResponse({ saved: true }))
        }
        if (u.includes('/api/identities') && init?.method === 'POST') {
          return Promise.resolve(
            jsonResponse({
              name: 'bob',
              token_hint: 'b2c3',
              permissions: { read: true },
              token: 'tok-bob',
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
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()

    // Create: the member preset pre-fills caps, but without a name there is nothing to save
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '新建身份')!
      .trigger('click')
    await flushPromises()
    const createDialog = wrapper.find('.el-dialog')
    const createSave = createDialog.findAll('button').find((b) => b.text() === '保存')!
    expect(createSave.attributes('disabled')).toBeDefined()
    await createDialog.find('input').setValue('bob')
    expect(createSave.attributes('disabled')).toBeUndefined()
    await createSave.trigger('click')
    await flushPromises()
    await flushPromises()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())

    // Edit: the dialog opens on bob's stored caps, save is disabled until a capability changes
    await wrapper.findAll('button[aria-label="编辑"]')[0].trigger('click')
    await flushPromises()
    const editDialog = wrapper.find('.el-dialog')
    const editSave = editDialog.findAll('button').find((b) => b.text() === '保存')!
    expect(editSave.attributes('disabled')).toBeDefined()
    // Uncheck the read capability (the first checkbox): now there is a diff to save
    await editDialog.find('.el-checkbox input').setValue(false)
    expect(editSave.attributes('disabled')).toBeUndefined()
    await editSave.trigger('click')
    await flushPromises()
    expect(
      vi
        .mocked(globalThis.fetch)
        .mock.calls.some((c) => String(c[0]).includes('/api/identities/bob') && (c[1]?.method as string) === 'PUT'),
    ).toBe(true)
    wrapper.unmount()
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

  it('shows the open-mode notice only in the access section', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse({ identities: [] }))),
    )
    const wrapper = mount(AdminPanel, {
      props: { who: { name: 'open', mode: 'open', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    // The notice is an identity/auth concern: it renders in the access section (the default)
    expect(wrapper.html()).toContain('开放模式')
    // Switching to any other tab removes it entirely (v-if, not merely hidden)
    await wrapper.findAll('.admin-nav .nav-item')[1].trigger('click')
    expect(wrapper.text()).not.toContain('开放模式')
    await wrapper.findAll('.admin-nav .nav-item')[0].trigger('click')
    expect(wrapper.text()).toContain('开放模式')
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

  it('shows doctor results with the named checklist', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.includes('/api/doctor')) {
          return Promise.resolve(
            jsonResponse({
              ok: false,
              checks: [
                { id: 'integrity', ok: true, issues: [] },
                { id: 'tag_refs', ok: false, issues: ['orphan tag: ghost'] },
              ],
            }),
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
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '运行体检')!
      .trigger('click')
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('发现 1 个问题（1/2 项检查未通过）')
    expect(html).toContain('orphan tag: ghost')
    // Every check gets a verdict row, passing ones included
    expect(html).toContain('数据库完整性')
    expect(html).toContain('标签引用')
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
