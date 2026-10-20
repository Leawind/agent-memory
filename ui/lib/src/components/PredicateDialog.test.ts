import { describe, it, expect, vi } from 'vitest'
import { mount, flushPromises, DOMWrapper } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../i18n'
import PredicateDialog from './PredicateDialog.vue'

describe('PredicateDialog', () => {
  it('hides delete without the dedicated capability and searches by name', async () => {
    setMemoryUILocale('zh')
    document.body.innerHTML = ''
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string) =>
        Promise.resolve({
          ok: true,
          status: 200,
          text: () =>
            Promise.resolve(
              JSON.stringify(
                String(url).includes('whoami')
                  ? { permissions: { read: true, admin: true, predicate_manage: false } }
                  : { predicates: [{ name: 'topic', predicate: 'a', description: 'selector' }] },
              ),
            ),
        }),
      ),
    )
    const wrapper = mount(PredicateDialog, { props: { visible: true }, global: { plugins: [ElementPlus] } })
    await flushPromises()
    await flushPromises()
    const dialog = new DOMWrapper(document.querySelector('.el-dialog')!)
    expect(dialog.text()).toContain('topic')
    expect(dialog.findAll('button').some((b) => b.text() === '删除')).toBe(false)
    await dialog
      .findAll('button')
      .find((b) => b.text() === '搜索')!
      .trigger('click')
    expect(wrapper.emitted('apply')).toEqual([['@topic']])
    wrapper.unmount()
  })
  it('creates a predicate with all required fields', async () => {
    setMemoryUILocale('zh')
    document.body.innerHTML = ''
    const writes: unknown[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string, init?: RequestInit) => {
        if (init?.method === 'PUT') writes.push(JSON.parse(String(init.body)))
        return Promise.resolve({
          ok: true,
          status: 200,
          text: () =>
            Promise.resolve(
              JSON.stringify(
                String(url).includes('whoami')
                  ? { permissions: { read: true, predicate_manage: true } }
                  : { predicates: [] },
              ),
            ),
        })
      }),
    )
    const wrapper = mount(PredicateDialog, { props: { visible: true }, global: { plugins: [ElementPlus] } })
    await flushPromises()
    const dialog = new DOMWrapper(document.querySelector('.el-dialog')!)
    const inputs = dialog.findAll('.el-form input:not([readonly])')
    await inputs[0].setValue('my_predicate')
    await inputs[1].setValue('a&!b')
    await inputs[2].setValue('description')
    await dialog
      .findAll('button')
      .find((b) => b.text() === '保存')!
      .trigger('click')
    await flushPromises()
    expect(writes).toEqual([{ name: 'my_predicate', predicate: 'a&!b', description: 'description' }])
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
  })
})
