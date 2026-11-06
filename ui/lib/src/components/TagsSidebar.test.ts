// TagsSidebar mount test: vertical tag list with descriptions, click-to-filter select,
// hover-revealed edit affordance, active-row highlight, keyed values nested under their
// key (collapsed by default, expandable, `key=value` filter expressions)
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TagsSidebar from './TagsSidebar.vue'

const tagsPayload = {
  tags: [
    { name: 'score', count: 6 },
    { name: 'rust', count: 5, description: '语言' },
    // Undescribed tags omit description entirely (sparse agent-facing shape)
    { name: 'convention', count: 4, description: '常驻约定', reserved: true },
    // Pointed at by score but also organizing on its own: top-level AND a value under score
    { name: 'nmmt', count: 3, keys: ['score'] },
    { name: 'infra', count: 2 },
    // Value-only tags: nest under score, never their own top-level entry
    { name: '5', count: 1, keys: ['score'], target_only: true },
    { name: '3', count: 1, keys: ['score'], target_only: true },
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

  it('renders every top-level tag with its description and memory count', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('score')
    expect(html).toContain('rust')
    expect(html).toContain('语言')
    expect(html).toContain('infra')
    expect(html).toContain('convention')
    // Reserved badge is surfaced on the reserved tag
    expect(html).toContain('保留')
    // Header count reflects the tag total (values included)
    expect(wrapper.find('.sb-count').text()).toBe('7')
    // Five top-level rows; value-only tags are not their own entries
    expect(wrapper.findAll('.sb-item')).toHaveLength(5)
    // Values start collapsed: no composed value row is visible yet
    expect(wrapper.text()).not.toContain('score:5')
    wrapper.unmount()
  })

  it('keeps value lists collapsed by default and expands on the twist', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    // Sorted by count desc: score (6) first; it carries three values (nmmt, 3, 5)
    const scoreRow = wrapper.findAll('.sb-item')[0]
    expect(scoreRow.text()).toContain('score')
    expect(wrapper.findAll('.sb-value')).toHaveLength(0)

    const twist = scoreRow.find('.sb-twist')
    expect(twist.exists()).toBe(true)
    expect(twist.attributes('aria-expanded')).toBe('false')
    // The twist swallows the click: it must not also act as a row (filter) click
    await twist.trigger('click')
    expect(wrapper.emitted('select')).toBeUndefined()

    await flushPromises()
    expect(twist.attributes('aria-expanded')).toBe('true')
    const values = wrapper.findAll('.sb-value')
    expect(values).toHaveLength(3)
    expect(values[0].text()).toContain('score:nmmt')
    expect(values[1].text()).toContain('score:3')
    expect(values[2].text()).toContain('score:5')
    // The dual-role tag keeps its own top-level row besides the nested one
    expect(wrapper.text()).toContain('nmmt')

    // Second click collapses again
    await scoreRow.find('.sb-twist').trigger('click')
    expect(wrapper.findAll('.sb-value')).toHaveLength(0)
    wrapper.unmount()
  })

  it('emits select with the clicked tag and its filter expression', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    // Top-level row: the tag's own name is the expression
    await wrapper.findAll('.sb-item')[1].trigger('click') // rust (5)
    expect(wrapper.emitted('select')![0]).toEqual([{ name: 'rust', count: 5, description: '语言' }, 'rust'])

    // Value row: the exact-edge atom `key=value`
    await wrapper.findAll('.sb-item')[0].find('.sb-twist').trigger('click')
    await wrapper.findAll('.sb-value')[2].trigger('click') // score:5
    expect(wrapper.emitted('select')![1]).toEqual([
      { name: '5', count: 1, keys: ['score'], target_only: true },
      'score=5',
    ])
    wrapper.unmount()
  })

  it('emits edit from the row affordance without selecting', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const first = wrapper.findAll('.sb-item')[0]
    // The edit affordance swallows the click: it must not also act as a row (filter) click
    await first.find('.sb-edit').trigger('click')
    expect(wrapper.emitted('edit')![0]).toEqual([{ name: 'score', count: 6 }])
    expect(wrapper.emitted('select')).toBeUndefined()
    wrapper.unmount()
  })

  it('marks the row whose expression equals the active expression', async () => {
    const wrapper = mount(TagsSidebar, { props: { active: ' rust ' } })
    await flushPromises()
    await flushPromises()
    const items = wrapper.findAll('.sb-item')
    expect(items[1].classes()).toContain('is-active') // rust
    expect(items[0].classes()).not.toContain('is-active')

    // A value's expression is `key=value`; the row only exists while expanded
    await wrapper.setProps({ active: 'score=5' })
    await wrapper.findAll('.sb-item')[0].find('.sb-twist').trigger('click')
    const values = wrapper.findAll('.sb-value')
    expect(values[2].classes()).toContain('is-active')
    expect(values[0].classes()).not.toContain('is-active')
    wrapper.unmount()
  })

  it('emits create from the header plus button', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await wrapper.find('.sb-add').trigger('click')
    expect(wrapper.emitted('create')).toHaveLength(1)
    wrapper.unmount()
  })

  it('has an all-memories reset row and quiet header tools', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const all = wrapper.find('.sb-all')
    expect(all.text()).toContain('全部记忆')
    // No expression applied yet: the reset row is the active entry
    expect(all.classes()).toContain('is-active')
    await all.trigger('click')
    expect(wrapper.emitted('clear')).toHaveLength(1)

    // An active tag turns the highlight over to that row
    await wrapper.setProps({ active: ' rust ' })
    expect(wrapper.find('.sb-all').classes()).not.toContain('is-active')
    expect(wrapper.find('.sb-item.is-active').exists()).toBe(true)

    // Two quiet tools: tag predicates, then create
    const tools = wrapper.findAll('.sb-tool')
    expect(tools).toHaveLength(2)
    await tools[0].trigger('click')
    expect(wrapper.emitted('predicates')).toHaveLength(1)
    wrapper.unmount()
  })

  it('refresh() refetches the tag list including edge-target-only values', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    fetchMock.mockClear()
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()
    const url = String(fetchMock.mock.calls[0]![0])
    expect(url.startsWith('/api/tags')).toBe(true)
    expect(url).toContain('all=true')
    wrapper.unmount()
  })
})
