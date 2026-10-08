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
//! Network calls always happen outside database transactions (guaranteed by callers): holding an IMMEDIATE lock across
//! network I/O in a write transaction is a forbidden deadlock shape, and stretching a read-only snapshot serves no purpose either.

use crate::search::{self, Hit};
use crate::store::{Store, TxMode};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

/// Embedding service configuration (an OpenAI-compatible `/embeddings` endpoint: cloud APIs and
/// local services such as Ollama/LM Studio share the same shape).
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
}

impl EmbedConfig {
    /// Stable, user-assigned cache identity, independent of order and display name.
    pub fn vector_key(&self) -> String {
        self.id.clone()
    }

    /// Cache validity is separate from identity. JSON avoids delimiter collisions.
    pub fn fingerprint(&self) -> String {
        json!([
            self.base_url.trim_end_matches('/'),
            self.model,
            self.query_prefix.as_deref().unwrap_or(""),
            self.passage_prefix.as_deref().unwrap_or("")
        ])
        .to_string()
    }

    fn endpoint(&self) -> String {
        format!("{}/embeddings", self.base_url.trim_end_matches('/'))
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
        })
    }

    /// The operational view: usable only when enabled and base_url / model are both non-empty.
    pub fn usable(&self) -> Option<EmbedConfig> {
        if !self.enabled || self.id.is_empty() || self.name.is_empty() {
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
            base_url: self.base_url.trim().to_string(),
            model: self.model.trim().to_string(),
            api_key: self.api_key.clone(),
            query_prefix: self.query_prefix.clone(),
            passage_prefix: self.passage_prefix.clone(),
            min_similarity: self.min_similarity.unwrap_or(DEFAULT_MIN_SIMILARITY),
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
pub struct Failover {
    preferred: AtomicUsize,
}

impl Failover {
    pub const fn new() -> Self {
        Failover {
            preferred: AtomicUsize::new(0),
        }
    }

    /// Run `op` against the first candidate that succeeds. Every failed attempt is logged (with
    /// its position) and the next candidate takes over; only when all fail does the last error
    /// surface. The sticky preferred index is a hint: a list edit merely makes it fall back to
    /// priority order (it is clamped, and a stale success index costs one failed attempt at most).
    pub fn first_available<'a, C, T>(
        &self,
        configs: &'a [C],
        mut op: impl FnMut(&C) -> Result<T, String>,
    ) -> Result<(usize, &'a C, T), String> {
        if configs.is_empty() {
            return Err("no candidate service is configured".to_string());
        }
        let start = self
            .preferred
            .load(Ordering::Relaxed)
            .min(configs.len() - 1);
        let mut last_err = String::new();
        for offset in 0..configs.len() {
            let idx = (start + offset) % configs.len();
            match op(&configs[idx]) {
                Ok(value) => {
                    self.preferred.store(idx, Ordering::Relaxed);
                    return Ok((idx, &configs[idx], value));
                }
                Err(e) => {
                    eprintln!("embedding candidate #{idx} failed, trying the next: {e}");
                    last_err = e;
                }
            }
        }
        Err(last_err)
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
const MAX_INPUT_CHARS: usize = 8_000;
/// The RRF (Reciprocal Rank Fusion) constant: earlier ranks contribute more, and K smooths out head-of-list weight differences.
const RRF_K: f64 = 60.0;

/// Default floor for the semantic recall channel: vector candidates below this cosine never
/// enter the fusion. Deliberately conservative — baselines are model-specific (bge-family
/// unrelated pairs score ~0.4, OpenAI text-embedding-3 ~0.2) — and configurable per model via
/// the entry's `min_similarity` field; 0 disables the floor entirely.
pub const DEFAULT_MIN_SIMILARITY: f32 = 0.30;

/// The memory text sent to the embedding service: title + blank line + content (truncated).
pub fn embed_memory_text(summary: &str, content: &str) -> String {
    let mut text = String::new();
    text.push_str(summary.trim());
    if !content.trim().is_empty() {
        text.push_str("\n\n");
        text.push_str(content.trim());
    }
    if text.chars().count() > MAX_INPUT_CHARS {
        text = text.chars().take(MAX_INPUT_CHARS).collect();
    }
    text
}

/// Query-side text: the model's query instruction prefix, then the query itself.
pub fn embed_query_text(cfg: &EmbedConfig, query: &str) -> String {
    let text = embed_memory_text(query, "");
    match &cfg.query_prefix {
        Some(p) => format!("{p}{text}"),
        None => text,
    }
}

/// Passage-side text: the model's passage prefix, then title + content.
pub fn embed_passage_text(cfg: &EmbedConfig, summary: &str, content: &str) -> String {
    let text = embed_memory_text(summary, content);
    match &cfg.passage_prefix {
        Some(p) => format!("{p}{text}"),
        None => text,
    }
}

/// Embed a batch. Any network/parsing failure returns Err; the caller decides how to degrade.
pub fn embed_texts(
    cfg: &EmbedConfig,
    texts: &[String],
    timeout: Duration,
) -> Result<Vec<Vec<f32>>, String> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let mut req = ureq::post(&cfg.endpoint()).timeout(timeout);
    if let Some(key) = &cfg.api_key {
        req = req.set("Authorization", &format!("Bearer {key}"));
    }
    let resp = req
        .send_json(json!({ "model": cfg.model, "input": texts }))
        .map_err(|e| format!("embedding request failed: {e}"))?;
    let status = resp.status();
    let body: Value = resp
        .into_json()
        .map_err(|e| format!("embedding response is not valid JSON: {e}"))?;
    if status != 200 {
        let detail = body["error"]["message"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("unknown error");
        return Err(format!("embedding service returned {status}: {detail}"));
    }
    let mut data = body["data"]
        .as_array()
        .ok_or("embedding response missing 'data' array")?
        .clone();
    data.sort_by_key(|item| item["index"].as_u64().unwrap_or(0));
    if data.len() != texts.len() {
        return Err(format!(
            "embedding service returned {} vectors for {} inputs",
            data.len(),
            texts.len()
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

/// Result of one backfill batch. The distinction between NotConfigured and Failed shapes caller behavior:
/// the write path stays silent for both (Failed logs to stderr), while the admin endpoint surfaces details to the UI.
pub enum EmbedOutcome {
    NotConfigured,
    Processed { processed: usize, remaining: usize },
    Failed(String),
}

impl EmbedOutcome {
    pub fn to_json(&self) -> Value {
        match self {
            EmbedOutcome::NotConfigured => json!({ "configured": false }),
            EmbedOutcome::Processed {
                processed,
                remaining,
            } => json!({
                "configured": true,
                "processed": processed,
                "remaining": remaining,
            }),
            EmbedOutcome::Failed(e) => json!({ "configured": true, "error": e }),
        }
    }
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

    let texts: Vec<String> = pending
        .iter()
        .map(|(_, summary, content)| embed_passage_text(cfg, summary, content))
        .collect();
    let vectors = embed_texts(cfg, &texts, BATCH_TIMEOUT)?;

    // Storing and counting share one write transaction: the count sees this batch's puts, so the
    // reported remaining is identical to a post-commit recount
    let remaining = crate::store::with_db_in(db_path, TxMode::Write, |st| {
        ensure_current_config(st, cfg)?;
        for ((id, _, _), vec) in pending.iter().zip(&vectors) {
            st.embedding_put(*id, &cfg.vector_key(), &cfg.fingerprint(), vec)?;
        }
        st.embedding_pending_count(&cfg.vector_key(), Some(&cfg.fingerprint()))
    })?;
    Ok(EmbedOutcome::Processed {
        processed: pending.len(),
        remaining,
    })
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
    match EMBED_FAILOVER.first_available(&configs, |cfg| process_pending_for(db_path, batch, cfg)) {
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

/// Single-flight guard for the background backfill worker: at most one drain loop runs at a time,
/// so a burst of writes cannot pile up threads.
static BACKFILL_RUNNING: AtomicBool = AtomicBool::new(false);

/// The write path's hook point (called after a memory-write transaction commits, on both server
/// faces). The backfill runs on a background thread: the memory is already saved, so a slow or
/// unavailable embedding service must never hold the write response hostage — vectors land
/// shortly after, or stay pending for the admin endpoint / CLI backfill (fallback semantics
/// unchanged). Writes landing while a worker drains are covered by that worker's loop; the only
/// miss window (a write committing after the worker's final count returned zero) is closed by the
/// next write or the next backfill run.
pub fn after_write(db_path: &Path) {
    use std::sync::atomic::Ordering;
    if BACKFILL_RUNNING.swap(true, Ordering::AcqRel) {
        return; // a worker is already draining the queue
    }
    let db_path = db_path.to_path_buf();
    std::thread::spawn(move || {
        loop {
            match process_pending(&db_path, 4) {
                EmbedOutcome::Processed { remaining, .. } if remaining > 0 => continue,
                EmbedOutcome::Failed(e) => {
                    eprintln!(
                        "embedding backfill failed (the memory itself is saved; it will be retried by the next backfill): {e}"
                    );
                    break;
                }
                _ => break, // queue drained, or semantic search not configured
            }
        }
        BACKFILL_RUNNING.store(false, Ordering::Release);
    });
}

// `use std::sync::atomic::AtomicBool` is spelled out at the top; the static above needs the type only.

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
    let vectors = embed_texts(
        cfg,
        &[embed_passage_text(cfg, &summary, &content)],
        QUERY_TIMEOUT,
    )?;
    let Some(vec) = vectors.into_iter().next() else {
        return Ok(());
    };
    crate::store::with_db_in(db_path, TxMode::Write, |st| {
        ensure_current_config(st, cfg)?;
        st.embedding_put(id, &cfg.vector_key(), &cfg.fingerprint(), &vec)
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
            .map(|(other, vec)| (*other, cosine(mine, vec)))
            .filter(|(_, s)| *s >= DEDUP_SIMILARITY)
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
    match EMBED_FAILOVER.first_available(&configs, |cfg| dedup_scan(db_path, id, cfg)) {
        Ok((_, _, hits)) if !hits.is_empty() => {
            result["similar_to"] = Value::Array(hits);
        }
        Ok(_) => {}
        Err(e) => eprintln!("semantic duplicate hint skipped: {e}"),
    }
}

/// RRF fusion ranking over keyword + vector recall.
///
/// Each channel contributes `1/(K+rank)` by rank, which naturally sidesteps the incomparable units of "keyword TF scores"
/// versus "cosine values". Vector-only hits (zero keyword hits but semantically close) are introduced by this function
/// — exactly the point of semantic search; their snippets take the fallback path from the start of the content.
///
/// The vector channel is capped at `vector_k` candidates (top cosine) before fusion: the keyword channel is already
/// self-limited by its all-terms AND, but without a cap the vector channel would pull in every stored vector, making
/// `total_matches` equal the store size and burying the head of the ranking in a noise tail. `vector_k` derives from
/// the caller's `limit` (roughly twice it, floored) so the cap scales with how much the caller actually reads.
///
/// On top of the rank cap, candidates below `min_similarity` cosine never qualify at all: a rank
/// cap alone cannot fix small stores (when every memory fits under the cap, a query matching
/// nothing still returns the whole store ranked). The floor is a query-time relevance bar —
/// model-specific baselines make it configurable rather than universal (`DEFAULT_MIN_SIMILARITY`).
pub fn hybrid_hits(
    memories: &[crate::model::Memory],
    keyword_hits: Vec<Hit>,
    table: &HashMap<i64, Vec<f32>>,
    query_vec: &[f32],
    vector_k: usize,
    min_similarity: f32,
) -> Vec<Hit> {
    // Vector pass: every memory with a stored vector above the floor is a candidate, ranked by
    // cosine descending (tag filtering narrows the fused hits one layer up, so both recall
    // channels stay symmetric)
    let mut vector_ranked: Vec<(usize, f32)> = memories
        .iter()
        .enumerate()
        .filter_map(|(idx, m)| {
            let id = Store::parse_id(&m.id).unwrap_or(0);
            table.get(&id).map(|v| (idx, cosine(query_vec, v)))
        })
        // `> 0.0` also guards zero vectors (cosine 0 = never recalled, see cosine())
        .filter(|(_, score)| *score > 0.0 && *score >= min_similarity)
        .collect();
    vector_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    vector_ranked.truncate(vector_k);

    let mut fused: HashMap<usize, f64> = HashMap::new();
    let mut keyword_snippets: HashMap<usize, &str> = HashMap::new();
    for (rank, hit) in keyword_hits.iter().enumerate() {
        *fused.entry(hit.idx).or_default() += 1.0 / (RRF_K + rank as f64);
        keyword_snippets.insert(hit.idx, hit.snippet.as_str());
    }
    for (rank, (idx, _)) in vector_ranked.iter().enumerate() {
        *fused.entry(*idx).or_default() += 1.0 / (RRF_K + rank as f64);
    }

    let mut items: Vec<(usize, f64)> = fused.into_iter().collect();
    // Same tie-break as the keyword pass: score → updated_at descending → id ascending
    items.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(memories[b.0].updated_at.cmp(&memories[a.0].updated_at))
            .then(memories[a.0].id.cmp(&memories[b.0].id))
    });

    items
        .into_iter()
        .map(|(idx, score)| {
            // RRF values are tiny (<0.033); scaling by 1e6 into integers preserves order, not precision — the score field
            // is an internal value for ordering only and promises no interpretability
            let score = (score * 1_000_000.0).round() as i64;
            let snippet = match keyword_snippets.get(&idx) {
                Some(s) => (*s).to_string(),
                None => search::fallback_snippet(&memories[idx].content),
            };
            Hit {
                idx,
                score,
                snippet,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Memory;

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
        }
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
        st.embedding_put(1, &original.id, &original.fingerprint(), &[1.0])
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
        table.insert(1i64, vec![1.0, 0.0]); // cosine 1.0 with the query
        table.insert(2i64, vec![0.9, 0.1]); // cosine ≈ 0.994
        table.insert(3i64, vec![0.6, 0.8]); // cosine 0.6

        let hits = hybrid_hits(&memories, Vec::new(), &table, &[1.0, 0.0], 10, 0.7);
        let ids: Vec<&str> = hits.iter().map(|h| memories[h.idx].id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["m1", "m2"],
            "the 0.6-cosine candidate is below the floor"
        );

        let hits = hybrid_hits(&memories, Vec::new(), &table, &[1.0, 0.0], 10, 0.0);
        assert_eq!(hits.len(), 3, "floor 0 keeps every positive candidate");

        let hits = hybrid_hits(&memories, Vec::new(), &table, &[1.0, 0.0], 10, 1.5);
        assert_eq!(hits.len(), 0, "nothing is close enough");

        let keyword_hit = Hit {
            idx: 2,
            score: 5,
            snippet: "s3".into(),
        };
        let hits = hybrid_hits(&memories, vec![keyword_hit], &table, &[1.0, 0.0], 10, 0.7);
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

        // First call: a (priority head) answers
        let (idx, _, value) = failover.first_available(&configs, probe).unwrap();
        assert_eq!((idx, value.as_str()), (0, "a"));

        // Head goes down: b takes over, and becomes the sticky choice
        a_down.set(true);
        let (idx, _, _) = failover.first_available(&configs, probe).unwrap();
        assert_eq!(idx, 1);

        // Next call starts at the sticky b (the still-dead a is never probed again)
        calls.borrow_mut().clear();
        let (idx, _, _) = failover.first_available(&configs, probe).unwrap();
        assert_eq!(idx, 1);
        assert!(
            !calls.borrow().iter().any(|m| m == "a"),
            "sticky skips the dead head: {:?}",
            calls.borrow()
        );

        // All candidates failing surfaces the last error
        let err = failover
            .first_available(&configs, |_| Err::<String, _>("down".into()))
            .unwrap_err();
        assert_eq!(err, "down");

        // An edited (shrunken) list: the stale sticky index is clamped, no panic
        let small = [cfg("x", None, None)];
        let (idx, _, _) = failover
            .first_available(&small, |c| Ok(c.model.clone()))
            .unwrap();
        assert_eq!(idx, 0);
    }

    /// Paraphrased recall: the keyword pass only hits m1, while the vector pass also brings in the semantically close m2,
    /// and m1 (hit by both channels) ranks ahead of m2 (vector only).
    #[test]
    fn hybrid_brings_in_vector_only_hits() {
        let memories = vec![
            mem(1, "token hashing", "sha256 of tokens"),
            mem(2, "密码保存", "哈希存储"),
        ];
        let keyword_hits = search::run(&memories, "hashing");
        assert_eq!(keyword_hits.len(), 1);

        let mut table = HashMap::new();
        table.insert(1i64, unit(4, 0)); // same direction as the query
        table.insert(2i64, unit(4, 1)); // orthogonal to the query (similarity 0, never enters)

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
        let tilted: Vec<f32> = vec![0.9, 0.1, 0.0, 0.0];
        let mut table2 = table.clone();
        table2.insert(2i64, tilted);
        let keyword_hits = search::run(&memories, "hashing");
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
        table.insert(2i64, vec![0.95, 0.31]);
        table.insert(3i64, vec![0.9, 0.44]);
        table.insert(1i64, vec![0.5, 0.87]);
        let hits = hybrid_hits(
            &[a, b, c],
            vec![k1, k2],
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
            table.insert((i + 1) as i64, vec![1.0, i as f32]);
        }
        let hits = hybrid_hits(&memories, Vec::new(), &table, &[1.0, 0.0], 3, 0.0);
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
        table.insert(4i64, vec![1.0, 0.0]);
        let hits = hybrid_hits(&memories, keyword_hits, &table, &[1.0, 0.0], 1, 0.0);
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
        st.embedding_put(1, "test-model", &fingerprint, &[1.0, 0.0])
            .unwrap();
        st.embedding_put(2, "test-model", &fingerprint, &[0.999, 0.045])
            .unwrap();
        st.embedding_put(3, "test-model", &fingerprint, &[0.0, 1.0])
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
        st2.embedding_put(1, "test-model", "unused", &[1.0])
            .unwrap();
        let mut result = serde_json::json!({"id": "m1"});
        dedup_hint(&path2, &mut result);
        assert!(result.get("similar_to").is_none());

        cleanup(&path);
        cleanup(&path2);
    }
}
