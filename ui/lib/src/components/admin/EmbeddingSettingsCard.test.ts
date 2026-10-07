// EmbeddingSettingsCard state-machine tests: the card mirrors the server's own state
// (effective config = switch on AND base_url AND model; health = probe; coverage = stats).
// Mounts the card directly with props; fetch is mocked per route.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../../index'
import EmbeddingSettingsCard from './EmbeddingSettingsCard.vue'
import type { StatsInfo } from '../../types'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

const COVERAGE: NonNullable<StatsInfo['embedding']> = { enabled: true, model: 'bge-m3', embedded: 2, pending: 0 }

function makeProps(over: Partial<ConstructorParameters<typeof EmbeddingSettingsCard>[0]> = {}) {
  return {
    embeddingEnabled: true,
    embeddingBaseUrl: 'http://svc:9/v1',
    embeddingModel: 'bge-m3',
    embeddingApiKey: 'key',
    stats: { embedding: COVERAGE } as StatsInfo | null,
    compact: false,
    ...over,
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
        // A route may hand back a Promise it controls (to hold a response in flight); pass it
        // through untouched — wrapping it here would stringify it into an empty object
        return out instanceof Promise ? out : Promise.resolve(jsonResponse(out))
      }
      return Promise.resolve(jsonResponse({}))
    }),
  )
  return calls
}

function mountCard(props = makeProps()) {
  return mount(EmbeddingSettingsCard, { props, global: { plugins: [ElementPlus] } })
}

function badge(wrapper: ReturnType<typeof mountCard>) {
  return wrapper.find('.el-card__header .el-tag').text()
}

function saveButton(wrapper: ReturnType<typeof mountCard>) {
  return wrapper.findAll('button').find((b) => b.text() === '保存并检测')!
}

async function fillInput(wrapper: ReturnType<typeof mountCard>, index: number, value: string) {
  await wrapper.findAll('.config-form input')[index].setValue(value)
}

beforeEach(() => {
  setMemoryUILocale('zh')
})

describe('EmbeddingSettingsCard states', () => {
  it('auto-opens the config form while unconfigured and keeps the switch disabled', () => {
    const calls = recordFetch([])
    const wrapper = mountCard(
      makeProps({
        embeddingEnabled: false,
        embeddingBaseUrl: '',
        embeddingModel: '',
        embeddingApiKey: '',
        stats: { embedding: { enabled: false } },
      }),
    )
    expect(badge(wrapper)).toBe('未配置')
    expect(wrapper.find('.el-switch').classes()).toContain('is-disabled')
    expect(wrapper.find('.config-form').exists()).toBe(true)
    expect(wrapper.text()).toContain('语义')
    // A save on an untouched empty form is a no-op: disabled
    expect(saveButton(wrapper).attributes('disabled')).toBeDefined()
    expect(calls).toHaveLength(0)
    wrapper.unmount()
  })

  it('saves only the three config fields while disabled and settles to the parked badge', async () => {
    const calls = recordFetch([
      (r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined),
    ])
    const props = makeProps({
      embeddingEnabled: false,
      embeddingBaseUrl: '',
      embeddingModel: '',
      embeddingApiKey: '',
      stats: { embedding: { enabled: false } },
    })
    const wrapper = mountCard(props)
    await fillInput(wrapper, 0, 'http://svc:9/v1')
    await fillInput(wrapper, 1, 'bge-m3')
    expect(saveButton(wrapper).attributes('disabled')).toBeUndefined()
    await saveButton(wrapper).trigger('click')
    await flushPromises()
    // The switch is not the form's business: the PUT carries the three fields only, and no
    // probe runs while the switch is off (the server would reject it as not-configured)
    const put = calls.find((c) => c.method === 'PUT')!
    expect(JSON.parse(put.body!)).toEqual({
      embedding_base_url: 'http://svc:9/v1',
      embedding_model: 'bge-m3',
      embedding_api_key: '',
    })
    expect(calls.some((c) => c.url.includes('/api/embeddings/test'))).toBe(false)
    expect(wrapper.emitted('changed')).toHaveLength(1)
    // The parent reload feeds the saved values back as props (both URL and model: that is what
    // makes the config complete server-side) — the card settles on the parked badge, form closed
    await wrapper.setProps({ embeddingBaseUrl: 'http://svc:9/v1', embeddingModel: 'bge-m3', embeddingApiKey: '' })
    await flushPromises()
    expect(badge(wrapper)).toBe('已停用')
    expect(wrapper.find('.config-form').exists()).toBe(false)
    wrapper.unmount()
  })

  it('applies the intent switch immediately with a dedicated PUT', async () => {
    const calls = recordFetch([
      (r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined),
    ])
    const wrapper = mountCard(makeProps({ embeddingEnabled: false }))
    expect(badge(wrapper)).toBe('已停用')
    await wrapper.find('.el-switch').trigger('click')
    await flushPromises()
    const put = calls.find((c) => c.method === 'PUT')!
    expect(JSON.parse(put.body!)).toEqual({ embedding_enabled: true })
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
  })

  it('probes on demand and turns the badge healthy with the dimension', async () => {
    const calls = recordFetch([
      (r) => (r.url.includes('/api/embeddings/test') ? { ok: true, dim: 1024, elapsed_ms: 42 } : undefined),
    ])
    const wrapper = mountCard()
    expect(badge(wrapper)).toBe('未检测')
    expect(wrapper.text()).toContain('向量覆盖 2/2')
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '检测连接')!
      .trigger('click')
    await flushPromises()
    expect(calls.some((c) => c.url.includes('/api/embeddings/test'))).toBe(true)
    expect(badge(wrapper)).toBe('已生效')
    expect(wrapper.text()).toContain('1024 维')
    // The label flips once a probe has settled
    expect(wrapper.findAll('button').some((b) => b.text() === '重新检测')).toBe(true)
    wrapper.unmount()
  })

  it('surfaces a broken probe verbatim under the broken badge', async () => {
    recordFetch([
      (r) =>
        r.url.includes('/api/embeddings/test')
          ? { ok: false, error: 'embedding service returned 401: bad key' }
          : undefined,
    ])
    const wrapper = mountCard()
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '检测连接')!
      .trigger('click')
    await flushPromises()
    expect(badge(wrapper)).toBe('服务异常')
    expect(wrapper.find('.broken-err').text()).toContain('bad key')
    wrapper.unmount()
  })

  it('warns about vector invalidation when the model changes and probes after saving', async () => {
    const calls = recordFetch([
      (r) => (r.url.includes('/api/settings') && r.method === 'PUT' ? { saved: true } : undefined),
      (r) => (r.url.includes('/api/embeddings/test') ? { ok: true, dim: 8 } : undefined),
    ])
    const wrapper = mountCard(
      makeProps({
        embeddingModel: 'old-model',
        stats: { embedding: { ...COVERAGE, model: 'old-model', embedded: 3 } },
      }),
    )
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '配置服务')!
      .trigger('click')
    await fillInput(wrapper, 1, 'new-model')
    expect(wrapper.text()).toContain('更换模型会使现有 3 条向量失效')
    expect(saveButton(wrapper).attributes('disabled')).toBeUndefined()
    await saveButton(wrapper).trigger('click')
    await flushPromises()
    await flushPromises()
    const put = calls.find((c) => c.method === 'PUT')!
    expect(JSON.parse(put.body!)).toEqual({
      embedding_base_url: 'http://svc:9/v1',
      embedding_model: 'new-model',
      embedding_api_key: 'key',
    })
    // Enabled: the save is followed by a probe, and success closes the form
    expect(calls.some((c) => c.url.includes('/api/embeddings/test'))).toBe(true)
    expect(badge(wrapper)).toBe('已生效')
    expect(wrapper.find('.config-form').exists()).toBe(false)
    wrapper.unmount()
  })

  it('shows live progress while a backfill drains the queue', async () => {
    // Hold the second batch in flight so the progress line can be observed mid-run (it clears
    // in the finally once the queue drains)
    let releaseSecond: (v: unknown) => void = () => {}
    let n = 0
    recordFetch([
      (r) => {
        if (!r.url.includes('/api/embeddings/backfill')) return undefined
        n += 1
        if (n === 1) return { configured: true, processed: 1, remaining: 1 }
        if (n === 2) return new Promise((res) => (releaseSecond = res))
        return { configured: true, processed: 0, remaining: 0 }
      },
    ])
    const wrapper = mountCard(makeProps({ stats: { embedding: { ...COVERAGE, embedded: 0, pending: 2 } } }))
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '补跑向量化')!
      .trigger('click')
    await flushPromises()
    await flushPromises()
    expect(wrapper.text()).toContain('补跑中 1/2')
    // Resolve the held batch with a Response-shaped object (the api layer calls res.text())
    releaseSecond(jsonResponse({ configured: true, processed: 1, remaining: 0 }))
    await flushPromises()
    await flushPromises()
    // Progress clears; the parent reload feeds the drained coverage back through props
    expect(wrapper.emitted('changed')).toHaveLength(1)
    await wrapper.setProps({ stats: { embedding: { ...COVERAGE, embedded: 2, pending: 0 } } as StatsInfo })
    await flushPromises()
    expect(wrapper.text()).toContain('向量覆盖 2/2')
    wrapper.unmount()
    document.querySelectorAll('.el-message').forEach((el) => el.remove())
  })
})
