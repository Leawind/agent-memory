//! Exact token counting for embedding inputs: a configured candidate can point at the model's
//! tokenizer on ModelScope (by id or a search keyword, `auto` = the candidate's model id), and the
//! tokenizer is downloaded once, cached on disk next to the database and loaded into memory.
//!
//! With a tokenizer available, embedding texts are truncated at token level — the only cut the
//! model actually sees. Without one, callers fall back to the character cap; the same fallback
//! applies while a download is in flight or has failed (network trouble must degrade, never
//! break, the same as the embedding services themselves).
//!
//! Resolution is deliberately two-tier: batch contexts (the backfill worker, CLI backfill) may
//! hit the network, while query-time text assembly only consults the caches — a first search
//! after a cold configuration must not stall on a multi-megabyte download. Until the first
//! backfill runs (or the CLI one), the character cap stays in effect for that candidate.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokenizers::Tokenizer;

use crate::embed::EmbedConfig;

/// Download budget for one tokenizer.json (tens of MB on a domestic CDN).
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(90);
/// Budget for one ModelScope search request.
const SEARCH_TIMEOUT: Duration = Duration::from_secs(10);
/// A tokenizer.json larger than this is not one (an HTML error page or a proxy gone wild).
const MAX_BYTES: u64 = 64 * 1024 * 1024;
/// After a failed resolution the source is not retried for this long (negative cache).
const MISS_COOLDOWN: Duration = Duration::from_secs(600);
/// How long a caller waits out a concurrent resolution of the same source before giving up.
const INFLIGHT_WAIT: Duration = Duration::from_secs(150);
/// Search hits probed for a tokenizer.json, best matches first.
const SEARCH_HITS_TRIED: usize = 3;
/// Token-count slack for the prefix/text BPE boundary and post-processor variance.
const BOUNDARY_MARGIN: usize = 2;
/// Never truncate a passage below this many tokens even if the prefix eats the budget.
const MIN_ALLOWANCE: usize = 32;

static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();
/// Resolved tokenizers by source id (`Vec` + linear scan on purpose: candidate lists are a
/// handful of entries and the statics must stay const-constructible, same as `embed::Failover`).
static LOADED: Mutex<Vec<(String, Arc<Tokenizer>)>> = Mutex::new(Vec::new());
/// Sources that failed to resolve recently: skip retrying within the cooldown window.
static MISSES: Mutex<Vec<(String, Instant)>> = Mutex::new(Vec::new());
/// Sources a thread is currently downloading, so concurrent callers wait instead of racing.
static RESOLVING: Mutex<Vec<String>> = Mutex::new(Vec::new());

const MODELSCOPE_BASE: &str = "https://modelscope.cn";
const MODELSCOPE_SEARCH: &str = "https://modelscope.cn/api/v1/dolphin/models";

/// Point the on-disk cache at the database's directory (first caller wins; later calls are no-ops).
pub fn init_cache_dir(db_path: &Path) {
    let dir = db_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("tokenizers");
    let _ = CACHE_DIR.set(dir);
}

/// The candidate's configured tokenizer source: `auto` maps to the model id, anything else is
/// used verbatim as a ModelScope id / search keyword. `None` disables the feature for the entry.
fn source_for(cfg: &EmbedConfig) -> Option<String> {
    let raw = cfg.tokenizer.as_deref()?.trim();
    if raw.is_empty() {
        return None;
    }
    if raw.eq_ignore_ascii_case("auto") {
        let model = cfg.model.trim();
        return (!model.is_empty()).then(|| model.to_string());
    }
    Some(raw.to_string())
}

/// Cache-only resolution (memory → disk): what query-time paths may use. Network failures leave
/// this returning `None`, and the caller's character-cap fallback takes over.
pub fn cached(cfg: &EmbedConfig) -> Option<Arc<Tokenizer>> {
    let source = source_for(cfg)?;
    cached_by_source(&source)
}

fn cached_by_source(source: &str) -> Option<Arc<Tokenizer>> {
    if let Some((_, tok)) = LOADED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .find(|(k, _)| k == source)
    {
        return Some(Arc::clone(tok));
    }
    let path = disk_path(source)?;
    let tok = Tokenizer::from_file(&path).ok()?;
    let tok = Arc::new(tok);
    LOADED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((source.to_string(), Arc::clone(&tok)));
    Some(tok)
}

/// Full resolution for batch contexts: cache → ModelScope → disk. A concurrent resolution of the
/// same source is waited out (bounded); a failed one is negatively cached for the cooldown.
pub fn resolve_eager(cfg: &EmbedConfig) -> Option<Arc<Tokenizer>> {
    let source = source_for(cfg)?;
    if let Some(tok) = cached_by_source(&source) {
        return Some(tok);
    }
    if miss_cooled(&source) {
        return None;
    }
    if !mark_resolving(&source) {
        return wait_out_resolving(&source);
    }
    let outcome = download_and_load(&source);
    release_resolving(&source);
    match outcome {
        Ok(tok) => {
            LOADED
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push((source, Arc::clone(&tok)));
            Some(tok)
        }
        Err(e) => {
            eprintln!("tokenizer download for '{source}' failed (falling back to character truncation): {e}");
            MISSES
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push((source, Instant::now()));
            None
        }
    }
}

fn miss_cooled(source: &str) -> bool {
    MISSES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .any(|(k, at)| k == source && at.elapsed() < MISS_COOLDOWN)
}

fn mark_resolving(source: &str) -> bool {
    let mut resolving = RESOLVING.lock().unwrap_or_else(PoisonError::into_inner);
    if resolving.iter().any(|k| k == source) {
        false
    } else {
        resolving.push(source.to_string());
        true
    }
}

fn release_resolving(source: &str) {
    RESOLVING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .retain(|k| k != source);
}

/// Another thread is already fetching this source: poll until it lands, then share its result.
fn wait_out_resolving(source: &str) -> Option<Arc<Tokenizer>> {
    let deadline = Instant::now() + INFLIGHT_WAIT;
    while Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(200));
        if !RESOLVING
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .any(|k| k == source)
        {
            break;
        }
    }
    cached_by_source(source)
}

/// Exact id first (when the source looks like `org/model`), then a keyword search whose top hits
/// are probed in turn. Returns the tokenizer file bytes.
fn download_bytes(source: &str) -> Result<Vec<u8>, String> {
    if source.contains('/') {
        match fetch_file(source) {
            Ok(bytes) => return Ok(bytes),
            Err(e) if is_not_found(&e) => {} // fall through to the keyword search
            Err(e) => return Err(e),
        }
    }
    let term = source
        .rsplit_once('/')
        .map(|(_, name)| name)
        .unwrap_or(source);
    for id in search_candidates(term)?.into_iter().take(SEARCH_HITS_TRIED) {
        if let Ok(bytes) = fetch_file(&id) {
            return Ok(bytes);
        }
    }
    Err(format!(
        "no ModelScope model matching '{term}' ships a tokenizer.json"
    ))
}

/// Whether `e` is ModelScope saying the model or file does not exist (as opposed to network
/// trouble): only an absence may fall through to the keyword search.
fn is_not_found(e: &str) -> bool {
    e.contains("404")
}

fn fetch_file(id: &str) -> Result<Vec<u8>, String> {
    let url = format!("{MODELSCOPE_BASE}/models/{id}/resolve/master/tokenizer.json");
    let resp = ureq::get(&url)
        .timeout(DOWNLOAD_TIMEOUT)
        .call()
        .map_err(|e| format!("tokenizer fetch failed: {e}"))?;
    let status = resp.status();
    let mut bytes = Vec::new();
    resp.into_reader()
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("tokenizer read failed: {e}"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(format!(
            "tokenizer.json of '{id}' exceeds the {MAX_BYTES}-byte cap"
        ));
    }
    // Content sanity: an object with a model section. Mirrors and proxies must not feed HTML
    // error pages into the parser on a 200.
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("tokenizer.json of '{id}' is not valid JSON: {e}"))?;
    if !value.is_object() || !value.get("model").is_some() {
        return Err(format!("tokenizer.json of '{id}' is not a tokenizer file"));
    }
    let _ = status;
    Ok(bytes)
}

/// ModelScope's model search: `PUT /api/v1/dolphin/models` with a JSON body (a GET query string
/// is not accepted). Returns `org/model` ids, skipping obvious binary conversions whose
/// tokenizers describe a different distribution than the source model's.
fn search_candidates(term: &str) -> Result<Vec<String>, String> {
    let resp = ureq::put(MODELSCOPE_SEARCH)
        .timeout(SEARCH_TIMEOUT)
        .send_json(json!({
            "Name": term,
            "PageSize": 8,
            "PageNumber": 1,
            "SortBy": "Default",
        }))
        .map_err(|e| format!("ModelScope search failed: {e}"))?;
    let body: Value = resp
        .into_json()
        .map_err(|e| format!("ModelScope search returned invalid JSON: {e}"))?;
    Ok(parse_search_models(&body))
}

fn parse_search_models(body: &Value) -> Vec<String> {
    body["Data"]["Model"]["Models"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|m| {
            let org = m["Path"].as_str()?.trim();
            let name = m["Name"].as_str()?.trim();
            if org.is_empty() || name.is_empty() {
                return None;
            }
            let upper = name.to_ascii_uppercase();
            if upper.contains("GGUF") || upper.contains("-MNN") || upper.contains("ONNX") {
                return None;
            }
            Some(format!("{org}/{name}"))
        })
        .collect()
}

fn download_and_load(source: &str) -> Result<Arc<Tokenizer>, String> {
    let bytes = download_bytes(source)?;
    let tok = Tokenizer::from_bytes(&bytes)
        .map_err(|e| format!("tokenizer.json failed to parse: {e}"))?;
    persist(source, &bytes);
    Ok(Arc::new(tok))
}

/// Best-effort disk persistence under `<db dir>/tokenizers/<sanitized id>/`; a full disk or an
/// unwritable directory only costs the download on the next process, never the current result.
fn persist(source: &str, bytes: &[u8]) {
    let Some(dir) = CACHE_DIR.get() else {
        return;
    };
    let target = disk_path(source);
    let Some(target) = target else {
        return;
    };
    if std::fs::create_dir_all(target.parent().unwrap_or(dir)).is_err() {
        return;
    }
    let tmp = target.with_extension("json.tmp");
    if std::fs::write(&tmp, bytes).is_ok() {
        let _ = std::fs::rename(&tmp, &target);
    }
    let meta = target.parent().unwrap_or(dir).join("meta.json");
    let _ = std::fs::write(
        &meta,
        json!({
            "source": source,
            "downloaded_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        })
        .to_string(),
    );
}

fn disk_path(source: &str) -> Option<PathBuf> {
    CACHE_DIR
        .get()
        .map(|dir| dir.join(sanitize(source)).join("tokenizer.json"))
}

/// Filesystem-safe directory name for a `org/model` source id.
fn sanitize(source: &str) -> String {
    source
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Cut `text` so that `prefix + result` fits `limit` tokens under the model's tokenizer: the
/// exact combined count decides whether anything must go, the prefix's own count (plus special
/// tokens and a boundary margin) sets the allowance for the passage. Returns an empty string on
/// tokenizer failure — the caller's character-cap fallback decides then.
pub fn truncate_to_tokens(tok: &Tokenizer, prefix: &str, text: &str, limit: usize) -> String {
    let Ok(full) = tok.encode(format!("{prefix}{text}"), true) else {
        return String::new();
    };
    if full.get_ids().len() <= limit {
        return text.to_string();
    }
    let prefix_tokens = tok
        .encode(prefix, true)
        .map(|e| e.get_ids().len())
        .unwrap_or(0);
    // Floor the allowance at MIN_ALLOWANCE so a token-hungry prefix cannot squeeze the passage
    // down to nothing — but never above the limit itself: exceeding the model window is exactly
    // what this cut exists to prevent.
    let allowance = limit
        .saturating_sub(prefix_tokens + BOUNDARY_MARGIN)
        .max(MIN_ALLOWANCE.min(limit.max(1)));
    let Ok(enc) = tok.encode(text, false) else {
        return String::new();
    };
    let ids = enc.get_ids();
    if ids.len() <= allowance {
        return text.to_string();
    }
    tok.decode(&ids[..allowance], true).unwrap_or_default()
}

/// A tiny WordPiece tokenizer shared by the chunking and embedding tests: whitespace
/// pre-tokenization, every fixture word one whole vocabulary entry, and a decoder that joins words
/// with spaces — so token counts and decode → re-encode round-trips are exactly predictable.
#[cfg(test)]
pub fn test_tokenizer() -> Tokenizer {
    use serde_json::{json, Value};
    let vocab: Value = json!({
        "[UNK]": 0, "he": 1, "llo": 2, "hello": 3, "world": 4, "helloworld": 5,
        "alpha": 6, "beta": 7, "gamma": 8, "delta": 9, "deploy": 10, "rollback": 11,
    });
    let config = json!({
        "version": "1.0",
        "truncation": null,
        "padding": null,
        "added_tokens": [],
        "normalizer": null,
        "pre_tokenizer": { "type": "Whitespace" },
        "post_processor": null,
        "decoder": { "type": "WordPiece", "prefix": "##", "cleanup": true },
        "model": {
            "type": "WordPiece",
            "unk_token": "[UNK]",
            "continuing_subword_prefix": "##",
            "max_input_chars_per_word": 100,
            "vocab": vocab,
        },
    });
    Tokenizer::from_bytes(config.to_string().as_bytes()).expect("tiny tokenizer builds")
}

/// Register a tokenizer as already resolved, so the embedding paths can be tested without a
/// download. Test-only: production entries only ever learn a tokenizer through `resolve_eager`.
#[cfg(test)]
pub fn inject_for_test(source: &str, tok: Tokenizer) {
    LOADED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push((source.to_string(), Arc::new(tok)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The tokenizer fixture shared with `tokenizer` tests: whitespace pre-tokenization with whole
    /// words in the vocabulary, so one word = one token and counts are easy to reason about.
    fn tiny_tokenizer() -> Tokenizer {
        test_tokenizer()
    }

    #[test]
    fn sanitize_keeps_path_safe() {
        assert_eq!(sanitize("BAAI/bge-m3"), "BAAI_bge-m3");
        assert_eq!(sanitize("a b/c..d"), "a_b_c..d");
    }

    #[test]
    fn parse_search_models_reads_org_name_pairs_and_skips_conversions() {
        let body = json!({
            "Code": 200,
            "Data": { "Model": { "Models": [
                { "Path": "iic", "Name": "gte_sentence-embedding_multilingual-base" },
                { "Path": "gpustack", "Name": "bge-m3-GGUF" },
                { "Path": "x", "Name": "bge-m3-MNN" },
                { "Path": "", "Name": "orphan" },
                { "Path": "org", "Name": "" },
            ]}},
        });
        assert_eq!(
            parse_search_models(&body),
            vec!["iic/gte_sentence-embedding_multilingual-base".to_string()]
        );
    }

    #[test]
    fn fit_text_keeps_short_input_and_cuts_long_input() {
        let tok = tiny_tokenizer();
        // "hello world" = 2 tokens: well under the limit, passes through verbatim
        assert_eq!(
            truncate_to_tokens(&tok, "", "hello world", 4),
            "hello world"
        );
        // "helloworld" is a whole vocab word (1 token); 50 copies = 50 tokens, cut to 32
        let long = "helloworld ".repeat(50);
        let cut = truncate_to_tokens(&tok, "", long.trim(), 32);
        assert!(!cut.is_empty());
        assert!(cut.chars().count() < long.trim().chars().count());
        let recount = tok.encode(cut.as_str(), true).unwrap().get_ids().len();
        assert!(recount <= 32, "truncated passage must fit the allowance");
    }

    /// A pathological limit below MIN_ALLOWANCE must still truncate to within the limit, never
    /// emit an over-window text.
    #[test]
    fn fit_text_clamps_floor_to_limit() {
        let tok = tiny_tokenizer();
        let long = "helloworld ".repeat(50);
        let cut = truncate_to_tokens(&tok, "", long.trim(), 10);
        let recount = tok.encode(cut.as_str(), true).unwrap().get_ids().len();
        assert!(recount <= 10, "floor must not push the cut past the limit");
    }

    #[test]
    fn fit_text_counts_prefix_tokens() {
        let tok = tiny_tokenizer();
        let long = "helloworld ".repeat(50);
        // "he llo" = 2 tokens of prefix budget; the passage gets 64-2-2=60 instead of 62
        let with_prefix = truncate_to_tokens(&tok, "he llo ", long.trim(), 64);
        let bare = truncate_to_tokens(&tok, "", long.trim(), 64);
        // The prefix's tokens are deducted before the passage is cut
        assert!(
            tok.encode(with_prefix.as_str(), true)
                .unwrap()
                .get_ids()
                .len()
                <= tok.encode(bare.as_str(), true).unwrap().get_ids().len()
        );
    }

    /// Live ModelScope round-trip: exact id, then search fallback. Ignored by default — run with
    /// `cargo test tokenizer -- --ignored` when touching the fetch path.
    #[test]
    #[ignore]
    fn modelscope_fetch_end_to_end() {
        let bytes = download_bytes("BAAI/bge-m3").expect("exact id fetches bge-m3's tokenizer");
        Tokenizer::from_bytes(&bytes).expect("downloaded file parses as a tokenizer");
        let searched = download_bytes("gte-multilingual-base").expect("keyword search finds gte");
        Tokenizer::from_bytes(&searched).expect("searched file parses as a tokenizer");
    }

    #[test]
    fn source_rules() {
        let mut cfg = crate::embed::EmbedConfig {
            id: "m3".into(),
            base_url: "https://api.siliconflow.cn/v1".into(),
            model: "BAAI/bge-m3".into(),
            api_key: None,
            query_prefix: None,
            passage_prefix: None,
            min_similarity: 0.0,
            tokenizer: None,
            max_tokens: None,
        };
        assert_eq!(source_for(&cfg), None);
        cfg.tokenizer = Some("auto".into());
        assert_eq!(source_for(&cfg), Some("BAAI/bge-m3".into()));
        cfg.tokenizer = Some(" AUTO ".into());
        assert_eq!(source_for(&cfg), Some("BAAI/bge-m3".into()));
        cfg.tokenizer = Some("gte-multilingual-base".into());
        assert_eq!(
            source_for(&cfg),
            Some("gte-multilingual-base".into()),
            "a keyword without / is searched as-is"
        );
    }
}
