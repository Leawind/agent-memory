// API 访问层：路径 = baseUrl + path，传输通道可被宿主替换（注入带鉴权的 fetch）。
// 统一错误处理约定：服务端业务错误以 {"error": "..."} 回报，非 2xx 时取出抛出。
import { useMemoryConfig } from '../config'
import type { ResolvedMemoryUIConfig } from '../config'
import { t } from '../i18n'

export interface ApiClient {
  get: <T = void>(path: string) => Promise<T>
  post: <T = void>(path: string, body?: unknown) => Promise<T>
  put: <T = void>(path: string, body?: unknown) => Promise<T>
  del: <T = void>(path: string) => Promise<T>
  /** 二进制下载（如导出备份）；非 2xx 抛错，消息取服务端 error 字段 */
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
        /* 非 JSON 响应体，用兜底文案 */
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

/** 在组件 setup 中取配置并创建 API 客户端（同一组件内多次调用各自新建，成本可忽略）。 */
export function useApiClient(): ApiClient {
  return createApiClient(useMemoryConfig())
}
