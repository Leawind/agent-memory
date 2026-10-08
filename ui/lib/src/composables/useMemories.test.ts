import { afterEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { useMemories, type MemoriesStore } from './useMemories'

const cursor = `s_${'a'.repeat(32)}`
let wrapper: VueWrapper | undefined
function response(status: number, body: unknown) {
  return { ok: status >= 200 && status < 300, status, text: () => Promise.resolve(JSON.stringify(body)) }
}
function result(token = cursor) {
  return {
    cursor: token,
    total_matches: 3,
    results: [{ id: 'm1', summary: 'alpha', tags: [], updated: 'now', score: 10, snippet: 'excerpt' }],
  }
}

async function harness(fetcher: (url: string) => unknown): Promise<MemoriesStore> {
  localStorage.clear()
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string) => {
      if (!String(url).includes('query=')) return Promise.resolve(response(200, {}))
      return Promise.resolve(fetcher(String(url)))
    }),
  )
  let store!: MemoriesStore
  wrapper = mount(
    defineComponent({
      setup() {
        store = useMemories()
        return () => h('div')
      },
    }),
  )
  await flushPromises()
  store.query.value = 'alpha'
  await store.onSearch()
  return store
}
afterEach(() => {
  wrapper?.unmount()
  vi.unstubAllGlobals()
})

describe('search snapshot pagination', () => {
  it('reuses cursors for pages and refreshes on a new intent or explicit reload', async () => {
    const urls: string[] = []
    const store = await harness((url) => {
      urls.push(url)
      return response(200, result())
    })
    store.page.value = 2
    await store.reloadPage()
    expect(urls.at(-1)).toContain(`cursor=${cursor}`)
    expect(urls.at(-1)).toContain('offset=20')
    await store.reload()
    expect(urls.at(-1)).not.toContain('cursor=')
    store.mode.value = 'keyword'
    await store.onSearch()
    expect(urls.at(-1)).not.toContain('cursor=')
    expect(store.page.value).toBe(1)
  })

  it('restarts at page one exactly once for the structured stale-cursor error', async () => {
    const urls: string[] = []
    const store = await harness((url) => {
      urls.push(url)
      return url.includes('cursor=')
        ? response(400, { error: 'Ranking changed', code: 'stale_search_cursor' })
        : response(200, result())
    })
    store.page.value = 2
    await store.reloadPage()
    expect(urls).toHaveLength(3)
    expect(urls[1]).toContain('cursor=')
    expect(urls[2]).not.toContain('cursor=')
    expect(urls[2]).toContain('offset=0')
    expect(store.page.value).toBe(1)
    expect(store.loading.value).toBe(false)
  })

  it('does not retry permission failures even if a response carries the stale code', async () => {
    const urls: string[] = []
    const store = await harness((url) => {
      urls.push(url)
      return url.includes('cursor=')
        ? response(403, { error: 'Forbidden', code: 'stale_search_cursor' })
        : response(200, result())
    })
    store.page.value = 2
    await expect(store.reloadPage()).rejects.toMatchObject({ status: 403, name: 'ApiError' })
    expect(urls).toHaveLength(2)
    expect(store.loading.value).toBe(false)
  })

  it('discarding a late stale response cannot reset the newer query', async () => {
    let complete!: (value: unknown) => void
    const store = await harness((url) =>
      url.includes('cursor=')
        ? new Promise((resolve) => {
            complete = resolve
          })
        : response(200, result(url.includes('beta') ? 'beta-cursor' : cursor)),
    )
    store.page.value = 2
    const oldPage = store.reloadPage()
    store.query.value = 'beta'
    await store.onSearch()
    complete(response(400, { error: 'Old ranking', code: 'stale_search_cursor' }))
    await oldPage
    expect(store.query.value).toBe('beta')
    store.page.value = 2
    // A fresh request must use the newer cursor, not the old cursor or a fallback.
    const request = store.reloadPage()
    expect(String(vi.mocked(globalThis.fetch).mock.calls.at(-1)?.[0])).toContain('cursor=beta-cursor')
    complete(response(200, result('beta-cursor')))
    await request
  })
})
