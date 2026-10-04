// TagsPanel mount smoke test: tag table renders from the API
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import TagsPanel from './TagsPanel.vue'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

describe('TagsPanel', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() =>
        Promise.resolve(
          jsonResponse({
            tags: [
              { name: 'rust', count: 3, description: '语言' },
              // Undescribed tags omit description entirely (sparse agent-facing shape)
              { name: 'infra', count: 0 },
            ],
          }),
        ),
      ),
    )
  })

  it('mounts and renders tag rows', async () => {
    const wrapper = mount(TagsPanel)
    await flushPromises()
    await flushPromises()
    const html = wrapper.html()
    expect(html).toContain('rust')
    expect(html).toContain('语言')
    expect(html).toContain('新建标签')
    wrapper.unmount()
  })

  it('refetches with the regex filter query param', async () => {
    const wrapper = mount(TagsPanel)
    await flushPromises()
    await flushPromises()
    const input = wrapper.find('.am-tag-filter input')
    expect(input.exists()).toBe(true)
    await input.setValue('^proj/')
    // After the 300ms debounce the request should be re-issued with the filter parameter
    await new Promise((r) => setTimeout(r, 350))
    await flushPromises()
    const calls = (fetch as ReturnType<typeof vi.fn>).mock.calls.map((c) => String(c[0]))
    expect(calls.some((u) => u.includes('/api/tags?filter=%5Eproj%2F'))).toBe(true)
    wrapper.unmount()
  })
})
