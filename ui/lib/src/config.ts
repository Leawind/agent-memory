// Component library config: the host injects it from any ancestor component via
// provideMemoryUI; falls back to defaults when not injected (same-origin root-path
// deployment), so the components work out of the box.
import { inject, provide } from 'vue'
import type { InjectionKey } from 'vue'
import { applyMemoryUILocalePreference } from './i18n'
import type { MemoryUILocaleOption } from './i18n'

export interface MemoryUIConfig {
  /** API prefix. Defaults to '' (same-origin deployment with the server); fill in the server address for cross-system integrations, e.g. 'http://127.0.0.1:8899' */
  baseUrl?: string
  /** Custom fetch (inject auth headers, error interception, etc.). Defaults to globalThis.fetch */
  fetch?: typeof fetch
  /** Default page size for lists. Defaults to 20 */
  defaultPageSize?: number
  /** UI language. 'auto' follows the browser (default); can also be pinned to 'zh' / 'en', switchable at runtime via setMemoryUILocale */
  locale?: MemoryUILocaleOption
  /** Where the "save to this browser" button lands in the one-time token display dialog
   *  shown after creating/resetting an identity in the admin panel.
   *  The button only renders when this is injected; the standalone site shell uses it
   *  to store the token in its own multi-identity token table. */
  onIdentityToken?: (name: string, token: string) => void
  /** Callback after the auth toggle is switched successfully (the standalone site shell uses it to re-resolve whoami so the top-bar identity area reflects the token mode immediately). */
  onAuthChanged?: (enabled: boolean) => void
}

export type ResolvedMemoryUIConfig = Omit<Required<MemoryUIConfig>, 'onIdentityToken' | 'onAuthChanged'> &
  Pick<MemoryUIConfig, 'onIdentityToken' | 'onAuthChanged'>

export const MemoryUIConfigKey: InjectionKey<MemoryUIConfig> = Symbol('memory-ui-config')

const defaults: ResolvedMemoryUIConfig = {
  baseUrl: '',
  fetch: (...args) => globalThis.fetch(...args),
  defaultPageSize: 20,
  locale: 'auto',
}

/** Inject the config inside a component setup (or use app.provide(MemoryUIConfigKey, config) directly). */
export function provideMemoryUI(config: MemoryUIConfig): void {
  if (config.locale) applyMemoryUILocalePreference(config.locale)
  provide(MemoryUIConfigKey, config)
}

/** Read the fully resolved config; must be called inside a component setup. */
export function useMemoryConfig(): ResolvedMemoryUIConfig {
  const raw = inject(MemoryUIConfigKey)
  return {
    baseUrl: (raw?.baseUrl ?? defaults.baseUrl).replace(/\/+$/, ''),
    fetch: raw?.fetch ?? defaults.fetch,
    defaultPageSize: raw?.defaultPageSize ?? defaults.defaultPageSize,
    locale: raw?.locale ?? defaults.locale,
    onIdentityToken: raw?.onIdentityToken,
    onAuthChanged: raw?.onAuthChanged,
  }
}
