// MemoryWorkspace integration test: sidebar + memory list render side by side, a sidebar row
// click narrows the list to that tag (click again clears), the edit affordance opens the tag
// dialog (plus button creates), and the resizer adjusts the sidebar width within bounds.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import MemoryWorkspace from './MemoryWorkspace.vue'

const listPayload = {
  total: 1,
  offset: 0,
  limit: 20,
  memories: [{ id: 'm1', summary: '列表模式的记忆', tags: ['t1'], updated: '2026-10-04 08:00' }],
}
const tagsPayload = {
  tags: [{ name: 'rust', count: 3, description: '语言' }],
}

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

function mockFetch(url: string | URL) {
  const u = String(url)
  if (u.startsWith('/api/tags')) return jsonResponse(tagsPayload)
  return jsonResponse(listPayload)
}

describe('MemoryWorkspace', () => {
  beforeEach(() => {
    localStorage.clear()
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => Promise.resolve(mockFetch(url))),
    )
  })

  it('renders the tag sidebar and the memory list side by side', async () => {
    const wrapper = mount(MemoryWorkspace)
    await flushPromises()
    await flushPromises()
    expect(wrapper.find('.am-tags-sidebar').exists()).toBe(true)
    expect(wrapper.find('.amws-resizer').exists()).toBe(true)
    expect(wrapper.find('.am-memory-card').exists()).toBe(true)
    expect(wrapper.find('.am-tags-sidebar').text()).toContain('rust')
    wrapper.unmount()
  })

  it('filters the list on a row click (again to clear); the edit affordance opens the dialog', async () => {
    const wrapper = mount(MemoryWorkspace)
    await flushPromises()
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)

    // Row click: the expression becomes the tag, the list refetches with it, the row highlights
    await wrapper.find('.sb-item').trigger('click')
    await flushPromises()
    await flushPromises()
    expect(fetchMock.mock.calls.some((c) => String(c[0]).includes(`tag_expr=${encodeURIComponent('rust')}`))).toBe(true)
    expect((wrapper.find('.tag-filter input').element as HTMLInputElement).value).toBe('rust')
    expect(wrapper.find('.sb-item.is-active').exists()).toBe(true)

    // Clicking the active tag clears the filter
    fetchMock.mockClear()
    await wrapper.find('.sb-item').trigger('click')
    await flushPromises()
    await flushPromises()
    const urls = fetchMock.mock.calls.map((c) => String(c[0]))
    expect(urls.some((u) => u.startsWith('/api/memories'))).toBe(true)
    expect(urls.some((u) => u.includes('tag_expr='))).toBe(false)
    expect((wrapper.find('.tag-filter input').element as HTMLInputElement).value).toBe('')
    expect(wrapper.find('.sb-item.is-active').exists()).toBe(false)

    // The hover-revealed edit affordance is what opens the tag dialog now
    await wrapper.findAll('.sb-item')[0].find('.sb-edit').trigger('click')
    await flushPromises()
    const tagDialog = wrapper.findComponent({ name: 'TagDialog' })
    // The dialog received the full tag for its 标签 #<name> title and dirty baseline
    expect(tagDialog.props('tag')).toEqual({ name: 'rust', count: 3, description: '语言' })
    expect(tagDialog.findComponent({ name: 'ElDialog' }).props('modelValue')).toBe(true)

    // Close again, then open create mode from the sidebar plus button
    await tagDialog.vm.$emit('update:visible', false)
    await flushPromises()
    await wrapper.find('.sb-add').trigger('click')
    await flushPromises()
    expect((wrapper.findComponent({ name: 'TagDialog' }).props('tag') as unknown) === null).toBe(true)
    expect(wrapper.findComponent({ name: 'TagDialog' }).findComponent({ name: 'ElDialog' }).props('modelValue')).toBe(
      true,
    )
    wrapper.unmount()
  })

  it('tag changes refresh the sidebar and the memory list', async () => {
    const wrapper = mount(MemoryWorkspace)
    await flushPromises()
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    fetchMock.mockClear()
    await wrapper.findAll('.sb-item')[0].find('.sb-edit').trigger('click')
    await flushPromises()
    // Save succeeds inside the dialog: the workspace must reload both sides afterwards
    const tagDialog = wrapper.findComponent({ name: 'TagDialog' })
    tagDialog.vm.$emit('saved')
    await flushPromises()
    const urls = fetchMock.mock.calls.map((c) => String(c[0]))
    expect(urls.some((u) => u.startsWith('/api/tags'))).toBe(true)
    expect(urls.some((u) => u.startsWith('/api/memories'))).toBe(true)
    wrapper.unmount()
  })

  it('dragging the resizer adjusts the sidebar width and clamps it', async () => {
    const wrapper = mount(MemoryWorkspace)
    await flushPromises()
    const resizer = wrapper.find('.amws-resizer')
    // happy-dom has no pointer capture implementation; the handlers only need the element ref
    const el = resizer.element as HTMLElement
    el.setPointerCapture = () => {}
    el.releasePointerCapture = () => {}
    await resizer.trigger('pointerdown', { clientX: 100, pointerId: 1 })
    await resizer.trigger('pointermove', { clientX: 160, pointerId: 1 })
    const aside = wrapper.find('.amws-aside').element as HTMLElement
    expect(aside.style.width).toBe('320px')
    // Beyond the max the width clamps (260 + 300 -> 420)
    await resizer.trigger('pointermove', { clientX: 400, pointerId: 1 })
    expect(aside.style.width).toBe('420px')
    await resizer.trigger('pointerup', { clientX: 400, pointerId: 1 })
    expect(localStorage.getItem('agent-memory-sidebar-width')).toBe('420')
    wrapper.unmount()
  })
})
