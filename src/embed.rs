//! Semantic search: embedding service calls, vector encoding/decoding and hybrid ranking.
//!
//! Fallback is this module's hard rule — an unavailable embedding service may only cause "degradation", never
//! "failure": the search path falls back to pure keywords on timeout/error; the write path just leaves vectors pending backfill,
//! since the memory itself is already saved; with no configuration, every entry point is a zero-cost pass-through.
//!
//! Network calls always happen outside database transactions (guaranteed by callers): holding an IMMEDIATE lock across
//! network I/O in a write transaction is a forbidden deadlock shape, and stretching a read-only snapshot serves no purpose either.

use crate::search::{self, Hit};
use crate::store::{Store, TxMode};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

/// Embedding service configuration (an OpenAI-compatible `/embeddings` endpoint: cloud APIs and
/// local services such as Ollama/LM Studio share the same shape).
#[derive(Clone, Debug)]
pub struct EmbedConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl EmbedConfig {
    fn endpoint(&self) -> String {
        format!("{}/embeddings", self.base_url.trim_end_matches('/'))
    }
}

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

/// Backfill one batch for memories missing vectors (or whose vector model is stale): read config and the pending list (read-only transaction)
/// → call the embedding service (outside transactions) → write vectors (a separate write transaction).
pub fn process_pending(db_path: &Path, batch: usize) -> EmbedOutcome {
    let batch = batch.clamp(1, MAX_BATCH);
    let (cfg, pending) = match crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
        let Some(cfg) = st.embedding_config()? else {
            return Ok(None);
        };
        let pending = st.embedding_pending_batch(&cfg.model, batch)?;
        Ok::<_, String>(Some((cfg, pending)))
    }) {
        Ok(Some(pair)) => pair,
        Ok(None) => return EmbedOutcome::NotConfigured,
        Err(e) => return EmbedOutcome::Failed(format!("cannot read database: {e}")),
    };
    if pending.is_empty() {
        return EmbedOutcome::Processed {
            processed: 0,
            remaining: 0,
        };
    }

    let texts: Vec<String> = pending
        .iter()
        .map(|(_, summary, content)| embed_memory_text(summary, content))
        .collect();
    let vectors = match embed_texts(&cfg, &texts, BATCH_TIMEOUT) {
        Ok(v) => v,
        Err(e) => return EmbedOutcome::Failed(e),
    };

    if let Err(e) = crate::store::with_db_in(db_path, TxMode::Write, |st| {
        for ((id, _, _), vec) in pending.iter().zip(&vectors) {
            st.embedding_put(*id, &cfg.model, vec)?;
        }
        Ok::<(), String>(())
    }) {
        return EmbedOutcome::Failed(format!("cannot store embeddings: {e}"));
    }

    match crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
        st.embedding_pending_count(&cfg.model)
    }) {
        Ok(remaining) => EmbedOutcome::Processed {
            processed: pending.len(),
            remaining,
        },
        Err(e) => EmbedOutcome::Failed(format!("cannot count pending: {e}")),
    }
}

/// The write path's hook point (called after the memory_create / memory_update transaction commits).
/// When the embedding service is unavailable, only log to stderr: the memory is saved, and vectors are left for backfill.
pub fn after_write(db_path: &Path) {
    if let EmbedOutcome::Failed(e) = process_pending(db_path, 4) {
        eprintln!(
            "embedding backfill after write failed (the memory itself is saved; it will be retried by the next backfill): {e}"
        );
    }
}

/// RRF fusion ranking over keyword + vector recall.
///
/// Each channel contributes `1/(K+rank)` by rank, which naturally sidesteps the incomparable units of "keyword TF scores"
/// versus "cosine values". Vector-only hits (zero keyword hits but semantically close) are introduced by this function
/// — exactly the point of semantic search; their snippets take the fallback path from the start of the content.
pub fn hybrid_hits(
    memories: &[crate::model::Memory],
    keyword_hits: Vec<Hit>,
    table: &HashMap<i64, Vec<f32>>,
    query_vec: &[f32],
    tag_filter: &[String],
    tag_regex: Option<&Regex>,
) -> Vec<Hit> {
    // Vector pass: through the same tag filtering (matching the keyword pass's recall semantics), ranked by cosine descending
    let mut vector_ranked: Vec<(usize, f32)> = memories
        .iter()
        .enumerate()
        .filter(|(_, m)| search::passes_tag_filters(m, tag_filter, tag_regex))
        .filter_map(|(idx, m)| {
            let id = Store::parse_id(&m.id).unwrap_or(0);
            table.get(&id).map(|v| (idx, cosine(query_vec, v)))
        })
        .filter(|(_, score)| *score > 0.0)
        .collect();
    vector_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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

    /// Paraphrased recall: the keyword pass only hits m1, while the vector pass also brings in the semantically close m2,
    /// and m1 (hit by both channels) ranks ahead of m2 (vector only).
    #[test]
    fn hybrid_brings_in_vector_only_hits() {
        let memories = vec![
            mem(1, "token hashing", "sha256 of tokens"),
            mem(2, "密码保存", "哈希存储"),
        ];
        let keyword_hits = search::run(&memories, "hashing", &[], None);
        assert_eq!(keyword_hits.len(), 1);

        let mut table = HashMap::new();
        table.insert(1i64, unit(4, 0)); // same direction as the query
        table.insert(2i64, unit(4, 1)); // orthogonal to the query (similarity 0, never enters)

        let hits = hybrid_hits(&memories, keyword_hits, &table, &unit(4, 0), &[], None);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);

        // m2 enters the results when its similarity to the query is 0.87 (non-orthogonal)
        let tilted: Vec<f32> = vec![0.9, 0.1, 0.0, 0.0];
        let mut table2 = table.clone();
        table2.insert(2i64, tilted);
        let keyword_hits = search::run(&memories, "hashing", &[], None);
        let hits = hybrid_hits(&memories, keyword_hits, &table2, &unit(4, 0), &[], None);
        assert_eq!(hits.len(), 2);
        assert_eq!(
            hits[0].idx, 0,
            "a double hit must rank ahead of a vector-only hit"
        );
        assert_eq!(hits[1].idx, 1);
        // Vector-only hits take the fallback snippet (start of content), plain text
        assert!(hits[1].snippet.contains("哈希存储"));
    }

    /// The vector pass honors the same tag filtering as the keyword pass.
    #[test]
    fn hybrid_respects_tag_filters() {
        let mut a = mem(1, "alpha", "a");
        a.tags = vec!["keep".to_string()];
        let mut b = mem(2, "beta", "b");
        b.tags = vec!["other".to_string()];
        let mut table = HashMap::new();
        table.insert(1i64, unit(2, 0));
        table.insert(2i64, unit(2, 0));
        let hits = hybrid_hits(
            &[a, b],
            Vec::new(),
            &table,
            &unit(2, 0),
            &["keep".to_string()],
            None,
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 0);
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
        let hits = hybrid_hits(&[a, b, c], vec![k1, k2], &table, &[1.0, 0.0], &[], None);
        let order: Vec<usize> = hits.iter().map(|h| h.idx).collect();
        assert_eq!(
            order,
            vec![1, 0, 2],
            "b, ranked on both channels, should beat a, the keyword-only leader"
        );
    }
}
