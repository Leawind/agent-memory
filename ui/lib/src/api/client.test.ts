// API 客户端测试：baseUrl 前缀、自定义 fetch 注入、错误处理与 JSON 解析约定
import { afterEach, describe, expect, it, vi } from 'vitest'
import { createApiClient } from './client'
import type { ResolvedMemoryUIConfig } from '../config'

function makeConfig(fetchImpl: typeof fetch, baseUrl = ''): ResolvedMemoryUIConfig {
  return { baseUrl, fetch: fetchImpl, defaultPageSize: 20 }
}

function mockFetch(status: number, body: unknown, raw = false): ReturnType<typeof vi.fn> {
  const text = raw ? String(body) : body === null ? '' : JSON.stringify(body)
  return vi.fn().mockResolvedValue({
    ok: status >= 200 && status < 300,
    status,
    text: () => Promise.resolve(text),
    blob: () => Promise.resolve(new Blob([text])),
  })
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('api client', () => {
  it('prefixes baseUrl and passes the custom fetch through', async () => {
    const fetchMock = mockFetch(200, { ok: true })
    const client = createApiClient(makeConfig(fetchMock as unknown as typeof fetch, 'http://127.0.0.1:8899'))
    await client.get('/api/stats')
    expect(fetchMock).toHaveBeenCalledWith('http://127.0.0.1:8899/api/stats', expect.anything())
  })

  it('returns parsed JSON on 200', async () => {
    const client = createApiClient(makeConfig(mockFetch(200, { total: 1, memories: [] }) as unknown as typeof fetch))
    const data = await client.get<{ total: number }>('/api/memories')
    expect(data.total).toBe(1)
  })

  it('returns null body for empty 2xx responses', async () => {
    const client = createApiClient(makeConfig(mockFetch(202, null) as unknown as typeof fetch))
    const data = await client.get<null>('/api/whatever')
    expect(data).toBeNull()
  })

  it('throws the server error message on 4xx', async () => {
    const client = createApiClient(
      makeConfig(mockFetch(404, { error: "memory 'm9' not found" }) as unknown as typeof fetch),
    )
    await expect(client.get('/api/memories/m9')).rejects.toThrow("memory 'm9' not found")
  })

  it('falls back to a generic message when body has no error field', async () => {
    const client = createApiClient(makeConfig(mockFetch(500, 'oops') as unknown as typeof fetch))
    await expect(client.get('/api/stats')).rejects.toThrow('请求失败 (HTTP 500)')
  })

  it('does not throw on 2xx with unparseable body', async () => {
    const client = createApiClient(makeConfig(mockFetch(200, 'not json', true) as unknown as typeof fetch))
    const data = await client.get<null>('/api/stats')
    expect(data).toBeNull()
  })

  it('post serializes the body as JSON', async () => {
    const fetchMock = mockFetch(200, null)
    const client = createApiClient(makeConfig(fetchMock as unknown as typeof fetch))
    await client.post('/api/memories', { summary: 's' })
    const [, init] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(init.method).toBe('POST')
    expect(init.body).toBe('{"summary":"s"}')
  })

  it('getBlob returns the binary body', async () => {
    const client = createApiClient(makeConfig(mockFetch(200, 'export-data') as unknown as typeof fetch))
    const blob = await client.getBlob('/api/export')
    expect(await blob.text()).toBe('"export-data"')
  })
})
