// BackupCard: predicate-filtered export and merge import ride the query string of the two
// admin endpoints; the import toast names the skipped duplicates. Mounts the card directly.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../../index'
import BackupCard from './BackupCard.vue'

function jsonResponse(body: unknown) {
  return {
    ok: true,
    status: 200,
    text: () => Promise.resolve(JSON.stringify(body)),
    blob: () => Promise.resolve(new Blob([JSON.stringify(body)])),
  }
}

function mountCard() {
  return mount(BackupCard, { global: { plugins: [ElementPlus] } })
}

beforeEach(() => {
  setMemoryUILocale('zh')
  vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:x')
  vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {})
})

describe('BackupCard', () => {
  it('loads tag names for completion and exports the typed predicate as a query param', async () => {
    const fetchMock = vi.fn((url: string | URL, init?: RequestInit) =>
      jsonResponse({ tags: [{ name: 'proj' }, { name: 'one' }] }),
    )
    vi.stubGlobal('fetch', fetchMock)
    const wrapper = mountCard()
    await flushPromises()
    expect(fetchMock).toHaveBeenCalledWith(expect.stringContaining('/api/tags?all=true'), expect.anything())
    const exportRow = wrapper.findAll('.row')[0]!
    await exportRow.find('input').setValue('proj=one')
    await exportRow.find('button').trigger('click')
    await flushPromises()
    const call = fetchMock.mock.calls.find(([url]) => String(url).includes('/api/export'))
    expect(String(call![0])).toContain('/api/export?predicate=proj%3Done')
    // An empty box exports the full backup: no query string at all
    await exportRow.find('input').setValue('')
    await exportRow.find('button').trigger('click')
    await flushPromises()
    const plain = fetchMock.mock.calls.filter(([url]) => String(url).includes('/api/export'))
    expect(String(plain[plain.length - 1]![0])).not.toContain('?')
    wrapper.unmount()
  })

  it('imports the picked file with merge and predicate in the query and reports skips', async () => {
    const fetchMock = vi.fn((url: string | URL, init?: RequestInit) => {
      if (String(url).includes('/api/import')) {
        return jsonResponse({ imported_memories: 3, imported_tags: 1, skipped_memories: 2 })
      }
      return jsonResponse({ tags: [] })
    })
    vi.stubGlobal('fetch', fetchMock)
    const wrapper = mountCard()
    await flushPromises()
    const importRow = wrapper.findAll('.row')[1]!
    await importRow.find('input').setValue('proj=one')
    await importRow.find('input[type="checkbox"]').setValue(true)
    const file = new File(['{}'], 'backup.json', { type: 'application/json' })
    const input = importRow.find('input[type="file"]')
    Object.defineProperty(input.element, 'files', { value: [file] })
    await input.trigger('change')
    await flushPromises()
    const call = fetchMock.mock.calls.find(([url]) => String(url).includes('/api/import'))
    expect(String(call![0])).toContain('/api/import?merge=1&predicate=proj%3Done')
    expect(call![1]).toMatchObject({ method: 'POST', body: {} })
    expect(document.body.textContent).toContain('跳过 2 条重复')
    wrapper.unmount()
  })
})
