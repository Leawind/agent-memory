import { describe, expect, it, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../../i18n'
import type { LifecyclePolicy } from '../../types'
import LifecyclePolicyCard from './LifecyclePolicyCard.vue'

const policy: LifecyclePolicy = {
  half_life_days: { fact: null, preference: 365, procedure: 730, context: 30, event: 7 },
  freshness_weight: 0.2,
  reinforcement_weight: 0.1,
}

describe('LifecyclePolicyCard', () => {
  it('keeps no-decay values nullable and runs maintenance only on explicit actions', async () => {
    setMemoryUILocale('zh')
    const calls: Array<{ path: string; method?: string; body?: unknown }> = []
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string, init?: RequestInit) => {
        calls.push({
          path: String(url),
          method: init?.method,
          body: init?.body ? JSON.parse(String(init.body)) : undefined,
        })
        return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve('{}') })
      }),
    )
    const wrapper = mount(LifecyclePolicyCard, { props: { policy, active: false }, global: { plugins: [ElementPlus] } })
    await flushPromises()
    expect(calls).toEqual([])
    await wrapper.findAll('input[type=checkbox]')[0].setValue(true)
    await wrapper
      .findAll('button')
      .find((button) => button.text() === '保存')!
      .trigger('click')
    await flushPromises()
    expect(calls[0]).toMatchObject({
      path: '/api/settings',
      method: 'PUT',
      body: { lifecycle_policy: { half_life_days: { fact: 365 } } },
    })
    expect(policy.half_life_days.fact).toBeNull()
    expect(wrapper.emitted('changed')).toHaveLength(1)
    await wrapper
      .findAll('button')
      .find((button) => button.text() === '重建分数投影')!
      .trigger('click')
    await flushPromises()
    expect(calls.slice(1).map((call) => call.path)).toEqual(['/api/access/rebuild', '/api/access/stats'])
    wrapper.unmount()
  })
})
