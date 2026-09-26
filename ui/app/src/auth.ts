// 访问令牌与身份管理：一个浏览器可保存多个身份的 token（localStorage），
// 可切换当前身份，之后所有请求自动携带其 Bearer 头。
// 与 lib 的 provideMemoryUI({ fetch }) 注入点配合，lib 自身不感知鉴权。
import type { WhoAmI } from '@agent-memory/ui'

export type { WhoAmI }

// 身份表：{ 身份名: token }；当前身份名单独存。
// 旧版单 token 键（agent-memory-token）在首次 whoami 成功后收编进身份表并清除。
const IDENTITY_MAP_KEY = 'agent-memory-identities'
const CURRENT_KEY = 'agent-memory-identity'
const LEGACY_TOKEN_KEY = 'agent-memory-token'

function readMap(): Record<string, string> {
  try {
    const raw = JSON.parse(localStorage.getItem(IDENTITY_MAP_KEY) ?? '{}')
    if (raw && typeof raw === 'object' && !Array.isArray(raw)) {
      return Object.fromEntries(Object.entries(raw).filter(([, v]) => typeof v === 'string' && v !== '')) as Record<
        string,
        string
      >
    }
  } catch {
    /* 隐私模式等场景存不了就算了 */
  }
  return {}
}

function writeMap(map: Record<string, string>): void {
  try {
    localStorage.setItem(IDENTITY_MAP_KEY, JSON.stringify(map))
  } catch {
    /* 忽略 */
  }
}

/** 本浏览器保存的全部身份名（按添加顺序） */
export function listIdentityNames(): string[] {
  return Object.keys(readMap())
}

/** token 首尾提示：前 4 位 + ... + 后 4 位，过短则原样返回 */
export function tokenHint(token: string): string {
  return token.length > 8 ? `${token.slice(0, 4)}...${token.slice(-4)}` : token
}

/** 全部身份及 token 首尾提示（按添加顺序），供身份下拉展示 */
export function listIdentities(): { name: string; hint: string }[] {
  return Object.entries(readMap()).map(([name, token]) => ({ name, hint: tokenHint(token) }))
}

/** 当前生效的身份名；null 表示未选择（开放模式或尚未添加） */
export function currentIdentityName(): string | null {
  try {
    return localStorage.getItem(CURRENT_KEY)
  } catch {
    return null
  }
}

function readLegacyToken(): string {
  try {
    return localStorage.getItem(LEGACY_TOKEN_KEY) ?? ''
  } catch {
    return ''
  }
}

/** 当前请求应携带的 token（无身份时回退旧版单 token，便于升级路径收编） */
export function readStoredToken(): string {
  const name = currentIdentityName()
  const map = readMap()
  if (name && map[name]) return map[name]
  return readLegacyToken()
}

/** 登记一个身份并切换过去；同时清掉旧版单 token 键 */
export function addIdentity(name: string, token: string): void {
  const map = readMap()
  map[name] = token
  writeMap(map)
  try {
    localStorage.setItem(CURRENT_KEY, name)
    localStorage.removeItem(LEGACY_TOKEN_KEY)
  } catch {
    /* 忽略 */
  }
}

/** 切换当前身份（不校验存在性，调用方从 listIdentityNames 取值） */
export function switchIdentity(name: string): void {
  try {
    localStorage.setItem(CURRENT_KEY, name)
  } catch {
    /* 忽略 */
  }
}

/** 移除一个身份的本地 token；若是当前身份，当前身份置空 */
export function removeIdentity(name: string): void {
  const map = readMap()
  delete map[name]
  writeMap(map)
  if (currentIdentityName() === name) {
    try {
      localStorage.removeItem(CURRENT_KEY)
    } catch {
      /* 忽略 */
    }
  }
}

/** 任意请求收到 401 时广播，App 壳监听后移除失效身份并弹出令牌输入框 */
export const UNAUTHORIZED_EVENT = 'agent-memory-unauthorized'

/** 带 Authorization 的 fetch 包装（token 缺省时行为与原生一致） */
export function authFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  const token = readStoredToken()
  if (!token) return globalThis.fetch(input, init)
  const headers = new Headers(init?.headers)
  headers.set('Authorization', `Bearer ${token}`)
  return globalThis.fetch(input, { ...init, headers }).then((res) => {
    if (res.status === 401 && !String(input).includes('/api/whoami')) {
      window.dispatchEvent(new CustomEvent(UNAUTHORIZED_EVENT))
    }
    return res
  })
}

export type WhoAmIResult = { ok: true; who: WhoAmI } | { ok: false; needToken: boolean }

/** 校验身份并取摘要；传 candidateToken 时校验的是候选 token（添加身份前先验证）。
 * 未知响应形态（如嵌入其他宿主）不触发令牌弹窗。 */
export async function fetchWhoAmI(candidateToken?: string): Promise<WhoAmIResult> {
  const token = candidateToken ?? readStoredToken()
  try {
    const res = await globalThis.fetch('/api/whoami', {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    })
    if (res.status === 401) return { ok: false, needToken: true }
    if (!res.ok) return { ok: false, needToken: false }
    const data = (await res.json()) as Partial<WhoAmI> | null
    if (data && typeof data === 'object' && typeof data.mode === 'string' && typeof data.name === 'string') {
      // 旧版单 token 首次验证成功：以其身份名收编进身份表
      if (!candidateToken && readLegacyToken() && !readMap()[data.name]) {
        addIdentity(data.name, readLegacyToken())
      }
      return { ok: true, who: data as WhoAmI }
    }
    return { ok: false, needToken: false }
  } catch {
    return { ok: false, needToken: false }
  }
}

export function can(who: WhoAmI | null, cap: string): boolean {
  return !!who && who.permissions?.[cap] === true
}
