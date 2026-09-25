// 组件库 i18n：独立作用域的 vue-i18n 实例。
// 不通过 app.use 注册全局 scope，避免与宿主自己的 vue-i18n 实例冲突；
// 组件直接引用 memoryUIi18n.global 的 composer（t 是响应式的，切语言即全库生效）。
import { createI18n } from 'vue-i18n'
import zh from './zh'
import en from './en'

export type MemoryUILocale = 'zh' | 'en'
/** locale 配置取值：固定语言或跟随浏览器 */
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
  // 面向宿主组件库，缺 key 时静默回退即可，不刷控制台
  missingWarn: false,
  fallbackWarn: false,
})

/** 库内组件统一从这里取 t（composer.t 为响应式） */
export const { t } = memoryUIi18n.global

export function setMemoryUILocale(locale: MemoryUILocale): void {
  memoryUIi18n.global.locale.value = locale
}

/** 应用 locale 偏好；'auto' 表示跟随浏览器语言 */
export function applyMemoryUILocalePreference(preference: MemoryUILocaleOption): void {
  setMemoryUILocale(preference === 'auto' ? detectLocale() : preference)
}

export function currentMemoryUILocale(): MemoryUILocale {
  return memoryUIi18n.global.locale.value as MemoryUILocale
}
