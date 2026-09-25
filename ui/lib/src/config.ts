// 组件库配置：宿主在任意祖先组件调用 provideMemoryUI 注入；未注入时回退默认值
// （同源根路径部署），保证组件开箱即用。
import { inject, provide } from 'vue'
import type { InjectionKey } from 'vue'

export interface MemoryUIConfig {
  /** API 前缀。默认 ''（与服务器同源部署）；跨系统集成时填服务器地址，如 'http://127.0.0.1:8899' */
  baseUrl?: string
  /** 自定义 fetch（注入鉴权头、错误拦截等）。默认 globalThis.fetch */
  fetch?: typeof fetch
  /** 列表默认分页大小。默认 20 */
  defaultPageSize?: number
}

export type ResolvedMemoryUIConfig = Required<MemoryUIConfig>

export const MemoryUIConfigKey: InjectionKey<MemoryUIConfig> = Symbol('memory-ui-config')

const defaults: ResolvedMemoryUIConfig = {
  baseUrl: '',
  fetch: (...args) => globalThis.fetch(...args),
  defaultPageSize: 20,
}

/** 在组件 setup 中注入配置（也可直接用 app.provide(MemoryUIConfigKey, config)）。 */
export function provideMemoryUI(config: MemoryUIConfig): void {
  provide(MemoryUIConfigKey, config)
}

/** 读取解析后的完整配置；必须在组件 setup 中调用。 */
export function useMemoryConfig(): ResolvedMemoryUIConfig {
  const raw = inject(MemoryUIConfigKey)
  return {
    baseUrl: (raw?.baseUrl ?? defaults.baseUrl).replace(/\/+$/, ''),
    fetch: raw?.fetch ?? defaults.fetch,
    defaultPageSize: raw?.defaultPageSize ?? defaults.defaultPageSize,
  }
}
