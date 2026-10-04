// @vitest-environment jsdom
// The editor defaults to rendered preview for existing memories (MarkdownView → DOMPurify),
// which needs its officially supported test DOM (happy-dom mis-strips block tags).
// Behavior tests for the dialog's open sequence: optimistic reset, the loading guard on save,
// and the edit/preview mode defaults (fetch is stubbed; Element Plus renders for real).
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import MemoryEditorDialog from './MemoryEditorDialog.vue'

const fullMemory = {
  id: 'm7',
  summary: '已有记忆摘要',
  content: '正文内容',
  tags: ['t1'],
  created_at: 1,
  updated_at: 2,
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
function saveButton() {
  return dialogEl().querySelector('.el-button--primary') as HTMLButtonElement
}
function summaryInput() {
  return dialogEl().querySelector('.el-form input') as HTMLInputElement
}

async function openDialog(memoryId: string | null) {
  const wrapper = mount(MemoryEditorDialog, {
    props: { visible: false, memoryId, tagOptions: ['t1'] },
    attachTo: document.body,
  })
  await wrapper.setProps({ visible: true })
  return wrapper
}

describe('MemoryEditorDialog open sequence', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse(fullMemory))),
    )
    document.body.innerHTML = ''
  })

  it('editing: shows a blank savable-locked form while fetching, then fills it', async () => {
    const wrapper = await openDialog('m7')
    // In flight: the optimistic reset must be visible (never the previous memory's data) and
    // saving must be locked, or a fast click would overwrite the memory with an empty form
    expect(summaryInput().value).toBe('')
    expect(saveButton().disabled).toBe(true)
    await flushPromises()
    // Loaded: fields filled, saving unlocked
    expect(summaryInput().value).toBe('已有记忆摘要')
    expect(saveButton().disabled).toBe(false)
    // Existing memories open rendered (no source textarea), per the progressive-preview default
    expect(dialogEl().querySelector('textarea')).toBeNull()
    wrapper.unmount()
  })

  it('creating: starts in the source editor and savable immediately, fetching nothing', async () => {
    const wrapper = await openDialog(null)
    await flushPromises()
    expect(saveButton().disabled).toBe(false)
    expect(dialogEl().querySelector('textarea')).not.toBeNull()
    const fetchMock = vi.mocked(globalThis.fetch)
    expect(fetchMock.mock.calls.filter((c) => String(c[0]).includes('/api/memories/'))).toHaveLength(0)
    wrapper.unmount()
  })

  it('a failed fetch closes the dialog and never leaves a savable empty form', async () => {
    vi.mocked(globalThis.fetch).mockImplementationOnce(() => Promise.resolve(jsonResponse({ error: 'not found' }, 404)))
    const wrapper = await openDialog('m7')
    await flushPromises()
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([false])
    expect(saveButton().disabled).toBe(true)
    wrapper.unmount()
  })
})
