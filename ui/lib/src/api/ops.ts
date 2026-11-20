// Ops endpoints: stats / doctor / import & export / version
import type { ApiClient } from './client'
import type {
  DoctorResp,
  EmbedBackfillResp,
  EmbedBackfillRunsResp,
  EmbedTestResp,
  HealthInfo,
  ImportResp,
  RerankTestResp,
  StatsInfo,
  VectorCacheDeleteResp,
  VectorCachesResp,
} from '../types'

export function getStats(client: ApiClient): Promise<StatsInfo> {
  return client.get<StatsInfo>('/api/stats')
}

export function getHealth(client: ApiClient): Promise<HealthInfo> {
  return client.get<HealthInfo>('/health')
}

export function runDoctorRemote(client: ApiClient): Promise<DoctorResp> {
  return client.get<DoctorResp>('/api/doctor')
}

/** Download the backup JSON; with `predicate`, only memories matching the tag expression are exported */
export function exportBackup(client: ApiClient, predicate?: string): Promise<Blob> {
  const suffix = predicate ? `?predicate=${encodeURIComponent(predicate)}` : ''
  return client.getBlob(`/api/export${suffix}`)
}

/**
 * Import a backup JSON (an object, not a string). Restore is the default and targets an empty
 * database (enforced server-side); `merge` imports incrementally into a populated database,
 * skipping verbatim duplicates. `predicate` limits the import to matching memories, evaluated
 * inside the file.
 */
export function importBackup(
  client: ApiClient,
  dump: unknown,
  options?: { merge?: boolean; predicate?: string },
): Promise<ImportResp> {
  const params = new URLSearchParams()
  if (options?.merge) params.set('merge', '1')
  if (options?.predicate) params.set('predicate', options.predicate)
  const suffix = params.size > 0 ? `?${params}` : ''
  return client.post<ImportResp>(`/api/import${suffix}`, dump)
}

/**
 * Start (or join) a backfill run for one model. The run drains in the background under the
 * server's per-model single-flight rule; progress is read through `listBackfillRuns`.
 */
export function startBackfill(client: ApiClient, modelKey: string): Promise<EmbedBackfillResp> {
  return client.post<EmbedBackfillResp>('/api/embeddings/backfill', { model_key: modelKey })
}

/** Active backfill runs for every model — the page shows them regardless of who started them */
export function listBackfillRuns(client: ApiClient): Promise<EmbedBackfillRunsResp> {
  return client.get<EmbedBackfillRunsResp>('/api/embeddings/backfill')
}

/** Ask a running backfill to stop after its batch in flight */
export function cancelBackfill(client: ApiClient, modelKey: string): Promise<{ cancelled: boolean }> {
  return client.post<{ cancelled: boolean }>('/api/embeddings/backfill/cancel', { model_key: modelKey })
}

/**
 * Run a connectivity test. With `entries`, exactly those candidates are probed (in request order,
 * one verdict per position) — the admin editor verifies what is on screen, including a candidate
 * that is disabled or not saved yet. Without it, every enabled candidate stored server-side is
 * probed: the list the search-time failover walks.
 */
export function testEmbeddings(client: ApiClient, entries?: unknown[]): Promise<EmbedTestResp> {
  return client.post<EmbedTestResp>('/api/embeddings/test', entries ? { entries } : {})
}

/** Run a one-document rerank against reranker candidates (same `entries` contract as testEmbeddings) */
export function testRerankers(client: ApiClient, entries?: unknown[]): Promise<RerankTestResp> {
  return client.post<RerankTestResp>('/api/rerank/test', entries ? { entries } : {})
}

/** List the vector caches per identity (rows with embedded/pending, configured or leftover) */
export function listVectorCaches(client: ApiClient): Promise<VectorCachesResp> {
  return client.get<VectorCachesResp>('/api/embeddings/caches')
}

/** Delete one cache identity's vectors outright (they go pending for backfill afterwards) */
export function deleteVectorCache(client: ApiClient, modelKey: string): Promise<VectorCacheDeleteResp> {
  return client.del<VectorCacheDeleteResp>('/api/embeddings/caches', { model_key: modelKey })
}
