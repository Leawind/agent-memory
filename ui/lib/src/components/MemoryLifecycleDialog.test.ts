import { describe, expect, it, vi } from 'vitest'
import { DOMWrapper, mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../index'
import MemoryLifecycleDialog from './MemoryLifecycleDialog.vue'

describe('MemoryLifecycleDialog', () => {
  it('restores an archive while preserving its explicit expiry and content kind', async () => {
    setMemoryUILocale('zh')
    document.body.innerHTML = ''
    const calls: unknown[] = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL, init?: RequestInit) => {
        calls.push({ path: String(url), method: init?.method, body: JSON.parse(String(init?.body)) })
        return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve('{"updated":true}') })
      }),
    )
    const wrapper = mount(MemoryLifecycleDialog, {
      props: {
        visible: true,
        memoryId: 'm7',
        metadata: { kind: 'event', expires_at: 1700000000, archived_at: 1690000000, pinned: true, state: 'archived' },
      },
      global: { plugins: [ElementPlus] },
    })
    await flushPromises()
    const dialog = new DOMWrapper(document.querySelector('.el-dialog')!)
    await dialog.findAll('input[type=checkbox]')[1].setValue(false)
    await dialog
      .findAll('button')
      .find((button) => button.text() === '保存')!
      .trigger('click')
    await flushPromises()
    expect(calls).toEqual([
      {
        path: '/api/memories/m7/lifecycle',
        method: 'PUT',
        body: { kind: 'event', pinned: true, archived: false, expires_at: 1700000000 },
      },
    ])
    expect(wrapper.emitted('saved')).toHaveLength(1)
    wrapper.unmount()
  })
})
