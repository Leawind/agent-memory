// Memory endpoints (paths map 1:1 to the server's /api/memories)
import type { ApiClient } from './client'
import type { MemoryFull, MemoryListResp, MemorySearchResp } from '../types'

/** Input for creating a memory (tags are the target set) */
export interface MemoryCreateInput {
  summary: string
  content: string
  tags: string[]
  /** The panel's tag select is allow-create: names the user typed are deliberate, so auto-create them */
  create_missing_tags?: boolean
}

/** Input for editing a memory: tags are expressed as add/remove sets (server convention add_tags / remove_tags) */
export interface MemoryUpdateInput {
  summary?: string
  content?: string
  add_tags?: string[]
  remove_tags?: string[]
  /** Same semantics as in MemoryCreateInput */
  create_missing_tags?: boolean
}

export function getMemory(client: ApiClient, id: string): Promise<MemoryFull> {
  return client.get<MemoryFull>(`/api/memories/${encodeURIComponent(id)}`)
}

export function createMemory(client: ApiClient, input: MemoryCreateInput): Promise<MemoryFull> {
  return client.post<MemoryFull>('/api/memories', input)
}

export function updateMemory(client: ApiClient, id: string, input: MemoryUpdateInput): Promise<MemoryFull> {
  return client.put<MemoryFull>(`/api/memories/${encodeURIComponent(id)}`, input)
}

export function deleteMemory(client: ApiClient, id: string): Promise<void> {
  return client.del(`/api/memories/${encodeURIComponent(id)}`)
}

export function fetchMemoriesPage(client: ApiClient, queryString: string): Promise<MemoryListResp | MemorySearchResp> {
  return client.get(`/api/memories?${queryString}`)
}
