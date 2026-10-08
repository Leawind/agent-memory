import { describe, expect, it, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../../i18n'
import type { SearchLimits } from '../../types'
import SearchLimitsCard from './SearchLimitsCard.vue'

describe('SearchLimitsCard', () => {
  it('requires recall to cover adaptive bounds and displays process runtime without changing saved limits', async () => {
    setMemoryUILocale('zh')
    const limits: SearchLimits = {
      semantic_candidates: 80,
      rerank_candidates: 50,
      adaptive: {
        enabled: false,
        min_candidates: 20,
        max_candidates: 100,
        target_latency_ms: 1000,
        max_input_chars: 400000,
      },
    }
    const calls: Array<{ path: string; body?: unknown }> = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string, init?: RequestInit) => {
        calls.push({ path: String(url), body: init?.body ? JSON.parse(String(init.body)) : undefined })
        const result = String(url).includes('/runtime')
          ? {
              process_local: true,
              adaptive_enabled: true,
              models: [
                {
                  id: 'local-model',
                  selected_candidates: 37,
                  last_candidates: 40,
                  last_latency_ms: 1100,
                  samples: 3,
                  failures: 1,
                },
              ],
            }
          : {}
        return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(result)) })
      }),
    )
    const wrapper = mount(SearchLimitsCard, { props: { limits, active: false }, global: { plugins: [ElementPlus] } })
    await flushPromises()
    expect(calls).toEqual([])
    await wrapper.find('.el-switch').trigger('click')
    const save = wrapper.findAll('button').find((button) => button.text() === '保存')!
    expect(save.attributes('disabled')).toBeDefined()
    await wrapper.findAll('input')[0].setValue('120')
    expect(save.attributes('disabled')).toBeUndefined()
    await save.trigger('click')
    await flushPromises()
    expect(calls[0]).toMatchObject({
      path: '/api/settings',
      body: { search_limits: { semantic_candidates: 120, adaptive: { enabled: true } } },
    })
    expect(limits.adaptive.enabled).toBe(false)
    await wrapper
      .findAll('button')
      .find((button) => button.text() === '刷新运行统计')!
      .trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('local-model')
    expect(wrapper.text()).toContain('1100 ms')
    expect(wrapper.emitted('changed')).toHaveLength(1)
    wrapper.unmount()
  })
})
