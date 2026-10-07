// EmbeddingSettingsCard tests for the multi-candidate editor: ordered list editing, canonical
// save payload, per-candidate probe verdicts, identity-change warning, and per-model cache
// management (backfill loop / delete-with-confirm). Mounts the card directly; fetch is mocked
// per route.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { ElMessageBox } from 'element-plus'
import { setMemoryUILocale } from '../../index'
import EmbeddingSettingsCard from './EmbeddingSettingsCard.vue'
import type { EmbedModelEntry, StatsInfo, VectorCacheInfo } from '../../types'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

const MODEL: EmbedModelEntry = {
  enabled: true,
  base_url: 'http://svc:9/v1',
  model: 'bge-m3',
  api_key: 'key',
  query_prefix: null,
  passage_prefix: null,
  min_similarity: 0.4,
}

const STATS: StatsInfo = {
  path: 'memory.db',
  memories: 2,
  tags: 0,
  file_size: 0,
  schema_version: 2,
  embedding: { enabled: true, model: 'bge-m3', embedded: 2, pending: 0 },
}

function makeProps(over: { models?: EmbedModelEntry[]; stats?: StatsInfo | null } = {}) {
  return {
    models: over.models ?? [MODEL],
    stats: 'stats' in over ? over.stats! : STATS,
  }
}

interface Recorded {
  url: string
  method?: string
  body?: string
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
        return Promise.resolve(jsonResponse(out))
      }
      return Promise.resolve(jsonResponse({}))
    }),
  )
  return calls
}

const CACHE: VectorCacheInfo = { key: 'bge-m3', model: 'bge-m3', embedded: 3, pending: 0, configured: true }

function cacheRoute(caches: VectorCacheInfo[]) {
  return (r: Recorded) => (r.url.includes('/api/embeddings/caches') && r.method !== 'DELETE' ? { caches } : undefined)
}

async function mountCard(props = makeProps(), routes: Array<(r: Recorded) => unknown | undefined> = []) {
  // Caller routes come first: the default caches route is the catch-all fallback
  const calls = recordFetch([...routes, cacheRoute([CACHE])])
  const wrapper = mount(EmbeddingSettingsCard, { props, global: { plugins: [ElementPlus] } })
  await flushPromises() // settle the caches fetch fired on mount
  return { wrapper, calls }
}

function badge(wrapper: ReturnType<typeof mount>) {
  return wrapper.find('.el-card__header .el-tag').text()
}

function saveButton(wrapper: ReturnType<typeof mount>) {
  return wrapper.findAll('button').find((b) => b.text() === '保存并检测')!
}

/** The model input of one entry (the el-switch also renders an input — always select by class) */
function modelInput(wrapper: ReturnType<typeof mount>, entry = 0) {
  return wrapper.findAll('.entry')[entry].find('.entry-model input')!
}
/** The five field inputs of one entry: baseUrl, apiKey, minSimilarity, queryPrefix, passagePrefix */
function fieldInputs(wrapper: ReturnType<typeof mount>, entry = 0) {
  return wrapper.findAll('.entry')[entry].findAll('.entry-fields input')
}

beforeEach(() => {
  setMemoryUILocale('zh')
})

describe('EmbeddingSettingsCard multi-candidate editor', () => {
  it('starts empty and unconfigured, with the editor ready and no requests beyond the cache list', async () => {
    const { wrapper, calls } = await mountCard(
      makeProps({ models: [], stats: { ...STATS, embedding: { enabled: false } } }),
      [],
    )
    expect(badge(wrapper)).toBe('未配置')
    expect(wrapper.findAll('.entry').length).toBe(0)
    expect(saveButton(wrapper).attributes('disabled')).toBeDefined()
    expect(wrapper.text()).toContain('语义')
    expect(calls.every((c) => c.url.includes('/api/embeddings/caches'))).toBe(true)
    wrapper.unmount()
  })

  it('adds a candidate and saves the canonical list, then probes because it is enabled', async () => {
    const { wrapper, calls } = await mountCard(
      makeProps({ models: [], stats: { ...STATS, embedding: { enabled: false } } }),
      [
        (r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined),
        (r) =>
          r.url.includes('/api/embeddings/test')
            ? { ok: true, results: [{ model: 'bge-m3-x', ok: true, dim: 1024, elapsed_ms: 42 }] }
            : undefined,
      ],
    )
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '添加候选模型')!
      .trigger('click')
    await modelInput(wrapper).setValue('bge-m3-x')
    await fieldInputs(wrapper)[0].setValue('http://svc:9/v1')
    expect(saveButton(wrapper).attributes('disabled')).toBeUndefined()
    await saveButton(wrapper).trigger('click')
    await flushPromises()
    await flushPromises()
    const put = calls.find((c) => c.method === 'PUT')!
    expect(JSON.parse(put.body!)).toEqual({
      embedding_models: [
        {
          enabled: true,
          base_url: 'http://svc:9/v1',
          model: 'bge-m3-x',
          api_key: null,
          query_prefix: null,
          passage_prefix: null,
          min_similarity: null,
        },
      ],
    })
    expect(calls.some((c) => c.url.includes('/api/embeddings/test'))).toBe(true)
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('reverts an in-progress draft without any write request', async () => {
    const { wrapper, calls } = await mountCard()
    await modelInput(wrapper).setValue('edited-model')
    expect(saveButton(wrapper).attributes('disabled')).toBeUndefined()
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '还原')!
      .trigger('click')
    expect((modelInput(wrapper).element as HTMLInputElement).value).toBe('bge-m3')
    expect(saveButton(wrapper).attributes('disabled')).toBeDefined()
    expect(calls.every((c) => c.method !== 'PUT')).toBe(true)
    wrapper.unmount()
  })

  it('shows per-candidate probe verdicts and the broken badge when a candidate fails', async () => {
    const { wrapper } = await mountCard(
      makeProps({
        models: [MODEL, { ...MODEL, model: 'dead-model', enabled: true, base_url: 'http://dead:9/v1' }],
      }),
      [
        (r) =>
          r.url.includes('/api/embeddings/test')
            ? {
                ok: false,
                results: [
                  { model: 'bge-m3', ok: true, dim: 1024, elapsed_ms: 42 },
                  { model: 'dead-model', ok: false, error: 'connection refused' },
                ],
              }
            : undefined,
      ],
    )
    expect(badge(wrapper)).toBe('未检测')
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '检测连接')!
      .trigger('click')
    await flushPromises()
    expect(badge(wrapper)).toBe('部分异常')
    const probes = wrapper.find('.probe-results').text()
    expect(probes).toContain('bge-m3')
    expect(probes).toContain('1024 维')
    expect(probes).toContain('dead-model')
    expect(probes).toContain('connection refused')
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('warns about vector invalidation when an identity changes, and preserves floors on save', async () => {
    const { wrapper, calls } = await mountCard(makeProps(), [
      (r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined),
      (r) =>
        r.url.includes('/api/embeddings/test')
          ? { ok: true, results: [{ model: 'new-model', ok: true, dim: 8 }] }
          : undefined,
    ])
    await modelInput(wrapper).setValue('new-model')
    expect(wrapper.text()).toContain('更换模型或前缀将使 3 条向量失效')
    await saveButton(wrapper).trigger('click')
    await flushPromises()
    await flushPromises()
    const put = JSON.parse(calls.find((c) => c.method === 'PUT')!.body!)
    expect(put.embedding_models[0].model).toBe('new-model')
    expect(put.embedding_models[0].min_similarity).toBe(0.4)
    expect(put.embedding_models[0].api_key).toBe('key')
    expect(calls.some((c) => c.url.includes('/api/embeddings/test'))).toBe(true)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('reorders candidates: the saved list order is the new failover priority', async () => {
    const second: EmbedModelEntry = { ...MODEL, model: 'e5', enabled: false, api_key: null }
    const { wrapper, calls } = await mountCard(makeProps({ models: [MODEL, second] }), [
      (r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined),
    ])
    // Move the second entry up
    await wrapper.findAll('.entry')[1].findAll('button')[0].trigger('click')
    await saveButton(wrapper).trigger('click')
    await flushPromises()
    const put = JSON.parse(calls.find((c) => c.method === 'PUT')!.body!)
    expect(put.embedding_models.map((e: EmbedModelEntry) => e.model)).toEqual(['e5', 'bge-m3'])
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('backfills one cache identity: loops with model_key until the queue drains', async () => {
    let n = 0
    const { wrapper, calls } = await mountCard(makeProps(), [
      cacheRoute([{ ...CACHE, pending: 2 }]),
      (r) => {
        if (!r.url.includes('/api/embeddings/backfill')) return undefined
        n += 1
        return n === 1
          ? { configured: true, processed: 1, remaining: 1 }
          : { configured: true, processed: 0, remaining: 1 }
      },
    ])
    expect(wrapper.find('.cache-section').exists()).toBe(true)
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '补跑')!
      .trigger('click')
    await flushPromises()
    await flushPromises()
    const posts = calls.filter((c) => c.url.includes('/api/embeddings/backfill'))
    expect(posts.length).toBe(2)
    expect(JSON.parse(posts[0].body!)).toEqual({ model_key: 'bge-m3' })
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('deletes a cache identity only after confirmation, carrying model_key on DELETE', async () => {
    const spy = vi.spyOn(ElMessageBox, 'confirm').mockResolvedValue('confirm' as never)
    const { wrapper, calls } = await mountCard(makeProps(), [
      (r) => (r.url.includes('/api/embeddings/caches') && r.method === 'DELETE' ? { deleted: 3 } : undefined),
    ])
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '删除缓存')!
      .trigger('click')
    await flushPromises()
    await flushPromises()
    expect(spy).toHaveBeenCalled()
    const del = calls.find((c) => c.method === 'DELETE')!
    expect(JSON.parse(del.body!)).toEqual({ model_key: 'bge-m3' })
    expect(wrapper.emitted('changed')).toHaveLength(1)
    spy.mockRestore()
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })
})
