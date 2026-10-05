// TagDialog behavior test: title format, dirty-gated save, reserved-tag restrictions and the
// delete flow (fetch is stubbed; Element Plus renders for real; dialogs teleport to body)
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { DOMWrapper, mount, flushPromises } from '@vue/test-utils'
import { ElMessageBox } from 'element-plus'
import TagDialog from './TagDialog.vue'
import type { TagView } from '../types'

const tag: TagView = { name: 'rust', count: 3, description: '语言' }

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
function deleteButton() {
  return dialogEl().querySelector('.am-dialog-head .el-button--danger') as HTMLButtonElement
}
function nameInput() {
  return dialogEl().querySelector('.el-form input') as HTMLInputElement
}
function descTextarea() {
  return dialogEl().querySelector('.el-form textarea') as HTMLTextAreaElement
}

async function openDialog(tag: TagView | null) {
  const wrapper = mount(TagDialog, {
    props: { visible: false, tag },
    attachTo: document.body,
  })
  await wrapper.setProps({ visible: true })
  await flushPromises()
  return wrapper
}

describe('TagDialog', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse())),
    )
    document.body.innerHTML = ''
  })

  it('editing: titled 标签 #<name>, save stays disabled until something changed', async () => {
    const wrapper = await openDialog(tag)
    expect(dialogTitle().textContent).toContain('标签 #rust')
    expect(nameInput().value).toBe('rust')
    expect(saveButton().disabled).toBe(true)
    // The title anchors on the original name while the rename is being typed
    await new DOMWrapper(nameInput()).setValue('rust2')
    expect(dialogTitle().textContent).toContain('标签 #rust')
    expect(saveButton().disabled).toBe(false)
    wrapper.unmount()
  })

  it('saving without a rename PUTs the description only', async () => {
    const wrapper = await openDialog(tag)
    await new DOMWrapper(descTextarea()).setValue('Rust 后端')
    await saveButton().click()
    await flushPromises()
    await flushPromises()
    const calls = vi.mocked(globalThis.fetch).mock.calls
    const put = calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'PUT')
    expect(put).toBeTruthy()
    expect(String(put![0])).toBe('/api/tags/rust')
    expect(JSON.parse(String(put![1]?.body))).toEqual({ description: 'Rust 后端' })
    expect(wrapper.emitted('saved')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([false])
    wrapper.unmount()
  })

  it('reserved tags: name locked, no delete button, description still editable', async () => {
    const wrapper = await openDialog({ name: 'conventions', count: 2, description: '常驻约定', reserved: true })
    expect(nameInput().disabled).toBe(true)
    expect(deleteButton()).toBeNull()
    await new DOMWrapper(descTextarea()).setValue('改描述')
    expect(saveButton().disabled).toBe(false)
    wrapper.unmount()
  })

  it('creating: titled 新建标签, save unlocks once the name is present', async () => {
    const wrapper = await openDialog(null)
    expect(dialogTitle().textContent).toContain('新建标签')
    expect(deleteButton()).toBeNull()
    expect(saveButton().disabled).toBe(true)
    await new DOMWrapper(nameInput()).setValue('fresh')
    expect(saveButton().disabled).toBe(false)
    await saveButton().click()
    await flushPromises()
    await flushPromises()
    const calls = vi.mocked(globalThis.fetch).mock.calls
    const post = calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'POST')
    expect(String(post![0])).toBe('/api/tags')
    expect(JSON.parse(String(post![1]?.body))).toEqual({ name: 'fresh', description: '' })
    expect(wrapper.emitted('saved')).toHaveLength(1)
    wrapper.unmount()
  })

  it('delete: opens the mode choice, detach deletes with mode=detach', async () => {
    const wrapper = await openDialog(tag)
    await deleteButton().click()
    await flushPromises()
    // The nested delete-mode dialog appeared with both modes offered
    const radios = [...document.querySelectorAll('.el-radio')]
    expect(radios.some((el) => el.textContent?.includes('仅摘除引用'))).toBe(true)
    expect(radios.some((el) => el.textContent?.includes('连带删除记忆'))).toBe(true)
    // Submit the danger button in the nested dialog footer
    const submit = document.querySelector('.el-dialog__footer .el-button--danger') as HTMLButtonElement
    expect(submit).toBeTruthy()
    await submit.click()
    await flushPromises()
    await flushPromises()
    const calls = vi.mocked(globalThis.fetch).mock.calls
    const del = calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'DELETE')
    expect(String(del![0])).toBe('/api/tags/rust?mode=detach')
    expect(wrapper.emitted('deleted')).toHaveLength(1)
    wrapper.unmount()
  })

  it('purge mode demands the extra confirm before deleting', async () => {
    const confirmSpy = vi.spyOn(ElMessageBox, 'confirm').mockImplementation(() => Promise.resolve(true) as never)
    const wrapper = await openDialog(tag)
    await deleteButton().click()
    await flushPromises()
    // Switch to purge (the radio input drives the group), then submit
    const purgeRadio = [...document.querySelectorAll('.el-radio__original')].find((el: Element) => {
      const label = el.closest('.el-radio')
      return label?.textContent?.includes('连带删除记忆')
    }) as HTMLInputElement
    await purgeRadio.click()
    await flushPromises()
    const submit = document.querySelector('.el-dialog__footer .el-button--danger') as HTMLButtonElement
    await submit.click()
    await flushPromises()
    await flushPromises()
    expect(confirmSpy).toHaveBeenCalled()
    expect(wrapper.emitted('deleted')).toHaveLength(1)
    wrapper.unmount()
  })
})
