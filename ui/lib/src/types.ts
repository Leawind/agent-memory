// Type definitions for server responses and entities (fields map 1:1 to what /api actually returns)

export interface MemorySummary {
  id: string
  tags: string[]
  summary: string
  /** Compact server-local wall clock "YYYY-MM-DD HH:MM"; summaries carry no creation time (the id encodes creation order) */
  updated: string
  lifecycle?: LifecycleMetadata
}

export interface LifecycleMetadata {
  kind: 'fact' | 'preference' | 'procedure' | 'context' | 'event'
  expires_at: number | null
  archived_at: number | null
  pinned: boolean
  state: 'active' | 'archived' | 'expired'
}

export interface MemoryFull extends MemorySummary {
  created: string
  content: string
  original_tags: string[]
  derived_tags: Array<{ tag: string; rules: string[] }>
  lifecycle: LifecycleMetadata
}

/** Write responses are echo-free: only facts the client cannot derive from its own request */
export interface MemoryCreateResp {
  id: string
  /** Compact server-local wall clock of the write */
  updated: string
  tags_autocreated?: string[]
  tags_missing_description?: string[]
  duplicate_of?: string[]
  /** Semantic near-duplicates (advisory): cosine >= the server threshold, best first */
  similar_to?: Array<{ id: string; similarity: number }>
}

export interface MemoryMergeResp {
  /** Memories still mentioning the absorbed id (those references now dangle) */
  referenced_by?: Array<{ id: string; summary: string }>
  note?: string
}

export interface MemoryUpdateResp {
  /** Whether anything actually changed */
  updated: boolean
  tags_autocreated?: string[]
  tags_missing_description?: string[]
}

export interface TagView {
  name: string
  count: number
  /** Omitted when the tag has no description */
  description?: string
  /** Present (true) only on the reserved tag (convention), which can be neither renamed nor deleted; the panel hides those actions */
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
  updated: string
  lifecycle?: LifecycleMetadata
}

export interface MemorySearchResp {
  total_matches: number
  /** Hybrid only: how many matches are literal keyword hits (the rest entered via the semantic channel) */
  keyword_matches?: number
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

/** One ordered embedding candidate (settings.embedding_models element, canonical server shape) */
export interface EmbedModelEntry {
  id: string
  name: string
  enabled: boolean
  base_url: string
  model: string
  api_key?: string | null
  query_prefix?: string | null
  passage_prefix?: string | null
  /** Absent/null = the server-side built-in default floor */
  min_similarity?: number | null
}

/** One ordered reranker candidate (settings.rerank_models element) */
export interface RerankModelEntry {
  id: string
  name: string
  enabled: boolean
  base_url: string
  model: string
  api_key?: string | null
}

export interface SearchLimits {
  semantic_candidates: number
  rerank_candidates: number
}

export interface LifecyclePolicy {
  half_life_days: Record<LifecycleMetadata['kind'], number | null>
  freshness_weight: number
  reinforcement_weight: number
}

export interface AccessStats {
  raw_events: number
  hourly_buckets: number
  tracked_memories: number
  dropped_read_events: number
}

/** Per-candidate verdict from the connection tests: `index` is the request position, so an editor
 *  can line results up with its rows even when a candidate is incomplete */
export interface EmbedTestResult {
  index: number
  model: string
  ok: boolean
  /** The probed candidate's vector identity (absent for an incomplete candidate) */
  key?: string
  dim?: number
  elapsed_ms?: number
  error?: string
}

export interface EmbedTestResp {
  ok: boolean
  results: EmbedTestResult[]
}

export interface RerankTestResult {
  index: number
  model: string
  ok: boolean
  scored?: number
  elapsed_ms?: number
  error?: string
}

export interface RerankTestResp {
  ok: boolean
  results: RerankTestResult[]
}

/** One vector-cache identity (GET /api/embeddings/caches): keys are user-assigned model IDs */
export interface VectorCacheInfo {
  key: string
  name?: string
  /** Configured API model name, or the ID of an unconfigured cache */
  model: string
  embedded: number
  pending: number
  /** Whether an enabled settings entry matches this key (unconfigured caches can only be deleted) */
  configured: boolean
}

export interface VectorCachesResp {
  caches: VectorCacheInfo[]
}

/** stats.embedding: semantic search vector coverage (enabled=false means not enabled) */
export interface EmbeddingCoverage {
  enabled: boolean
  model?: string
  embedded?: number
  pending?: number
  /** Per-candidate coverage (the head candidate's numbers stay top-level for convenience) */
  models?: Array<{
    key: string
    model: string
    enabled: boolean
    embedded?: number
    pending?: number
  }>
}

export interface EmbedBackfillResp {
  configured: boolean
  processed?: number
  remaining?: number
  error?: string
}

export interface VectorCacheDeleteResp {
  deleted: number
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
  checks: DoctorCheck[]
}

/** One named hygiene check: id is a stable machine identifier (the UI localizes it) */
export interface DoctorCheck {
  id: string
  ok: boolean
  issues: string[]
}

export interface ImportResp {
  imported_memories: number
  imported_tags: number
}

export interface TagRule {
  name: string
  expression: string
}

export interface TagRulePreview {
  valid: boolean
  total_violations: number
  violations: Array<{ id: string; summary: string; rules: string[] }>
  positive_cycles: boolean
}

/** Caller identity summary returned by /api/whoami (mode=open means full capabilities in permissions; mode=anonymous means the configured capability set of the anonymous identity) */
export interface WhoAmI {
  name: string
  mode: 'open' | 'anonymous' | 'token'
  permissions: Record<string, boolean>
}
