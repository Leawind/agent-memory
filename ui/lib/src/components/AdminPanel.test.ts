// AdminPanel 挂载测试：stub fetch + 真实渲染 Element Plus 组件
// （el-* 导入由 unplugin-vue-components 在构建期注入，测试无需全局注册）
// 覆盖：身份表/鉴权开关/自定义提示词渲染、非 admin 提示、备份与体检区
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../i18n'
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
          return Promise.resolve(jsonResponse({ instructions: 'team rules', conventions: null, auth_required: true }))
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
    // 留空即默认：两个字段各带一条留空行为提示，不再有「恢复默认」按钮
    expect(html).toContain('留空时使用内置默认提示词')
    expect(html).toContain('追加在基础提示词之后')
    expect(html).not.toContain('恢复默认')
    // 备份与体检集中在管理面板
    expect(html).toContain('备份导入导出')
    expect(html).toContain('导出备份')
    expect(html).toContain('数据体检')
    expect(html).toContain('新建身份')
    wrapper.unmount()
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
})
