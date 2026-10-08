//! Access events are the scoring source; projections and hourly buckets are derived data.

use crate::store::{self, TxMode};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub const HALF_LIFE_SECONDS: u64 = 30 * 86400;
pub const RAW_RETENTION_SECONDS: u64 = 30 * 86400;
pub const CREDIT_WINDOW_SECONDS: u64 = 60;
pub const SCORING_VERSION: u32 = 1;
pub static DROPPED_READ_EVENTS: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Read,
    Use,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub actor: String,
    pub event_key: String,
    pub kind: Kind,
    pub occurred_at: u64,
    pub weight: f64,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bucket {
    pub end: u64,
    pub score: f64,
    pub reads: i64,
    pub uses: i64,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct History {
    pub scoring_version: u32,
    pub events: Vec<Event>,
    pub buckets: Vec<Bucket>,
}

impl History {
    pub fn validate(&self) -> Result<(), String> {
        if self.scoring_version != SCORING_VERSION {
            return Err("unsupported access scoring version".into());
        }
        for event in &self.events {
            if event.actor.is_empty()
                || event.actor.chars().count() > 100
                || event.event_key.is_empty()
                || event.event_key.chars().count() > 100
                || event.event_key.chars().any(char::is_control)
                || event.occurred_at > crate::lifecycle::MAX_TIMESTAMP
                || !event.weight.is_finite()
                || !(event.weight == 0.0 || event.weight == event.kind.weight())
            {
                return Err("invalid access event".into());
            }
        }
        let mut ends = std::collections::HashSet::new();
        for bucket in &self.buckets {
            if bucket.end == 0
                || bucket.end % 3600 != 0
                || bucket.end > crate::lifecycle::MAX_TIMESTAMP
                || !ends.insert(bucket.end)
                || bucket.reads < 0
                || bucket.uses < 0
                || !bucket.score.is_finite()
                || bucket.score < 0.0
                || bucket.score > bucket.reads as f64 * 0.25 + bucket.uses as f64 + 1e-9
            {
                return Err("invalid access bucket".into());
            }
        }
        Ok(())
    }
}
impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Use => "use",
        }
    }
    pub fn weight(self) -> f64 {
        match self {
            Self::Read => 0.25,
            Self::Use => 1.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Projection {
    pub score: f64,
    pub as_of: u64,
    pub reads: i64,
    pub uses: i64,
}

pub fn decay(score: f64, from: u64, to: u64) -> f64 {
    score
        * (-std::f64::consts::LN_2 * to.saturating_sub(from) as f64 / HALF_LIFE_SECONDS as f64)
            .exp()
}

impl Projection {
    pub fn value(&self, now: u64) -> f64 {
        decay(self.score, self.as_of, now)
    }
    pub fn add(&mut self, score: f64, at: u64, reads: i64, uses: i64) {
        let reference = self.as_of.max(at);
        self.score = decay(self.score, self.as_of, reference) + decay(score, at, reference);
        self.as_of = reference;
        self.reads = self.reads.saturating_add(reads);
        self.uses = self.uses.saturating_add(uses);
    }
}

/// Automatic read telemetry commits after the read snapshot closes. A short write
/// deadline keeps contention from failing or delaying the successful read indefinitely.
pub fn record_reads(path: &Path, actor: &str, ids: &[i64]) {
    if ids.is_empty() {
        return;
    }
    let result = store::with_db_in_timeout(
        path,
        TxMode::Write,
        std::time::Duration::from_millis(50),
        |st| -> Result<(), String> {
            let now = crate::model::now();
            for id in ids {
                if st.memory_exists(*id)? {
                    st.access_append(*id, actor, Kind::Read, None, now)?;
                }
            }
            Ok(())
        },
    );
    if let Err(error) = result {
        DROPPED_READ_EVENTS.fetch_add(ids.len() as u64, Ordering::Relaxed);
        eprintln!("read telemetry skipped: {error}");
    }
}

/// Bounded hourly compaction and daily reconstruction from the durable sources.
pub fn start_maintenance(path: &Path) {
    let path = path.to_owned();
    std::thread::spawn(move || {
        let mut hours = 0u64;
        loop {
            let started = std::time::Instant::now();
            for _ in 0..20 {
                let result = store::with_db_in(&path, TxMode::Write, |st| {
                    st.access_compact(crate::model::now())
                });
                match result {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("access compaction failed: {error}");
                        break;
                    }
                }
                if started.elapsed() > std::time::Duration::from_secs(2) {
                    break;
                }
                std::thread::yield_now();
            }
            if hours % 24 == 0 {
                if let Err(error) =
                    store::with_db_in(&path, TxMode::Write, |st| st.access_rebuild())
                {
                    eprintln!("access projection rebuild failed: {error}");
                }
            }
            hours += 1;
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exponential_projection_handles_order_and_query_time_decay() {
        let mut state = Projection::default();
        state.add(1.0, 100, 0, 1);
        state.add(1.0, 100 + HALF_LIFE_SECONDS, 0, 1);
        assert!((state.score - 1.5).abs() < 1e-12);
        assert!((state.value(100 + 2 * HALF_LIFE_SECONDS) - 0.75).abs() < 1e-12);
        state.add(0.25, 100, 1, 0);
        assert!((state.score - 1.625).abs() < 1e-12);
        assert_eq!((state.reads, state.uses), (1, 2));
    }
}
