// 服务端响应与实体的类型定义（字段与 /api 实际返回一一对应）

export interface MemorySummary {
  id: string
  tags: string[]
  summary: string
  created_at: number
  updated_at: number
}

export interface MemoryFull extends MemorySummary {
  content: string
}

export interface TagView {
  name: string
  description: string
  memory_count: number
  last_used_at: number | null
}

export interface TagListResp {
  total_tags: number
  total_memories: number
  tags: TagView[]
}

export interface MemoryListResp {
  total: number
  offset: number
  limit: number
  memories: MemorySummary[]
  note?: string
}

export interface SearchResult {
  id: string
  tags: string[]
  summary: string
  score: number
  snippet: string
  updated_at: number
}

export interface MemorySearchResp {
  query: string
  total_matches: number
  offset: number
  returned: number
  results: SearchResult[]
}

export interface StatsInfo {
  path: string
  memories: number
  tags: number
  next_id: string
  file_size: number
  newest_update: { id: string; updated_at: number } | null
  schema_version: number
  version?: string
}

export interface HealthInfo {
  status: string
  version: string
}

export interface DoctorResp {
  ok: boolean
  issues: string[]
}

export interface ImportResp {
  imported_memories: number
  imported_tags: number
}
