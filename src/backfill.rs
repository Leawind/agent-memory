//! Background backfill: per-model drains that every trigger can start, and that any client can
//! watch.
//!
//! A drain is a run: it belongs to one embedding candidate, runs in its own thread, and is
//! registered while it works. That registration is what makes the three triggers safe to combine —
//! service start, the periodic sweep and a settings save all ask for a run, and a run that already
//! exists is reported instead of duplicated. The registry is also the status surface: the admin
//! page lists active runs, so progress is visible whether the operator started it or the sweep did.
//!
//! The sweep asks the same three questions every tick: which candidate is current (the one the
//! failover would pick — priority order, starting from the last one that answered), may it run
//! unattended (`auto_backfill`), does it really have pending memories (one cheap count — no
//! backlog means no traffic), and only then start the run. A memory write takes the same path but
//! ignores the opt-in switch: saving a memory always embeds it; the switch governs unattended
//! sweeps, not the consequence of a write.
//!
//! Failures keep the search path's semantics: the candidate enters the shared failure cooldown and
//! the run ends, so a dead service costs one timeout per cooldown window rather than one per tick.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, PoisonError};
use std::time::Duration;

use crate::embed::{self, EmbedConfig, EmbedOutcome};
use crate::store::TxMode;

/// How often the worker sweeps on its own.
pub const TICK: Duration = Duration::from_secs(600);
/// Memories per embedding round inside one run.
const BATCH: usize = embed::MAX_BATCH;
/// Upper bound on rounds per run, so one run cannot monopolize its worker when writes keep
/// arriving; whatever remains is picked up by the next run.
const MAX_ROUNDS: usize = 256;

/// Who asked for a run. Reported to the admin page so the operator can tell an automatic sweep
/// from their own click.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Auto,
    Manual,
    Write,
}

impl Trigger {
    pub fn as_str(self) -> &'static str {
        match self {
            Trigger::Auto => "auto",
            Trigger::Manual => "manual",
            Trigger::Write => "write",
        }
    }
}

/// A run's observable progress. `total` grows with newly pending memories, so it always equals
/// `done + remaining`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progress {
    pub model_key: String,
    pub trigger: Trigger,
    pub started_at: u64,
    pub done: usize,
    pub total: usize,
    pub remaining: usize,
}

impl Progress {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "model_key": self.model_key,
            "trigger": self.trigger.as_str(),
            "started_at": self.started_at,
            "done": self.done,
            "total": self.total,
            "remaining": self.remaining,
        })
    }
}

struct RunState {
    progress: Progress,
    finished: bool,
}

/// One drain in flight. Held in the registry while it works (so `runs()` and a second trigger can
/// see it) and by whoever waits for it to finish.
pub struct Run {
    state: Mutex<RunState>,
    cancel: AtomicBool,
    settle: Condvar,
}

impl Run {
    fn new(progress: Progress) -> Self {
        Run {
            state: Mutex::new(RunState {
                progress,
                finished: false,
            }),
            cancel: AtomicBool::new(false),
            settle: Condvar::new(),
        }
    }

    pub fn snapshot(&self) -> Progress {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .progress
            .clone()
    }

    fn update(&self, processed: usize, remaining: usize) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.progress.done += processed;
        state.progress.remaining = remaining;
        state.progress.total = state.progress.done + remaining;
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    fn finish(&self) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.finished = true;
        drop(state);
        self.settle.notify_all();
    }

    /// Block until the run ends (used by the `wait` mode, which lets a script or a test drive a
    /// full drain through one request).
    pub fn wait(&self) -> Progress {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        while !state.finished {
            state = self
                .settle
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        state.progress.clone()
    }
}

/// Outcome of asking for a run.
pub enum Start {
    Started(Arc<Run>),
    /// The model already has a run: shared, never duplicated (whatever the trigger).
    AlreadyRunning(Arc<Run>),
    /// Nothing pending for this model — no run was created.
    Idle,
}

static RUNS: Mutex<Vec<Arc<Run>>> = Mutex::new(Vec::new());
static WORKER: OnceLock<()> = OnceLock::new();

/// Start the periodic worker (idempotent). The first sweep runs immediately: that is the startup
/// trigger, and it is what repairs a store whose cache was rebuilt or whose candidate was enabled
/// while the server was idle.
pub fn start(db_path: &Path) {
    if WORKER.set(()).is_err() {
        return;
    }
    let db_path = db_path.to_path_buf();
    std::thread::spawn(move || loop {
        sweep(&db_path);
        std::thread::sleep(TICK);
    });
}

/// Active runs, for the admin page's status and progress display.
pub fn runs() -> Vec<Progress> {
    RUNS.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .map(|run| run.snapshot())
        .collect()
}

#[cfg(test)]
pub fn run_for(key: &str) -> Option<Progress> {
    RUNS.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .find(|run| run.snapshot().model_key == key)
        .map(|run| run.snapshot())
}

/// Ask a run to stop after its batch in flight (which still lands). Returns whether a run existed.
pub fn cancel(key: &str) -> bool {
    let runs = RUNS.lock().unwrap_or_else(PoisonError::into_inner);
    match runs.iter().find(|run| run.snapshot().model_key == key) {
        Some(run) => {
            run.cancel.store(true, Ordering::Relaxed);
            true
        }
        None => false,
    }
}

/// Start (or join) a run for one candidate.
pub fn start_run(db_path: &Path, cfg: &EmbedConfig, trigger: Trigger) -> Result<Start, String> {
    let key = cfg.vector_key();
    if let Some(run) = share(&key) {
        return Ok(Start::AlreadyRunning(run));
    }
    let pending = crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
        st.embedding_pending_count(&key, Some(&cfg.fingerprint()))
    })?;
    if pending == 0 {
        return Ok(Start::Idle);
    }
    let run = Arc::new(Run::new(Progress {
        model_key: key.clone(),
        trigger,
        started_at: crate::model::now(),
        done: 0,
        total: pending,
        remaining: pending,
    }));
    {
        // Re-check under the registry lock: two triggers racing here must not both start.
        let mut runs = RUNS.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(existing) = runs.iter().find(|r| r.snapshot().model_key == key) {
            return Ok(Start::AlreadyRunning(Arc::clone(existing)));
        }
        runs.push(Arc::clone(&run));
    }
    let cfg = cfg.clone();
    let db_path = db_path.to_path_buf();
    let worker = Arc::clone(&run);
    std::thread::spawn(move || drain(db_path, cfg, worker));
    Ok(Start::Started(run))
}

/// The registered run for a model, if any.
fn share(key: &str) -> Option<Arc<Run>> {
    RUNS.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .find(|run| run.snapshot().model_key == key)
        .cloned()
}

fn drain(db_path: PathBuf, cfg: EmbedConfig, run: Arc<Run>) {
    let mut rounds = 0;
    while rounds < MAX_ROUNDS && !run.cancelled() {
        rounds += 1;
        match embed::process_pending_for(&db_path, BATCH, &cfg) {
            Ok(EmbedOutcome::Processed {
                processed,
                remaining,
            }) => {
                run.update(processed, remaining);
                if processed == 0 || remaining == 0 {
                    break;
                }
            }
            Ok(_) => break,
            Err(e) => {
                eprintln!("backfill of '{}' stopped: {e}", cfg.vector_key());
                embed::EMBED_FAILOVER.fail(&cfg.circuit_identity());
                break;
            }
        }
    }
    let progress = run.snapshot();
    if progress.done > 0 {
        eprintln!(
            "backfill of '{}' ({}): {} memories embedded, {} left",
            progress.model_key,
            progress.trigger.as_str(),
            progress.done,
            progress.remaining
        );
    }
    run.finish();
    RUNS.lock()
        .unwrap_or_else(PoisonError::into_inner)
        .retain(|r| !Arc::ptr_eq(r, &run));
}

/// One sweep: walk the candidates in the failover's own order (starting from the one that
/// answered last) and start a run for the first candidate that opted into automatic backfill, is
/// not in its failure cooldown, and actually has pending memories. Returns that candidate's key.
pub fn sweep(db_path: &Path) -> Option<String> {
    let configs =
        match crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| st.embedding_configs()) {
            Ok(configs) => configs,
            Err(e) => {
                eprintln!("automatic backfill skipped: {e}");
                return None;
            }
        };
    for cfg in rotation(&configs) {
        if !cfg.auto_backfill || embed::EMBED_FAILOVER.is_open(&cfg.circuit_identity()) {
            continue;
        }
        if share(&cfg.vector_key()).is_some() {
            continue;
        }
        match start_run(db_path, cfg, Trigger::Auto) {
            Ok(Start::Started(_)) => return Some(cfg.vector_key()),
            Ok(_) => continue,
            Err(e) => {
                eprintln!("automatic backfill skipped: {e}");
                return None;
            }
        }
    }
    None
}

/// The write path: drain the candidate that would answer a query, regardless of its opt-in
/// switch — saving a memory must embed it. A candidate that is cooling down or already running is
/// skipped; the run already in flight covers the new write.
pub fn start_write_run(db_path: &Path) {
    let Ok(configs) =
        crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| st.embedding_configs())
    else {
        return;
    };
    for cfg in rotation(&configs) {
        if embed::EMBED_FAILOVER.is_open(&cfg.circuit_identity()) {
            continue;
        }
        match start_run(db_path, cfg, Trigger::Write) {
            // Only the candidate that would answer a query is drained by a write. Idle means it
            // already covered the write (the create-time dedup hint embeds synchronously), and
            // an existing run covers it too; spilling into the next model would spend API budget
            // on a cache the query path does not consult while the head is healthy.
            Ok(Start::Started(_) | Start::AlreadyRunning(_) | Start::Idle) => return,
            Err(_) => continue,
        }
    }
}

/// Candidates in failover order, starting at the one that answered last.
fn rotation(configs: &[EmbedConfig]) -> impl Iterator<Item = &EmbedConfig> {
    let start = if configs.is_empty() {
        0
    } else {
        embed::EMBED_FAILOVER.preferred() % configs.len()
    };
    (0..configs.len()).map(move |offset| &configs[(start + offset) % configs.len()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::test_support::{cleanup, temp_db};

    /// Poll the registry until the condition holds (runs finish on their own thread).
    fn eventually(mut done: impl FnMut() -> bool) {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !done() {
            assert!(std::time::Instant::now() < deadline, "condition never held");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn cfg(id: &str, auto: bool) -> EmbedConfig {
        EmbedConfig {
            id: id.into(),
            base_url: "http://127.0.0.1:9/v1".into(),
            model: id.into(),
            api_key: None,
            query_prefix: None,
            passage_prefix: None,
            min_similarity: 0.0,
            tokenizer: None,
            max_tokens: None,
            auto_backfill: auto,
        }
    }

    /// Runs are shared per model: the second trigger joins the first instead of starting another,
    /// and different models run independently.
    #[test]
    fn a_model_has_at_most_one_run() {
        let path = temp_db("backfill-single-flight");
        cleanup(&path);
        let st = crate::store::Store::open(&path).unwrap();
        st.insert_memory("a", "body", &[], 1, 1).unwrap();
        st.insert_memory("b", "body", &[], 1, 1).unwrap();

        let a = cfg("single-a", true);
        let b = cfg("single-b", true);
        let first = match start_run(&path, &a, Trigger::Manual) {
            Ok(Start::Started(run)) => run,
            Ok(_) => panic!("a pending model must start a run"),
            Err(e) => panic!("{e}"),
        };
        // Same model: joined, with the original trigger
        match start_run(&path, &a, Trigger::Auto) {
            Ok(Start::AlreadyRunning(run)) => {
                assert_eq!(run.snapshot().trigger, Trigger::Manual)
            }
            _ => panic!("a second run for the same model must be refused"),
        }
        // A different model is independent
        assert!(matches!(
            start_run(&path, &b, Trigger::Auto),
            Ok(Start::Started(_))
        ));

        assert!(cancel("single-a") && cancel("single-b"));
        first.wait();
        drop(st);
        cleanup(&path);
    }

    /// A model with nothing pending never creates a run, so an idle store costs no traffic.
    #[test]
    fn idle_model_starts_nothing() {
        let path = temp_db("backfill-idle");
        cleanup(&path);
        let st = crate::store::Store::open(&path).unwrap();
        st.insert_memory("a", "body", &[], 1, 1).unwrap();
        let a = cfg("idle-a", true);
        st.embedding_put(1, &a.vector_key(), &a.fingerprint(), &[vec![1.0]])
            .unwrap();
        assert!(matches!(
            start_run(&path, &a, Trigger::Auto),
            Ok(Start::Idle)
        ));
        assert!(runs().is_empty());
        drop(st);
        cleanup(&path);
    }

    /// The sweep skips models that did not opt in, and the write path does not.
    #[test]
    fn sweep_respects_the_opt_in_the_write_path_does_not() {
        let path = temp_db("backfill-opt-in");
        cleanup(&path);
        let st = crate::store::Store::open(&path).unwrap();
        st.insert_memory("a", "body", &[], 1, 1).unwrap();
        st.settings_put(
            crate::store::Store::SETTING_EMBEDDING_MODELS,
            &serde_json::json!([{
                "id": "optin-a", "name": "optin-a", "enabled": true,
                "base_url": "http://127.0.0.1:9/v1", "model": "optin-a",
                "auto_backfill": false
            }])
            .to_string(),
        )
        .unwrap();
        assert_eq!(sweep(&path), None, "an opted-out model is left alone");
        assert!(run_for("optin-a").is_none());

        start_write_run(&path);
        assert!(
            run_for("optin-a").is_some(),
            "a write still embeds its memory"
        );
        cancel("optin-a");
        eventually(|| run_for("optin-a").is_none());
        drop(st);
        cleanup(&path);
    }
}
