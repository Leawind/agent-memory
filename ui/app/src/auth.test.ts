// 多身份令牌存储：localStorage 里的身份表与当前身份指针
import { describe, expect, it, vi, beforeEach } from 'vitest'
import {
  addIdentity,
  currentIdentityName,
  listIdentities,
  listIdentityNames,
  readStoredToken,
  removeIdentity,
  switchIdentity,
  tokenHint,
} from './auth'

describe('multi-identity token store', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  it('stores several identities and switches the current one', () => {
    addIdentity('admin', 'tok-a')
    addIdentity('viewer', 'tok-v')
    expect(listIdentityNames()).toEqual(['admin', 'viewer'])
    // addIdentity 切换当前身份
    expect(currentIdentityName()).toBe('viewer')
    expect(readStoredToken()).toBe('tok-v')

    switchIdentity('admin')
    expect(currentIdentityName()).toBe('admin')
    expect(readStoredToken()).toBe('tok-a')
  })

  it('re-adding an identity overwrites its token and keeps order', () => {
    addIdentity('admin', 'tok-a')
    addIdentity('viewer', 'tok-v')
    addIdentity('admin', 'tok-a2')
    expect(listIdentityNames()).toEqual(['admin', 'viewer'])
    expect(readStoredToken()).toBe('tok-a2')
  })

  it('removing the current identity clears the pointer, others keep it', () => {
    addIdentity('admin', 'tok-a')
    addIdentity('viewer', 'tok-v')
    switchIdentity('admin')

    removeIdentity('viewer')
    expect(listIdentityNames()).toEqual(['admin'])
    expect(currentIdentityName()).toBe('admin')

    removeIdentity('admin')
    expect(listIdentityNames()).toEqual([])
    expect(currentIdentityName()).toBe(null)
    expect(readStoredToken()).toBe('')
  })

  it('lists identities with ellipsis + tail token hints in insertion order', () => {
    expect(listIdentities()).toEqual([])
    addIdentity('admin', 'abcdefghijklmnop')
    addIdentity('viewer', 'qwertyuiop')
    expect(listIdentities()).toEqual([
      { name: 'admin', hint: '…mnop' },
      { name: 'viewer', hint: '…uiop' },
    ])
  })

  it('masks tokens as ellipsis plus the last four characters', () => {
    expect(tokenHint('')).toBe('')
    expect(tokenHint('abc')).toBe('…abc')
    expect(tokenHint('sk_abcdefghijklmnop')).toBe('…mnop')
  })

  it('adopts a legacy single token under its identity name after first whoami success', async () => {
    // 模拟旧版升级：只有单 token 键
    localStorage.setItem('agent-memory-token', 'tok-legacy')
    expect(readStoredToken()).toBe('tok-legacy')

    // fetchWhoAmI 成功返回身份名后收编（stub 全局 fetch）
    const fetchMock = vi.fn(() =>
      Promise.resolve({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ name: 'admin', mode: 'token', permissions: { read: true } }),
      }),
    )
    vi.stubGlobal('fetch', fetchMock)
    const { fetchWhoAmI } = await import('./auth')
    const r = await fetchWhoAmI()
    expect(r.ok).toBe(true)

    expect(localStorage.getItem('agent-memory-token')).toBe(null)
    expect(listIdentityNames()).toEqual(['admin'])
    expect(currentIdentityName()).toBe('admin')
    expect(readStoredToken()).toBe('tok-legacy')
  })
})
