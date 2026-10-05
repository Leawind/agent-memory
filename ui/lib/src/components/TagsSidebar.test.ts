// TagsSidebar mount test: vertical tag list with descriptions, select/create emissions
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TagsSidebar from './TagsSidebar.vue'

const tagsPayload = {
  tags: [
    { name: 'rust', count: 3, description: '语言' },
    // Undescribed tags omit description entirely (sparse agent-facing shape)
    { name: 'infra', count: 1 },
    { name: 'conventions', count: 2, description: '常驻约定', reserved: true },
  ],
}

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('TagsSidebar', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse(tagsPayload))),
    )
  })

  it('renders every tag with its description and memory count', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('rust')
    expect(html).toContain('语言')
    expect(html).toContain('infra')
    expect(html).toContain('conventions')
    // Reserved badge is surfaced on the reserved tag
    expect(html).toContain('保留')
    // Header count reflects the tag total
    expect(wrapper.find('.sb-count').text()).toBe('3')
    wrapper.unmount()
  })

  it('emits select with the clicked tag', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const items = wrapper.findAll('.sb-item')
    expect(items).toHaveLength(3)
    // Sorted by count desc: rust (3) first
    await items[0].trigger('click')
    expect(wrapper.emitted('select')![0]).toEqual([{ name: 'rust', count: 3, description: '语言' }])
    wrapper.unmount()
  })

  it('emits create from the header plus button', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await wrapper.find('.sb-add').trigger('click')
    expect(wrapper.emitted('create')).toHaveLength(1)
    wrapper.unmount()
  })

  it('refresh() refetches the tag list', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    fetchMock.mockClear()
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()
    expect(fetchMock.mock.calls.some((c) => String(c[0]).startsWith('/api/tags'))).toBe(true)
    wrapper.unmount()
  })
})
