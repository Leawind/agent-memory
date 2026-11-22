// Type definitions for server responses and entities (fields map 1:1 to what /api actually returns)

/** Scoped values of one memory (sparse: omitted entirely when the memory carries none). A key is
 *  a scope name; a string value is the value half of the composed tag name (`system::pc1sw0` →
 *  "pc1sw0"), numeric spellings canonicalize to JSON numbers. null declares that the memory takes
 *  no tag in that scope — legal only on a scope required on every write. */
export type MemoryAttributes = Record<string, string | number | null>

export interface MemorySummary {
  id: string
  tags: string[]
  summary: string
  attributes?: MemoryAttributes
  /** Compact server-local wall clock "YYYY-MM-DD HH:MM"; summaries carry no creation time (the id encodes creation order) */
  updated: string
}

export interface MemoryFull extends MemorySummary {
  created: string
  content: string
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
  /** The scope half when the name is scoped (`system::pc1sw0` → `system`); absent on bare tags */
  scope?: string
  /** Present (true) only on the reserved tag (convention), which can be neither renamed nor deleted; the panel hides those actions */
  reserved?: true
}

/** One scope — the `system` in `system::pc1sw0`: a vocabulary of scoped values with its own
 *  description and write contract. */
export interface ScopeView {
  name: string
  /** Memories carrying at least one tag under this scope */
  count: number
  /** Distinct tags under this scope */
  tag_count: number
  /** Omitted when the scope has no description */
  description?: string
  /** Present (true) when every new memory must declare this scope (a value, or null for none) */
  required?: true
}

export interface TagListResp {
  tags: TagView[]
  /** Both namespaces side by side; GET /api/scopes answers with the same whole shape */
  scopes: ScopeView[]
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
}

export interface MemorySearchResp {
  cursor?: string
  cursor_expires_at?: number
  cursor_unavailable?: string
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
  /** ModelScope tokenizer source: 'auto' resolves by the model id, any other id/keyword is used
   *  verbatim; absent/null keeps the character-cap truncation */
  tokenizer?: string | null
  /** The model's maximum input tokens, applied when a tokenizer is configured; absent/null = the
   *  built-in default limit */
  max_tokens?: number | null
  /** Whether the background scheduler may drain this model's backlog unattended (start, timer,
   *  settings save); absent/null = off */
  auto_backfill?: boolean | null
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
  adaptive: {
    enabled: boolean
    min_candidates: number
    max_candidates: number
    target_latency_ms: number
    max_input_chars: number
  }
}

export interface SearchRuntime {
  process_local: boolean
  adaptive_enabled: boolean
  models: Array<{
    id: string
    model: string
    samples: number
    failures: number
    selected_candidates: number
    last_candidates: number
    last_latency_ms: number
    ewma_ms_per_document: number | null
  }>
}

export interface LifecyclePolicy {
  rules: Array<{ predicate: string; half_life_days: number | null }>
  default_half_life_days: number | null
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
  /** Upstream HTTP status when the service answered; absent for transport failures */
  status?: number
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
  /** Upstream HTTP status when the service answered; absent for transport failures */
  status?: number
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

/** One backfill run as reported by the server (it belongs to one model and is serialized there) */
export interface EmbedBackfillRun {
  model_key: string
  trigger: 'auto' | 'manual' | 'write'
  started_at: number
  done: number
  total: number
  remaining: number
}

export interface EmbedBackfillResp {
  configured: boolean
  model_key?: string
  /** A run exists after this request (started now, or already running and joined) */
  running?: boolean
  /** This request created the run (false when it joined an existing one) */
  started?: boolean
  processed?: number
  remaining?: number
  total?: number
  run?: EmbedBackfillRun
  error?: string
}

export interface EmbedBackfillRunsResp {
  runs: EmbedBackfillRun[]
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
  /** Present in merge mode: memories skipped because the target already carries them verbatim */
  skipped_memories: number
  /** Merge mode into a store with required scopes: memories that arrived without the declaration */
  unclassified_memories?: number
}

export interface NamedPredicate {
  name: string
  predicate: string
  description: string
}

/** Caller identity summary returned by /api/whoami (mode=open means full capabilities in permissions; mode=anonymous means the configured capability set of the anonymous identity) */
export interface WhoAmI {
  name: string
  mode: 'open' | 'anonymous' | 'token'
  permissions: Record<string, boolean>
}
