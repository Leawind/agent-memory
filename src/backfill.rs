//! Background backfill scheduler: keeps vectors caught up without waiting for someone to click
//! the backfill button.
//!
//! Three moments ask for a sweep and all of them wake the same worker: service start, a timer,
//! and a successful settings save (which is where a candidate becomes usable, or where a changed
//! fingerprint makes the whole store pending). The write path joins in by nudging instead of
//! spawning its own drain, so one worker owns the queue and two drains never race.
//!
//! Each sweep answers the same three questions: which candidate is worth draining (order =
//! priority, only candidates that opted in via `auto_backfill`), does it actually have pending
//! work (one cheap count — no backlog means no traffic at all), and then drain it in batches.
//! Failure semantics are the search path's own: a candidate that cannot answer enters the shared
//! failure cooldown and the next sweep moves on, so a dead service costs one timeout per window
//! rather than one per tick. Draining is bounded per sweep; whatever remains is picked up by the
//! next round or the next tick.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::OnceLock;
use std::time::Duration;

use crate::embed::{self, EmbedConfig, EmbedOutcome};
use crate::store::TxMode;

/// How often the worker sweeps on its own.
pub const TICK: Duration = Duration::from_secs(600);
/// Memories per embedding round inside one sweep.
const BATCH: usize = embed::MAX_BATCH;
/// Upper bound on rounds per sweep, so one tick cannot monopolize the worker when writes keep
/// arriving; the remainder waits for the next round or the next tick.
const MAX_ROUNDS: usize = 256;
/// A failure this early in a sweep means the service is down: stop instead of walking every
/// candidate through its own timeout.
const MIN_ROUNDS_BEFORE_FAILURE: usize = 1;

/// The worker's wake-up handle. Capacity 1 with `try_send`: nudges coalesce, so a burst of writes
/// during a drain leaves exactly one follow-up sweep queued.
static WAKE: OnceLock<SyncSender<()>> = OnceLock::new();

/// Start the worker (idempotent: a second call is a no-op). The first sweep runs immediately —
/// that is the startup trigger, and it is what repairs a store whose cache was rebuilt or whose
/// candidate was just enabled while the server was idle.
pub fn start(db_path: &Path) {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    if WAKE.set(tx).is_err() {
        return; // already started
    }
    let db_path = db_path.to_path_buf();
    std::thread::spawn(move || worker(db_path, rx));
}

/// Ask for a sweep. Returns false when no worker runs (a foreground context such as tests or the
/// CLI), letting the caller fall back to its own drain.
pub fn nudge() -> bool {
    match WAKE.get() {
        Some(tx) => match tx.try_send(()) {
            Ok(()) | Err(TrySendError::Full(())) => true,
            Err(TrySendError::Disconnected(())) => false,
        },
        None => false,
    }
}

fn worker(db_path: PathBuf, rx: std::sync::mpsc::Receiver<()>) {
    loop {
        sweep(&db_path);
        // Wait for the next nudge or the timer, whichever comes first; a disconnected channel
        // means the process is shutting down.
        match rx.recv_timeout(TICK) {
            Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
        }
    }
}

/// One sweep: pick the first opted-in candidate with pending work and drain it. Returns the
/// number of memories embedded, for tests and logging.
pub fn sweep(db_path: &Path) -> usize {
    let configs =
        match crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| st.embedding_configs()) {
            Ok(configs) => configs,
            Err(e) => {
                eprintln!("automatic backfill skipped: {e}");
                return 0;
            }
        };
    if configs.is_empty() {
        return 0;
    }
    let Some(target) = pick(&configs, |cfg| {
        crate::store::with_db_in(db_path, TxMode::ReadOnly, |st| {
            st.embedding_pending_count(&cfg.vector_key(), Some(&cfg.fingerprint()))
        })
        .unwrap_or(0)
    }) else {
        return 0;
    };
    drain(db_path, &configs[target])
}

/// Which candidate the sweep should work on: the first one, in priority order, that opted into
/// automatic backfill, is not in its failure cooldown, and has pending memories. Everything else
/// keeps its backlog until a later tick (or the operator's own backfill).
fn pick(configs: &[EmbedConfig], pending: impl Fn(&EmbedConfig) -> usize) -> Option<usize> {
    configs.iter().position(|cfg| {
        cfg.auto_backfill
            && !embed::EMBED_FAILOVER.is_open(&cfg.circuit_identity())
            && pending(cfg) > 0
    })
}

fn drain(db_path: &Path, cfg: &EmbedConfig) -> usize {
    let mut done = 0;
    for round in 0..MAX_ROUNDS {
        match embed::process_pending_for(db_path, BATCH, cfg) {
            Ok(EmbedOutcome::Processed { processed, .. }) => {
                done += processed;
                if processed == 0 {
                    break;
                }
            }
            Ok(_) => break,
            Err(e) => {
                eprintln!("automatic backfill of '{}' stopped: {e}", cfg.vector_key());
                if round >= MIN_ROUNDS_BEFORE_FAILURE {
                    embed::EMBED_FAILOVER.fail(&cfg.circuit_identity());
                }
                break;
            }
        }
    }
    if done > 0 {
        eprintln!(
            "automatic backfill of '{}': {done} memories embedded",
            cfg.vector_key()
        );
    }
    done
}

#[cfg(test)]
mod tests {
    use super::*;

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

    /// Only opted-in candidates with real work are picked, in priority order; with none, a sweep
    /// returns nothing and stays silent.
    #[test]
    fn pick_prefers_the_first_opted_in_candidate_with_work() {
        let configs = [cfg("a", false), cfg("b", true), cfg("c", true)];
        let pending = |c: &EmbedConfig| match c.id.as_str() {
            "b" => 0,
            _ => 5,
        };
        assert_eq!(pick(&configs, pending), Some(2), "b has nothing to do");

        let all_idle = |_: &EmbedConfig| 0;
        assert_eq!(pick(&configs, all_idle), None);

        let nobody_opted_in = [cfg("a", false), cfg("b", false)];
        assert_eq!(pick(&nobody_opted_in, pending), None);
    }

    /// A candidate in its failure cooldown is skipped without counting its backlog, so a dead
    /// service is not queried on every tick.
    #[test]
    fn pick_skips_candidates_in_failure_cooldown() {
        let configs = [cfg("cool", true), cfg("next", true)];
        embed::EMBED_FAILOVER.fail(&configs[0].circuit_identity());
        let pending = |_: &EmbedConfig| 3;
        assert_eq!(
            pick(&configs, pending),
            Some(1),
            "the cooled candidate is skipped"
        );
        embed::EMBED_FAILOVER.clear(&configs[0].circuit_identity());
        assert_eq!(pick(&configs, pending), Some(0), "clearing releases it");
    }
}
