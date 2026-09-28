// Presentation formatting helpers (pure functions). Time format follows the library's current language.
import { memoryUIi18n } from './i18n'

export function formatTime(ts: number | null | undefined): string {
  if (!ts) return '—'
  const locale = memoryUIi18n.global.locale.value === 'en' ? 'en-US' : 'zh-CN'
  return new Date(ts * 1000).toLocaleString(locale, { hour12: false })
}

export function formatSize(bytes: number | null | undefined): string {
  if (bytes == null) return '—'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`
}
