import { DoctorResp, ImportResp, StatsInfo } from '../types';
export declare function useOps(): {
    stats: import('vue').Ref<{
        path?: string | undefined;
        memories?: number | undefined;
        tags?: number | undefined;
        next_id?: string | undefined;
        file_size?: number | undefined;
        newest_update?: {
            id: string;
            updated_at: number;
        } | null | undefined;
        schema_version?: number | undefined;
        version?: string | undefined;
    }, Partial<StatsInfo> | {
        path?: string | undefined;
        memories?: number | undefined;
        tags?: number | undefined;
        next_id?: string | undefined;
        file_size?: number | undefined;
        newest_update?: {
            id: string;
            updated_at: number;
        } | null | undefined;
        schema_version?: number | undefined;
        version?: string | undefined;
    }>;
    version: import('vue').Ref<string, string>;
    doctor: import('vue').Ref<{
        ok: boolean;
        issues: string[];
    }, DoctorResp | {
        ok: boolean;
        issues: string[];
    }>;
    doctorRan: import('vue').Ref<boolean, boolean>;
    doctorLoading: import('vue').Ref<boolean, boolean>;
    exporting: import('vue').Ref<boolean, boolean>;
    importing: import('vue').Ref<boolean, boolean>;
    sizeText: import('vue').ComputedRef<string>;
    reload: () => Promise<void>;
    runDoctor: () => Promise<void>;
    exportData: () => Promise<void>;
    importFile: (file: File) => Promise<ImportResp>;
};
export type OpsStore = ReturnType<typeof useOps>;
