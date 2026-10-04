// Panel reusability props: show-header / title / subtitle
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import MemoriesPanel from './MemoriesPanel.vue'

const listPayload = {
  total: 1,
  offset: 0,
  limit: 20,
  memories: [{ id: 'm1', summary: '列表模式的记忆', tags: ['t1'], updated: '2026-10-04 08:00' }],
}
const tagsPayload = { tags: [] }

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('MemoriesPanel reusable props', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        const u = String(url)
        if (u.startsWith('/api/tags')) return Promise.resolve(jsonResponse(tagsPayload))
        return Promise.resolve(jsonResponse(listPayload))
      }),
    )
  })

  it('renders the default header by default', async () => {
    const wrapper = mount(MemoriesPanel)
    await flushPromises()
    expect(wrapper.find('.am-panel-header').exists()).toBe(true)
    expect(wrapper.find('.am-panel-title').text()).toBe('记忆')
    wrapper.unmount()
  })

  it('hides the header with show-header=false', async () => {
    const wrapper = mount(MemoriesPanel, { props: { showHeader: false } })
    await flushPromises()
    expect(wrapper.find('.am-panel-header').exists()).toBe(false)
    // The toolbar and table are unaffected
    expect(wrapper.find('.am-toolbar').exists()).toBe(true)
    expect(wrapper.html()).toContain('列表模式的记忆')
    wrapper.unmount()
  })

  it('overrides title text and subtitle tooltip content', async () => {
    const wrapper = mount(MemoriesPanel, {
      props: { title: '自定义标题', subtitle: '自定义副标题' },
    })
    await flushPromises()
    expect(wrapper.find('.am-panel-title').text()).toBe('自定义标题')
    // The subtitle is no longer rendered flat; it is the hover tooltip of the info icon next to the title
    const tooltip = wrapper.findComponent({ name: 'ElTooltip' })
    expect(tooltip.exists()).toBe(true)
    expect(tooltip.props('content')).toBe('自定义副标题')
    expect(wrapper.find('.am-info').exists()).toBe(true)
    wrapper.unmount()
  })
})
