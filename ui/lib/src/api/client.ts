// API access layer: path = baseUrl + path; the transport channel can be replaced by the host
// (inject a fetch with auth).
// Unified error handling convention: server-side business errors are reported as {"error": "..."},
// extracted and thrown on non-2xx.
import { useMemoryConfig } from '../config'
import type { ResolvedMemoryUIConfig } from '../config'
import { t } from '../i18n'

export interface ApiClient {
  get: <T = void>(path: string) => Promise<T>
  post: <T = void>(path: string, body?: unknown) => Promise<T>
  put: <T = void>(path: string, body?: unknown) => Promise<T>
  del: <T = void>(path: string) => Promise<T>
  /** Binary download (e.g. backup export); throws on non-2xx with the message taken from the server's error field */
  getBlob: (path: string) => Promise<Blob>
}

export function createApiClient(config: ResolvedMemoryUIConfig): ApiClient {
  async function api<T = void>(path: string, options: RequestInit = {}): Promise<T> {
    const res = await config.fetch(config.baseUrl + path, {
      headers: { 'Content-Type': 'application/json' },
      ...options,
    })
    const text = await res.text()
    let data: unknown = null
    try {
      data = text ? JSON.parse(text) : null
    } catch {
      data = null
    }
    if (!res.ok) {
      const msg = (data as { error?: string } | null)?.error ?? t('errors.http', { status: res.status })
      throw new Error(msg)
    }
    return data as T
  }

  async function getBlob(path: string): Promise<Blob> {
    const res = await config.fetch(config.baseUrl + path)
    if (!res.ok) {
      const text = await res.text().catch(() => '')
      let msg = t('errors.http', { status: res.status })
      try {
        msg = (JSON.parse(text) as { error?: string })?.error ?? msg
      } catch {
        /* Non-JSON response body, fall back to the default message */
      }
      throw new Error(msg)
    }
    return res.blob()
  }

  return {
    get: <T = void>(path: string) => api<T>(path),
    post: <T = void>(path: string, body?: unknown) =>
      api<T>(path, { method: 'POST', body: JSON.stringify(body ?? {}) }),
    put: <T = void>(path: string, body?: unknown) => api<T>(path, { method: 'PUT', body: JSON.stringify(body ?? {}) }),
    del: <T = void>(path: string) => api<T>(path, { method: 'DELETE' }),
    getBlob,
  }
}

/** Grab the config inside a component setup and create an API client (each call creates a fresh one; the cost is negligible). */
export function useApiClient(): ApiClient {
  return createApiClient(useMemoryConfig())
}
