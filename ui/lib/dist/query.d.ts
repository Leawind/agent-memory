export interface MemoriesQueryState {
    query: string;
    tagFilter: string;
    sort: string;
    order: string;
    page: number;
    pageSize: number;
}
export declare function buildMemoriesQuery(s: MemoriesQueryState): string;
export declare function isSearchMode(query: string): boolean;
