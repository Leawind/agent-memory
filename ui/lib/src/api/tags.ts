// Tag endpoints
import type { ApiClient } from './client'
import type { TagListResp } from '../types'

export type TagDeleteMode = 'detach' | 'purge'

/** filter is a tag-name regex (filtered server-side); the server responds 400 to invalid regexes.
 *  all=true adds tags that only ever appear as edge targets (the value half of key:value edges). */
export function listTags(client: ApiClient, filter?: string, all?: boolean): Promise<TagListResp> {
  const params = new URLSearchParams()
  if (filter) params.set('filter', filter)
  if (all) params.set('all', 'true')
  const suffix = params.size > 0 ? `?${params}` : ''
  return client.get<TagListResp>(`/api/tags${suffix}`)
}

/** Create is echo-free: an empty object on a clean create, the case-variant hint when one exists */
export function createTag(
  client: ApiClient,
  name: string,
  description: string,
): Promise<{ similar_existing?: string; note?: string }> {
  return client.post('/api/tags', { name, description })
}

/** Edit a tag: an empty newName or one equal to the original name means no rename. Echo-free: only the changed flags come back */
export function updateTag(
  client: ApiClient,
  name: string,
  newName: string,
  description: string,
): Promise<{ renamed: boolean; description_updated: boolean }> {
  const body: { description: string; new_name?: string } = { description }
  const trimmed = newName.trim()
  if (trimmed && trimmed !== name) body.new_name = trimmed
  return client.put(`/api/tags/${encodeURIComponent(name)}`, body)
}

/** Delete is echo-free: only the impact count comes back (dry_run previews via memories_affected) */
export function deleteTag(
  client: ApiClient,
  name: string,
  mode: TagDeleteMode,
): Promise<{ memories_updated?: number; memories_deleted?: number }> {
  return client.del(`/api/tags/${encodeURIComponent(name)}?mode=${mode}`)
}
