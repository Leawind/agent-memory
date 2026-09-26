import { ApiClient } from './client';
import { DoctorResp, EmbedBackfillResp, EmbedTestResp, HealthInfo, ImportResp, StatsInfo } from '../types';
export declare function getStats(client: ApiClient): Promise<StatsInfo>;
export declare function getHealth(client: ApiClient): Promise<HealthInfo>;
export declare function runDoctorRemote(client: ApiClient): Promise<DoctorResp>;
export declare function exportBackup(client: ApiClient): Promise<Blob>;
/** 导入备份 JSON（对象，非字符串）；仅空库可用，服务端把关 */
export declare function importBackup(client: ApiClient, dump: unknown): Promise<ImportResp>;
/** 补跑一批语义搜索向量，返回 processed / remaining（UI 循环调用直到 remaining=0） */
export declare function backfillEmbeddings(client: ApiClient): Promise<EmbedBackfillResp>;
/** 用服务端已保存的 embedding 配置做一次连通性测试 */
export declare function testEmbeddings(client: ApiClient): Promise<EmbedTestResp>;
