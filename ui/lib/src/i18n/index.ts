// Component library i18n: a standalone-scoped vue-i18n instance.
// It is not registered as a global scope via app.use, avoiding conflicts with the host's own
// vue-i18n instance; components reference the composer from memoryUIi18n.global directly
// (t is reactive, so switching the language takes effect across the whole library).
// Adding a language takes two steps: (1) create a dictionary file (e.g. ja.ts, keys matching
// zh.ts one-to-one) and register it in messages below;
// (2) extend the MemoryUILocale type. Then add one entry to the site shell's language menu —
// the components themselves need zero changes.
import { createI18n } from 'vue-i18n'
import zh from './zh'
import en from './en'

export type MemoryUILocale = 'zh' | 'en'
/** Values for the locale config: a fixed language or follow the browser */
export type MemoryUILocaleOption = MemoryUILocale | 'auto'

function detectLocale(): MemoryUILocale {
  if (typeof navigator === 'undefined') return 'zh'
  return navigator.language?.toLowerCase().startsWith('zh') ? 'zh' : 'en'
}

export const memoryUIi18n = createI18n({
  legacy: false,
  locale: detectLocale(),
  fallbackLocale: 'zh',
  messages: { zh, en },
  // Built for a hostable component library: missing keys silently fall back, no console spam
  missingWarn: false,
  fallbackWarn: false,
})

/** Components in this library all take t from here (composer.t is reactive).
 * Explicit annotation: under pnpm's isolated layout TS cannot portably name the inferred type (TS2742), and the d.ts output needs a named reference. */
type MemoryUIComposer = typeof memoryUIi18n.global
export const t: MemoryUIComposer['t'] = memoryUIi18n.global.t

export function setMemoryUILocale(locale: MemoryUILocale): void {
  memoryUIi18n.global.locale.value = locale
}

/** Apply the locale preference; 'auto' means follow the browser language */
export function applyMemoryUILocalePreference(preference: MemoryUILocaleOption): void {
  setMemoryUILocale(preference === 'auto' ? detectLocale() : preference)
}

export function currentMemoryUILocale(): MemoryUILocale {
  return memoryUIi18n.global.locale.value as MemoryUILocale
}
