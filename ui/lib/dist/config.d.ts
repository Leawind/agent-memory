import { InjectionKey } from 'vue';
export interface MemoryUIConfig {
    /** API 前缀。默认 ''（与服务器同源部署）；跨系统集成时填服务器地址，如 'http://127.0.0.1:8899' */
    baseUrl?: string;
    /** 自定义 fetch（注入鉴权头、错误拦截等）。默认 globalThis.fetch */
    fetch?: typeof fetch;
    /** 列表默认分页大小。默认 20 */
    defaultPageSize?: number;
}
export type ResolvedMemoryUIConfig = Required<MemoryUIConfig>;
export declare const MemoryUIConfigKey: InjectionKey<MemoryUIConfig>;
/** 在组件 setup 中注入配置（也可直接用 app.provide(MemoryUIConfigKey, config)）。 */
export declare function provideMemoryUI(config: MemoryUIConfig): void;
/** 读取解析后的完整配置；必须在组件 setup 中调用。 */
export declare function useMemoryConfig(): ResolvedMemoryUIConfig;
