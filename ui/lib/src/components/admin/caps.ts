// Shared registries and pure functions for the AdminPanel family: capability list, presets,
// display helpers, error wrapping.
// CAPS keys match the JSON keys of the server's auth::Cap (the single source of truth).
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

/** Not a single capability (an empty anonymous capability set = anonymous access denied entirely). */
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

// Tokens are stored hashed only; the list shows just the suffix hint
export function maskToken(hint: string): string {
  return hint ? `…${hint}` : '—'
}

/** Unified error wrapper at the panel layer: on failure toast and return undefined, optionally toast on success. */
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
