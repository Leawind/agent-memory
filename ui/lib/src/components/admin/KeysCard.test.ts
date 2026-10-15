// KeysCard tests: registry table rendering (values overflow behind a tooltip), the create /
// edit dialog gating, and the delete confirmation. Fetch is stubbed; Element Plus renders for real.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { ElMessageBox } from 'element-plus'
import KeysCard from './KeysCard.vue'
import type { KeyView } from '../../types'

function jsonResponse(body: unknown, status = 200) {
  return { ok: status < 400, status, text: () => Promise.resolve(JSON.stringify(body)) } as unknown as Response
}

const rows: KeyView[] = [
  { name: 'phase', type: 'enum', multi: true, description: '阶段（可多个）', values: ['design', 'review'] },
  {
    name: 'project',
    type: 'enum',
    multi: false,
    description: '所属项目',
    values: ['a1', 'a2', 'a3', 'a4', 'a5', 'a6', 'a7', 'a8'],
  },
  { name: 'score', type: 'number', multi: false, description: '' },
]

function table(wrapper: ReturnType<typeof mount>) {
  return wrapper.find('.keys-card .el-table')
}

describe('KeysCard', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse({}))),
    )
  })

  it('renders every key with type/multi tags and its known values', async () => {
    const wrapper = mount(KeysCard, { props: { keys: rows }, global: { plugins: [ElementPlus] } })
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('phase')
    expect(html).toContain('阶段（可多个）')
    expect(html).toContain('多值')
    expect(html).toContain('score')
    // The empty description renders as a dash, not blank
    expect(table(wrapper).text()).toContain('—')
    // Values show inline up to the cap, the rest behind a +N tooltip
    expect(table(wrapper).text()).toContain('a1')
    expect(table(wrapper).text()).toContain('+2')
    expect(table(wrapper).text()).not.toContain('a8')
    wrapper.unmount()
  })

  it('create: save unlocks with a name and PUTs the canonical key row', async () => {
    const puts: unknown[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((_url: string | URL, init?: RequestInit) => {
        if (init?.method === 'PUT') {
          puts.push(JSON.parse(String(init.body)))
          return Promise.resolve(jsonResponse({}))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(KeysCard, { props: { keys: rows }, global: { plugins: [ElementPlus] } })
    await flushPromises()
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '新建键')!
      .trigger('click')
    await flushPromises()
    const dialog = wrapper.find('.el-dialog')
    const save = dialog.findAll('button').find((b) => b.text() === '保存')!
    expect(save.attributes('disabled')).toBeDefined()
    await dialog.find('input').setValue('env')
    expect(save.attributes('disabled')).toBeUndefined()
    await save.trigger('click')
    await flushPromises()
    expect(puts).toEqual([{ name: 'env', key_type: 'enum', multi: false, description: '' }])
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('edit: the name is locked and save needs a real reconfiguration', async () => {
    const puts: unknown[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((_url: string | URL, init?: RequestInit) => {
        if (init?.method === 'PUT') {
          puts.push(JSON.parse(String(init.body)))
          return Promise.resolve(jsonResponse({}))
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mount(KeysCard, { props: { keys: rows }, global: { plugins: [ElementPlus] } })
    await flushPromises()
    await table(wrapper)
      .findAll('button')
      .find((b) => b.text() === '编辑')!
      .trigger('click')
    await flushPromises()
    const dialog = wrapper.find('.el-dialog')
    const save = dialog.findAll('button').find((b) => b.text() === '保存')!
    // Pristine re-open of the stored row: nothing to save
    expect(save.attributes('disabled')).toBeDefined()
    const nameInput = dialog.find('input')
    expect((nameInput.element as HTMLInputElement).disabled).toBe(true)
    // Toggling multi is a schema decision worth saving (phase starts multi-valued: off it goes)
    await dialog.find('.el-switch').trigger('click')
    expect(save.attributes('disabled')).toBeUndefined()
    await save.trigger('click')
    await flushPromises()
    expect(puts).toEqual([{ name: 'phase', key_type: 'enum', multi: false, description: '阶段（可多个）' }])
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('delete: confirms first, reports the detached value-tag count', async () => {
    const confirmSpy = vi.spyOn(ElMessageBox, 'confirm').mockImplementation(() => Promise.resolve(true) as never)
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse({ deleted: true, memories_updated: 2 }))),
    )
    const wrapper = mount(KeysCard, { props: { keys: rows }, global: { plugins: [ElementPlus] } })
    await flushPromises()
    await table(wrapper)
      .findAll('button')
      .find((b) => b.text() === '删除')!
      .trigger('click')
    await flushPromises()
    expect(confirmSpy).toHaveBeenCalled()
    const del = vi
      .mocked(globalThis.fetch)
      .mock.calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'DELETE')
    expect(String(del![0])).toContain('/api/keys/phase')
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })
})
