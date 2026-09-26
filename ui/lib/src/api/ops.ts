// 运维端点：统计 / 体检 / 导入导出 / 版本
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

/** 导入备份 JSON（对象，非字符串）；仅空库可用，服务端把关 */
export function importBackup(client: ApiClient, dump: unknown): Promise<ImportResp> {
  return client.post<ImportResp>('/api/import', dump)
}

/** 补跑一批语义搜索向量，返回 processed / remaining（UI 循环调用直到 remaining=0） */
export function backfillEmbeddings(client: ApiClient): Promise<EmbedBackfillResp> {
  return client.post<EmbedBackfillResp>('/api/embeddings/backfill', {})
}

/** 用服务端已保存的 embedding 配置做一次连通性测试 */
export function testEmbeddings(client: ApiClient): Promise<EmbedTestResp> {
  return client.post<EmbedTestResp>('/api/embeddings/test', {})
}
