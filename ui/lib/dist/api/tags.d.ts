import { ApiClient } from './client';
import { TagListResp } from '../types';
export type TagDeleteMode = 'detach' | 'purge';
/** filter 为标签名正则（服务端过滤）；非法正则由服务端 400 */
export declare function listTags(client: ApiClient, filter?: string): Promise<TagListResp>;
export declare function createTag(client: ApiClient, name: string, description: string): Promise<void>;
/** 编辑标签：newName 为空或与原名相同表示不改名 */
export declare function updateTag(client: ApiClient, name: string, newName: string, description: string): Promise<void>;
export declare function deleteTag(client: ApiClient, name: string, mode: TagDeleteMode): Promise<void>;
