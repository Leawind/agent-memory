// AdminPanel 家族的共享登记表与纯函数：能力清单、预设、展示辅助、错误包装。
// CAPS 的 key 与服务端 auth::Cap 的 JSON 键一致（唯一登记表）。
import { t } from '../../i18n'
import { toastError, toastSuccess } from '../../toast'

export const CAPS = [
  { key: 'read', labelKey: 'capRead' },
  { key: 'create', labelKey: 'capCreate' },
  { key: 'update', labelKey: 'capUpdate' },
  { key: 'delete', labelKey: 'capDelete' },
  { key: 'tag_manage', labelKey: 'capTagManage' },
  { key: 'admin', labelKey: 'capAdmin' },
] as const

export interface IdentityRow {
  name: string
  token_hint: string
  permissions: Record<string, boolean>
  created_at: number
}

export type PresetKey = 'admin' | 'member' | 'viewer' | 'custom'

export function emptyCaps(): Record<string, boolean> {
  return Object.fromEntries(CAPS.map((c) => [c.key, false]))
}

/** 一个能力都没有（匿名能力集为空 = 匿名访问被整体拒绝）。 */
export function isEmptyCaps(p?: Record<string, boolean> | null): boolean {
  return !p || CAPS.every((c) => p[c.key] !== true)
}

export const PRESETS: Record<string, string[]> = {
  admin: CAPS.map((c) => c.key),
  member: ['read', 'create', 'update', 'tag_manage'],
  viewer: ['read'],
}

export function capLabel(key: string): string {
  const found = CAPS.find((c) => c.key === key)
  return found ? t(`access.${found.labelKey}`) : key
}

export function enabledCaps(row: IdentityRow): string[] {
  return CAPS.map((c) => c.key).filter((k) => row.permissions?.[k] === true)
}

// token 只存哈希，列表只有尾缀提示
export function maskToken(hint: string): string {
  return hint ? `…${hint}` : '—'
}

/** 面板层统一的错误包装：动作失败 toast 并返回 undefined，成功可选 toast。 */
export async function run<T>(action: () => Promise<T>, successMsg?: string): Promise<T | undefined> {
  try {
    const out = await action()
    if (successMsg) toastSuccess(successMsg)
    return out
  } catch (e) {
    toastError(e instanceof Error ? e.message : String(e))
    return undefined
  }
}
