// RerankerSettingsCard tests: the reranker editor mirrors the embedding card (collapsed line per
// candidate, expansion into labeled fields, per-row and whole-list probes, drag reordering,
// canonical save) minus the vector cache — rerankers keep no derived data.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import type { SortableEvent, SortableOptions } from 'sortablejs'
import { setMemoryUILocale } from '../../index'
import RerankerSettingsCard from './RerankerSettingsCard.vue'
import type { RerankModelEntry } from '../../types'

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

const ENTRY: RerankModelEntry = {
  enabled: true,
  base_url: 'http://rerank:9/v1',
  model: 'bge-reranker-v2-m3',
  api_key: null,
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
  const wrapper = mount(RerankerSettingsCard, {
    props: { models: models ?? [ENTRY] },
    global: { plugins: [ElementPlus] },
  })
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
function button(wrapper: Wrapper, text: string, scope?: ReturnType<Wrapper['find']>) {
  return (scope ?? wrapper).findAll('button').find((b) => b.text() === text)
}

beforeEach(() => {
  sortable.created.length = 0
  setMemoryUILocale('zh')
})

describe('RerankerSettingsCard', () => {
  it('renders unconfigured with an empty list, no cache section and disabled actions', async () => {
    const { wrapper, calls } = mountCard([])
    expect(badge(wrapper)).toBe('未配置')
    expect(wrapper.text()).toContain('重排')
    expect(wrapper.find('.cache-leftovers').exists()).toBe(false)
    expect(wrapper.findAll('.entry')).toHaveLength(0)
    expect(button(wrapper, '保存')!.attributes('disabled')).toBeDefined()
    expect(button(wrapper, '检测')!.attributes('disabled')).toBeDefined()
    expect(calls).toHaveLength(0)
    wrapper.unmount()
  })

  it('collapses a candidate to one line and expands it into labeled fields', async () => {
    const { wrapper } = await mountCard()
    expect(wrapper.findAll('.entry')).toHaveLength(1)
    expect(button(wrapper, '删除', row(wrapper))).toBeUndefined()
    await expand(wrapper)
    expect(
      row(wrapper)
        .findAll('.field-label')
        .map((l) => l.text()),
    ).toEqual(['服务地址（base_url）', '模型名', 'API Key'])
    expect(button(wrapper, '删除', row(wrapper))).toBeTruthy()
    expect(button(wrapper, '检测', row(wrapper))).toBeTruthy()
    wrapper.unmount()
  })

  it('saves the canonical rerank_models list after an edit', async () => {
    const { wrapper, calls } = await mountCard(
      [],
      [(r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined)],
    )
    await button(wrapper, '添加')!.trigger('click')
    await row(wrapper).find('.f-model input').setValue('bge-reranker-v2-m3')
    await row(wrapper).find('.f-base-url input').setValue('http://rerank:9/v1')
    await button(wrapper, '保存')!.trigger('click')
    await flushPromises()
    const put = JSON.parse(calls.find((c) => c.method === 'PUT')!.body!)
    expect(put).toEqual({
      rerank_models: [{ enabled: true, base_url: 'http://rerank:9/v1', model: 'bge-reranker-v2-m3', api_key: null }],
    })
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('probes one candidate in place, with the values on screen', async () => {
    const { wrapper, calls } = await mountCard(
      [ENTRY, { ...ENTRY, model: 'dead', base_url: 'http://dead:9/v1' }],
      [
        (r) =>
          r.url.includes('/api/rerank/test')
            ? { ok: true, results: [{ index: 0, model: 'dead', ok: true, scored: 1, elapsed_ms: 30 }] }
            : undefined,
      ],
    )
    // The row is edited first: the probe must follow the draft, not the saved entry
    await expand(wrapper, 1)
    await row(wrapper, 1).find('.f-model input').setValue('dead-2')
    await button(wrapper, '检测', row(wrapper, 1))!.trigger('click')
    await flushPromises()
    expect(JSON.parse(calls.find((c) => c.url.includes('/api/rerank/test'))!.body!)).toEqual({
      entries: [{ enabled: true, base_url: 'http://dead:9/v1', model: 'dead-2', api_key: null }],
    })
    expect(row(wrapper, 1).find('.entry-verdict').text()).toBe('正常 · 1 篇已打分 · 30ms')
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('surfaces a failed probe under the broken badge with the error verbatim', async () => {
    const { wrapper } = await mountCard(
      [ENTRY],
      [
        (r) =>
          r.url.includes('/api/rerank/test')
            ? {
                ok: false,
                results: [{ index: 0, model: 'bge-reranker-v2-m3', ok: false, error: 'connection refused' }],
              }
            : undefined,
      ],
    )
    expect(badge(wrapper)).toBe('未检测')
    await button(wrapper, '检测')!.trigger('click')
    await flushPromises()
    expect(badge(wrapper)).toBe('部分异常')
    expect(row(wrapper).find('.entry-verdict').text()).toBe('连接失败')
    expect(wrapper.find('.probe-results').text()).toContain('connection refused')
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })

  it('reorders candidates by dragging the handle', async () => {
    const { wrapper, calls } = await mountCard(
      [ENTRY, { ...ENTRY, model: 'second' }],
      [(r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined)],
    )
    expect(sortable.created[0].el).toBe(wrapper.find('.entry-list').element)
    drop(1, 0)
    await flushPromises()
    expect(wrapper.findAll('.entry-name').map((n) => n.text())).toEqual(['second', 'bge-reranker-v2-m3'])
    await button(wrapper, '保存')!.trigger('click')
    await flushPromises()
    const put = JSON.parse(calls.find((c) => c.method === 'PUT')!.body!)
    expect(put.rerank_models.map((e: RerankModelEntry) => e.model)).toEqual(['second', 'bge-reranker-v2-m3'])
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })
})
