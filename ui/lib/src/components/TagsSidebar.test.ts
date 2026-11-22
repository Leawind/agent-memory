// TagsSidebar mount test: bare tags as their own entries, scope sections with the required
// badge, values nested under their scope (collapsed by default, expandable, prefix stripped),
// click-to-filter expressions (a tag's name or a scope's ::* wildcard), hover-revealed edit
// affordances (tag dialog for tags, scope dialog for scopes), active-row highlight
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TagsSidebar from './TagsSidebar.vue'

const tagsPayload = {
  tags: [
    // Bare tags stand on their own; undescribed tags omit description entirely (sparse shape)
    { name: 'rust', count: 5, description: '语言' },
    { name: 'convention', count: 4, description: '常驻约定', reserved: true },
    { name: 'infra', count: 2 },
    // Scoped tags: the scope half rides along; they never stand alone in the sidebar
    { name: 'score::3', count: 3, scope: 'score' },
    { name: 'score::5', count: 1, scope: 'score' },
    { name: 'phase::design', count: 2, scope: 'phase' },
  ],
  scopes: [
    { name: 'score', count: 4, tag_count: 2, required: true, description: '分值' },
    { name: 'phase', count: 2, tag_count: 1 },
    { name: 'unused', count: 0, tag_count: 0 },
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

  it('renders bare tags on their own and scope sections with the required badge', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('rust')
    expect(html).toContain('语言')
    expect(html).toContain('infra')
    // Reserved badge is surfaced on the reserved tag
    expect(html).toContain('保留')
    // The required scope carries the badge; the plain scope does not
    const scoreHeader = wrapper.findAll('.sb-scope')[0]
    expect(scoreHeader.text()).toContain('score')
    expect(scoreHeader.text()).toContain('必填')
    expect(scoreHeader.find('.sb-desc').text()).toBe('分值')
    expect(wrapper.findAll('.sb-scope')[1].text()).not.toContain('必填')
    // Header count reflects the tag total (scoped tags included)
    expect(wrapper.find('.sb-count').text()).toBe('6')
    // Three bare rows + three scope headers; values start collapsed (no value row at all)
    expect(wrapper.findAll('.sb-item')).toHaveLength(6)
    expect(wrapper.findAll('.sb-value')).toHaveLength(0)
    wrapper.unmount()
  })

  it('nests a scope’s values under it, collapsed by default, prefix stripped', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    // Sorted by use: rust (5), convention (4), infra (2); scopes: score (4) then phase (2)
    const items = wrapper.findAll('.sb-item')
    expect(items[3].classes()).toContain('sb-scope')
    const scoreHeader = items[3]
    const twist = scoreHeader.find('.sb-twist')
    expect(twist.attributes('aria-expanded')).toBe('false')
    // The twist swallows the click: it must not also act as a row (filter) click
    await twist.trigger('click')
    expect(wrapper.emitted('select')).toBeUndefined()
    await flushPromises()
    expect(twist.attributes('aria-expanded')).toBe('true')

    const values = wrapper.findAll('.sb-value')
    expect(values).toHaveLength(2)
    // The scope prefix is dropped — nesting under the scope already says it; the full
    // `scope::value` name stays available as the row's hover title
    expect(values[0].text()).toContain('3')
    expect(values[1].text()).toContain('5')
    expect(values[1].attributes('title')).toBe('score::5')
    // A scope without any value keeps the empty twist placeholder and stays header-only
    const unusedHeader = wrapper.findAll('.sb-scope')[2]
    expect(unusedHeader.text()).toContain('unused')
    expect(unusedHeader.find('.sb-twist-empty').exists()).toBe(true)

    // Second click collapses again
    await scoreHeader.find('.sb-twist').trigger('click')
    expect(wrapper.findAll('.sb-value')).toHaveLength(0)
    wrapper.unmount()
  })

  it('emits select with the row’s expression: a tag name, a scope::value, or the scope wildcard', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const items = wrapper.findAll('.sb-item')

    // Bare row: the tag's own name is the expression
    await items[0].trigger('click') // rust
    expect(wrapper.emitted('select')![0]).toEqual(['rust'])

    // Scope header: the ::* wildcard
    await items[3].trigger('click') // score
    expect(wrapper.emitted('select')![1]).toEqual(['score::*'])

    // Value row: the exact scoped tag name
    await items[3].find('.sb-twist').trigger('click')
    await wrapper.findAll('.sb-value')[1].trigger('click') // score::5
    expect(wrapper.emitted('select')![2]).toEqual(['score::5'])
    wrapper.unmount()
  })

  it('edit affordances open the tag dialog for tags and the scope dialog for scopes', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await flushPromises()
    const items = wrapper.findAll('.sb-item')

    // The affordance swallows the click: it must not also act as a row (filter) click
    await items[0].find('.sb-edit').trigger('click') // rust
    expect(wrapper.emitted('edit')![0]).toEqual([{ name: 'rust', count: 5, description: '语言' }])
    expect(wrapper.emitted('select')).toBeUndefined()

    // A scope header's affordance carries the scope instead
    await items[3].find('.sb-edit').trigger('click') // score
    expect(wrapper.emitted('editScope')![0]).toEqual([
      { name: 'score', count: 4, tag_count: 2, required: true, description: '分值' },
    ])

    // A value row's affordance opens the tag dialog on the full scoped name
    await items[3].find('.sb-twist').trigger('click')
    const valueRow = wrapper.findAll('.sb-value')[0]
    expect(valueRow.find('.sb-edit').exists()).toBe(true)
    await valueRow.find('.sb-edit').trigger('click')
    expect(wrapper.emitted('edit')![1]).toEqual([{ name: 'score::3', count: 3, scope: 'score' }])
    wrapper.unmount()
  })

  it('marks the row whose expression equals the active expression', async () => {
    const wrapper = mount(TagsSidebar, { props: { active: ' rust ' } })
    await flushPromises()
    await flushPromises()
    const items = wrapper.findAll('.sb-item')
    expect(items[0].classes()).toContain('is-active') // rust
    expect(items[3].classes()).not.toContain('is-active')

    // A scope header goes active on its wildcard; its values only while expanded
    await wrapper.setProps({ active: 'score::*' })
    expect(wrapper.findAll('.sb-item')[3].classes()).toContain('is-active')
    await wrapper.setProps({ active: 'score::5' })
    await wrapper.findAll('.sb-item')[3].find('.sb-twist').trigger('click')
    const values = wrapper.findAll('.sb-value')
    expect(values[1].classes()).toContain('is-active')
    expect(values[0].classes()).not.toContain('is-active')
    wrapper.unmount()
  })

  it('emits create/create-scope from the header tools and predicates from the first', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    await wrapper.find('.sb-add').trigger('click')
    expect(wrapper.emitted('create')).toHaveLength(1)
    const tools = wrapper.findAll('.sb-tool')
    expect(tools).toHaveLength(3)
    await tools[0].trigger('click')
    expect(wrapper.emitted('predicates')).toHaveLength(1)
    await tools[1].trigger('click')
    expect(wrapper.emitted('createScope')).toHaveLength(1)
    wrapper.unmount()
  })

  it('has an all-memories reset row that hands the highlight back when a filter applies', async () => {
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
    wrapper.unmount()
  })

  it('refresh() refetches the tag list (no all switch anymore)', async () => {
    const wrapper = mount(TagsSidebar)
    await flushPromises()
    const fetchMock = vi.mocked(globalThis.fetch)
    fetchMock.mockClear()
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()
    const url = String(fetchMock.mock.calls[0]![0])
    expect(url.startsWith('/api/tags')).toBe(true)
    expect(url).not.toContain('all=')
    wrapper.unmount()
  })
})
