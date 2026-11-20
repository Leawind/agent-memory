//! Semantic search: embedding service calls, vector encoding/decoding and hybrid ranking.
//!
//! Fallback is this module's hard rule — an unavailable embedding service may only cause "degradation", never
//! "failure": the search path falls back to pure keywords on timeout/error; the write path just leaves vectors pending backfill,
//! since the memory itself is already saved; with no configuration, every entry point is a zero-cost pass-through.
//!
//! Several embedding services can be configured at once (an ordered candidate list in settings, each
//! independently enabled). At runtime the first candidate whose actual call succeeds wins; a failure
//! moves the attempt to the next candidate, so a restarting or partially down local service degrades
//! instead of breaking. Each candidate's vectors are keyed by its own identity (`vector_key`), so the
//! caches of all configured models coexist in the embeddings table and per-model backfill/delete is a
//! pure bookkeeping operation.
//!
//! Network calls never happen inside a database write transaction (guaranteed by callers): holding
//! an IMMEDIATE lock across network I/O is a forbidden deadlock shape. The search path does hold
//! its read-only snapshot across the embedding/reranker calls — under WAL that blocks nothing; it
//! only pins checkpointing for the model latency, the accepted cost of scoring one consistent
//! candidate snapshot (rerank.rs documents the same trade-off from the reranker's side).

use crate::search::{self, Hit};
use crate::store::{Store, TxMode};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};
use tokenizers::Tokenizer;

/// Embedding service configuration (an OpenAI-compatible `/embeddings` endpoint: cloud APIs and
/// local services such as Ollama/LM Studio share the same shape). `base_url` is the service
/// root; a value pasted with the trailing `/embeddings` is normalized to the root (see
/// `service_root`), so both spellings share the endpoint and the cache identity.
#[derive(Clone, Debug)]
pub struct EmbedConfig {
    pub id: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    /// Text prepended to search queries before embedding. Asymmetric models train with
    /// instruction prefixes and lose noticeable quality without them (E5: "query: "; original
    /// BGE: a "Represent this sentence for searching relevant passages:" instruction);
    /// symmetric models like bge-m3 leave this unset. Applied verbatim — a trailing space is
    /// part of the instruction.
    pub query_prefix: Option<String>,
    /// Text prepended to stored-memory texts before embedding (E5: "passage: ").
    pub passage_prefix: Option<String>,
    /// Cosine floor for the semantic recall channel (query-time filter only — not part of the
    /// vector identity, changing it never re-embeds). 0.0 disables the floor.
    pub min_similarity: f32,
    /// Tokenizer source for exact token counting: `auto` resolves against this candidate's model
    /// id, any other non-empty value is used verbatim as a ModelScope id / search keyword. None
    /// keeps the character cap. Changing the resolved truncation regime re-embeds (see
    /// `fingerprint`), so this participates in cache validity.
    pub tokenizer: Option<String>,
    /// The embedding model's maximum input tokens, applied when a tokenizer is configured
    /// (absent = `DEFAULT_TOKEN_LIMIT`). Inert without a tokenizer — and therefore not part of
    /// the fingerprint in that case.
    pub max_tokens: Option<usize>,
    /// Whether the background scheduler may drain this candidate's backlog on its own (at
    /// service start, on a timer, and after settings are saved). Off by default: automatic
    /// embedding spends the operator's API budget, so it is opt-in per candidate. A memory write
    /// still embeds that memory — this governs only the unattended sweeps.
    pub auto_backfill: bool,
}

/// Token budget assumed when a tokenizer is configured but the model's real limit is not known:
/// fits the long-context class (bge-m3, OpenAI text-embedding-3); smaller models must set
/// `max_tokens` explicitly.
pub const DEFAULT_TOKEN_LIMIT: usize = 8192;

/// Normalize a configured service base_url: surrounding whitespace and trailing slashes off, and
/// a trailing full-endpoint path (`/embeddings`, `/rerank`) reduced to the root. Provider docs
/// show complete URLs and they get pasted into the settings form, so both spellings must select
/// the same endpoint and carry the same cache identity. Case-sensitive on purpose: URL paths
/// are, and the compatibility ecosystem serves these paths lowercase.
pub fn service_root(base_url: &str, endpoint: &str) -> String {
    let root = base_url.trim().trim_end_matches('/');
    match root.strip_suffix(endpoint) {
        Some(stripped) => stripped.trim_end_matches('/').to_string(),
        None => root.to_string(),
    }
}

impl EmbedConfig {
    /// Stable, user-assigned cache identity, independent of order and display name.
    pub fn vector_key(&self) -> String {
        self.id.clone()
    }

    /// Failure-circuit identity (see `Failover::identity`): shared format with the reranker
    /// candidates, so both kinds key their cooldowns the same way.
    pub fn circuit_identity(&self) -> String {
        Failover::identity(&self.id, &self.base_url, &self.model, &self.api_key)
    }

    /// Cache validity is separate from identity. JSON avoids delimiter collisions. The truncation
    /// regime travels with the fingerprint (tokenizer source + effective token limit + chunking
    /// policy version): a vector embedded under a different cut is a different vector. An
    /// unconfigured tokenizer is a constant, so a stray `max_tokens` on a tokenizer-less entry
    /// never invalidates anything.
    pub fn fingerprint(&self) -> String {
        json!([
            self.base_url.trim_end_matches('/'),
            self.model,
            self.query_prefix.as_deref().unwrap_or(""),
            self.passage_prefix.as_deref().unwrap_or(""),
            self.tokenizer.as_deref().unwrap_or(""),
            match self.tokenizer {
                Some(_) => self.max_tokens.unwrap_or(DEFAULT_TOKEN_LIMIT),
                None => 0,
            },
            // Changing how bodies are split changes every vector, so the split layout is part of
            // the cache identity — but only where splitting happens at all.
            match self.tokenizer {
                Some(_) => crate::chunk::POLICY_TAG,
                None => "",
            }
        ])
        .to_string()
    }

    /// The exact-token truncation budget when a tokenizer is configured, None = character cap.
    pub fn token_limit(&self) -> Option<usize> {
        self.tokenizer
            .is_some()
            .then(|| self.max_tokens.unwrap_or(DEFAULT_TOKEN_LIMIT))
    }

    fn endpoint(&self) -> String {
        format!("{}/embeddings", service_root(&self.base_url, "/embeddings"))
    }
}

/// One configured embedding candidate (an element of the ordered `embedding_models` settings
/// list). Carries the raw settings fields so the admin UI can round-trip them verbatim; the
/// operational view is `config` (usable only when enabled and fully specified).
#[derive(Clone, Debug)]
pub struct EmbedEntry {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub query_prefix: Option<String>,
    pub passage_prefix: Option<String>,
    /// Per-candidate cosine floor: baselines are model-specific (bge-family ~0.4, OpenAI-3
    /// ~0.2), so the floor travels with the model instead of being one global knob. None = the
    /// built-in default (`DEFAULT_MIN_SIMILARITY`).
    pub min_similarity: Option<f32>,
    /// See `EmbedConfig::tokenizer`.
    pub tokenizer: Option<String>,
    /// See `EmbedConfig::max_tokens`.
    pub max_tokens: Option<usize>,
    /// See `EmbedConfig::auto_backfill`.
    pub auto_backfill: bool,
}

impl EmbedEntry {
    /// Lenient parse of one settings-array element: wrong inner types normalize to their
    /// defaults instead of failing (the REST layer validates on write; this only guards
    /// hand-edited databases). Order in the array is the failover priority.
    pub fn from_json(v: &Value) -> Option<EmbedEntry> {
        let obj = v.as_object()?;
        let string = |key: &str| {
            obj.get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .filter(|s| !s.is_empty())
        };
        Some(EmbedEntry {
            id: string("id").unwrap_or_default().trim().to_string(),
            name: string("name").unwrap_or_default().trim().to_string(),
            enabled: obj.get("enabled").and_then(Value::as_bool).unwrap_or(true),
            base_url: string("base_url").unwrap_or_default(),
            model: string("model").unwrap_or_default(),
            api_key: string("api_key"),
            query_prefix: string("query_prefix"),
            passage_prefix: string("passage_prefix"),
            min_similarity: obj
                .get("min_similarity")
                .and_then(Value::as_f64)
                .map(|f| f as f32)
                .filter(|f| (0.0..=1.0).contains(f)),
            tokenizer: string("tokenizer"),
            max_tokens: obj
                .get("max_tokens")
                .and_then(Value::as_u64)
                .map(|n| n as usize)
                .filter(|n| (64..=262_144).contains(n)),
            auto_backfill: obj
                .get("auto_backfill")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
    }

    /// Canonical JSON for the settings round-trip (GET → UI → PUT). The floor is rounded to
    /// three decimals: f32 → f64 conversion noise (0.45 → 0.44999998...) is meaningless at
    /// hint precision and would make every GET→PUT cycle churn the stored value.
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "enabled": self.enabled,
            "base_url": self.base_url,
            "model": self.model,
            "api_key": self.api_key,
            "query_prefix": self.query_prefix,
            "passage_prefix": self.passage_prefix,
            "min_similarity": self
                .min_similarity
                .map(|f| (f as f64 * 1000.0).round() / 1000.0),
            "tokenizer": self.tokenizer,
            "max_tokens": self.max_tokens,
            "auto_backfill": self.auto_backfill,
        })
    }

    /// The operational view: usable only when enabled and base_url / model are both non-empty.
    pub fn usable(&self) -> Option<EmbedConfig> {
        if !self.enabled || self.id.is_empty() {
            return None;
        }
        self.config()
    }

    /// The same view without the enabled gate: what a candidate would do if it were switched on.
    /// The admin connection probe tests exactly this, so a candidate can be verified before it is
    /// enabled or saved.
    pub fn config(&self) -> Option<EmbedConfig> {
        if self.base_url.trim().is_empty() || self.model.trim().is_empty() {
            return None;
        }
        Some(EmbedConfig {
            id: self.id.clone(),
            base_url: service_root(&self.base_url, "/embeddings"),
            model: self.model.trim().to_string(),
            api_key: self.api_key.clone(),
            query_prefix: self.query_prefix.clone(),
            passage_prefix: self.passage_prefix.clone(),
            min_similarity: self.min_similarity.unwrap_or(DEFAULT_MIN_SIMILARITY),
            tokenizer: self.tokenizer.clone(),
            max_tokens: self.max_tokens,
            auto_backfill: self.auto_backfill,
        })
    }

    /// A disabled model's cache remains manageable under the same ID.
    pub fn vector_key(&self) -> String {
        self.id.clone()
    }

    pub fn fingerprint(&self) -> Option<String> {
        self.config().map(|cfg| cfg.fingerprint())
    }
}

/// Runtime selection among the configured candidates: try them in priority order (starting from
/// the last candidate that actually answered — a dead first choice must not tax every later call
/// with its connect timeout), and hand the first one whose real call succeeds to the caller. The
/// probe IS the caller's operation: no extra health-check traffic, and "available" means "the
/// actual embedding just worked", not merely "the port is open".
///
/// A failed attempt also opens a per-candidate circuit: for the cooldown window the candidate is
/// skipped without any traffic, so a burst of searches against a dead service pays the connect
/// failure once instead of per request (a refused connect can take seconds to surface at the OS
/// layer even when the port is long gone). After the window the next call probes the candidate
/// again (half-open), and a success clears the record — a restarted service is picked up by the
/// first call past the cooldown at the latest.
pub struct Failover {
    preferred: AtomicUsize,
    /// Negative cache keyed by the candidate's circuit identity (`Failover::identity` — id,
    /// endpoint, model and credential, so a reconfigured candidate is never skipped for its
    /// predecessor's failures). A Vec on purpose: candidate lists are a handful of entries,
    /// and the struct must be constructible in a `const` context (the shared statics below).
    cooled: Mutex<Vec<(String, Instant)>>,
    /// Cooldown window; `new()` fixes it at `FAILURE_COOLDOWN`, tests shrink it.
    cooldown: Duration,
}

/// How long a failed candidate stays skipped without a probe.
pub const FAILURE_COOLDOWN: Duration = Duration::from_secs(30);

impl Failover {
    /// The negative-cache key shared by both model kinds: everything that decides whether a
    /// call to this candidate succeeds (id, endpoint, model, credential). Deliberately not
    /// the vector-cache fingerprint — fixing an invalid api_key must release the candidate
    /// from its cooldown instead of waiting it out, and the instruction prefixes don't
    /// affect reachability at all.
    pub fn identity(id: &str, base_url: &str, model: &str, api_key: &Option<String>) -> String {
        json!([id, base_url.trim_end_matches('/'), model, api_key]).to_string()
    }

    pub const fn new() -> Self {
        Failover {
            preferred: AtomicUsize::new(0),
            cooled: Mutex::new(Vec::new()),
            cooldown: FAILURE_COOLDOWN,
        }
    }

    /// A constructor for tests: the production cooldown is tuned for real-world pacing, too long
    /// to wait out in a unit test.
    #[cfg(test)]
    fn with_cooldown(cooldown: Duration) -> Self {
        Failover {
            preferred: AtomicUsize::new(0),
            cooled: Mutex::new(Vec::new()),
            cooldown,
        }
    }

    /// Run `op` against the first candidate that succeeds. `identity` names the candidate in the
    /// failure circuit (stable across calls, distinct per configuration — see the struct docs).
    /// Every failed attempt is logged (with its position) and the next candidate takes over; only
    /// when all fail does the last error surface. The sticky preferred index is a hint: a list
    /// edit merely makes it fall back to priority order (it is clamped, and a stale success index
    /// costs one failed attempt at most).
    pub fn first_available<'a, C, T, E: std::fmt::Display>(
        &self,
        configs: &'a [C],
        identity: impl Fn(&C) -> String,
        mut op: impl FnMut(&C) -> Result<T, E>,
    ) -> Result<(usize, &'a C, T), String> {
        if configs.is_empty() {
            return Err("no candidate service is configured".to_string());
        }
        let cooldown = self.cooldown;
        let start = self
            .preferred
            .load(Ordering::Relaxed)
            .min(configs.len() - 1);
        let mut last_err = String::new();
        let mut tried = 0;
        for offset in 0..configs.len() {
            let idx = (start + offset) % configs.len();
            let id = identity(&configs[idx]);
            // Circuit open for this identity: skip without traffic. The lock is held only around
            // the record access, never across `op` (the network call).
            let open = |s: &Self, id: &str| {
                s.cooled
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .iter()
                    .any(|(k, at)| k == id && at.elapsed() < cooldown)
            };
            if open(self, &id) {
                continue;
            }
            tried += 1;
            match op(&configs[idx]) {
                Ok(value) => {
                    let mut cooled = self.cooled.lock().unwrap_or_else(PoisonError::into_inner);
                    cooled.retain(|(k, _)| k != &id);
                    drop(cooled);
                    self.preferred.store(idx, Ordering::Relaxed);
                    return Ok((idx, &configs[idx], value));
                }
                Err(e) => {
                    eprintln!("embedding candidate #{idx} failed, trying the next: {e}");
                    let mut cooled = self.cooled.lock().unwrap_or_else(PoisonError::into_inner);
                    match cooled.iter_mut().find(|(k, _)| k == &id) {
                        Some((_, at)) => *at = Instant::now(),
                        None => cooled.push((id, Instant::now())),
                    }
                    last_err = e.to_string();
                }
            }
        }
        if tried == 0 {
            // Every candidate's circuit is open: degrade immediately instead of re-paying the
            // connect failure the cooldown exists to prevent.
            return Err("all candidates are in failure cooldown; skipping the probe".to_string());
        }
        Err(last_err)
    }

    /// Manual circuit release: the admin connection probe verified a candidate directly (not via
    /// `first_available`), so any failure record it holds is stale — clear it so the next search
    /// picks the candidate up immediately instead of waiting out the cooldown. A candidate that
    /// never failed is a no-op.
    pub fn clear(&self, identity: &str) {
        self.cooled
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|(k, _)| k != identity);
    }

    /// Whether a candidate is inside its failure cooldown. Callers that pick a candidate
    /// themselves (the background backfill runs one candidate at a time) consult this before
    /// paying a connect timeout, and report failures through `fail` so the cooldown is shared
    /// with the search path.
    pub fn is_open(&self, identity: &str) -> bool {
        self.cooled
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .any(|(k, at)| k == identity && at.elapsed() < self.cooldown)
    }

    /// Record a failure for a candidate this caller attempted directly — the same record
    /// `first_available` would have written, so both paths share one cooldown.
    pub fn fail(&self, identity: &str) {
        let mut cooled = self.cooled.lock().unwrap_or_else(PoisonError::into_inner);
        match cooled.iter_mut().find(|(k, _)| k == identity) {
            Some((_, at)) => *at = Instant::now(),
            None => cooled.push((identity.to_string(), Instant::now())),
        }
    }

    /// The candidate that answered last: the starting point for callers that select candidates
    /// themselves (the background sweep walks the list from here).
    pub fn preferred(&self) -> usize {
        self.preferred.load(Ordering::Relaxed)
    }
}

/// Selection state for the embedding candidate list (shared by search / write hook / backfill).
pub static EMBED_FAILOVER: Failover = Failover::new();

/// Timeout for query embedding (the search path): give up when it expires and fall back to keyword search.
pub const QUERY_TIMEOUT: Duration = Duration::from_secs(3);
/// Timeout for batch embedding (the write hook / backfill).
pub const BATCH_TIMEOUT: Duration = Duration::from_secs(30);
/// Maximum number of texts per request (a conservative value against provider batch limits).
pub const MAX_BATCH: usize = 16;
/// Character truncation length applied to each text before it goes to the embedding service. bge-m3's 8K token window
/// is roughly 8K Chinese characters; smaller 512-token models truncate server-side — this only guards against extremely
/// long texts blowing up the request body.
pub const MAX_INPUT_CHARS: usize = 8_000;
/// Default floor for the semantic recall channel: vector candidates below this cosine never
/// enter the fusion. Deliberately conservative — baselines are model-specific (bge-family
/// unrelated pairs score ~0.4, OpenAI text-embedding-3 ~0.2) — and configurable per model via
/// the entry's `min_similarity` field; 0 disables the floor entirely.
pub const DEFAULT_MIN_SIMILARITY: f32 = 0.30;

/// The memory text sent to the embedding service: title + blank line + content, truncated to the
/// character cap. The reranker's document assembly shares this shape; embedding-side truncation
/// happens in `embed_passage_text` / `embed_query_text` instead.
pub fn embed_memory_text(summary: &str, content: &str) -> String {
    cap_chars(&assemble_memory_text(summary, content), MAX_INPUT_CHARS)
}

/// The uncapped assembly: title + blank line + content.
fn assemble_memory_text(summary: &str, content: &str) -> String {
    let mut text = String::new();
    text.push_str(summary.trim());
    if !content.trim().is_empty() {
        text.push_str("\n\n");
        text.push_str(content.trim());
    }
    text
}

fn cap_chars(text: &str, max: usize) -> String {
    if text.chars().count() > max {
        text.chars().take(max).collect()
    } else {
        text.to_string()
    }
}

/// Fit one embedding input under the candidate's truncation regime: with a resolved tokenizer the
/// cut is exact at token level; otherwise (no tokenizer, not yet downloaded, or download failed)
/// the character cap applies. Cache-only — resolution must have happened in a batch context.
fn fit_embedding_input(cfg: &EmbedConfig, prefix: &str, text: &str) -> String {
    if let Some(limit) = cfg.token_limit() {
        if let Some(tok) = crate::tokenizer::cached(cfg) {
            let fitted = crate::tokenizer::truncate_to_tokens(&tok, prefix, text, limit);
            if !fitted.is_empty() {
                return format!("{prefix}{fitted}");
            }
        }
    }
    format!("{prefix}{}", cap_chars(text, MAX_INPUT_CHARS))
}

/// Query-side text: the model's query instruction prefix, then the query itself.
pub fn embed_query_text(cfg: &EmbedConfig, query: &str) -> String {
    fit_embedding_input(cfg, cfg.query_prefix.as_deref().unwrap_or(""), query.trim())
}

/// Passage-side text: the model's passage prefix, then title + content.
pub fn embed_passage_text(cfg: &EmbedConfig, summary: &str, content: &str) -> String {
    fit_embedding_input(
        cfg,
        cfg.passage_prefix.as_deref().unwrap_or(""),
        &assemble_memory_text(summary, content),
    )
}

/// Every embedding input a memory produces: one text normally, several when the body is long
/// enough to be split. Splitting needs an exact token count, so it applies only when the
/// candidate's tokenizer is resolved; otherwise the single truncated text is the whole answer.
/// Chunks are cut so that prefix + text fits the model's token limit on its own.
pub fn embed_passage_texts(cfg: &EmbedConfig, summary: &str, content: &str) -> Vec<String> {
    let Some(limit) = cfg.token_limit() else {
        return vec![embed_passage_text(cfg, summary, content)];
    };
    let Some(tok) = crate::tokenizer::cached(cfg) else {
        return vec![embed_passage_text(cfg, summary, content)];
    };
    crate::chunk::chunk_texts(&tok, summary, content, limit)
        .into_iter()
        .map(|text| {
            let prefix = cfg.passage_prefix.as_deref().unwrap_or("");
            format!("{prefix}{}", fit_tokens(&tok, prefix, &text, limit))
        })
        .collect()
}

/// Token-exact fit of one already-assembled chunk, with the character cap as the last resort.
fn fit_tokens(tok: &Tokenizer, prefix: &str, text: &str, limit: usize) -> String {
    let fitted = crate::tokenizer::truncate_to_tokens(tok, prefix, text, limit);
    if fitted.is_empty() {
        cap_chars(text, MAX_INPUT_CHARS)
    } else {
        fitted
    }
}

/// One failed embedding/reranker service call: the upstream HTTP status when the service
/// answered (absent for transport and shape failures) and the human-readable detail. Display
/// renders only the detail, so logging call sites read exactly as before; the admin connection
/// probe additionally surfaces the status for a classified verdict.
#[derive(Clone, Debug)]
pub struct ServiceError {
    pub status: Option<u16>,
    pub message: String,
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<ServiceError> for String {
    fn from(e: ServiceError) -> String {
        e.message
    }
}

/// Embed a batch. Any network/parsing failure returns Err; the caller decides how to degrade.
pub fn embed_texts(
    cfg: &EmbedConfig,
    texts: &[String],
    timeout: Duration,
) -> Result<Vec<Vec<f32>>, ServiceError> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let mut req = ureq::post(&cfg.endpoint()).timeout(timeout);
    if let Some(key) = &cfg.api_key {
        req = req.set("Authorization", &format!("Bearer {key}"));
    }
    let resp = req
        .send_json(json!({ "model": cfg.model, "input": texts }))
        .map_err(|e| ServiceError {
            status: None,
            message: format!("embedding request failed: {e}"),
        })?;
    let status = resp.status();
    let body: Value = resp.into_json().map_err(|e| ServiceError {
        status: Some(status),
        message: format!("embedding response is not valid JSON: {e}"),
    })?;
    if status != 200 {
        let detail = body["error"]["message"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("unknown error");
        return Err(ServiceError {
            status: Some(status),
            message: format!("embedding service returned {status}: {detail}"),
        });
    }
    // A 200 whose body does not answer the request is a service bug, not an HTTP condition
    parse_vectors(body, texts.len()).map_err(|message| ServiceError {
        status: None,
        message,
    })
}

/// Shape-check and decode the `data` array of a 200 embedding response.
fn parse_vectors(body: Value, inputs: usize) -> Result<Vec<Vec<f32>>, String> {
    let mut data = body["data"]
        .as_array()
        .ok_or("embedding response missing 'data' array")?
        .clone();
    data.sort_by_key(|item| item["index"].as_u64().unwrap_or(0));
    if data.len() != inputs {
        return Err(format!(
            "embedding service returned {} vectors for {} inputs",
            data.len(),
            inputs
        ));
    }
    data.into_iter()
        .map(|item| {
            item["embedding"]
                .as_array()
                .ok_or("embedding entry missing 'embedding' array")?
                .iter()
                .map(|n| {
                    n.as_f64()
                        .map(|f| f as f32)
                        .ok_or_else(|| "embedding contains a non-number".to_string())
                })
                .collect()
        })
        .collect()
}

/// f32 vector → BLOB (little-endian; SQLite has no vector type, and cross-platform byte order is guaranteed by the binding).
pub fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(vec.len() * 4);
    for v in vec {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

/// BLOB → f32 vector; trailing partial bytes are dropped when the length is not a multiple of 4 (corrupt rows count as vector-less).
pub fn blob_to_vec(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Cosine similarity; returns 0 for unequal lengths or zero vectors (meaning it never enters vector recall).
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let (mut dot, mut na, mut nb) = (0f32, 0f32, 0f32);
    for (x, y) in a.iter().zip(b) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// Best similarity between two memories' chunk lists: the closest pair on either side. A memory
/// is as relevant as its most relevant passage, not as the average across its body.
pub fn best_cosine(a: &[Vec<f32>], b: &[Vec<f32>]) -> f32 {
    let mut best = 0.0f32;
    for x in a {
        for y in b {
            best = best.max(cosine(x, y));
        }
    }
    best
}

/// Result of one backfill batch. The distinction between NotConfigured and Failed shapes caller behavior:
/// the write path stays silent for both (Failed logs to stderr), while the admin endpoint surfaces details to the UI.
pub enum EmbedOutcome {
    NotConfigured,
    Processed { processed: usize, remaining: usize },
    Failed(String),
}

fn ensure_current_config(st: &Store, cfg: &EmbedConfig) -> Result<(), String> {
    if st
        .embedding_configs()?
        .iter()
        .any(|current| current.id == cfg.id && current.fingerprint() == cfg.fingerprint())
    {
        Ok(())
    } else {
        Err("embedding configuration changed while the request was in flight".into())
    }
}

/// Backfill one batch for the given candidate's identity: read the pending list (read-only transaction)
/// → call the embedding service (outside transactions) → write vectors and count what is left (one write
/// transaction — the same connection serves both, so a full pass opens two databases, not three).
/// `Err` means "this candidate could not do the work" (service or storage failure) — the failover
/// wrapper treats it as a reason to try the next candidate.
pub fn process_pending_for(
    db_path: &Path,
    batch: usize,
    cfg: &EmbedConfig,
) -> Result<EmbedOutcome, String> {
    // Batch context: this is where a configured tokenizer gets downloaded (and disk-cached) so the
    // query-time, cache-only assembly can cut exactly from here on. Failure degrades to the
    // character cap for this batch and is negatively cached inside the resolver.
    crate::tokenizer::resolve_eager(cfg);
    let batch = batch.clamp(1, MAX_BATCH);
    let pending = crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
        st.embedding_pending_batch(&cfg.vector_key(), &cfg.fingerprint(), batch)
    })?;
    if pending.is_empty() {
        return Ok(EmbedOutcome::Processed {
            processed: 0,
            remaining: 0,
        });
    }

    // One memory may need several texts when its body is split. The round stops adding memories
    // once the request budget is filled (always keeping at least the first one), so a round stays
    // one request's worth of work and the rest of the queue waits for the next round.
    let mut per_memory: Vec<(i64, usize)> = Vec::new();
    let mut texts: Vec<String> = Vec::new();
    for (id, summary, content) in &pending {
        let parts = embed_passage_texts(cfg, summary, content);
        if !per_memory.is_empty() && texts.len() + parts.len() > MAX_BATCH {
            break;
        }
        per_memory.push((*id, parts.len()));
        texts.extend(parts);
    }
    let vectors = embed_texts_batched(cfg, &texts, BATCH_TIMEOUT)?;

    // Storing and counting share one write transaction: the count sees this batch's puts, so the
    // reported remaining is identical to a post-commit recount
    let remaining = crate::store::with_db_in(db_path, TxMode::Write, |st| {
        ensure_current_config(st, cfg)?;
        write_vectors(st, cfg, &per_memory, &vectors)?;
        st.embedding_pending_count(&cfg.vector_key(), Some(&cfg.fingerprint()))
    })?;
    Ok(EmbedOutcome::Processed {
        processed: per_memory.len(),
        remaining,
    })
}

/// Hand each memory's share of the flat vector list to storage, in the caller's transaction.
fn write_vectors(
    st: &Store,
    cfg: &EmbedConfig,
    per_memory: &[(i64, usize)],
    vectors: &[Vec<f32>],
) -> Result<(), String> {
    let mut offset = 0;
    for (id, count) in per_memory {
        let end = offset + count;
        let slice = vectors.get(offset..end).unwrap_or_default();
        st.embedding_put(*id, &cfg.vector_key(), &cfg.fingerprint(), slice)?;
        offset = end;
    }
    Ok(())
}

/// Embed a flat text list in request-sized groups; the group boundaries never split a memory's
/// texts in a way that matters — vectors are matched back by position.
fn embed_texts_batched(
    cfg: &EmbedConfig,
    texts: &[String],
    timeout: Duration,
) -> Result<Vec<Vec<f32>>, ServiceError> {
    let mut out = Vec::with_capacity(texts.len());
    for group in texts.chunks(MAX_BATCH) {
        out.extend(embed_texts(cfg, group, timeout)?);
    }
    Ok(out)
}

/// Backfill one batch for the highest-priority candidate that can do the work (the write hook and
/// the default admin/CLI backfill run through here). The explicit per-model variant
/// (`process_pending_for` + a resolved candidate) serves the cache-management surface.
pub fn process_pending(db_path: &Path, batch: usize) -> EmbedOutcome {
    let batch = batch.clamp(1, MAX_BATCH);
    let configs =
        match crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| st.embedding_configs()) {
            Ok(configs) => configs,
            Err(e) => return EmbedOutcome::Failed(format!("cannot read database: {e}")),
        };
    if configs.is_empty() {
        return EmbedOutcome::NotConfigured;
    }
    match EMBED_FAILOVER.first_available(
        &configs,
        |cfg| cfg.circuit_identity(),
        |cfg| process_pending_for(db_path, batch, cfg),
    ) {
        Ok((_, _, outcome)) => outcome,
        Err(e) => EmbedOutcome::Failed(e),
    }
}

/// Whether a tool's committed write changes memory content and thereby invalidates vectors: the
/// single predicate both server faces (MCP / REST) use to decide whether a write triggers the
/// backfill hook. Merge rewrites the target's content (its vector was invalidated in the store
/// layer) and cascade-deletes the source's row, so it belongs here exactly like an update.
pub fn needs_backfill(tool: &str) -> bool {
    use crate::tools::{MEMORY_CREATE, MEMORY_EDIT, MEMORY_MERGE, MEMORY_UPDATE};
    matches!(
        tool,
        MEMORY_CREATE | MEMORY_EDIT | MEMORY_MERGE | MEMORY_UPDATE
    )
}

/// The write path's hook point (called after a memory-write transaction commits, on both server
/// faces). The backfill runs on a background thread: the memory is already saved, so a slow or
/// unavailable embedding service must never hold the write response hostage — vectors land
/// shortly after, or stay pending for the admin endpoint / CLI backfill (fallback semantics
/// unchanged). Saving a memory always embeds it, whether or not the candidate opted into
/// automatic sweeps.
pub fn after_write(db_path: &Path) {
    // A thin hand-off: the backfill module owns runs, per-model serialization and progress. The
    // thread only keeps the response from waiting on a pending-count read.
    let db_path = db_path.to_path_buf();
    std::thread::spawn(move || crate::backfill::start_write_run(&db_path));
}

/// Cosine similarity at which a stored memory counts as a near-duplicate of a freshly created one.
/// Advisory threshold: false positives cost a harmless hint, misses cost nothing (the exact-summary
/// duplicate_of check runs regardless).
pub const DEDUP_SIMILARITY: f32 = 0.90;

/// Make sure the memory has a vector under the given candidate's identity, embedding it
/// synchronously if pending. Every failure is an Err for the failover wrapper (the hint is
/// advisory and degrades silently upstream; the vector stays queued for the backfill).
fn ensure_vector_for(db_path: &Path, id: i64, cfg: &EmbedConfig) -> Result<(), String> {
    let loaded = crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
        let Some(row) = st.embedding_pending_for(&cfg.vector_key(), &cfg.fingerprint(), id)? else {
            return Ok(None); // vector already stored under this identity (or the memory is gone)
        };
        Ok::<_, String>(Some(row))
    })?;
    let Some((_, summary, content)) = loaded else {
        return Ok(());
    };
    let texts = embed_passage_texts(cfg, &summary, &content);
    let vectors = embed_texts_batched(cfg, &texts, QUERY_TIMEOUT)?;
    if vectors.is_empty() {
        return Ok(());
    }
    crate::store::with_db_in(db_path, TxMode::Write, |st| {
        ensure_current_config(st, cfg)?;
        st.embedding_put(id, &cfg.vector_key(), &cfg.fingerprint(), &vectors)
    })
}

/// The near-duplicate scan for one candidate: make sure the new memory has a vector under this
/// candidate's identity, then scan the identity's stored vectors. The whole scan is the failover
/// unit — a candidate that cannot embed falls through to the next one wholesale.
fn dedup_scan(db_path: &Path, id: i64, cfg: &EmbedConfig) -> Result<Vec<Value>, String> {
    ensure_vector_for(db_path, id, cfg)?;
    crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
        let table = st.embeddings_active(&cfg.vector_key(), &cfg.fingerprint())?;
        let Some(mine) = table.get(&id) else {
            return Ok(Vec::new());
        };
        let mut hits: Vec<(i64, f32)> = table
            .iter()
            .filter(|(other, _)| **other != id)
            .filter_map(|(other, theirs)| {
                // A memory's similarity is its best matching chunk on either side.
                let score = best_cosine(mine, theirs);
                (score >= DEDUP_SIMILARITY).then_some((*other, score))
            })
            .collect();
        hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let hits = hits
            .into_iter()
            .take(5)
            .map(|(other, s)| {
                json!({
                    "id": Store::format_id(other),
                    // Rounded: the third decimal is noise at hint precision
                    "similarity": (s * 1000.0).round() / 1000.0,
                })
            })
            .collect();
        Ok::<_, String>(hits)
    })
}

/// After a memory_create commits and its vector was backfilled, scan stored embeddings for
/// near-duplicates of the new memory and attach them as `similar_to` hints on the tool result —
/// closing the loop the way `duplicate_of` does for exact summaries, but for paraphrases. Pure
/// fallback semantics: no configuration, no vector yet (service down), or any read failure leaves
/// the result untouched, because the hint is a companion to the write, never a gate on it.
pub fn dedup_hint(db_path: &Path, result: &mut Value) {
    let Some(id_str) = result["id"].as_str() else {
        return;
    };
    let Some(id) = Store::parse_id(id_str) else {
        return;
    };
    let configs =
        match crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| st.embedding_configs()) {
            Ok(configs) if !configs.is_empty() => configs,
            Ok(_) => return,
            Err(e) => {
                eprintln!("semantic duplicate hint skipped: {e}");
                return;
            }
        };
    match EMBED_FAILOVER.first_available(
        &configs,
        |cfg| cfg.circuit_identity(),
        |cfg| dedup_scan(db_path, id, cfg),
    ) {
        Ok((_, _, hits)) if !hits.is_empty() => {
            result["similar_to"] = Value::Array(hits);
        }
        Ok(_) => {}
        Err(e) => eprintln!("semantic duplicate hint skipped: {e}"),
    }
}

/// Fuse content, tags and bounded semantic recall. Similarity must clear the
/// model-specific floor before an independent semantic candidate qualifies.
pub fn semantic_hits(
    memories: &[crate::model::Memory],
    table: &HashMap<i64, Vec<Vec<f32>>>,
    query_vec: &[f32],
    vector_k: usize,
    min_similarity: f32,
) -> Vec<Hit> {
    // The caller supplies eligible memories only, so top-k cannot be consumed by
    // memories outside the tag filter. A multi-chunk memory scores as its best chunk.
    let mut vector_ranked: Vec<(usize, f32)> = memories
        .iter()
        .enumerate()
        .filter_map(|(idx, m)| {
            let id = Store::parse_id(&m.id).unwrap_or(0);
            let chunks = table.get(&id)?;
            let score = chunks
                .iter()
                .map(|v| cosine(query_vec, v))
                .fold(0.0f32, f32::max);
            Some((idx, score))
        })
        // `> 0.0` also guards zero vectors (cosine 0 = never recalled, see cosine())
        .filter(|(_, score)| *score > 0.0 && *score >= min_similarity)
        .collect();
    vector_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    vector_ranked.truncate(vector_k);

    vector_ranked
        .into_iter()
        .map(|(idx, score)| Hit {
            idx,
            // Positive float bits preserve score order and exact ties without rounding.
            score: i64::from(score.to_bits()),
            snippet: search::fallback_snippet(&memories[idx].content),
        })
        .collect()
}

#[cfg(test)]
fn hybrid_hits(
    memories: &[crate::model::Memory],
    literal: [Vec<Hit>; 2],
    table: &HashMap<i64, Vec<Vec<f32>>>,
    query_vec: &[f32],
    vector_k: usize,
    min_similarity: f32,
) -> Vec<Hit> {
    let vector_hits = semantic_hits(memories, table, query_vec, vector_k, min_similarity);
    search::fuse(
        memories,
        &[(&literal[0], 1.0), (&literal[1], 1.0), (&vector_hits, 1.0)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Memory;

    /// The failure-circuit identity covers the credential and the model: fixing an invalid
    /// api_key or swapping the model releases the candidate from its predecessor's cooldown,
    /// while a trailing slash on the endpoint does not split one candidate into two identities.
    #[test]
    fn circuit_identity_tracks_credential_and_model_not_endpoint_slash() {
        let cfg = EmbedConfig {
            id: "local".into(),
            base_url: "http://x/v1/".into(),
            model: "bge".into(),
            api_key: Some("bad".into()),
            query_prefix: None,
            passage_prefix: None,
            min_similarity: 0.0,
            tokenizer: None,
            max_tokens: None,
            auto_backfill: false,
        };
        let same = EmbedConfig {
            base_url: "http://x/v1".into(),
            ..cfg.clone()
        };
        assert_eq!(cfg.circuit_identity(), same.circuit_identity());
        let fixed_key = EmbedConfig {
            api_key: Some("good".into()),
            ..cfg.clone()
        };
        assert_ne!(cfg.circuit_identity(), fixed_key.circuit_identity());
        let other_model = EmbedConfig {
            model: "e5".into(),
            ..cfg
        };
        assert_ne!(same.circuit_identity(), other_model.circuit_identity());
    }

    fn mem(id: i64, summary: &str, content: &str) -> Memory {
        Memory {
            id: Store::format_id(id),
            summary: summary.to_string(),
            content: content.to_string(),
            tags: Vec::new(),
            created_at: 1,
            updated_at: 1,
        }
    }

    fn unit(dim: usize, hot: usize) -> Vec<f32> {
        (0..dim).map(|i| if i == hot { 1.0 } else { 0.0 }).collect()
    }

    #[test]
    fn blob_roundtrip() {
        let v = vec![0.25f32, -1.5, 3.0e-7, f32::MIN];
        let back = blob_to_vec(&vec_to_blob(&v));
        assert_eq!(v, back);
        // Corrupt row: partial bytes are dropped instead of panicking
        let mut blob = vec_to_blob(&v);
        blob.push(0xAA);
        assert_eq!(blob_to_vec(&blob).len(), 4);
    }

    #[test]
    fn cosine_basics() {
        let a = unit(3, 0);
        assert!((cosine(&a, &a) - 1.0).abs() < 1e-6);
        assert!(cosine(&a, &unit(3, 1)).abs() < 1e-6);
        assert_eq!(
            cosine(&a, &unit(4, 0)),
            0.0,
            "unequal dimensions = 0, must not panic"
        );
        assert_eq!(cosine(&[0.0, 0.0], &[0.0, 0.0]), 0.0);
    }

    #[test]
    fn embed_memory_text_truncates() {
        let long = "x".repeat(MAX_INPUT_CHARS + 100);
        let text = embed_memory_text("s", &long);
        assert_eq!(text.chars().count(), MAX_INPUT_CHARS);
        assert!(text.starts_with("s\n\nx"));
    }

    fn cfg(model: &str, q: Option<&str>, p: Option<&str>) -> EmbedConfig {
        EmbedConfig {
            id: model.into(),
            base_url: "http://x".into(),
            model: model.into(),
            api_key: None,
            query_prefix: q.map(str::to_string),
            passage_prefix: p.map(str::to_string),
            min_similarity: 0.0,
            tokenizer: None,
            max_tokens: None,
            auto_backfill: false,
        }
    }

    /// Prefixes change cache validity while the user-assigned ID stays stable; the truncation
    /// regime travels with the fingerprint too, but an inert `max_tokens` (no tokenizer) never
    /// invalidates a cache.
    #[test]
    fn tokenizer_config_changes_fingerprint() {
        let mut bare = cfg("bge-m3", None, None);
        bare.tokenizer = Some("auto".into());
        assert_ne!(bare.fingerprint(), cfg("bge-m3", None, None).fingerprint());
        assert_eq!(bare.token_limit(), Some(DEFAULT_TOKEN_LIMIT));

        bare.max_tokens = Some(512);
        assert_eq!(bare.token_limit(), Some(512));
        assert_ne!(bare.fingerprint(), cfg("bge-m3", None, None).fingerprint());

        // Inert without a tokenizer: the limit alone changes nothing
        let mut limit_only = cfg("bge-m3", None, None);
        limit_only.max_tokens = Some(512);
        assert_eq!(limit_only.token_limit(), None);
        assert_eq!(
            limit_only.fingerprint(),
            cfg("bge-m3", None, None).fingerprint()
        );
    }

    /// Without a resolved tokenizer a body is one text, split or not; with one, a long body
    /// becomes several texts that each carry the title and stay inside the model window.
    #[test]
    fn long_passages_are_split_only_with_a_tokenizer() {
        let mut plain = cfg("m", None, None);
        plain.tokenizer = Some("embed-split-test".into());
        let body = "alpha ".repeat(300);
        // Configured but never resolved: still a single truncated text
        assert_eq!(embed_passage_texts(&plain, "title", &body).len(), 1);

        crate::tokenizer::inject_for_test("embed-split-test", crate::tokenizer::test_tokenizer());
        let mut split = plain;
        split.max_tokens = Some(64);
        let texts = embed_passage_texts(&split, "title", &body);
        assert!(texts.len() > 1, "a 300-token body does not fit 64 tokens");
        let tok = crate::tokenizer::test_tokenizer();
        for text in &texts {
            assert!(text.starts_with("title"), "every chunk keeps the title");
            let n = tok.encode(text.as_str(), true).unwrap().get_ids().len();
            assert!(n <= 64, "chunk of {n} tokens exceeds the window");
        }

        // A short body stays one text, unchanged from the single-vector form
        let short = embed_passage_texts(&split, "title", "alpha beta");
        assert_eq!(
            short,
            vec![embed_passage_text(&split, "title", "alpha beta")]
        );
    }

    /// A multi-chunk memory ranks by its best chunk, not by the average across its body.
    #[test]
    fn chunked_memory_scores_as_its_best_chunk() {
        let memories = vec![mem(1, "long", "c"), mem(2, "other", "c")];
        let mut table: HashMap<i64, Vec<Vec<f32>>> = HashMap::new();
        // m1: one irrelevant chunk and one exactly on the query
        table.insert(1, vec![unit(3, 2), unit(3, 0)]);
        table.insert(2, vec![unit(3, 1)]);
        let hits = semantic_hits(&memories, &table, &unit(3, 0), 10, 0.9);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);
    }

    /// Entry round-trip keeps the new fields; out-of-range values normalize to defaults
    /// (hand-edited databases must not crash the parse).
    #[test]
    fn entry_round_trips_tokenizer_fields() {
        let entry = EmbedEntry {
            id: "m3".into(),
            name: "bge-m3".into(),
            enabled: true,
            base_url: "https://api.siliconflow.cn/v1".into(),
            model: "BAAI/bge-m3".into(),
            api_key: None,
            query_prefix: None,
            passage_prefix: None,
            min_similarity: None,
            tokenizer: Some("auto".into()),
            max_tokens: Some(512),
            auto_backfill: false,
        };
        let parsed = EmbedEntry::from_json(&entry.to_json()).unwrap();
        assert_eq!(parsed.tokenizer.as_deref(), Some("auto"));
        assert_eq!(parsed.max_tokens, Some(512));
        assert_eq!(parsed.config().unwrap().token_limit(), Some(512));

        let mut junk = entry.to_json();
        junk["max_tokens"] = json!(3);
        assert_eq!(EmbedEntry::from_json(&junk).unwrap().max_tokens, None);
        junk["max_tokens"] = json!("not-a-number");
        assert_eq!(EmbedEntry::from_json(&junk).unwrap().max_tokens, None);
        assert_eq!(EmbedEntry::from_json(&json!({})).unwrap().tokenizer, None);
    }

    #[test]
    fn changed_configuration_rejects_in_flight_vectors() {
        use crate::store::test_support::{cleanup, temp_db};
        let path = temp_db("embedding-stale-config");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let mut entry =
            json!({"id":"service", "name":"Service", "model":"model", "base_url":"http://x"});
        st.settings_put(
            Store::SETTING_EMBEDDING_MODELS,
            &json!([entry.clone()]).to_string(),
        )
        .unwrap();
        let original = st.embedding_configs().unwrap()[0].clone();
        assert!(ensure_current_config(&st, &original).is_ok());
        st.insert_memory("s", "body", &[], 1, 1).unwrap();
        st.embedding_put(1, &original.id, &original.fingerprint(), &[vec![1.0]])
            .unwrap();
        entry["model"] = json!("replacement");
        st.settings_put(Store::SETTING_EMBEDDING_MODELS, &json!([entry]).to_string())
            .unwrap();
        assert!(ensure_current_config(&st, &original).is_err());
        assert!(st
            .embeddings_active(&original.id, &original.fingerprint())
            .unwrap()
            .is_empty());
        assert_eq!(
            st.embedding_pending_count(&original.id, Some(&original.fingerprint()))
                .unwrap(),
            1
        );
        drop(st);
        cleanup(&path);
    }

    /// The semantic recall floor: weak candidates never enter the fusion, so a query matching
    /// nothing returns nothing instead of the whole store ranked. Keyword hits face no floor.
    #[test]
    fn semantic_floor_filters_weak_candidates() {
        let memories: Vec<Memory> = (0..3).map(|i| mem(i + 1, "s", "c")).collect();
        let mut table = HashMap::new();
        table.insert(1i64, vec![vec![1.0, 0.0]]); // cosine 1.0 with the query
        table.insert(2i64, vec![vec![0.9, 0.1]]); // cosine ≈ 0.994
        table.insert(3i64, vec![vec![0.6, 0.8]]); // cosine 0.6

        let hits = hybrid_hits(
            &memories,
            [Vec::new(), Vec::new()],
            &table,
            &[1.0, 0.0],
            10,
            0.7,
        );
        let ids: Vec<&str> = hits.iter().map(|h| memories[h.idx].id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["m1", "m2"],
            "the 0.6-cosine candidate is below the floor"
        );

        let hits = hybrid_hits(
            &memories,
            [Vec::new(), Vec::new()],
            &table,
            &[1.0, 0.0],
            10,
            0.0,
        );
        assert_eq!(hits.len(), 3, "floor 0 keeps every positive candidate");

        let hits = hybrid_hits(
            &memories,
            [Vec::new(), Vec::new()],
            &table,
            &[1.0, 0.0],
            10,
            1.5,
        );
        assert_eq!(hits.len(), 0, "nothing is close enough");

        let keyword_hit = Hit {
            idx: 2,
            score: 5,
            snippet: "s3".into(),
        };
        let hits = hybrid_hits(
            &memories,
            [vec![keyword_hit], Vec::new()],
            &table,
            &[1.0, 0.0],
            10,
            0.7,
        );
        assert_eq!(
            hits.len(),
            3,
            "keyword hits are AND-limited, not similarity-ranked: no floor applies"
        );
    }

    /// Prefixes change cache validity while the user-assigned ID stays stable.
    #[test]
    fn prefixes_apply_per_side_and_change_fingerprint() {
        let bare = cfg("bge-m3", None, None);
        assert_eq!(bare.vector_key(), "bge-m3");
        assert_eq!(embed_query_text(&bare, "find x"), "find x");
        assert_eq!(embed_passage_text(&bare, "t", "b"), "t\n\nb");

        let e5 = cfg("e5", Some("query: "), Some("passage: "));
        assert_eq!(e5.vector_key(), "e5");
        assert_ne!(e5.fingerprint(), cfg("e5", None, None).fingerprint());
        // Trailing space of "query: " is part of the instruction — never trimmed
        assert_eq!(embed_query_text(&e5, "find x"), "query: find x");
        assert_eq!(embed_passage_text(&e5, "t", "b"), "passage: t\n\nb");

        // Only one side set: the key reflects exactly what is configured
        assert_ne!(
            cfg("e5", Some("query: "), None).fingerprint(),
            e5.fingerprint()
        );
    }

    /// A service URL pasted with the full endpoint path (provider docs show complete URLs) must
    /// be indistinguishable from the root spelling: same endpoint, same cache identity.
    #[test]
    fn pasted_endpoint_base_url_matches_root_spelling() {
        let entry = |base_url: &str| {
            EmbedEntry::from_json(&json!({
                "id": "bge", "name": "bge", "enabled": true,
                "base_url": base_url, "model": "BAAI/bge-m3",
            }))
            .expect("entry parses")
            .config()
            .expect("complete")
        };
        let root = entry("https://api.siliconflow.cn/v1");
        assert_eq!(
            root.endpoint(),
            entry("https://api.siliconflow.cn/v1/embeddings").endpoint()
        );
        assert_eq!(
            root.fingerprint(),
            entry("https://api.siliconflow.cn/v1/embeddings/").fingerprint()
        );
        assert_eq!(
            root.circuit_identity(),
            entry("https://api.siliconflow.cn/v1/embeddings/").circuit_identity()
        );
        // The settings round-trip keeps what the user typed
        assert_eq!(
            EmbedEntry::from_json(&json!({
                "id": "bge", "name": "bge", "enabled": true,
                "base_url": "https://api.siliconflow.cn/v1/embeddings/", "model": "m",
            }))
            .expect("entry parses")
            .base_url,
            "https://api.siliconflow.cn/v1/embeddings/"
        );
    }

    /// Entry parsing (settings array elements): defaults applied leniently, disabled or
    /// incomplete entries excluded from the operational pool but preserved for the round-trip,
    /// and the cache identity follows the user-assigned ID.
    #[test]
    fn entry_parsing_lenient_defaults_and_usable_view() {
        let full = EmbedEntry::from_json(&json!({
            "enabled": false,
            "base_url": " http://x/v1 ",
            "id": "e5", "name": "e5", "model": " e5 ",
            "api_key": "sk",
            "query_prefix": "query: ",
            "passage_prefix": "passage: ",
            "min_similarity": 0.45,
        }))
        .expect("full entry parses");
        assert!(!full.enabled);
        assert_eq!(full.vector_key(), "e5");
        assert!(full.usable().is_none(), "disabled = not operational");
        // ... but a disabled candidate still has an operational view: the connection probe checks
        // exactly what was configured, before the switch is turned on
        assert_eq!(
            full.config().expect("complete entry").model,
            "e5",
            "the enabled gate lives in usable(), not in config()"
        );
        // The usable view is the operational config (trimmed), enabled via the entry
        let enabled = EmbedEntry {
            enabled: true,
            ..full
        };
        let cfg = enabled.usable().expect("enabled + complete = usable");
        assert_eq!(cfg.base_url, "http://x/v1", "trimmed");
        assert_eq!(cfg.min_similarity, 0.45);
        let unnamed = EmbedEntry {
            name: String::new(),
            ..enabled
        };
        assert_eq!(
            unnamed.usable().expect("name is optional").vector_key(),
            "e5"
        );

        // Minimal entry: enabled by default, floor defaults, unusable while fields are blank
        let bare = EmbedEntry::from_json(&json!({})).expect("empty object still parses");
        assert!(bare.enabled);
        assert_eq!(bare.base_url, "");
        assert_eq!(bare.min_similarity, None);
        assert!(bare.usable().is_none(), "blank base_url/model = not usable");

        // Wrong inner types normalize instead of failing; out-of-range floors fall back to default
        let odd = EmbedEntry::from_json(&json!({
            "base_url": "http://x", "id": "m", "name": "m", "model": "m", "enabled": "yes", "min_similarity": 7.0,
        }))
        .expect("lenient parse");
        let usable = odd.usable().expect("model+url present = usable");
        assert!((usable.min_similarity - DEFAULT_MIN_SIMILARITY).abs() < 1e-6);

        // Round-trip: to_json carries every raw field back (min_similarity None stays null)
        let round = EmbedEntry::from_json(&odd.to_json()).expect("round-trip parses");
        assert_eq!(round.model, "m");
        assert_eq!(round.min_similarity, None);
    }

    /// The candidate failover: priority order wins when everything works, the first healthy
    /// candidate takes over when an earlier one fails, and the sticky hint survives across calls
    /// (while tolerating list edits by clamping).
    #[test]
    fn failover_tries_candidates_in_priority_order() {
        let failover = Failover::new();
        let configs = [
            cfg("a", None, None),
            cfg("b", None, None),
            cfg("c", None, None),
        ];
        let calls = std::cell::RefCell::new(Vec::new());
        let a_down = std::cell::Cell::new(false);
        let probe = |c: &EmbedConfig| {
            calls.borrow_mut().push(c.model.clone());
            if c.model == "a" && a_down.get() {
                Err("a is down".to_string())
            } else {
                Ok(c.model.clone())
            }
        };
        let identity = |c: &EmbedConfig| c.id.clone();

        // First call: a (priority head) answers
        let (idx, _, value) = failover.first_available(&configs, identity, probe).unwrap();
        assert_eq!((idx, value.as_str()), (0, "a"));

        // Head goes down: b takes over, and becomes the sticky choice
        a_down.set(true);
        let (idx, _, _) = failover.first_available(&configs, identity, probe).unwrap();
        assert_eq!(idx, 1);

        // Next call starts at the sticky b (the still-dead a is never probed again)
        calls.borrow_mut().clear();
        let (idx, _, _) = failover.first_available(&configs, identity, probe).unwrap();
        assert_eq!(idx, 1);
        assert!(
            !calls.borrow().iter().any(|m| m == "a"),
            "sticky skips the dead head: {:?}",
            calls.borrow()
        );

        // All candidates failing surfaces the last error
        let err = failover
            .first_available(&configs, identity, |_| Err::<String, String>("down".into()))
            .unwrap_err();
        assert_eq!(err, "down");

        // An edited (shrunken) list: the stale sticky index is clamped, no panic
        let small = [cfg("x", None, None)];
        let (idx, _, _) = failover
            .first_available(&small, identity, |c| Ok::<String, String>(c.model.clone()))
            .unwrap();
        assert_eq!(idx, 0);
    }

    /// The failure circuit: candidates that just failed are skipped without traffic for the
    /// cooldown window (so a search burst against a dead service pays the connect failure once),
    /// half-open after the window, and a success clears the record.
    #[test]
    fn failure_cooldown_skips_dead_candidates_then_half_opens() {
        // The identity is the production shape: id + fingerprint, so a reconfigured candidate is
        // never skipped for its predecessor's failure
        let identity = |c: &EmbedConfig| format!("{} {}", c.vector_key(), c.fingerprint());
        let failover = Failover::with_cooldown(Duration::from_millis(80));
        let configs = [cfg("a", None, None), cfg("b", None, None)];
        let calls = std::cell::RefCell::new(Vec::new());
        let b_down = std::cell::Cell::new(true);
        let probe = |c: &EmbedConfig| {
            calls.borrow_mut().push(c.model.clone());
            if c.model == "a" || b_down.get() {
                Err(format!("{} is down", c.model))
            } else {
                Ok(c.model.clone())
            }
        };

        // First call: every attempt fails — both circuits arm
        let err = failover
            .first_available(&configs, identity, probe)
            .unwrap_err();
        assert_eq!(err, "b is down", "the last error surfaces");
        assert_eq!(calls.borrow().as_slice(), ["a", "b"]);

        // Within the cooldown nothing is probed: degrade immediately with the circuit message
        b_down.set(false);
        let err = failover
            .first_available(&configs, identity, probe)
            .unwrap_err();
        assert!(err.contains("cooldown"), "circuit message: {err}");
        assert_eq!(
            calls.borrow().as_slice(),
            ["a", "b"],
            "no traffic while every circuit is open"
        );

        // Half-open: past the window the candidates are probed again, and b's success clears
        // its record while a's failure re-arms its circuit
        std::thread::sleep(Duration::from_millis(100));
        calls.borrow_mut().clear();
        let (idx, _, _) = failover.first_available(&configs, identity, probe).unwrap();
        assert_eq!(idx, 1, "b recovered and answers");
        assert_eq!(calls.borrow().as_slice(), ["a", "b"]);

        // b's circuit is clear and it is sticky: an immediate call probes b only, the still
        // cooled-down a stays untouched
        calls.borrow_mut().clear();
        let (idx, _, _) = failover.first_available(&configs, identity, probe).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(calls.borrow().as_slice(), ["b"]);

        // Manual release (what the admin connection probe does): a has recovered (the user
        // restarted the service) but its cooldown is still running, and sticky b just went
        // down — without the release the walk skips the recovered candidate
        let a_down = std::cell::Cell::new(false);
        let probe = |c: &EmbedConfig| {
            calls.borrow_mut().push(c.model.clone());
            if (c.model == "a" && a_down.get()) || (c.model == "b" && b_down.get()) {
                Err(format!("{} is down", c.model))
            } else {
                Ok(c.model.clone())
            }
        };
        b_down.set(true);
        calls.borrow_mut().clear();
        let err = failover
            .first_available(&configs, identity, probe)
            .unwrap_err();
        assert_eq!(err, "b is down");
        assert_eq!(
            calls.borrow().as_slice(),
            ["b"],
            "recovered-but-cooled a stays skipped: {:?}",
            calls.borrow()
        );
        // The probe's clear: the next call reaches a immediately. b, having failed in the
        // previous call, is now the one behind a circuit and is skipped — the walk starts at
        // the cleared, recovered candidate and answers.
        failover.clear(&format!(
            "{} {}",
            configs[0].vector_key(),
            configs[0].fingerprint()
        ));
        calls.borrow_mut().clear();
        let (idx, _, _) = failover.first_available(&configs, identity, probe).unwrap();
        assert_eq!(idx, 0, "cleared a answers right after the probe");
        assert_eq!(
            calls.borrow().as_slice(),
            ["a"],
            "b is now the cooled one; a is probed directly: {:?}",
            calls.borrow()
        );
    }

    /// Paraphrased recall: the keyword pass only hits m1, while the vector pass also brings in the semantically close m2,
    /// and m1 (hit by both channels) ranks ahead of m2 (vector only).
    #[test]
    fn hybrid_brings_in_vector_only_hits() {
        let memories = vec![
            mem(1, "token hashing", "sha256 of tokens"),
            mem(2, "密码保存", "哈希存储"),
        ];
        let keyword_hits = search::literal_matches(&memories, "hashing").channels;
        assert_eq!(keyword_hits[0].len(), 1);

        let mut table = HashMap::new();
        table.insert(1i64, vec![unit(4, 0)]); // same direction as the query
        table.insert(2i64, vec![unit(4, 1)]); // orthogonal to the query (similarity 0, never enters)

        let hits = hybrid_hits(
            &memories,
            keyword_hits,
            &table,
            &unit(4, 0),
            usize::MAX,
            0.0,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);

        // m2 enters the results when its similarity to the query is 0.87 (non-orthogonal)
        let tilted: Vec<Vec<f32>> = vec![vec![0.9, 0.1, 0.0, 0.0]];
        let mut table2 = table.clone();
        table2.insert(2i64, tilted);
        let keyword_hits = search::literal_matches(&memories, "hashing").channels;
        let hits = hybrid_hits(
            &memories,
            keyword_hits,
            &table2,
            &unit(4, 0),
            usize::MAX,
            0.0,
        );
        assert_eq!(hits.len(), 2);
        assert_eq!(
            hits[0].idx, 0,
            "a double hit must rank ahead of a vector-only hit"
        );
        assert_eq!(hits[1].idx, 1);
        // Vector-only hits take the fallback snippet (start of content), plain text
        assert!(hits[1].snippet.contains("哈希存储"));
    }

    /// Fusion when keyword and vector rankings conflict: an item ranking high on both channels must beat either channel's top single-channel item.
    #[test]
    fn hybrid_fuses_both_ranks() {
        let a = mem(1, "aa", "a");
        let b = mem(2, "bb", "b");
        let c = mem(3, "cc", "c");
        // Keyword ranks: a first, b second; vector ranks: b first, c second, a third
        // (vectors take directions at distinct angles from the query, distinguishable by cosine)
        let k1 = Hit {
            idx: 0,
            score: 100,
            snippet: "sa".into(),
        };
        let k2 = Hit {
            idx: 1,
            score: 50,
            snippet: "sb".into(),
        };
        let mut table = HashMap::new();
        table.insert(2i64, vec![vec![0.95, 0.31]]);
        table.insert(3i64, vec![vec![0.9, 0.44]]);
        table.insert(1i64, vec![vec![0.5, 0.87]]);
        let hits = hybrid_hits(
            &[a, b, c],
            [vec![k1, k2], Vec::new()],
            &table,
            &[1.0, 0.0],
            usize::MAX,
            0.0,
        );
        let order: Vec<usize> = hits.iter().map(|h| h.idx).collect();
        assert_eq!(
            order,
            vec![1, 0, 2],
            "b, ranked on both channels, should beat a, the keyword-only leader"
        );
    }

    /// The semantic channel is capped at `vector_k` candidates: only the highest-cosine memories
    /// enter the fusion, so total_matches stays bounded no matter how large the store grows.
    #[test]
    fn hybrid_caps_vector_channel() {
        let memories: Vec<Memory> = (0..6).map(|i| mem(i + 1, "s", "c")).collect();
        let mut table = HashMap::new();
        // Vectors tilted progressively further from the query: cosine descends with i, so m1 is
        // the top candidate and m6 the weakest
        for i in 0..6usize {
            table.insert((i + 1) as i64, vec![vec![1.0, i as f32]]);
        }
        let hits = hybrid_hits(
            &memories,
            [Vec::new(), Vec::new()],
            &table,
            &[1.0, 0.0],
            3,
            0.0,
        );
        assert_eq!(hits.len(), 3);
        let ids: Vec<&str> = hits.iter().map(|h| memories[h.idx].id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["m1", "m2", "m3"],
            "only the top-3 cosine candidates survive the cap"
        );
    }

    /// The cap trims the vector channel only: keyword hits are already AND-limited and must all
    /// survive fusion even when they outnumber vector_k.
    #[test]
    fn vector_cap_never_truncates_keyword_hits() {
        let memories: Vec<Memory> = (0..4).map(|i| mem(i + 1, "s", "c")).collect();
        let keyword_hits = vec![
            Hit {
                idx: 0,
                score: 10,
                snippet: "sa".into(),
            },
            Hit {
                idx: 1,
                score: 5,
                snippet: "sb".into(),
            },
            Hit {
                idx: 2,
                score: 3,
                snippet: "sc".into(),
            },
        ];
        let mut table = HashMap::new();
        table.insert(4i64, vec![vec![1.0, 0.0]]);
        let hits = hybrid_hits(
            &memories,
            [keyword_hits, Vec::new()],
            &table,
            &[1.0, 0.0],
            1,
            0.0,
        );
        let ids: std::collections::HashSet<usize> = hits.iter().map(|h| h.idx).collect();
        assert!(
            ids.is_superset(&[0usize, 1, 2].into_iter().collect()),
            "all three keyword hits survive a vector_k of 1"
        );
        assert_eq!(
            hits.len(),
            4,
            "3 keyword hits + at most 1 vector candidate (the cap) — not the whole table"
        );
    }

    /// The create-time dedup hint: near-identical vectors above the threshold land in similar_to
    /// (newest hit first, self excluded); unrelated or missing vectors leave the result untouched,
    /// and so does missing configuration — the hint is a companion, never a gate.
    #[test]
    fn dedup_hint_flags_near_duplicates_only() {
        use crate::store::test_support::{cleanup, temp_db};

        let path = temp_db("dedup-hint");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        // Configure semantic search (the hint reads the same settings the search path does)
        st.settings_put(
            crate::store::Store::SETTING_EMBEDDING_MODELS,
            r#"[{"base_url": "http://127.0.0.1:9", "id": "test-model", "name": "test-model", "model": "test-model"}]"#,
        )
        .unwrap();

        let fingerprint = st.embedding_configs().unwrap()[0].fingerprint();
        // m2 is a near-duplicate of m1 (similarity 1), m3 is unrelated
        st.insert_memory("a", "body a", &[], 1, 1).unwrap();
        st.insert_memory("b", "body b", &[], 1, 1).unwrap();
        st.insert_memory("c", "body c", &[], 1, 1).unwrap();
        st.embedding_put(1, "test-model", &fingerprint, &[vec![1.0, 0.0]])
            .unwrap();
        st.embedding_put(2, "test-model", &fingerprint, &[vec![0.999, 0.045]])
            .unwrap();
        st.embedding_put(3, "test-model", &fingerprint, &[vec![0.0, 1.0]])
            .unwrap();

        let mut result = serde_json::json!({"id": "m1"});
        dedup_hint(&path, &mut result);
        let hits = result["similar_to"].as_array().expect("hint attached");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["id"], "m2");
        assert!(hits[0]["similarity"].as_f64().unwrap() >= DEDUP_SIMILARITY as f64);

        // No vector for the created memory yet (service was down): silent no-op
        let mut result = serde_json::json!({"id": "m9"});
        dedup_hint(&path, &mut result);
        assert!(result.get("similar_to").is_none());

        // No configuration at all: silent no-op
        let path2 = temp_db("dedup-hint-noconf");
        cleanup(&path2);
        let st2 = Store::open(&path2).unwrap();
        st2.insert_memory("a", "body", &[], 1, 1).unwrap();
        st2.embedding_put(1, "test-model", "unused", &[vec![1.0]])
            .unwrap();
        let mut result = serde_json::json!({"id": "m1"});
        dedup_hint(&path2, &mut result);
        assert!(result.get("similar_to").is_none());

        cleanup(&path);
        cleanup(&path2);
    }
}
