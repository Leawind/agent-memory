// 访问令牌管理：一次输入存 localStorage，之后所有请求自动携带 Bearer 头。
// 与 lib 的 provideMemoryUI({ fetch }) 注入点配合，lib 自身不感知鉴权。
export interface WhoAmI {
  name: string
  mode: 'open' | 'token'
  permissions: Record<string, boolean>
}

const TOKEN_KEY = 'agent-memory-token'

export function readStoredToken(): string {
  try {
    return localStorage.getItem(TOKEN_KEY) ?? ''
  } catch {
    return ''
  }
}

export function storeToken(token: string): void {
  try {
    localStorage.setItem(TOKEN_KEY, token)
  } catch {
    /* 隐私模式等场景存不了就算了 */
  }
}

export function clearToken(): void {
  try {
    localStorage.removeItem(TOKEN_KEY)
  } catch {
    /* 忽略 */
  }
}

/** 任意请求收到 401 时广播，App 壳监听后弹出令牌输入框 */
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

/** 校验当前 token 并取身份摘要；未知响应形态（如嵌入其他宿主）不触发令牌弹窗 */
export async function fetchWhoAmI(): Promise<WhoAmIResult> {
  try {
    const res = await globalThis.fetch('/api/whoami', {
      headers: authHeaders(),
    })
    if (res.status === 401) return { ok: false, needToken: true }
    if (!res.ok) return { ok: false, needToken: false }
    const data = (await res.json()) as Partial<WhoAmI> | null
    if (data && typeof data === 'object' && typeof data.mode === 'string' && typeof data.name === 'string') {
      return { ok: true, who: data as WhoAmI }
    }
    return { ok: false, needToken: false }
  } catch {
    return { ok: false, needToken: false }
  }
}

function authHeaders(): Record<string, string> {
  const token = readStoredToken()
  return token ? { Authorization: `Bearer ${token}` } : {}
}

export function can(who: WhoAmI | null, cap: string): boolean {
  return !!who && who.permissions?.[cap] === true
}
