// @vitest-environment jsdom
// The editor defaults to rendered preview for existing memories (MarkdownView → DOMPurify),
// which needs its officially supported test DOM (happy-dom mis-strips block tags).
// Behavior tests for the dialog: open sequence, the dirty-gated save button in the title row,
// the two tag containers (bare chips + scope values) and the delete/merge flows (fetch is
// stubbed; Element Plus renders for real).
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { DOMWrapper, mount, flushPromises } from '@vue/test-utils'
import { ElMessageBox } from 'element-plus'
import MemoryEditorDialog from './MemoryEditorDialog.vue'
import type { MemoryDraft, ScopeColumn } from '../composables/useMemories'

const fullMemory = {
  id: 'm7',
  summary: '已有记忆摘要',
  content: '正文内容',
  tags: ['t1', 'project::demo', 'score::3'],
  created: '2026-10-04 08:00',
  updated: '2026-10-04 08:00',
  attributes: { project: 'demo', score: 3 },
}

const scopeOptions: ScopeColumn[] = [
  { name: 'project', required: false, values: ['demo', 'ui'] },
  { name: 'score', required: true, values: ['3', '5'] },
]

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

async function openDialog(memoryId: string | null, extraProps: Record<string, unknown> = {}) {
  const wrapper = mount(MemoryEditorDialog, {
    props: {
      visible: false,
      memoryId,
      tagOptions: ['t1'],
      scopeOptions,
      requiredScopes: ['score'],
      ...extraProps,
    },
    attachTo: document.body,
  })
  await wrapper.setProps({ visible: true })
  await flushPromises()
  return wrapper
}

/** Route fetch by URL substring + method, recording write bodies */
function routeFetch(routes: Array<{ match: string; method?: string; body: unknown }>) {
  const writes: Array<Record<string, unknown>> = []
  vi.stubGlobal(
    'fetch',
    vi.fn((input: RequestInfo | URL, init?: RequestInit) => {
      const url = String(input)
      const method = String(init?.method ?? 'GET').toUpperCase()
      for (const r of routes) {
        if (url.includes(r.match) && (r.method === undefined || method === r.method)) {
          if (init?.body) writes.push(JSON.parse(String(init.body)))
          return Promise.resolve(jsonResponse(r.body))
        }
      }
      return Promise.resolve(jsonResponse({}))
    }),
  )
  return writes
}

describe('MemoryEditorDialog', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse(fullMemory))),
    )
    document.body.innerHTML = ''
  })

  it('editing: removing a bare tag ships remove_tags by name, nothing else travels', async () => {
    const writes = routeFetch([
      { match: '/api/memories/m7', method: 'PUT', body: { updated: true } },
      { match: '/api/memories/', body: fullMemory },
    ])
    const wrapper = await openDialog('m7')
    // The chips select carries bare names only; scoped names live in the scope rows below
    const select = dialogEl().querySelector('.el-select')!
    expect(select.textContent).toContain('t1')
    expect(select.textContent).not.toContain('project::demo')
    await new DOMWrapper(select.querySelector('.el-tag__close')!).trigger('click')
    await new DOMWrapper(saveButton()).trigger('click')
    await flushPromises()
    expect(writes[0].remove_tags).toEqual(['t1'])
    expect(writes[0].attributes).toBeUndefined()
    wrapper.unmount()
  })

  it('editing scope values: a changed value travels through attributes, scoped to its key', async () => {
    const writes = routeFetch([
      { match: '/api/memories/m7', method: 'PUT', body: { updated: true } },
      { match: '/api/memories/', body: fullMemory },
    ])
    const wrapper = await openDialog('m7')
    const vm = wrapper.vm as unknown as { setAttr: (scope: string, value: string) => void }
    vm.setAttr('score', '5')
    await flushPromises()
    expect(saveButton().disabled).toBe(false)
    await new DOMWrapper(saveButton()).trigger('click')
    await flushPromises()
    // Only the changed scope travels: the upsert carries the new value, and no remove is
    // emitted (the scope itself stays)
    expect(writes[0].attributes).toEqual({ score: '5' })
    expect(writes[0].tags).toBeUndefined()
    expect(writes[0].remove_tags).toBeUndefined()
    wrapper.unmount()
  })

  it('editing: clearing an optional scope removes the composed tag by full name', async () => {
    const writes = routeFetch([
      { match: '/api/memories/m7', method: 'PUT', body: { updated: true } },
      { match: '/api/memories/', body: fullMemory },
    ])
    const wrapper = await openDialog('m7')
    const vm = wrapper.vm as unknown as { setAttr: (scope: string, value: string) => void }
    vm.setAttr('project', '') // cleared: the declaration itself goes away
    await flushPromises()
    expect(saveButton().disabled).toBe(false)
    await new DOMWrapper(saveButton()).trigger('click')
    await flushPromises()
    expect(writes[0].remove_tags).toEqual(['project::demo'])
    expect(writes[0].attributes).toBeUndefined()
    wrapper.unmount()
  })

  it('editing: dropping a required scope’s value submits the null declaration, not a removal', async () => {
    const writes = routeFetch([
      { match: '/api/memories/m7', method: 'PUT', body: { updated: true } },
      { match: '/api/memories/', body: fullMemory },
    ])
    const wrapper = await openDialog('m7')
    const vm = wrapper.vm as unknown as { form: MemoryDraft }
    vm.form.attributes.score = null // the explicit "takes no tag in this scope"
    await flushPromises()
    expect(saveButton().disabled).toBe(false)
    await new DOMWrapper(saveButton()).trigger('click')
    await flushPromises()
    expect(writes[0].attributes).toEqual({ score: null })
    expect(writes[0].remove_tags).toBeUndefined()
    wrapper.unmount()
  })

  it('the scope rows render the loaded values and the required flag', async () => {
    const wrapper = await openDialog('m7')
    const rows = [...dialogEl().querySelectorAll('.scope-row')]
    expect(rows).toHaveLength(2)
    expect(rows[0].textContent).toContain('project')
    expect(rows[1].textContent).toContain('score')
    // The required scope carries the badge and the no-value option
    expect(rows[1].textContent).toContain('必填')
    const selects = [
      ...dialogEl().querySelectorAll('.scope-select .el-select__wrapper, .scope-select .el-input__inner'),
    ]
    expect(selects.length).toBeGreaterThanOrEqual(2)
    wrapper.unmount()
  })

  it('adding a typed token clears the select input, so the next keystroke starts fresh', async () => {
    // Regression: the add-token entry is a regular option (not allow-create), so element-plus's
    // default reserve-keyword left the landed token in the input where the next keystroke
    // concatenated onto it. reserve-keyword=false makes every selection clear the query.
    const wrapper = await openDialog(null)
    const input = () => dialogEl().querySelector('.el-select__input') as HTMLInputElement
    await new DOMWrapper(input()).setValue('t2')
    await flushPromises()
    const option = [...document.querySelectorAll('.el-select-dropdown__item')].find((o) =>
      o.textContent?.includes('t2'),
    )
    expect(option, 'the add-token entry should be offered for a new token').toBeTruthy()
    await new DOMWrapper(option!).trigger('click')
    await flushPromises()
    expect(input().value).toBe('')
    expect(dialogEl().querySelector('.el-select')!.textContent).toContain('t2')
    wrapper.unmount()
  })

  it('tag entry: the dropdown offers the add action for legal tokens, a red refusal for illegal ones', async () => {
    const wrapper = await openDialog('m7')
    const vm = wrapper.vm as unknown as {
      onTagQuery: (q: string) => void
      tagQueryEntry: { token: string; label: string; problem: string } | null
    }
    // A legal, unknown bare token offers the add action
    vm.onTagQuery('fresh-tag')
    await flushPromises()
    expect(vm.tagQueryEntry?.problem).toBe('')
    expect(vm.tagQueryEntry?.label).toContain('fresh-tag')
    // An illegal name is refused outright — the chip can never land, so the explanation
    // carries the offending token
    vm.onTagQuery('am-debug)invalid')
    await flushPromises()
    expect(vm.tagQueryEntry?.problem).toBe('name')
    expect(vm.tagQueryEntry?.label).toContain('am-debug)invalid')
    // A scoped name does not belong in the bare chips — the scope rows own it
    vm.onTagQuery('score::9')
    await flushPromises()
    expect(vm.tagQueryEntry?.problem).toBe('scoped')
    // So does the exact scoped tag the memory already carries
    vm.onTagQuery('project::demo')
    await flushPromises()
    expect(vm.tagQueryEntry?.problem).toBe('scoped')
    // A duplicate bare chip is refused with the toggle hint
    vm.onTagQuery('t1')
    await flushPromises()
    expect(vm.tagQueryEntry).toBeNull() // t1 is in the vocabulary — the real option serves
    wrapper.unmount()
  })

  it('editing: titled 记忆 #<id>, locked while fetching, save stays disabled until edited', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse(fullMemory))),
    )
    const wrapper = mount(MemoryEditorDialog, {
      props: { visible: false, memoryId: 'm7', tagOptions: ['t1'], scopeOptions, requiredScopes: ['score'] },
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

  it('creating: a required scope must be declared with a value before save unlocks', async () => {
    const writes = routeFetch([
      { match: '/api/memories', method: 'POST', body: { id: 'm9', updated: '2026-10-04 08:00' } },
    ])
    const wrapper = await openDialog(null, { tagOptions: [], requiredScopes: ['score'] })
    await new DOMWrapper(summaryInput()).setValue('scoped fact')
    // A summary alone is not enough: the store requires a declaration
    expect(saveButton().disabled).toBe(true)
    expect(dialogEl().querySelector('.required-hint')?.textContent).toContain('score')
    // The declaration is a value on the scope
    const vm = wrapper.vm as unknown as { form: MemoryDraft }
    vm.form.attributes.score = '3'
    await flushPromises()
    expect(saveButton().disabled).toBe(false)
    expect(dialogEl().querySelector('.required-hint')).toBeNull()
    await new DOMWrapper(saveButton()).trigger('click')
    await flushPromises()
    expect(writes[0].attributes).toEqual({ score: '3' })
    expect(writes[0].tags).toBeUndefined()
    wrapper.unmount()
  })

  it('creating: declaring a required scope as none submits the null declaration', async () => {
    const writes = routeFetch([
      { match: '/api/memories', method: 'POST', body: { id: 'm9', updated: '2026-10-04 08:00' } },
    ])
    const wrapper = await openDialog(null, { tagOptions: [], requiredScopes: ['score'] })
    await new DOMWrapper(summaryInput()).setValue('scoped fact')
    expect(saveButton().disabled).toBe(true)
    const vm = wrapper.vm as unknown as { form: MemoryDraft }
    vm.form.attributes.score = null
    await flushPromises()
    expect(saveButton().disabled).toBe(false)
    await new DOMWrapper(saveButton()).trigger('click')
    await flushPromises()
    expect(writes[0].attributes).toEqual({ score: null })
    wrapper.unmount()
  })

  it('creating: titled 新建记忆, a summary alone unlocks save (content is optional)', async () => {
    const wrapper = await openDialog(null, { requiredScopes: undefined, scopeOptions: [] })
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
    // The memory detail is what fails here (the tag-registry probe failing is non-fatal by design)
    vi.mocked(globalThis.fetch).mockImplementation((input: RequestInfo | URL) => {
      const u = String(input)
      if (u.includes('/api/memories/')) return Promise.resolve(jsonResponse({ error: 'not found' }, 404))
      return Promise.resolve(jsonResponse(fullMemory))
    })
    const wrapper = mount(MemoryEditorDialog, {
      props: { visible: false, memoryId: 'm7', tagOptions: ['t1'], scopeOptions, requiredScopes: ['score'] },
      attachTo: document.body,
    })
    await wrapper.setProps({ visible: true })
    await flushPromises()
    expect(wrapper.emitted('update:visible')?.at(-1)).toEqual([false])
    expect(saveButton().disabled).toBe(true)
    wrapper.unmount()
  })

  it('the title-row delete button confirms and emits deleted', async () => {
    routeFetch([
      { match: '/api/memories/m7', method: 'GET', body: fullMemory },
      { match: '/api/memories/m7', method: 'DELETE', body: null },
    ])
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
    const wrapper = await openDialog(null, { scopeOptions: [], requiredScopes: [] })
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
    const wrapper = await openDialog(null, { scopeOptions: [], requiredScopes: [] })
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
    routeFetch([
      { match: '/api/memories/m7', method: 'GET', body: fullMemory },
      { match: '/api/memories/merge', method: 'POST', body: {} },
    ])
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
