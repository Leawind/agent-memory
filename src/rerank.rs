//! Reranker client: cross-encoder re-scoring of search candidates.
//!
//! A reranker is a second-stage relevance model: it reads (query, document) pairs through a
//! cross-encoder and returns calibrated relevance scores, re-ordering the fused candidate pool
//! that recall (keywords + vectors) produced. This module only speaks HTTP to an OpenAI-ish
//! `/rerank` endpoint (Cohere-style request, tolerant response parsing for the shapes returned
//! by common local servers); ordering policy lives in the search handler.
//!
//! Failure is always a silent degradation: an unavailable reranker leaves the fused order in
//! place — better ranking is an improvement, never a dependency. Like every other outbound call,
//! reranking never runs inside a write transaction (the search path holds a read snapshot).

use serde_json::{json, Value};
use std::time::Duration;

/// Reranker service configuration (see `Failover` reuse in the search handler).
#[derive(Clone, Debug)]
pub struct RerankConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl RerankConfig {
    fn endpoint(&self) -> String {
        format!("{}/rerank", self.base_url.trim_end_matches('/'))
    }
}

/// One configured reranker candidate (an element of the ordered `rerank_models` settings list),
/// mirroring `embed::EmbedEntry`: raw settings fields for the admin round-trip, `usable` as the
/// operational view.
#[derive(Clone, Debug)]
pub struct RerankEntry {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
}

impl RerankEntry {
    /// Lenient parse of one settings-array element (same discipline as embeddings entries).
    pub fn from_json(v: &Value) -> Option<RerankEntry> {
        let obj = v.as_object()?;
        let string = |key: &str| {
            obj.get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
                .filter(|s| !s.is_empty())
        };
        Some(RerankEntry {
            id: string("id").unwrap_or_default().trim().to_string(),
            name: string("name").unwrap_or_default().trim().to_string(),
            enabled: obj.get("enabled").and_then(Value::as_bool).unwrap_or(true),
            base_url: string("base_url").unwrap_or_default(),
            model: string("model").unwrap_or_default(),
            api_key: string("api_key"),
        })
    }

    /// Canonical JSON for the settings round-trip (GET → UI → PUT).
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "enabled": self.enabled,
            "base_url": self.base_url,
            "model": self.model,
            "api_key": self.api_key,
        })
    }

    /// The operational view: enabled and fully specified.
    pub fn usable(&self) -> Option<RerankConfig> {
        if !self.enabled || self.id.is_empty() || self.name.is_empty() {
            return None;
        }
        self.config()
    }

    /// The same view without the enabled gate (see `embed::EmbedEntry::config`): the admin probe
    /// verifies what is on screen, before it is enabled or saved.
    pub fn config(&self) -> Option<RerankConfig> {
        if self.base_url.trim().is_empty() || self.model.trim().is_empty() {
            return None;
        }
        Some(RerankConfig {
            base_url: self.base_url.trim().to_string(),
            model: self.model.trim().to_string(),
            api_key: self.api_key.clone(),
        })
    }
}

/// Selection state for the reranker candidate list (shared by search / connection test).
pub static RERANK_FAILOVER: crate::embed::Failover = crate::embed::Failover::new();

/// Timeout for one rerank call on the search path: a local cross-encoder over a few dozen short
/// documents is fast on GPU and takes a few seconds on CPU — past this the fused order is kept.
pub const RERANK_TIMEOUT: Duration = Duration::from_secs(15);

/// How many of the fused candidates enter the rerank stage (the funnel: recall wide, rerank
/// narrow). The tail keeps its fused order behind the reranked head.
pub const CANDIDATE_POOL: usize = 50;

/// Rerank `documents` against `query`, returning `(document index, relevance score)` pairs in
/// descending score order. Any network/parsing failure is an Err; the caller degrades to the
/// incoming order.
pub fn rerank(
    cfg: &RerankConfig,
    query: &str,
    documents: &[String],
    timeout: Duration,
) -> Result<Vec<(usize, f64)>, String> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }
    let mut req = ureq::post(&cfg.endpoint()).timeout(timeout);
    if let Some(key) = &cfg.api_key {
        req = req.set("Authorization", &format!("Bearer {key}"));
    }
    let resp = req
        .send_json(json!({
            "model": cfg.model,
            "query": query,
            "documents": documents,
            "top_n": documents.len(),
        }))
        .map_err(|e| format!("rerank request failed: {e}"))?;
    let status = resp.status();
    let body: Value = resp
        .into_json()
        .map_err(|e| format!("rerank response is not valid JSON: {e}"))?;
    if status != 200 {
        let detail = body["error"]["message"]
            .as_str()
            .or_else(|| body["message"].as_str())
            .unwrap_or("unknown error");
        return Err(format!("rerank service returned {status}: {detail}"));
    }
    parse_response(&body, documents.len())
}

/// Parse a rerank response: `{"results": [{"index": N, "relevance_score": F}, ...]}` is the
/// common shape (Cohere / Jina / Xinference / SiliconFlow); bare arrays and `score` field names
/// (TEI and friends) are accepted too. Strict about coverage: every document must be scored
/// exactly once — a partial answer cannot be told apart from a broken one, and silently keeping
/// unscored candidates in place would weld two incompatible scales into one ranking.
pub fn parse_response(body: &Value, docs: usize) -> Result<Vec<(usize, f64)>, String> {
    let results = match body {
        Value::Array(_) => body,
        _ => body
            .get("results")
            .ok_or("rerank response missing 'results' array")?,
    };
    let arr = results
        .as_array()
        .ok_or("rerank response 'results' is not an array")?;
    if arr.len() != docs {
        return Err(format!(
            "rerank service scored {} of {} documents",
            arr.len(),
            docs
        ));
    }
    let mut scored = Vec::with_capacity(docs);
    for item in arr {
        let index = item
            .get("index")
            .and_then(Value::as_u64)
            .ok_or("rerank result entry missing 'index'")? as usize;
        if index >= docs {
            return Err(format!(
                "rerank result index {index} out of range ({docs} documents)"
            ));
        }
        let score = item
            .get("relevance_score")
            .or_else(|| item.get("score"))
            .and_then(Value::as_f64)
            .ok_or("rerank result entry missing 'relevance_score'/'score'")?;
        scored.push((index, score));
    }
    if scored
        .iter()
        .map(|(i, _)| *i)
        .collect::<std::collections::HashSet<_>>()
        .len()
        != docs
    {
        return Err("rerank response contains duplicate document indices".to_string());
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    Ok(scored)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Response parsing accepts the common shapes (wrapped results, bare array, `score` naming),
    /// sorts by score descending, and rejects partial/ambiguous answers outright.
    #[test]
    fn parse_response_shapes_and_strictness() {
        let cohere = json!({"results": [
            {"index": 0, "relevance_score": 0.3},
            {"index": 2, "relevance_score": 0.9},
            {"index": 1, "relevance_score": 0.5},
        ]});
        let parsed = parse_response(&cohere, 3).unwrap();
        assert_eq!(
            parsed,
            vec![(2, 0.9), (1, 0.5), (0, 0.3)],
            "sorted descending"
        );

        // TEI-style: bare array, `score` field
        let tei = json!([{"index": 1, "score": 0.7}, {"index": 0, "score": 0.1}]);
        assert_eq!(parse_response(&tei, 2).unwrap(), vec![(1, 0.7), (0, 0.1)]);

        // Missing results / wrong count / out-of-range / duplicate indices / missing score
        assert!(parse_response(&json!({}), 2).is_err());
        assert!(parse_response(
            &json!({"results": [{"index": 0, "relevance_score": 1.0}]}),
            2
        )
        .is_err());
        assert!(parse_response(
            &json!({"results": [{"index": 5, "relevance_score": 1.0}]}),
            2
        )
        .is_err());
        assert!(parse_response(
            &json!({"results": [{"index": 0, "relevance_score": 1.0}, {"index": 0, "relevance_score": 0.5}]}),
            2
        )
        .is_err());
        assert!(parse_response(&json!({"results": [{"index": 0}]}), 1).is_err());
    }
}
