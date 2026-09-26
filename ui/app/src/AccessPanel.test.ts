// AccessPanel 挂载测试：stub fetch + 真实渲染 Element Plus 组件
// （el-* 导入由 unplugin-vue-components 在构建期注入，测试无需全局注册）
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '@agent-memory/ui'
import AccessPanel from './AccessPanel.vue'
import type { WhoAmI } from '../auth'

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

describe('AccessPanel', () => {
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
              identities: [
                {
                  name: 'bob',
                  token_hint: 'a1b2',
                  permissions: { read: true },
                  created_at: 1700000000,
                },
              ],
            }),
          )
        }
        if (u.includes('/api/settings')) {
          return Promise.resolve(
            jsonResponse({ instructions: 'team rules', conventions: null, default_instructions: 'built-in default' }),
          )
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(AccessPanel, {
      props: { who: { name: 'alice', mode: 'token', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('bob')
    // 列表只有尾缀提示（服务端只存哈希），并带重置入口
    expect(html).toContain('…a1b2')
    expect(html).toContain('重置 Token')
    // textarea 的值是 DOM property，不在 innerHTML 里
    const textareas = wrapper.findAll('textarea')
    expect(textareas.length).toBe(2)
    expect((textareas[0].element as HTMLTextAreaElement).value).toBe('team rules')
    expect((textareas[1].element as HTMLTextAreaElement).value).toBe('')
    // 两个提示词字段各带一个"恢复默认"
    expect(html.split('恢复默认').length - 1).toBe(2)
    expect(html).toContain('新建身份')
    wrapper.unmount()
  })

  it('hides management for non-admin identities', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse({}))),
    )
    const wrapper = mount(AccessPanel, {
      props: { who: { name: 'bob', mode: 'token', permissions: { read: true } } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    expect(wrapper.html()).toContain('需要 admin 权限')
    // 隐藏的 el-dialog 标题仍在 DOM 里，这里断言的是操作入口：无「新建身份」按钮
    expect(wrapper.find('.panel-head button').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows the open-mode notice with admin capabilities', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse({ identities: [] }))),
    )
    const wrapper = mount(AccessPanel, {
      props: { who: { name: 'local', mode: 'open', permissions: ALL_TRUE } },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    expect(wrapper.html()).toContain('开放模式')
    wrapper.unmount()
  })
})
