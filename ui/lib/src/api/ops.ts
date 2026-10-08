// Ops endpoints: stats / doctor / import & export / version
import type { ApiClient } from './client'
import type {
  DoctorResp,
  EmbedBackfillResp,
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

export function exportBackup(client: ApiClient): Promise<Blob> {
  return client.getBlob('/api/export')
}

/** Import a backup JSON (an object, not a string); only allowed on an empty database, enforced server-side */
export function importBackup(client: ApiClient, dump: unknown): Promise<ImportResp> {
  return client.post<ImportResp>('/api/import', dump)
}

/**
 * Backfill a batch of semantic search vectors, returning processed / remaining (the UI loops
 * until remaining=0). With modelKey, drains that specific cache identity instead of the
 * active candidate's.
 */
export function backfillEmbeddings(client: ApiClient, modelKey?: string): Promise<EmbedBackfillResp> {
  return client.post<EmbedBackfillResp>('/api/embeddings/backfill', modelKey ? { model_key: modelKey } : {})
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
