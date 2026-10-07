// RerankerSettingsCard tests: the reranker list editor mirrors the embedding card's discipline
// (ordered candidates, canonical save, per-candidate probe) minus the vector cache.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../../index'
import RerankerSettingsCard from './RerankerSettingsCard.vue'
import type { RerankModelEntry } from '../../types'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

const ENTRY: RerankModelEntry = {
  enabled: true,
  base_url: 'http://rerank:9/v1',
  model: 'bge-reranker-v2-m3',
  api_key: null,
}

function makeProps(models?: RerankModelEntry[]) {
  return { models: models ?? [ENTRY] }
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

function mountCard(models?: RerankModelEntry[], routes: Array<(r: Recorded) => unknown | undefined> = []) {
  const calls = recordFetch(routes)
  return {
    wrapper: mount(RerankerSettingsCard, { props: makeProps(models), global: { plugins: [ElementPlus] } }),
    calls,
  }
}

function badge(wrapper: ReturnType<typeof mount>) {
  return wrapper.find('.el-card__header .el-tag').text()
}

beforeEach(() => {
  setMemoryUILocale('zh')
})

describe('RerankerSettingsCard', () => {
  it('renders the reranker editor unconfigured with no cache section', () => {
    const { wrapper, calls } = mountCard([])
    expect(badge(wrapper)).toBe('未配置')
    expect(wrapper.text()).toContain('重排')
    expect(wrapper.find('.cache-section').exists()).toBe(false)
    expect(calls).toHaveLength(0)
    wrapper.unmount()
  })

  it('saves the canonical rerank_models list and probes because an entry is enabled', async () => {
    const { wrapper, calls } = mountCard(
      [],
      [
        (r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined),
        (r) =>
          r.url.includes('/api/rerank/test')
            ? { ok: true, results: [{ model: 'bge-reranker-v2-m3', ok: true, scored: 1, elapsed_ms: 30 }] }
            : undefined,
      ],
    )
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '添加候选模型')!
      .trigger('click')
    await wrapper.findAll('.entry')[0].find('.entry-model input')!.setValue('bge-reranker-v2-m3')
    await wrapper.findAll('.entry')[0].findAll('.entry-fields input')[0].setValue('http://rerank:9/v1')
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '保存并检测')!
      .trigger('click')
    await flushPromises()
    await flushPromises()
    const put = JSON.parse(calls.find((c) => c.method === 'PUT')!.body!)
    expect(put).toEqual({
      rerank_models: [{ enabled: true, base_url: 'http://rerank:9/v1', model: 'bge-reranker-v2-m3', api_key: null }],
    })
    expect(calls.some((c) => c.url.includes('/api/rerank/test'))).toBe(true)
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('surfaces a failed probe under the broken badge with the error verbatim', async () => {
    const { wrapper } = mountCard(
      [ENTRY],
      [
        (r) =>
          r.url.includes('/api/rerank/test')
            ? { ok: false, results: [{ model: 'bge-reranker-v2-m3', ok: false, error: 'rerank service returned 500' }] }
            : undefined,
      ],
    )
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '检测连接')!
      .trigger('click')
    await flushPromises()
    expect(badge(wrapper)).toBe('部分异常')
    expect(wrapper.find('.probe-results').text()).toContain('rerank service returned 500')
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })
})
