// Tag endpoints
import type { ApiClient } from './client'
import type { TagListResp } from '../types'

export type TagDeleteMode = 'detach' | 'purge'

/** filter is a tag-name regex (filtered server-side); the server responds 400 to invalid regexes */
export function listTags(client: ApiClient, filter?: string): Promise<TagListResp> {
  const suffix = filter ? `?filter=${encodeURIComponent(filter)}` : ''
  return client.get<TagListResp>(`/api/tags${suffix}`)
}

export function createTag(client: ApiClient, name: string, description: string): Promise<void> {
  return client.post('/api/tags', { name, description })
}

/** Edit a tag: an empty newName or one equal to the original name means no rename */
export function updateTag(client: ApiClient, name: string, newName: string, description: string): Promise<void> {
  const body: { description: string; new_name?: string } = { description }
  const trimmed = newName.trim()
  if (trimmed && trimmed !== name) body.new_name = trimmed
  return client.put(`/api/tags/${encodeURIComponent(name)}`, body)
}

export function deleteTag(client: ApiClient, name: string, mode: TagDeleteMode): Promise<void> {
  return client.del(`/api/tags/${encodeURIComponent(name)}?mode=${mode}`)
}
