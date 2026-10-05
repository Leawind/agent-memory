// Root element regression: the app shell and MemoryAdmin toggle the panels with v-show,
// and Vue cannot apply display:none to a fragment-root component — with ElConfigProvider
// as the template root every panel stayed visible at once. The top-level components must
// keep a single element root, with ElConfigProvider wrapping the content from inside.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import AdminPanel from './AdminPanel.vue'
import MemoriesPanel from './MemoriesPanel.vue'
import MemoryAdmin from './MemoryAdmin.vue'
import MemoryWorkspace from './MemoryWorkspace.vue'
import TagsSidebar from './TagsSidebar.vue'

beforeEach(() => {
  vi.stubGlobal(
    'fetch',
    vi.fn(() =>
      Promise.resolve({
        ok: true,
        status: 200,
        text: () => Promise.resolve(JSON.stringify({ total: 0, memories: [], tags: [] })),
      }),
    ),
  )
})

function expectElementRoot(wrapper: VueWrapper<any>, className: string): void {
  const el = wrapper.vm.$.subTree.el as HTMLElement | null
  expect(el, 'the component root must be a real element, not a fragment anchor').toBeTruthy()
  expect(el!.nodeType).toBe(1)
  expect(el!.classList.contains(className)).toBe(true)
}

describe('top-level components keep a single element root (host v-show must work)', () => {
  it('MemoriesPanel root is the am-panel div', () => {
    const wrapper = mount(MemoriesPanel)
    expectElementRoot(wrapper, 'am-panel')
    wrapper.unmount()
  })

  it('TagsSidebar root is a single element', () => {
    const wrapper = mount(TagsSidebar)
    expectElementRoot(wrapper, 'am-tags-sidebar')
    wrapper.unmount()
  })

  it('AdminPanel root is the am-panel div', () => {
    const wrapper = mount(AdminPanel)
    expectElementRoot(wrapper, 'am-panel')
    wrapper.unmount()
  })

  it('MemoryAdmin root is the memory-ui div', async () => {
    const wrapper = mount(MemoryAdmin)
    await flushPromises()
    expectElementRoot(wrapper, 'memory-ui')
    wrapper.unmount()
  })

  it('MemoryWorkspace root is the am-workspace div', async () => {
    const wrapper = mount(MemoryWorkspace)
    await flushPromises()
    expectElementRoot(wrapper, 'am-workspace')
    wrapper.unmount()
  })
})
