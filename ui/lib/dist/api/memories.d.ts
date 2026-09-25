import { ApiClient } from './client';
import { MemoryFull, MemoryListResp, MemorySearchResp } from '../types';
/** 新建记忆的输入（标签为目标集合） */
export interface MemoryCreateInput {
    summary: string;
    content: string;
    tags: string[];
}
/** 编辑记忆的输入：标签以增删集合表达（服务端约定 add_tags / remove_tags） */
export interface MemoryUpdateInput {
    summary?: string;
    content?: string;
    add_tags?: string[];
    remove_tags?: string[];
}
export declare function getMemory(client: ApiClient, id: string): Promise<MemoryFull>;
export declare function createMemory(client: ApiClient, input: MemoryCreateInput): Promise<MemoryFull>;
export declare function updateMemory(client: ApiClient, id: string, input: MemoryUpdateInput): Promise<MemoryFull>;
export declare function deleteMemory(client: ApiClient, id: string): Promise<void>;
export declare function fetchMemoriesPage(client: ApiClient, queryString: string): Promise<MemoryListResp | MemorySearchResp>;
