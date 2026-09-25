import { MemorySummary, SearchResult } from '../types';
/** 编辑器表单的形状：id 为 null 表示新建 */
export interface MemoryDraft {
    id: string | null;
    summary: string;
    content: string;
    tags: string[];
}
export declare function useMemories(): {
    query: import('vue').Ref<string, string>;
    tagFilter: import('vue').Ref<string, string>;
    sort: import('vue').Ref<"updated_at" | "created_at", "updated_at" | "created_at">;
    order: import('vue').Ref<"asc" | "desc", "asc" | "desc">;
    page: import('vue').Ref<number, number>;
    pageSize: import('vue').Ref<number, number>;
    rows: import('vue').Ref<{
        id: string;
        tags: string[];
        summary: string;
        created_at: number;
        updated_at: number;
    }[], MemorySummary[] | {
        id: string;
        tags: string[];
        summary: string;
        created_at: number;
        updated_at: number;
    }[]>;
    searchResults: import('vue').Ref<{
        id: string;
        tags: string[];
        summary: string;
        score: number;
        snippet: string;
        updated_at: number;
    }[], SearchResult[] | {
        id: string;
        tags: string[];
        summary: string;
        score: number;
        snippet: string;
        updated_at: number;
    }[]>;
    total: import('vue').Ref<number, number>;
    note: import('vue').Ref<string, string>;
    loading: import('vue').Ref<boolean, boolean>;
    tagOptions: import('vue').Ref<string[], string[]>;
    searching: import('vue').ComputedRef<boolean>;
    onSearch: () => Promise<void>;
    reload: () => Promise<void>;
    loadTagOptions: () => Promise<void>;
    saveMemory: (draft: MemoryDraft) => Promise<void>;
    removeMemory: (id: string) => Promise<void>;
};
export type MemoriesStore = ReturnType<typeof useMemories>;
