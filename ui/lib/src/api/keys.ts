// Key registry endpoints (paths map 1:1 to the server's /api/keys)
import type { ApiClient } from './client'
import type { KeyListResp } from '../types'

export type KeyType = 'enum' | 'number'

/** Set is echo-free: the caller just stated name/type/multi/description, so a clean set answers with an empty object */
export interface KeySetInput {
  name: string
  key_type?: KeyType
  multi?: boolean
  description?: string
}

export function listKeys(client: ApiClient): Promise<KeyListResp> {
  return client.get<KeyListResp>('/api/keys')
}

/** Create or reconfigure a key. Changes apply to future writes; existing data is grandfathered. */
export function setKey(client: ApiClient, input: KeySetInput): Promise<Record<string, never>> {
  return client.put('/api/keys', input)
}

/** Delete is echo-free: only the detached value-tag count comes back */
export function deleteKey(client: ApiClient, name: string): Promise<{ memories_updated: number }> {
  return client.del(`/api/keys/${encodeURIComponent(name)}`)
}
