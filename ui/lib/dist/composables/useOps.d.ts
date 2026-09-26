import { StatsInfo } from '../types';
export declare function useOps(): {
    stats: import('vue').Ref<{
        path?: string | undefined;
        memories?: number | undefined;
        tags?: number | undefined;
        file_size?: number | undefined;
        schema_version?: number | undefined;
        version?: string | undefined;
    }, Partial<StatsInfo> | {
        path?: string | undefined;
        memories?: number | undefined;
        tags?: number | undefined;
        file_size?: number | undefined;
        schema_version?: number | undefined;
        version?: string | undefined;
    }>;
    version: import('vue').Ref<string, string>;
    sizeText: import('vue').ComputedRef<string>;
    reload: () => Promise<void>;
};
export type OpsStore = ReturnType<typeof useOps>;
