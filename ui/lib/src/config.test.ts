// 配置注入测试：provide 覆盖默认值、未 provide 回退默认值、baseUrl 规范化
import { describe, expect, it } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount } from '@vue/test-utils'
import { provideMemoryUI, useMemoryConfig } from './config'
import type { ResolvedMemoryUIConfig } from './config'

// 在组件 setup 中捕获 useMemoryConfig 的返回值供断言
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
