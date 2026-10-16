// Memory endpoints (paths map 1:1 to the server's /api/memories)
import type { ApiClient } from './client'
import type {
  MemoryCreateResp,
  MemoryFull,
  MemoryListResp,
  MemoryMergeResp,
  MemorySearchResp,
  MemoryUpdateResp,
} from '../types'

/** One tags object entry: identifier word, number, or null for a bare tag */
export type TagValue = string | number | null

/** Input for creating a memory: the tags object maps each tag key to its value (null = bare tag) */
export interface MemoryCreateInput {
  summary: string
  /** Optional body: an empty string stores a summary-only memory */
  content: string
  tags: Record<string, TagValue>
  /** The panel's tag select is allow-create: names the user typed are deliberate, so auto-create them */
  create_missing_tags?: boolean
}

/** Input for editing a memory: incremental — add_tags upserts keys, remove_tags drops keys by name */
export interface MemoryUpdateInput {
  summary?: string
  content?: string
  add_tags?: Record<string, TagValue>
  remove_tags?: string[]
  /** Same semantics as in MemoryCreateInput */
  create_missing_tags?: boolean
}

export function getMemory(client: ApiClient, id: string): Promise<MemoryFull> {
  return client.get<MemoryFull>(`/api/memories/${encodeURIComponent(id)}`)
}

/** Create is echo-free: only the new id, timestamp and sparse classifications come back */
export function createMemory(client: ApiClient, input: MemoryCreateInput): Promise<MemoryCreateResp> {
  return client.post<MemoryCreateResp>('/api/memories', input)
}

/** Update is echo-free: only the changed flag and sparse tag classifications come back */
export function updateMemory(client: ApiClient, id: string, input: MemoryUpdateInput): Promise<MemoryUpdateResp> {
  return client.put<MemoryUpdateResp>(`/api/memories/${encodeURIComponent(id)}`, input)
}

export function deleteMemory(client: ApiClient, id: string): Promise<void> {
  return client.del(`/api/memories/${encodeURIComponent(id)}`)
}

/** Merge absorbs `source` into `target`: source is deleted, target keeps its id. Echo-free:
 * a clean merge answers with an empty object (only reference reports carry facts). */
export function mergeMemory(client: ApiClient, target: string, source: string): Promise<MemoryMergeResp> {
  return client.post('/api/memories/merge', { target, source })
}

export function fetchMemoriesPage(client: ApiClient, queryString: string): Promise<MemoryListResp | MemorySearchResp> {
  return client.get(`/api/memories?${queryString}`)
}
