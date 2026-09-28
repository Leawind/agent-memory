// Access tokens and identity management: one browser can store tokens for multiple identities
// (localStorage), switch the current identity, and all subsequent requests automatically carry
// its Bearer header.
// Works with the lib's provideMemoryUI({ fetch }) injection point; the lib itself stays
// auth-unaware.
import type { WhoAmI } from '@agent-memory/ui'

export type { WhoAmI }

// Identity table: { identity name: token }; the current identity name is stored separately.
// The legacy single-token key (agent-memory-token) is folded into the identity table and cleared
// after the first successful whoami.
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
    /* If storage is unavailable (e.g. private mode), just give up */
  }
  return {}
}

function writeMap(map: Record<string, string>): void {
  try {
    localStorage.setItem(IDENTITY_MAP_KEY, JSON.stringify(map))
  } catch {
    /* ignore */
  }
}

/** All identity names saved in this browser (in insertion order) */
export function listIdentityNames(): string[] {
  return Object.keys(readMap())
}

/** Token suffix hint: ellipsis + last 4 characters, same style as the admin UI (lib's maskToken).
 * Tokens have a fixed sk_ prefix with no distinguishing head, which is never exposed. */
export function tokenHint(token: string): string {
  return token ? `…${token.slice(-4)}` : ''
}

/** All identities with token suffix hints (in insertion order), for the identity dropdown */
export function listIdentities(): { name: string; hint: string }[] {
  return Object.entries(readMap()).map(([name, token]) => ({ name, hint: tokenHint(token) }))
}

/** The currently active identity name; null means none selected (open mode or nothing added yet) */
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

/** The token the current request should carry (falls back to the legacy single token when no identity is selected, easing the upgrade path) */
export function readStoredToken(): string {
  const name = currentIdentityName()
  const map = readMap()
  if (name && map[name]) return map[name]
  return readLegacyToken()
}

/** Register an identity and switch to it; also clears the legacy single-token key */
export function addIdentity(name: string, token: string): void {
  const map = readMap()
  map[name] = token
  writeMap(map)
  try {
    localStorage.setItem(CURRENT_KEY, name)
    localStorage.removeItem(LEGACY_TOKEN_KEY)
  } catch {
    /* ignore */
  }
}

/** Switch the current identity (no existence check; the caller takes values from listIdentityNames) */
export function switchIdentity(name: string): void {
  try {
    localStorage.setItem(CURRENT_KEY, name)
  } catch {
    /* ignore */
  }
}

/** Remove an identity's local token; if it is the current one, the current selection is cleared */
export function removeIdentity(name: string): void {
  const map = readMap()
  delete map[name]
  writeMap(map)
  if (currentIdentityName() === name) {
    try {
      localStorage.removeItem(CURRENT_KEY)
    } catch {
      /* ignore */
    }
  }
}

/** Broadcast on any 401 response; the App shell listens, removes the invalid identity and pops the token input dialog */
export const UNAUTHORIZED_EVENT = 'agent-memory-unauthorized'

/** fetch wrapper carrying Authorization (behaves identically to native fetch when no token is set) */
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

/** Validate the identity and fetch its summary; when candidateToken is given, the candidate token is validated (verify before adding an identity).
 * Unknown response shapes (e.g. embedded in another host) do not trigger the token dialog. */
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
      // Legacy single token validated for the first time: fold it into the identity table under its identity name
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
