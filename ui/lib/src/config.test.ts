// Config injection tests: provide overrides defaults, missing provide falls back to defaults,
// baseUrl normalization
import { describe, expect, it } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import { provideMemoryUI, useMemoryConfig } from './config'
import type { ResolvedMemoryUIConfig } from './config'

// Capture useMemoryConfig's return value inside a component setup for assertions
let captured: ResolvedMemoryUIConfig | null = null
const Probe = defineComponent({
  setup() {
    captured = useMemoryConfig()
    return () => h('div')
  },
})

const Host = defineComponent({
  setup() {
    provideMemoryUI({ baseUrl: 'http://127.0.0.1:8899/', defaultPageSize: 7 })
  },
  render() {
    return h(Probe)
  },
})

describe('memory UI config', () => {
  it('uses provided values and strips the trailing slash from baseUrl', () => {
    mount(Host)
    expect(captured?.baseUrl).toBe('http://127.0.0.1:8899')
    expect(captured?.defaultPageSize).toBe(7)
  })

  it('falls back to same-origin defaults without a provider', () => {
    mount(Probe)
    expect(captured?.baseUrl).toBe('')
    expect(captured?.defaultPageSize).toBe(20)
    expect(typeof captured?.fetch).toBe('function')
  })
})
