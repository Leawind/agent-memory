// 运维端点：统计 / 体检 / 导入导出 / 版本
import type { ApiClient } from './client'
import type { DoctorResp, HealthInfo, ImportResp, StatsInfo } from '../types'

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
