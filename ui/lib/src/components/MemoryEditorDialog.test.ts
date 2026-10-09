// @vitest-environment jsdom
// The editor defaults to rendered preview for existing memories (MarkdownView → DOMPurify),
// which needs its officially supported test DOM (happy-dom mis-strips block tags).
// Behavior tests for the dialog: open sequence, the dirty-gated save button in the title row,
// and the delete flow (fetch is stubbed; Element Plus renders for real).
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { DOMWrapper, mount, flushPromises } from '@vue/test-utils'
import { ElMessageBox } from 'element-plus'
import MemoryEditorDialog from './MemoryEditorDialog.vue'

const fullMemory = {
  id: 'm7',
  summary: '已有记忆摘要',
  content: '正文内容',
  tags: ['t1'],
  original_tags: ['t1'],
  derived_tags: [],
  lifecycle: { expires_at: null, archived_at: null, pinned: false, state: 'active' },
  created: '2026-10-04 08:00',
  updated: '2026-10-04 08:00',
}

function jsonResponse(body: unknown, status = 200) {
  // The client only reads ok/status/text; cast the minimal stub to the full Response type once, here
  return {
    ok: status < 400,
    status,
    text: () => Promise.resolve(JSON.stringify(body)),
  } as unknown as Response
}

/** The dialog teleports to document.body (append-to-body); query it there. */
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
function summaryInput() {
  return dialogEl().querySelector('.el-dialog__body input') as HTMLInputElement
}

async function openDialog(memoryId: string | null) {
  const wrapper = mount(MemoryEditorDialog, {
    props: { visible: false, memoryId, tagOptions: ['t1'] },
    attachTo: document.body,
  })
  await wrapper.setProps({ visible: true })
  await flushPromises()
  return wrapper
}

describe('MemoryEditorDialog', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse(fullMemory))),
    )
    document.body.innerHTML = ''
  })

  it('edits only original sources and shows derived tags separately', async () => {
    const writes: Array<{ remove_tags: string[]; add_tags: string[] }> = []
    vi.stubGlobal(
      'fetch',
      vi.fn((_url: string | URL, init?: RequestInit) => {
        if (init?.method === 'PUT') {
          writes.push(JSON.parse(String(init.body)))
          return Promise.resolve(jsonResponse({ updated: true }))
        }
        return Promise.resolve(
          jsonResponse({
            ...fullMemory,
            tags: ['vue3', 'web'],
            original_tags: ['vue3'],
            derived_tags: [{ tag: 'web', rules: ['framework domain'] }],
          }),
        )
      }),
    )
    const wrapper = await openDialog('m7')
    const select = dialogEl().querySelector('.el-select')!
    expect(select.textContent).toContain('vue3')
    expect(select.textContent).not.toContain('web')
    expect(dialogEl().querySelector('.derived-tags')?.textContent).toContain('web')
    await new DOMWrapper(select.querySelector('.el-tag__close')!).trigger('click')
    await new DOMWrapper(saveButton()).trigger('click')
    await flushPromises()
    expect(writes[0].remove_tags).toEqual(['vue3'])
    expect(writes[0].add_tags).toEqual([])
    wrapper.unmount()
  })

  it('editing: titled 记忆 #<id>, locked while fetching, save stays disabled until edited', async () => {
    const wrapper = mount(MemoryEditorDialog, {
      props: { visible: false, memoryId: 'm7', tagOptions: ['t1'] },
      attachTo: document.body,
    })
    await wrapper.setProps({ visible: true })
    // In flight: the optimistic reset must be visible (never the previous memory's data) and
    // saving must be locked, or a fast click would overwrite the memory with an empty form
    expect(dialogTitle().textContent).toContain('记忆 #m7')
    expect(summaryInput().value).toBe('')
    expect(saveButton().disabled).toBe(true)
    await flushPromises()
    // Loaded: fields filled, but the form is pristine — per the workspace spec the save button
    // stays disabled until something actually changed
    expect(summaryInput().value).toBe('已有记忆摘要')
    expect(saveButton().disabled).toBe(true)
    // Existing memories open rendered (no source textarea), per the progressive-preview default
    expect(dialogEl().querySelector('textarea')).toBeNull()
    // Editing the summary unlocks save
    await new DOMWrapper(summaryInput()).setValue('改成新的摘要')
    expect(saveButton().disabled).toBe(false)
    wrapper.unmount()
  })

  it('creating: titled 新建记忆, a summary alone unlocks save (content is optional)', async () => {
    const wrapper = await openDialog(null)
    expect(dialogTitle().textContent).toContain('新建记忆')
    expect(deleteButton()).toBeNull()
    // Starts in the source editor and fetches nothing
    expect(dialogEl().querySelector('textarea')).not.toBeNull()
    const fetchMock = vi.mocked(globalThis.fetch)
    expect(fetchMock.mock.calls.filter((c) => String(c[0]).includes('/api/memories/'))).toHaveLength(0)
    // Empty form: not savable
    expect(saveButton().disabled).toBe(true)
    await new DOMWrapper(summaryInput()).setValue('新摘要')
    // Summary-only memory: content stays empty and save is already unlocked
    expect(saveButton().disabled).toBe(false)
    wrapper.unmount()
  })

  it('a failed fetch closes the dialog and never leaves a savable empty form', async () => {
    vi.mocked(globalThis.fetch).mockImplementationOnce(() => Promise.resolve(jsonResponse({ error: 'not found' }, 404)))
    const wrapper = mount(MemoryEditorDialog, {
      props: { visible: false, memoryId: 'm7', tagOptions: ['t1'] },
      attachTo: document.body,
    })
    await wrapper.setProps({ visible: true })
    await flushPromises()
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([false])
    expect(saveButton().disabled).toBe(true)
    wrapper.unmount()
  })

  it('the title-row delete button confirms and emits deleted', async () => {
    const confirmSpy = vi.spyOn(ElMessageBox, 'confirm').mockImplementation(() => Promise.resolve(true) as never)
    const wrapper = await openDialog('m7')
    expect(deleteButton()).toBeTruthy()
    await deleteButton().click()
    await flushPromises()
    await flushPromises()
    expect(confirmSpy).toHaveBeenCalled()
    const calls = vi.mocked(globalThis.fetch).mock.calls
    const deleteCall = calls.find((c) => String(c[1]?.method ?? '').toUpperCase() === 'DELETE')
    expect(deleteCall).toBeTruthy()
    expect(String(deleteCall![0])).toContain('/api/memories/m7')
    expect(wrapper.emitted('deleted')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([false])
    wrapper.unmount()
  })

  /** Route fetch by URL substring + method for create/merge flows */
  function routeFetch(routes: Array<{ match: string; method?: string; body: unknown }>) {
    vi.mocked(globalThis.fetch).mockImplementation((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input)
      const method = String(init?.method ?? 'GET').toUpperCase()
      for (const r of routes) {
        if (url.includes(r.match) && (r.method === undefined || method === r.method)) {
          return Promise.resolve(jsonResponse(r.body))
        }
      }
      return Promise.resolve(jsonResponse({}))
    })
  }

  it('creating: a similar_to hint offers the merge; accepting absorbs the new memory', async () => {
    routeFetch([
      {
        match: '/api/memories',
        method: 'POST',
        body: { id: 'm9', updated: 'x', similar_to: [{ id: 'm3', similarity: 0.92 }] },
      },
      { match: '/api/memories/merge', method: 'POST', body: {} },
    ])
    const confirmSpy = vi.spyOn(ElMessageBox, 'confirm').mockImplementation(() => Promise.resolve(true) as never)
    const wrapper = await openDialog(null)
    await new DOMWrapper(summaryInput()).setValue('新摘要')
    await saveButton().click()
    await flushPromises()
    await flushPromises()
    await flushPromises()
    expect(confirmSpy).toHaveBeenCalled()
    const mergeCall = vi.mocked(globalThis.fetch).mock.calls.find((c) => String(c[0]).includes('/api/memories/merge'))
    expect(mergeCall).toBeTruthy()
    expect(JSON.parse(String(mergeCall![1]?.body))).toEqual({ target: 'm3', source: 'm9' })
    // The list refresh carries the final state (the merged-away memory is gone)
    expect(wrapper.emitted('saved')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([false])
    wrapper.unmount()
  })

  it('creating: declining the merge hint keeps both memories', async () => {
    routeFetch([{ match: '/api/memories', method: 'POST', body: { id: 'm9', updated: 'x', duplicate_of: ['m3'] } }])
    const confirmSpy = vi.spyOn(ElMessageBox, 'confirm').mockImplementation(() => Promise.reject('cancel') as never)
    const wrapper = await openDialog(null)
    await new DOMWrapper(summaryInput()).setValue('新摘要')
    await saveButton().click()
    await flushPromises()
    await flushPromises()
    expect(confirmSpy).toHaveBeenCalled()
    expect(vi.mocked(globalThis.fetch).mock.calls.some((c) => String(c[0]).includes('/api/memories/merge'))).toBe(false)
    // Kept: the saved refresh still fires (the list must gain the new memory)
    expect(wrapper.emitted('saved')).toHaveLength(1)
    wrapper.unmount()
  })

  it('the title-row merge button prompts for a source id and merges into the edited memory', async () => {
    routeFetch([{ match: '/api/memories/merge', method: 'POST', body: {} }])
    const promptSpy = vi
      .spyOn(ElMessageBox, 'prompt')
      .mockImplementation(() => Promise.resolve({ value: 'm12' }) as never)
    const wrapper = await openDialog('m7')
    const mergeButton = dialogEl().querySelector('.am-dialog-head .el-button[aria-label*="合并"]') as HTMLButtonElement
    expect(mergeButton).toBeTruthy()
    await mergeButton.click()
    await flushPromises()
    await flushPromises()
    expect(promptSpy).toHaveBeenCalled()
    const mergeCall = vi.mocked(globalThis.fetch).mock.calls.find((c) => String(c[0]).includes('/api/memories/merge'))
    expect(mergeCall).toBeTruthy()
    expect(JSON.parse(String(mergeCall![1]?.body))).toEqual({ target: 'm7', source: 'm12' })
    expect(wrapper.emitted('saved')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([false])
    wrapper.unmount()
  })
})
