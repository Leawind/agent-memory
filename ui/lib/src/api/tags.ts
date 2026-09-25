// 标签端点
import type { ApiClient } from './client'
import type { TagListResp } from '../types'

export type TagDeleteMode = 'detach' | 'purge'

export function listTags(client: ApiClient): Promise<TagListResp> {
  return client.get<TagListResp>('/api/tags')
}

export function createTag(client: ApiClient, name: string, description: string): Promise<void> {
  return client.post('/api/tags', { name, description })
}

/** 编辑标签：newName 为空或与原名相同表示不改名 */
export function updateTag(client: ApiClient, name: string, newName: string, description: string): Promise<void> {
  const body: { description: string; new_name?: string } = { description }
  const trimmed = newName.trim()
  if (trimmed && trimmed !== name) body.new_name = trimmed
  return client.put(`/api/tags/${encodeURIComponent(name)}`, body)
}

export function deleteTag(client: ApiClient, name: string, mode: TagDeleteMode): Promise<void> {
  return client.del(`/api/tags/${encodeURIComponent(name)}?mode=${mode}`)
}
