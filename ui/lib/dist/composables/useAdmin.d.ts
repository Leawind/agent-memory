import { DoctorResp, ImportResp } from '../types';
export declare function useAdmin(): {
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
    runDoctor: () => Promise<void>;
    exportData: () => Promise<void>;
    importFile: (file: File) => Promise<ImportResp>;
};
export type AdminStore = ReturnType<typeof useAdmin>;
