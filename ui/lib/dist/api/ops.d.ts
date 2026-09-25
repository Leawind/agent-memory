import { ApiClient } from './client';
import { DoctorResp, HealthInfo, ImportResp, StatsInfo } from '../types';
export declare function getStats(client: ApiClient): Promise<StatsInfo>;
export declare function getHealth(client: ApiClient): Promise<HealthInfo>;
export declare function runDoctorRemote(client: ApiClient): Promise<DoctorResp>;
export declare function exportBackup(client: ApiClient): Promise<Blob>;
/** 导入备份 JSON（对象，非字符串）；仅空库可用，服务端把关 */
export declare function importBackup(client: ApiClient, dump: unknown): Promise<ImportResp>;
