export interface MemoriesQueryState {
    query: string;
    tagFilter: string;
    /** 搜索模式：auto（服务端按配置决定）/ keyword / hybrid；auto 不传参数 */
    mode: string;
    sort: string;
    order: string;
    page: number;
    pageSize: number;
}
export declare function buildMemoriesQuery(s: MemoriesQueryState): string;
export declare function isSearchMode(query: string): boolean;
