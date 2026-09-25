// 记忆端点（路径与服务端 /api/memories 一一对应）
import type { ApiClient } from './client'
import type { MemoryFull, MemoryListResp, MemorySearchResp } from '../types'

/** 新建记忆的输入（标签为目标集合） */
export interface MemoryCreateInput {
  summary: string
  content: string
  tags: string[]
}

/** 编辑记忆的输入：标签以增删集合表达（服务端约定 add_tags / remove_tags） */
export interface MemoryUpdateInput {
  summary?: string
  content?: string
  add_tags?: string[]
  remove_tags?: string[]
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
