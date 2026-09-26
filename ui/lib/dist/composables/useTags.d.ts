import { TagDeleteMode } from '../api/tags';
import { TagView } from '../types';
export declare function useTags(): {
    rows: import('vue').Ref<{
        name: string;
        description: string;
        memory_count: number;
        last_used_at: number | null;
    }[], TagView[] | {
        name: string;
        description: string;
        memory_count: number;
        last_used_at: number | null;
    }[]>;
    loading: import('vue').Ref<boolean, boolean>;
    filter: import('vue').Ref<string, string>;
    reload: () => Promise<void>;
    create: (name: string, description: string) => Promise<void>;
    rename: (oldName: string, newName: string, description: string) => Promise<void>;
    remove: (name: string, mode: TagDeleteMode) => Promise<void>;
};
export type TagsStore = ReturnType<typeof useTags>;
