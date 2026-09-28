// Ops endpoints: stats / doctor / import & export / version
import type { ApiClient } from './client'
import type { DoctorResp, EmbedBackfillResp, EmbedTestResp, HealthInfo, ImportResp, StatsInfo } from '../types'

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

/** Backfill a batch of semantic search vectors, returning processed / remaining (the UI loops until remaining=0) */
export function backfillEmbeddings(client: ApiClient): Promise<EmbedBackfillResp> {
  return client.post<EmbedBackfillResp>('/api/embeddings/backfill', {})
}

/** Run a connectivity test using the embedding config saved on the server */
export function testEmbeddings(client: ApiClient): Promise<EmbedTestResp> {
  return client.post<EmbedTestResp>('/api/embeddings/test', {})
}
