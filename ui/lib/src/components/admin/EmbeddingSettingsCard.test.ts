// EmbeddingSettingsCard tests for the multi-candidate editor: one collapsed line per candidate,
// expansion into labeled fields, drag reordering, per-row and whole-list probes, canonical save
// payload, identity-change warning, and per-model cache management (backfill with a progress bar
// that can be interrupted, delete-with-confirm). Mounts the card directly; fetch is mocked per
// route.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { ElMessageBox } from 'element-plus'
import type { SortableEvent, SortableOptions } from 'sortablejs'
import { setMemoryUILocale } from '../../index'
import EmbeddingSettingsCard from './EmbeddingSettingsCard.vue'
import type { EmbedModelEntry, VectorCacheInfo } from '../../types'

// Reordering is delegated to SortableJS, which owns the DOM half of a drag; the tests drive the
// callback it fires on a drop, so the geometry that produces those two indices stays its business
const sortable = vi.hoisted(() => ({ created: [] as Array<{ el: Element; options: SortableOptions }> }))

vi.mock('sortablejs', () => ({
  default: {
    create: (el: Element, options: SortableOptions) => {
      const instance = { el, options, destroy: () => {} }
      sortable.created.push(instance)
      return instance
    },
  },
}))

/** Drop the row at `from` onto the position `to`, the way SortableJS reports it */
function drop(from: number, to: number): void {
  sortable.created[sortable.created.length - 1].options.onEnd!({ oldIndex: from, newIndex: to } as SortableEvent)
}

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

const MODEL: EmbedModelEntry = {
  id: 'local-model',
  name: 'bge-m3',
  enabled: true,
  base_url: 'http://svc:9/v1',
  model: 'bge-m3',
  api_key: 'key',
  query_prefix: null,
  passage_prefix: null,
  min_similarity: 0.4,
}

const CACHE: VectorCacheInfo = { key: 'local-model', model: 'bge-m3', embedded: 3, pending: 0, configured: true }

interface Recorded {
  url: string
  method?: string
  body?: string
}

/** A route answer the test resolves by hand (for asserting an in-progress state) */
function deferred<T>() {
  let resolve!: (v: T) => void
  const promise = new Promise<T>((res) => (resolve = res))
  return { promise, resolve }
}

function recordFetch(routes: Array<(r: Recorded) => unknown | undefined>) {
  const calls: Recorded[] = []
  vi.stubGlobal(
    'fetch',
    vi.fn((url: string | URL, init?: RequestInit) => {
      const rec: Recorded = {
        url: String(url),
        method: init?.method,
        body: typeof init?.body === 'string' ? init.body : undefined,
      }
      calls.push(rec)
      for (const route of routes) {
        const out = route(rec)
        if (out === undefined) continue
        // A route may answer with a promise of its own: the test decides when the batch lands
        return out instanceof Promise ? out.then(jsonResponse) : Promise.resolve(jsonResponse(out))
      }
      return Promise.resolve(jsonResponse({}))
    }),
  )
  return calls
}

function cacheRoute(caches: VectorCacheInfo[]) {
  return (r: Recorded) => (r.url.includes('/api/embeddings/caches') && r.method !== 'DELETE' ? { caches } : undefined)
}

async function mountCard(
  models: EmbedModelEntry[] = [MODEL],
  routes: Array<(r: Recorded) => unknown | undefined> = [],
) {
  // Caller routes come first: the default caches route is the catch-all fallback
  const calls = recordFetch([...routes, cacheRoute([CACHE])])
  const wrapper = mount(EmbeddingSettingsCard, { props: { models }, global: { plugins: [ElementPlus] } })
  await flushPromises() // settle the caches fetch fired on mount
  return { wrapper, calls }
}

type Wrapper = ReturnType<typeof mount>

function badge(wrapper: Wrapper) {
  return wrapper.find('.el-card__header .el-tag').text()
}
function row(wrapper: Wrapper, i = 0) {
  return wrapper.findAll('.entry')[i]
}
async function expand(wrapper: Wrapper, i = 0) {
  await row(wrapper, i).find('.entry-toggle').trigger('click')
}
function field(wrapper: Wrapper, cls: string, i = 0) {
  return row(wrapper, i).find(`${cls} input`)
}
/** A button by its text, scoped to one row's expanded area or to the card's bottom action row */
function button(wrapper: Wrapper, text: string, scope?: ReturnType<Wrapper['find']>) {
  return (scope ?? wrapper).findAll('button').find((b) => b.text() === text)
}

beforeEach(() => {
  sortable.created.length = 0
  setMemoryUILocale('zh')
})

describe('EmbeddingSettingsCard candidate editor', () => {
  it('uses display names and preserves caches and open rows by ID across reordered settings', async () => {
    const first = { ...MODEL, name: '本地服务' }
    const second = { ...MODEL, id: 'cloud-model', name: '云端服务' }
    const { wrapper } = await mountCard([first, second])
    expect(wrapper.findAll('.entry-name').map((n) => n.text())).toEqual(['本地服务', '云端服务'])
    expect(wrapper.find('.entry-index').exists()).toBe(false)
    await expand(wrapper, 0)
    await wrapper.setProps({ models: [second, { ...first, name: '本地向量' }] })
    await flushPromises()
    expect(row(wrapper, 0).find('.entry-detail').exists()).toBe(false)
    expect(row(wrapper, 1).find('.entry-detail').exists()).toBe(true)
    expect(row(wrapper, 1).find('.entry-name').text()).toBe('本地向量')
    expect(row(wrapper, 1).find('.entry-usage').text()).toBe('向量 3 · 待补 0')
    expect(wrapper.find('.settings-hint').exists()).toBe(false)
    wrapper.unmount()
  })
  it('starts empty and unconfigured, with both actions disabled', async () => {
    const { wrapper, calls } = await mountCard([], [])
    expect(badge(wrapper)).toBe('未配置')
    expect(wrapper.findAll('.entry')).toHaveLength(0)
    expect(button(wrapper, '保存')!.attributes('disabled')).toBeDefined()
    expect(button(wrapper, '检测')!.attributes('disabled')).toBeDefined()
    expect(calls.every((c) => c.url.includes('/api/embeddings/caches'))).toBe(true)
    wrapper.unmount()
  })

  it('collapses each candidate to one line: name, coverage and the switch, nothing expanded', async () => {
    const second: EmbedModelEntry = { ...MODEL, id: 'e5', name: 'e5', model: 'e5', enabled: false }
    const { wrapper } = await mountCard([MODEL, second])
    expect(wrapper.findAll('.entry')).toHaveLength(2)
    expect(wrapper.findAll('.entry-detail')).toHaveLength(0)
    expect(row(wrapper, 0).find('.entry-name').text()).toBe('bge-m3')
    expect(wrapper.find('.entry-index').exists()).toBe(false)
    expect(row(wrapper, 0).find('.entry-usage').text()).toBe('向量 3 · 待补 0')
    expect(wrapper.find('.entry-index').exists()).toBe(false)
    // The enable switch sits behind the row's text, and the row actions are all collapsed away
    expect(row(wrapper, 0).find('.entry-switch').exists()).toBe(true)
    expect(button(wrapper, '删除缓存', row(wrapper, 0))).toBeUndefined()
    expect(button(wrapper, '删除', row(wrapper, 0))).toBeUndefined()
    // No revert button; adding uses an icon button with the short label
    expect(wrapper.findAll('button').some((b) => b.text() === '还原')).toBe(false)
    expect(button(wrapper, '添加')!.find('.el-icon').exists()).toBe(true)
    wrapper.unmount()
  })

  it('expands a candidate into labeled fields and only then shows its actions', async () => {
    const { wrapper } = await mountCard()
    await expand(wrapper)
    const detail = row(wrapper).find('.entry-detail')
    expect(detail.findAll('.field-label').map((l) => l.text())).toEqual([
      '模型 ID',
      '显示名称',
      '服务地址（base_url）',
      'API 模型名',
      'API Key',
      '查询指令前缀（可选）',
      '文档指令前缀（可选）',
      '语义召回最低相似度',
    ])
    expect((field(wrapper, '.f-model').element as HTMLInputElement).value).toBe('bge-m3')
    expect(button(wrapper, '检测', detail)).toBeTruthy()
    expect(button(wrapper, '删除', detail)).toBeTruthy()
    // 3 vectors cached and nothing pending: delete is offered, backfill is not
    expect(button(wrapper, '删除缓存', detail)).toBeTruthy()
    expect(button(wrapper, '补跑', detail)).toBeUndefined()
    // Collapsing again hides them
    await expand(wrapper)
    expect(row(wrapper).find('.entry-detail').exists()).toBe(false)
    wrapper.unmount()
  })

  it('saves the canonical list after an edit', async () => {
    const { wrapper, calls } = await mountCard(
      [MODEL, { ...MODEL, id: 'e5', name: 'e5', model: 'e5', query_prefix: 'query: ' }],
      [(r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined)],
    )
    await expand(wrapper, 1)
    await field(wrapper, '.f-model', 1).setValue('e5-mini')
    await button(wrapper, '保存')!.trigger('click')
    await flushPromises()
    const put = JSON.parse(calls.find((c) => c.method === 'PUT')!.body!)
    expect(put).toEqual({
      embedding_models: [
        {
          id: 'local-model',
          name: 'bge-m3',
          enabled: true,
          base_url: 'http://svc:9/v1',
          model: 'bge-m3',
          api_key: 'key',
          query_prefix: null,
          passage_prefix: null,
          min_similarity: 0.4,
        },
        {
          id: 'e5',
          name: 'e5',
          enabled: true,
          base_url: 'http://svc:9/v1',
          model: 'e5-mini',
          api_key: 'key',
          query_prefix: 'query: ',
          passage_prefix: null,
          min_similarity: 0.4,
        },
      ],
    })
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('offers backfill only for an identity that is committed server-side', async () => {
    const { wrapper } = await mountCard(
      [MODEL],
      [
        cacheRoute([CACHE, { key: 'e5', model: 'e5', embedded: 0, pending: 4, configured: true }]),
        (r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined),
      ],
    )
    // A brand-new candidate whose identity matches a pending cache: nothing can drain it through a
    // saved entry yet, so the button must not be there
    await button(wrapper, '添加')!.trigger('click')
    await field(wrapper, '.f-id', 1).setValue('e5')
    await field(wrapper, '.f-name', 1).setValue('e5')
    await field(wrapper, '.f-model', 1).setValue('e5')
    await field(wrapper, '.f-base-url', 1).setValue('http://svc:9/v1')
    await field(wrapper, '.f-query-prefix', 1).setValue('query: ')
    await flushPromises()
    expect(row(wrapper, 1).find('.entry-usage').text()).toBe('向量 0 · 待补 4')
    expect(button(wrapper, '补跑', row(wrapper, 1))).toBeUndefined()
    await button(wrapper, '保存')!.trigger('click')
    await flushPromises()
    // The parent reloads the settings and hands the committed list back down: now it can drain
    await wrapper.setProps({
      models: [MODEL, { ...MODEL, id: 'e5', name: 'e5', model: 'e5', query_prefix: 'query: ' }],
    })
    await flushPromises()
    expect(button(wrapper, '补跑', row(wrapper, 1))).toBeTruthy()
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('re-reads the per-model caches whenever the settings arrive fresh', async () => {
    const { wrapper, calls } = await mountCard([MODEL])
    const cacheGets = () =>
      calls.filter((c) => c.url.includes('/api/embeddings/caches') && c.method !== 'DELETE').length
    expect(cacheGets()).toBe(1)
    // The host reloads the panel on every reopen: coverage and pending counts move with writes made
    // elsewhere, so they must not stay stale behind that reload
    await wrapper.setProps({ models: [{ ...MODEL, id: 'bge-m3', name: 'bge-m3', model: 'bge-m3' }] })
    await flushPromises()
    expect(cacheGets()).toBe(2)
    wrapper.unmount()
  })

  it('probes one candidate with its own values and keeps the verdict on that row', async () => {
    const { wrapper, calls } = await mountCard(
      [MODEL, { ...MODEL, id: 'dead', name: 'dead', model: 'dead', base_url: 'http://dead:9/v1' }],
      [
        (r) =>
          r.url.includes('/api/embeddings/test')
            ? { ok: true, results: [{ index: 0, key: 'dead', model: 'dead', ok: true, dim: 1024, elapsed_ms: 42 }] }
            : undefined,
      ],
    )
    await expand(wrapper, 1)
    await button(wrapper, '检测', row(wrapper, 1))!.trigger('click')
    await flushPromises()
    const post = calls.find((c) => c.url.includes('/api/embeddings/test'))!
    // Only the clicked row travels, with the values currently on screen
    expect(JSON.parse(post.body!)).toEqual({
      entries: [
        {
          id: 'dead',
          name: 'dead',
          enabled: true,
          base_url: 'http://dead:9/v1',
          model: 'dead',
          api_key: 'key',
          query_prefix: null,
          passage_prefix: null,
          min_similarity: 0.4,
        },
      ],
    })
    expect(row(wrapper, 1).find('.entry-verdict').text()).toBe('正常 · 1024 维 · 42ms')
    expect(row(wrapper, 0).find('.entry-verdict').exists()).toBe(false)
    // A verdict belongs to the values it was taken for: editing them invalidates it
    await field(wrapper, '.f-base-url', 1).setValue('http://other:9/v1')
    await flushPromises()
    expect(row(wrapper, 1).find('.entry-verdict').exists()).toBe(false)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('probes every enabled candidate at once and lines the verdicts up with the rows', async () => {
    const { wrapper } = await mountCard(
      [MODEL, { ...MODEL, id: 'dead', name: 'dead', model: 'dead' }],
      [
        (r) =>
          r.url.includes('/api/embeddings/test')
            ? {
                ok: false,
                results: [
                  { index: 0, key: 'bge-m3', model: 'bge-m3', ok: true, dim: 1024, elapsed_ms: 42 },
                  { index: 1, model: 'dead', ok: false, error: 'connection refused' },
                ],
              }
            : undefined,
      ],
    )
    expect(badge(wrapper)).toBe('未检测')
    await button(wrapper, '检测')!.trigger('click')
    await flushPromises()
    expect(badge(wrapper)).toBe('部分异常')
    expect(row(wrapper, 0).find('.entry-verdict').text()).toBe('正常 · 1024 维 · 42ms')
    expect(row(wrapper, 1).find('.entry-verdict').text()).toBe('连接失败')
    // The reason stays readable without expanding anything
    expect(wrapper.find('.probe-results').text()).toContain('dead')
    expect(wrapper.find('.probe-results').text()).toContain('connection refused')
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('reorders candidates by dragging the handle: the saved order becomes the new priority', async () => {
    const second: EmbedModelEntry = { ...MODEL, id: 'e5', name: 'e5', model: 'e5' }
    const { wrapper, calls } = await mountCard(
      [MODEL, second],
      [(r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined)],
    )
    expect(sortable.created[0].el).toBe(wrapper.find('.entry-list').element)
    drop(1, 0)
    await flushPromises()
    expect(wrapper.findAll('.entry-name').map((n) => n.text())).toEqual(['e5', 'bge-m3'])
    await button(wrapper, '保存')!.trigger('click')
    await flushPromises()
    const put = JSON.parse(calls.find((c) => c.method === 'PUT')!.body!)
    expect(put.embedding_models.map((e: EmbedModelEntry) => e.model)).toEqual(['e5', 'bge-m3'])
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('warns about vector invalidation when an identity changes', async () => {
    const { wrapper } = await mountCard()
    await expand(wrapper)
    await field(wrapper, '.f-model').setValue('new-model')
    expect(wrapper.text()).toContain('更换 ID、服务地址、模型或前缀将使 3 条向量失效')
    wrapper.unmount()
  })

  it('backfills one identity with a progress bar and stops when interrupted', async () => {
    const batches: Array<ReturnType<typeof deferred<unknown>>> = []
    const { wrapper, calls } = await mountCard(
      [MODEL],
      [
        cacheRoute([{ ...CACHE, embedded: 0, pending: 3 }]),
        (r) => {
          if (!r.url.includes('/api/embeddings/backfill')) return undefined
          const d = deferred<unknown>()
          batches.push(d)
          return d.promise
        },
      ],
    )
    await expand(wrapper)
    // Pending vectors and a committed entry: backfill is offered
    expect(button(wrapper, '补跑', row(wrapper))).toBeTruthy()
    await button(wrapper, '补跑', row(wrapper))!.trigger('click')
    await flushPromises()
    expect(batches).toHaveLength(1)
    // The batch lands: the progress bar and its percentage show the run in flight
    batches[0].resolve({ configured: true, processed: 1, remaining: 2 })
    await flushPromises()
    await flushPromises()
    expect(row(wrapper).find('.entry-progress').exists()).toBe(true)
    expect(row(wrapper).find('.el-progress__text').text()).toBe('33%')
    expect(row(wrapper).find('.progress-text').text()).toBe('补跑中 1/3')
    expect(batches).toHaveLength(2)
    // Interrupting stops the loop after the batch in flight (which still lands)
    await button(wrapper, '中断', row(wrapper))!.trigger('click')
    batches[1].resolve({ configured: true, processed: 1, remaining: 1 })
    await flushPromises()
    await flushPromises()
    expect(calls.filter((c) => c.url.includes('/api/embeddings/backfill'))).toHaveLength(2)
    expect(row(wrapper).find('.entry-progress').exists()).toBe(false)
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('deletes one identity cache only after confirmation, carrying model_key on DELETE', async () => {
    const spy = vi.spyOn(ElMessageBox, 'confirm').mockResolvedValue('confirm' as never)
    const { wrapper, calls } = await mountCard(
      [MODEL],
      [(r) => (r.url.includes('/api/embeddings/caches') && r.method === 'DELETE' ? { deleted: 3 } : undefined)],
    )
    await expand(wrapper)
    await button(wrapper, '删除缓存', row(wrapper))!.trigger('click')
    await flushPromises()
    await flushPromises()
    expect(spy).toHaveBeenCalled()
    const del = calls.find((c) => c.method === 'DELETE')!
    expect(JSON.parse(del.body!)).toEqual({ model_key: 'local-model' })
    expect(wrapper.emitted('changed')).toHaveLength(1)
    spy.mockRestore()
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('lists caches whose identity is no longer configured as deletable leftovers', async () => {
    const spy = vi.spyOn(ElMessageBox, 'confirm').mockResolvedValue('confirm' as never)
    const { wrapper, calls } = await mountCard(
      [MODEL],
      [
        cacheRoute([CACHE, { key: 'old-model', model: 'old-model', embedded: 5, pending: 2, configured: false }]),
        (r) => (r.url.includes('/api/embeddings/caches') && r.method === 'DELETE' ? { deleted: 5 } : undefined),
      ],
    )
    const leftovers = wrapper.find('.cache-leftovers')
    expect(leftovers.text()).toContain('old-model')
    expect(leftovers.text()).toContain('向量 5 · 待补 2')
    expect(leftovers.text()).toContain('未配置')
    await button(wrapper, '删除缓存', leftovers)!.trigger('click')
    await flushPromises()
    await flushPromises()
    expect(JSON.parse(calls.find((c) => c.method === 'DELETE')!.body!)).toEqual({ model_key: 'old-model' })
    spy.mockRestore()
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })
})
