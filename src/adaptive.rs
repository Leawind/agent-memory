//! Process-local, bounded rerank latency control. Quality requires external evaluation.

use crate::{rerank::RerankConfig, search::Limits};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::Path,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const MAX_MODELS: usize = 64;
const IDLE_TTL: Duration = Duration::from_secs(600);
static STATES: OnceLock<Mutex<HashMap<String, State>>> = OnceLock::new();

struct State {
    current: usize,
    samples: u64,
    failures: u64,
    ms_per_doc: Option<f64>,
    chars_per_doc: Option<f64>,
    last_candidates: usize,
    last_ms: f64,
    updated_at: u64,
    touched: Instant,
}

impl State {
    fn new(limits: &Limits) -> Self {
        Self {
            current: limits.rerank_candidates,
            samples: 0,
            failures: 0,
            ms_per_doc: None,
            chars_per_doc: None,
            last_candidates: 0,
            last_ms: 0.0,
            updated_at: 0,
            touched: Instant::now(),
        }
    }

    fn observe(&mut self, limits: &Limits, docs: usize, chars: usize, elapsed_ms: f64, ok: bool) {
        if docs == 0 {
            return;
        }
        self.samples += 1;
        self.last_candidates = docs;
        self.last_ms = elapsed_ms;
        self.updated_at = crate::model::now();
        self.touched = Instant::now();
        let a = &limits.adaptive;
        if !ok {
            self.failures += 1;
            self.current = (docs * 7 / 10).clamp(a.min_candidates, a.max_candidates);
            return;
        }
        let ewma = |old: Option<f64>, measured: f64| {
            old.map_or(measured, |previous| 0.8 * previous + 0.2 * measured)
        };
        self.ms_per_doc = Some(ewma(self.ms_per_doc, (elapsed_ms / docs as f64).max(0.01)));
        self.chars_per_doc = Some(ewma(self.chars_per_doc, chars as f64 / docs as f64));
        let desired = (a.target_latency_ms as f64 / self.ms_per_doc.unwrap()).floor() as usize;
        let desired = desired.clamp(a.min_candidates, a.max_candidates);
        if elapsed_ms > a.target_latency_ms as f64 * 1.1 {
            // Current slow evidence wins over an EWMA still dominated by older fast calls.
            let immediate =
                (docs as f64 * a.target_latency_ms as f64 / elapsed_ms).floor() as usize;
            self.current = desired
                .min(immediate)
                .clamp(a.min_candidates, a.max_candidates);
        } else if elapsed_ms < a.target_latency_ms as f64 * 0.8 {
            let growth = if docs < self.current {
                self.current
            } else {
                docs + docs.div_ceil(10)
            };
            self.current = desired
                .min(growth)
                .clamp(a.min_candidates, a.max_candidates);
        } else {
            self.current = docs.clamp(a.min_candidates, a.max_candidates);
        }
    }

    fn budget(&self, limits: &Limits, document_chars: &[usize]) -> usize {
        let a = &limits.adaptive;
        if !a.enabled {
            return limits.rerank_candidates.min(document_chars.len());
        }
        let mut count = self
            .current
            .clamp(a.min_candidates, a.max_candidates)
            .min(document_chars.len());
        let mut chars: usize = document_chars.iter().take(count).sum();
        while count > a.min_candidates && chars > a.max_input_chars {
            count -= 1;
            chars -= document_chars[count];
        }
        if let (Some(ms), Some(previous_chars)) = (self.ms_per_doc, self.chars_per_doc) {
            // Length is a cost proxy, never a relevance signal. Stay within the minimum.
            let estimated = |n: usize, size: usize| {
                ms * n as f64
                    * (size as f64 / n.max(1) as f64 / previous_chars.max(1.0)).clamp(0.5, 2.0)
            };
            while count > a.min_candidates
                && estimated(count, chars) > a.target_latency_ms as f64 * 1.1
            {
                count -= 1;
                chars -= document_chars[count];
            }
        }
        count
    }
}

fn key(path: &Path, cfg: &RerankConfig, limits: &Limits) -> String {
    let path = std::fs::canonicalize(path).unwrap_or_else(|_| crate::store::normalize_path(path));
    crate::util::sha256_hex(
        json!([
            path.to_string_lossy(),
            cfg.id,
            cfg.base_url.trim_end_matches('/'),
            cfg.model,
            cfg.api_key,
            limits.to_json()
        ])
        .to_string()
        .as_bytes(),
    )
}

fn states() -> &'static Mutex<HashMap<String, State>> {
    STATES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn maintain(states: &mut HashMap<String, State>, incoming: &str) {
    states.retain(|_, state| state.touched.elapsed() < IDLE_TTL);
    if !states.contains_key(incoming) && states.len() >= MAX_MODELS {
        if let Some(oldest) = states
            .iter()
            .min_by_key(|(_, state)| state.touched)
            .map(|(key, _)| key.clone())
        {
            states.remove(&oldest);
        }
    }
}

pub fn budget(path: &Path, cfg: &RerankConfig, limits: &Limits, document_chars: &[usize]) -> usize {
    let key = key(path, cfg, limits);
    let mut states = states()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    maintain(&mut states, &key);
    let state = states.entry(key).or_insert_with(|| State::new(limits));
    state.touched = Instant::now();
    state.budget(limits, document_chars)
}

pub fn observe(
    path: &Path,
    cfg: &RerankConfig,
    limits: &Limits,
    docs: usize,
    chars: usize,
    elapsed: Duration,
    ok: bool,
) {
    let key = key(path, cfg, limits);
    let mut states = states()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    maintain(&mut states, &key);
    states
        .entry(key)
        .or_insert_with(|| State::new(limits))
        .observe(limits, docs, chars, elapsed.as_secs_f64() * 1000.0, ok);
}

pub fn view(path: &Path, configs: &[RerankConfig], limits: &Limits) -> Value {
    let states = states()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let models: Vec<_> = configs.iter().map(|cfg| {
        let default = State::new(limits);
        let state = states.get(&key(path, cfg, limits)).filter(|s| s.touched.elapsed() < IDLE_TTL).unwrap_or(&default);
        json!({"id":cfg.id,"model":cfg.model,"samples":state.samples,"failures":state.failures,
            "selected_candidates":if limits.adaptive.enabled {state.current} else {limits.rerank_candidates},
            "last_candidates":state.last_candidates,"last_latency_ms":state.last_ms,
            "ewma_ms_per_document":state.ms_per_doc,"ewma_chars_per_document":state.chars_per_doc,
            "updated_at":state.updated_at})
    }).collect();
    json!({"process_local":true,"adaptive_enabled":limits.adaptive.enabled,"models":models})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> Limits {
        Limits::parse(&json!({"semantic_candidates":100,"rerank_candidates":50,
            "adaptive":{"enabled":true,"min_candidates":10,"max_candidates":100,"target_latency_ms":1000}})).unwrap()
    }

    #[test]
    fn full_cache_preserves_existing_feedback_and_evicts_only_for_a_new_model() {
        let limits = limits();
        let mut states = HashMap::new();
        for index in 0..MAX_MODELS {
            let mut state = State::new(&limits);
            state.samples = index as u64 + 1;
            states.insert(index.to_string(), state);
        }
        maintain(&mut states, "0");
        assert_eq!(states.len(), MAX_MODELS);
        assert_eq!(states["0"].samples, 1);
        maintain(&mut states, "new");
        assert_eq!(states.len(), MAX_MODELS - 1);
        states.insert("new".into(), State::new(&limits));
        assert_eq!(states.len(), MAX_MODELS);
    }

    #[test]
    fn controller_shrinks_fast_grows_slowly_and_stays_bounded() {
        let limits = limits();
        let mut state = State::new(&limits);
        let chars = vec![100; 200];
        assert_eq!(state.budget(&limits, &chars), 50);
        state.observe(&limits, 50, 5000, 5000.0, true);
        assert_eq!(state.current, 10);
        assert_eq!(state.budget(&limits, &chars), 10);
        state.observe(&limits, 10, 1000, 1.0, true);
        assert!(state.current <= 11);
        for _ in 0..100 {
            state.observe(&limits, state.current, state.current * 100, 1.0, true);
        }
        assert_eq!(state.current, 100);
        state.observe(&limits, 100, 10000, 10.0, false);
        assert_eq!(state.current, 70);
        assert_eq!(state.failures, 1);
        assert_eq!(state.budget(&limits, &[100, 100, 100]), 3);
    }

    #[test]
    fn length_budget_and_disabled_mode_have_explicit_bounds() {
        let mut limits = limits();
        limits.adaptive.max_input_chars = 80_000;
        let state = State::new(&limits);
        assert_eq!(state.budget(&limits, &vec![8000; 100]), 10);
        limits.adaptive.enabled = false;
        assert_eq!(state.budget(&limits, &vec![8000; 100]), 50);
    }

    #[test]
    fn sparse_fast_queries_do_not_shrink_the_next_large_query() {
        let limits = limits();
        let mut state = State::new(&limits);
        state.observe(&limits, 3, 300, 1.0, true);
        assert_eq!(state.current, 50);
    }

    #[test]
    fn telemetry_is_scoped_to_database_model_credentials_and_settings() {
        let limits = limits();
        let path = Path::new("adaptive-scope-test.db");
        let cfg = RerankConfig {
            id: "model-1".into(),
            base_url: "http://local/v1".into(),
            model: "cross-encoder".into(),
            api_key: Some("secret-token".into()),
        };
        observe(path, &cfg, &limits, 50, 5000, Duration::from_secs(5), true);
        let first = view(path, std::slice::from_ref(&cfg), &limits);
        assert_eq!(first["models"][0]["selected_candidates"], 10);
        assert!(!first.to_string().contains("secret-token"));
        assert_eq!(
            view(Path::new("other.db"), std::slice::from_ref(&cfg), &limits)["models"][0]
                ["samples"],
            0
        );
        let mut changed = cfg.clone();
        changed.api_key = Some("another-token".into());
        assert_eq!(view(path, &[changed], &limits)["models"][0]["samples"], 0);
        let mut changed_limits = limits;
        changed_limits.adaptive.target_latency_ms = 500;
        assert_eq!(
            view(path, &[cfg], &changed_limits)["models"][0]["samples"],
            0
        );
    }
}
