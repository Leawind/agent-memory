import { describe, expect, it, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../../index'
import TagRulesCard from './TagRulesCard.vue'

describe('TagRulesCard', () => {
  it('previews conflicts without saving and clears stale results when the draft changes', async () => {
    setMemoryUILocale('zh')
    const calls: Array<{ path: string; method?: string; body: unknown }> = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        calls.push({ path: String(url), method: init?.method, body: JSON.parse(String(init?.body)) })
        return Promise.resolve({
          ok: true,
          status: 200,
          text: () =>
            Promise.resolve(
              JSON.stringify({
                valid: false,
                total_violations: 1,
                violations: [{ id: 'm1', summary: 'summary only', rules: ['exclusivity'] }],
              }),
            ),
        })
      }),
    )
    const wrapper = mount(TagRulesCard, {
      props: { rules: [{ name: 'exclusivity', expression: 'mutex(a,b)' }] },
      global: { plugins: [ElementPlus] },
    })
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '检查现有记忆')!
      .trigger('click')
    await flushPromises()
    expect(calls).toEqual([
      {
        path: '/api/tag-rules/preview',
        method: 'POST',
        body: { constraints: [{ name: 'exclusivity', expression: 'mutex(a,b)' }] },
      },
    ])
    expect(wrapper.text()).toContain('有 1 条记忆违反规则')
    expect(wrapper.text()).toContain('summary only')
    await wrapper.findAll('input')[1].setValue('!a|b')
    expect(wrapper.find('.el-alert').exists()).toBe(false)
    wrapper.unmount()
  })

  it('requires named expressions and publishes the complete draft atomically', async () => {
    setMemoryUILocale('zh')
    const writes: unknown[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((_url: string | URL, init?: RequestInit) => {
        writes.push(JSON.parse(String(init?.body)))
        return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve('{"saved":true}') })
      }),
    )
    const wrapper = mount(TagRulesCard, { props: { rules: [] }, global: { plugins: [ElementPlus] } })
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '添加约束')!
      .trigger('click')
    const save = () => wrapper.findAll('button').find((b) => b.text() === '保存')!
    expect(save().attributes('disabled')).toBeDefined()
    await wrapper.findAll('input')[0].setValue('family')
    await wrapper.findAll('input')[1].setValue('!child|parent')
    await save().trigger('click')
    await flushPromises()
    expect(writes).toEqual([{ constraints: [{ name: 'family', expression: '!child|parent' }] }])
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
  })
})
