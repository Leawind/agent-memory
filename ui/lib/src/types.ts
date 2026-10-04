// Type definitions for server responses and entities (fields map 1:1 to what /api actually returns)

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
  count: number
  /** Omitted when the tag has no description */
  description?: string
  /** Present (true) only on the reserved tag (conventions), which can be neither renamed nor deleted; the panel hides those actions */
  reserved?: true
}

export interface TagListResp {
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
  /** The search mode actually used: hybrid (keyword + semantic) or keyword */
  mode?: 'hybrid' | 'keyword'
  /** True when a hybrid request fell back to keyword search because the embedding service was unavailable */
  semantic_fallback?: boolean
  hint?: string
  note?: string
  results: SearchResult[]
}

/** stats.embedding: semantic search vector coverage (enabled=false means not enabled) */
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

/** Caller identity summary returned by /api/whoami (mode=open means full capabilities in permissions; mode=anonymous means the configured capability set of the anonymous identity) */
export interface WhoAmI {
  name: string
  mode: 'open' | 'anonymous' | 'token'
  permissions: Record<string, boolean>
}
