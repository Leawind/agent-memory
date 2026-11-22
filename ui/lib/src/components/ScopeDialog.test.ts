// ScopeDialog behavior test: create/edit flows, the immutable name in edit mode, the required
// switch, and the request shapes (fetch is stubbed; Element Plus renders for real; dialogs
// teleport to body)
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { DOMWrapper, mount, flushPromises } from '@vue/test-utils'
import ScopeDialog from './ScopeDialog.vue'
import type { ScopeView } from '../types'

const scope: ScopeView = { name: 'score', count: 4, tag_count: 2, description: '分值' }

function jsonResponse(body: unknown = null) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

/** The dialog teleports to document.body (append-to-body); query it there. setValue goes through
 * a DOMWrapper so Vue's v-model picks the change up. */
function dialogEl() {
  return document.querySelector('.am-form-dialog') as HTMLElement
}
function dialogTitle() {
  return dialogEl().querySelector('.am-dialog-title') as HTMLElement
}
function saveButton() {
  return dialogEl().querySelector('.am-dialog-head .el-button--primary') as HTMLButtonElement
}
function nameInput() {
  return dialogEl().querySelector('.el-form input') as HTMLInputElement
}
function descTextarea() {
  return dialogEl().querySelector('.el-form textarea') as HTMLTextAreaElement
}

async function openDialog(target: ScopeView | null) {
  const wrapper = mount(ScopeDialog, {
    props: { visible: false, scope: target },
    attachTo: document.body,
  })
  await wrapper.setProps({ visible: true })
  await flushPromises()
  return wrapper
}

describe('ScopeDialog', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse())),
    )
    document.body.innerHTML = ''
  })

  it('creating: titled 新建作用域, save unlocks once the name is present', async () => {
    const wrapper = await openDialog(null)
    expect(dialogTitle().textContent).toContain('新建作用域')
    expect(saveButton().disabled).toBe(true)
    await new DOMWrapper(nameInput()).setValue('system')
    expect(saveButton().disabled).toBe(false)
    await new DOMWrapper(descTextarea()).setValue('机器归属')
    expect(saveButton().disabled).toBe(false)
    wrapper.unmount()
  })

  it('creating with the required switch on POSTs the flag in the same call', async () => {
    const wrapper = await openDialog(null)
    await new DOMWrapper(nameInput()).setValue('system')
    await new DOMWrapper(descTextarea()).setValue('机器归属')
    await new DOMWrapper(dialogEl().querySelector('.el-switch') as HTMLElement).trigger('click')
    await saveButton().click()
    await flushPromises()
    await flushPromises()
    const calls = vi.mocked(globalThis.fetch).mock.calls
    const post = calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'POST')
    expect(String(post![0])).toBe('/api/scopes')
    expect(JSON.parse(String(post![1]?.body))).toEqual({
      name: 'system',
      description: '机器归属',
      required: true,
    })
    expect(wrapper.emitted('saved')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([false])
    wrapper.unmount()
  })

  it('creating with the switch off omits the flag entirely', async () => {
    const wrapper = await openDialog(null)
    await new DOMWrapper(nameInput()).setValue('system')
    await new DOMWrapper(descTextarea()).setValue('机器归属')
    await saveButton().click()
    await flushPromises()
    await flushPromises()
    const calls = vi.mocked(globalThis.fetch).mock.calls
    const post = calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'POST')
    expect(JSON.parse(String(post![1]?.body))).toEqual({ name: 'system', description: '机器归属' })
    wrapper.unmount()
  })

  it('editing: titled 作用域 #<name>, the name is locked, description and flag are editable', async () => {
    const wrapper = await openDialog(scope)
    expect(dialogTitle().textContent).toContain('作用域 #score')
    expect(nameInput().disabled).toBe(true)
    expect(descTextarea().value).toBe('分值')
    expect(saveButton().disabled).toBe(true)

    // Description alone: only the description half travels
    await new DOMWrapper(descTextarea()).setValue('优先级')
    expect(saveButton().disabled).toBe(false)
    await saveButton().click()
    await flushPromises()
    await flushPromises()
    const put = vi.mocked(globalThis.fetch).mock.calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'PUT')
    expect(String(put![0])).toBe('/api/scopes/score')
    expect(JSON.parse(String(put![1]?.body))).toEqual({ description: '优先级' })
    expect(wrapper.emitted('saved')).toHaveLength(1)
    wrapper.unmount()
  })

  it('editing: flipping the required switch sends the flag, and only the flag half changes', async () => {
    const wrapper = await openDialog({ ...scope, required: true })
    // The description is unchanged; the switch is the only difference (the description half
    // always travels, like the tag dialog's; the flag only when it flipped)
    await new DOMWrapper(dialogEl().querySelector('.el-switch') as HTMLElement).trigger('click')
    expect(saveButton().disabled).toBe(false)
    await saveButton().click()
    await flushPromises()
    await flushPromises()
    const put = vi.mocked(globalThis.fetch).mock.calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'PUT')
    expect(JSON.parse(String(put![1]?.body))).toEqual({ description: '分值', required: false })
    wrapper.unmount()
  })

  it('an illegal name is flagged under the field and keeps save disabled', async () => {
    const wrapper = await openDialog(null)
    await new DOMWrapper(nameInput()).setValue('a::b')
    expect(saveButton().disabled).toBe(true)
    expect(dialogEl().textContent).toContain('非法作用域名')
    // A scope name is one identifier word; fixing it clears the flag and unlocks save
    await new DOMWrapper(nameInput()).setValue('machines')
    expect(saveButton().disabled).toBe(false)
    expect(dialogEl().textContent).not.toContain('非法作用域名')
    wrapper.unmount()
  })
})
