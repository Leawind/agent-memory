import { ResolvedMemoryUIConfig } from '../config';
export interface ApiClient {
    get: <T = void>(path: string) => Promise<T>;
    post: <T = void>(path: string, body?: unknown) => Promise<T>;
    put: <T = void>(path: string, body?: unknown) => Promise<T>;
    del: <T = void>(path: string) => Promise<T>;
    /** 二进制下载（如导出备份）；非 2xx 抛错，消息取服务端 error 字段 */
    getBlob: (path: string) => Promise<Blob>;
}
export declare function createApiClient(config: ResolvedMemoryUIConfig): ApiClient;
/** 在组件 setup 中取配置并创建 API 客户端（同一组件内多次调用各自新建，成本可忽略）。 */
export declare function useApiClient(): ApiClient;
