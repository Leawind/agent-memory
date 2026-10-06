// TagsSidebar mount test: vertical tag list with descriptions, click-to-filter select,
// hover-revealed edit affordance, active-row highlight
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TagsSidebar from './TagsSidebar.vue'

const tagsPayload = {
  tags: [
    { name: 'rust', count: 3, description: '语言' },
    // Undescribed tags omit description entirely (sparse agent-facing shape)
    { name: 'infra', count: 1 },
    { name: 'convention', count: 2, description: '常驻约定', reserved: true },
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
    expect(html).toContain('convention')
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

  it('emits edit from the row affordance without selecting', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const first = wrapper.findAll('.sb-item')[0]
    // The edit affordance swallows the click: it must not also act as a row (filter) click
    await first.find('.sb-edit').trigger('click')
    expect(wrapper.emitted('edit')![0]).toEqual([{ name: 'rust', count: 3, description: '语言' }])
    expect(wrapper.emitted('select')).toBeUndefined()
    wrapper.unmount()
  })

  it('marks the row whose tag equals the active expression', async () => {
    const wrapper = mount(TagsSidebar, { props: { active: ' rust ' } })
    await flushPromises()
    await flushPromises()
    const items = wrapper.findAll('.sb-item')
    expect(items[0].classes()).toContain('is-active')
    expect(items[1].classes()).not.toContain('is-active')

    await wrapper.setProps({ active: 'infra' })
    expect(items[0].classes()).not.toContain('is-active')
    // Sorted by count desc: rust (3), convention (2), infra (1)
    expect(items[2].classes()).toContain('is-active')
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
