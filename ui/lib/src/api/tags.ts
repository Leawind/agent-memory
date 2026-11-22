// Tag and scope endpoints
import type { ApiClient } from './client'
import type { TagListResp } from '../types'

export type TagDeleteMode = 'detach' | 'purge'

/** filter is a name regex narrowing both the tag and the scope lists, filtered server-side; the
 *  server responds 400 to invalid regexes. */
export function listTags(client: ApiClient, filter?: string): Promise<TagListResp> {
  const suffix = filter ? `?filter=${encodeURIComponent(filter)}` : ''
  return client.get<TagListResp>(`/api/tags${suffix}`)
}

/** Create is echo-free: an empty object on a clean create, the case-variant hint when one exists.
 *  The name may be scoped (`system::pc1sw0`); then the scope must already exist. */
export function createTag(
  client: ApiClient,
  name: string,
  description: string,
): Promise<{ similar_existing?: string; note?: string }> {
  return client.post('/api/tags', { name, description })
}

/** Edit a tag: an empty newName or one equal to the original name means no rename. The
 *  required-on-every-write contract lives on scopes (PUT /api/scopes), not here. Echo-free: only
 *  the changed flags come back */
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

/** Create a scope, optionally arming its write contract in the same call; echo-free beyond the
 *  fact of the create. */
export function createScope(
  client: ApiClient,
  name: string,
  description: string,
  required?: boolean,
): Promise<{ created: boolean }> {
  return client.post('/api/scopes', {
    name,
    description,
    ...(required === undefined ? {} : { required }),
  })
}

/** Edit a scope: the name is the identity and never changes, so only the description and the
 *  required flag travel, each only when actually provided. */
export function updateScope(
  client: ApiClient,
  name: string,
  description?: string,
  required?: boolean,
): Promise<{ description_updated: boolean; required_updated: boolean }> {
  const body: { description?: string; required?: boolean } = {}
  if (description !== undefined) body.description = description
  if (required !== undefined) body.required = required
  return client.put(`/api/scopes/${encodeURIComponent(name)}`, body)
}

/** The scope listing answers with the same whole shape as GET /api/tags (tags + scopes
 *  together), so hosts reuse one parser. */
export function listScopes(client: ApiClient): Promise<TagListResp> {
  return client.get<TagListResp>('/api/scopes')
}
