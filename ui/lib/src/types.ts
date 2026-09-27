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
  created_at: number
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
  total_matches: number
  offset: number
  returned: number
  /** 实际使用的搜索方式：hybrid（关键词+语义）或 keyword */
  mode?: 'hybrid' | 'keyword'
  /** hybrid 请求因 embedding 服务不可用而回退关键词时为 true */
  semantic_fallback?: boolean
  hint?: string
  note?: string
  results: SearchResult[]
}

/** stats.embedding：语义搜索向量覆盖率（enabled=false 表示未启用） */
export interface EmbeddingCoverage {
  enabled: boolean
  model?: string
  embedded?: number
  pending?: number
}

export interface EmbedBackfillResp {
  configured: boolean
  processed?: number
  remaining?: number
  error?: string
}

export interface EmbedTestResp {
  ok: boolean
  dim?: number
  elapsed_ms?: number
  error?: string
}

export interface StatsInfo {
  path: string
  memories: number
  tags: number
  file_size: number
  schema_version: number
  embedding?: EmbeddingCoverage
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

/** /api/whoami 返回的调用者身份摘要（mode=open 时 permissions 为全能力；mode=anonymous 时为匿名身份的配置能力集） */
export interface WhoAmI {
  name: string
  mode: 'open' | 'anonymous' | 'token'
  permissions: Record<string, boolean>
}
