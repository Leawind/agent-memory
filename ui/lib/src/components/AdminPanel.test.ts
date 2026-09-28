// AdminPanel 挂载测试：stub fetch + 真实渲染 Element Plus 组件
// （el-* 导入由 unplugin-vue-components 在构建期注入，测试无需全局注册）
// 覆盖：身份表/鉴权开关/自定义提示词渲染、非 admin 提示、备份与体检区、
// token 一次性弹窗的「保存到本浏览器」钩子、开启鉴权的无 admin 预检
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
              conventions: null,
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
    // 列表只有尾缀提示（服务端只存哈希），并带重置入口
    expect(html).toContain('…a1b2')
    expect(html).toContain('重置 Token')
    // 鉴权开关与身份列表同屏可见
    expect(html).toContain('Token 鉴权')
    // textarea 的值是 DOM property，不在 innerHTML 里
    const textareas = wrapper.findAll('textarea')
    expect(textareas.length).toBe(2)
    expect((textareas[0].element as HTMLTextAreaElement).value).toBe('team rules')
    expect((textareas[1].element as HTMLTextAreaElement).value).toBe('')
    // 用法说明收进标签旁的 ⓘ（面板标题 + 两个字段 = 至少 3 个）；编辑/预览切换在卡片头部
    expect(wrapper.findAll('.am-info').length).toBeGreaterThanOrEqual(3)
    expect(wrapper.findAll('.el-radio-button input').length).toBe(2)
    // 切到预览：两个 textarea 换成 Markdown 渲染
    const radioInputs = wrapper.findAll('.el-radio-button input')
    ;(radioInputs[1].element as HTMLInputElement).checked = true
    await radioInputs[1].trigger('change')
    await flushPromises()
    expect(wrapper.findAll('textarea').length).toBe(0)
    expect(wrapper.find('.md-body').exists()).toBe(true)
    // 切回编辑
    ;(radioInputs[0].element as HTMLInputElement).checked = true
    await radioInputs[0].trigger('change')
    await flushPromises()
    expect(wrapper.findAll('textarea').length).toBe(2)
    const resetBtn = wrapper.findAll('button').find((b) => b.text() === '恢复默认')
    expect(resetBtn).toBeTruthy()
    expect(resetBtn!.attributes('disabled')).toBeUndefined()
    // 备份与体检集中在管理面板
    expect(html).toContain('备份导入导出')
    expect(html).toContain('导出备份')
    expect(html).toContain('数据体检')
    expect(html).toContain('新建身份')
    // 新建身份入口在鉴权卡内身份小节行的右侧，紧贴身份表（三卡已合并为一张鉴权卡）
    const createBtn = wrapper.findAll('button').find((b) => b.text() === '新建身份')
    expect(createBtn).toBeTruthy()
    expect(createBtn!.element.closest('.el-card__header')).toBeNull()
    expect(createBtn!.element.closest('.el-card__body')).toBeTruthy()
    // 语义搜索只有一张卡：配置 + 向量覆盖率小节同卡，不再出现第二张同名卡
    expect(html.match(/语义搜索（embedding）/g)).toHaveLength(1)
    expect(html).toContain('向量覆盖率')
    expect(html).toContain('补跑向量化')
    expect(html).toContain('1 条记忆缺最新向量')
    // 缺键回退不再出现（vue-i18n 缺键时会把键名原样回显）
    expect(html).not.toContain('access.embedding')
    expect(html).not.toContain('access.identityCard')
    wrapper.unmount()
  })

  // 保存行里「保存」与弹窗内同名按钮共存，这里只找卡片里的那个
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
          return Promise.resolve(
            jsonResponse({ instructions: null, conventions: null, default_instructions: 'BUILT-IN DEFAULT PROMPT' }),
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
    // 未设置时编辑框直接预填内置默认（所见即生效）
    const textarea = wrapper.findAll('textarea')[0]
    expect((textarea.element as HTMLTextAreaElement).value).toBe('BUILT-IN DEFAULT PROMPT')
    // 内容等于默认 → 「恢复默认」无可做之事，禁用
    const resetBtn = wrapper.findAll('button').find((b) => b.text() === '恢复默认')!
    expect(resetBtn.attributes('disabled')).toBeDefined()
    // 未修改直接保存：与默认一致归一为空串（继续跟随默认而非冻结快照）
    await promptCardSave(wrapper)!.trigger('click')
    await flushPromises()
    expect(settingsPuts).toEqual([JSON.stringify({ instructions: '', conventions: '' })])
    wrapper.unmount()
    // 清掉本用例触发的 toast，避免残留干扰后续按内容挑 toast 的断言
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('reset-to-default clears the override without touching conventions', async () => {
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
          return Promise.resolve(
            jsonResponse({ instructions: 'team rules', conventions: null, default_instructions: 'BUILT-IN' }),
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
      .find((b) => b.text() === '恢复默认')!
      .trigger('click')
    await flushPromises()
    // 只清 instructions；conventions 键省略 = 服务端不改动
    expect(settingsPuts).toEqual([JSON.stringify({ instructions: '' })])
    // 编辑框填回内置默认，进入淡色默认态
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
    // 隐藏的 el-dialog 标题仍在 DOM 里，这里断言的是操作入口：头部无「新建身份」按钮
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
    // 身份未就绪不预取管理数据（无 who 时服务端会 403）
    expect(identityCalls.length).toBe(0)
    await wrapper.setProps({ who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } })
    await flushPromises()
    // 回归：admin 能力就绪后必须补加载
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

  // 走完「新建身份 → token 一次性弹窗」流程的辅助：返回弹窗包装
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
    // 限定在创建弹窗内找「保存」，避免命中自定义提示词卡片的同名按钮
    await wrapper
      .find('.el-dialog')
      .findAll('button')
      .find((b) => b.text() === '保存')!
      .trigger('click')
    await flushPromises()
    await flushPromises()
    return wrapper
  }

  // token 一次性弹窗（DOM 里同时存在隐藏的创建弹窗，按内容挑第二个）
  function tokenDialog(wrapper: ReturnType<typeof mount>) {
    const dialogs = wrapper.findAll('.el-dialog')
    const found = dialogs.find((d) => d.text().includes('复制 Token'))
    expect(found, 'token-once dialog should be visible').toBeTruthy()
    return found!
  }

  it('token-once dialog has no save-to-browser without the host hook', async () => {
    const wrapper = await createIdentityAndOpenTokenDialog()
    const dialog = tokenDialog(wrapper)
    expect(dialog.text()).not.toContain('保存到本浏览器')
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
    // 保存成功有反馈 toast（弹窗经过渡关闭，happy-dom 里 DOM 状态不可靠）
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
    // 找到鉴权开关并触发切换（before-change 应拦下）
    const switchEl = wrapper.find('.el-switch')
    expect(switchEl.exists()).toBe(true)
    await switchEl.trigger('click')
    await flushPromises()
    // 预检拦截：不发 PUT，开关保持关
    expect(settingsPuts).toEqual([])
    expect(wrapper.html()).not.toContain('el-switch.is-checked')
    wrapper.unmount()
  })
})
