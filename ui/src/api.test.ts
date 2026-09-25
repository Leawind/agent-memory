// api 封装层测试：mock fetch，验证错误处理与 JSON 解析约定
import { afterEach, describe, expect, it, vi } from 'vitest'
import { get } from './api'

function mockFetch(status: number, body: unknown, raw = false): void {
  const text = raw ? String(body) : body === null ? '' : JSON.stringify(body)
  globalThis.fetch = vi.fn().mockResolvedValue({
    ok: status >= 200 && status < 300,
    status,
    text: () => Promise.resolve(text),
  }) as unknown as typeof fetch
}

afterEach(() => {
  vi.restoreAllMocks()
})

describe('api wrapper', () => {
  it('returns parsed JSON on 200', async () => {
    mockFetch(200, { total: 1, memories: [] })
    const data = await get<{ total: number }>('/api/memories')
    expect(data.total).toBe(1)
  })

  it('returns null body for empty 2xx responses', async () => {
    mockFetch(202, null)
    const data = await get<null>('/api/whatever')
    expect(data).toBeNull()
  })

  it('throws the server error message on 4xx', async () => {
    mockFetch(404, { error: "memory 'm9' not found" })
    await expect(get('/api/memories/m9')).rejects.toThrow("memory 'm9' not found")
  })

  it('falls back to a generic message when body has no error field', async () => {
    mockFetch(500, 'oops')
    await expect(get('/api/stats')).rejects.toThrow('请求失败 (HTTP 500)')
  })

  it('does not throw on 2xx with unparseable body', async () => {
    mockFetch(200, 'not json', true)
    const data = await get<null>('/api/stats')
    expect(data).toBeNull()
  })
})
