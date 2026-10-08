//! Short-lived summary-only ranking snapshots, scoped to one caller and database.

use crate::{auth::IdentityCtx, store::Store, tools::ToolError};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const TTL_SECONDS: u64 = 120;
const MAX_ENTRIES: usize = 128;
const MAX_BYTES: usize = 16 * 1024 * 1024;
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

struct Entry {
    scope: String,
    revision: u64,
    payload: Value,
    expires_at: u64,
    deadline: Instant,
    created: Instant,
    bytes: usize,
}

#[derive(Default)]
struct Cache {
    entries: HashMap<String, Entry>,
    bytes: usize,
}

fn stale() -> ToolError {
    ToolError::StaleSearch(
        "search cursor expired or invalid; repeat the same search without cursor".into(),
    )
}

fn cache() -> &'static Mutex<Cache> {
    CACHE.get_or_init(|| Mutex::new(Cache::default()))
}

pub fn scope(st: &Store, ctx: &IdentityCtx, filters: &Value) -> String {
    let path =
        std::fs::canonicalize(&st.path).unwrap_or_else(|_| crate::store::normalize_path(&st.path));
    crate::util::sha256_hex(
        json!([path.to_string_lossy(), ctx.summary(), filters])
            .to_string()
            .as_bytes(),
    )
}

pub fn page(payload: &Value, offset: u64, limit: u64) -> Value {
    let mut fields: serde_json::Map<String, Value> = payload
        .as_object()
        .expect("search payload is an object")
        .iter()
        .filter(|(key, _)| key.as_str() != "results")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    if offset != 0 {
        fields.remove("hint");
    }
    let results: Vec<_> = payload["results"]
        .as_array()
        .into_iter()
        .flatten()
        .skip(usize::try_from(offset).unwrap_or(usize::MAX))
        .take(limit as usize)
        .cloned()
        .collect();
    fields.insert("results".into(), json!(results));
    fields.insert("returned".into(), json!(results.len()));
    fields.insert("offset".into(), json!(offset));
    Value::Object(fields)
}

impl Cache {
    fn prune(&mut self, now: u64) {
        let instant = Instant::now();
        self.entries
            .retain(|_, entry| entry.expires_at > now && entry.deadline > instant);
        self.bytes = self.entries.values().map(|entry| entry.bytes).sum();
    }

    fn insert(&mut self, token: String, entry: Entry, now: u64) -> bool {
        self.prune(now);
        if entry.bytes > MAX_BYTES || entry.expires_at <= now {
            return false;
        }
        while self.entries.len() >= MAX_ENTRIES || self.bytes + entry.bytes > MAX_BYTES {
            let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.created)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            if let Some(removed) = self.entries.remove(&oldest) {
                self.bytes -= removed.bytes;
            }
        }
        if let Some(removed) = self.entries.remove(&token) {
            self.bytes -= removed.bytes;
        }
        self.bytes += entry.bytes;
        self.entries.insert(token, entry);
        true
    }

    fn get(
        &mut self,
        token: &str,
        scope: &str,
        revision: u64,
        now: u64,
        offset: u64,
        limit: u64,
    ) -> Result<Value, ToolError> {
        self.prune(now);
        let entry = self
            .entries
            .get(token)
            .filter(|entry| entry.scope == scope && entry.revision == revision)
            .ok_or_else(stale)?;
        Ok(page(&entry.payload, offset, limit))
    }
}

pub fn get(
    token: &str,
    scope: &str,
    revision: u64,
    offset: u64,
    limit: u64,
) -> Result<Value, ToolError> {
    cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(token, scope, revision, crate::model::now(), offset, limit)
}

pub fn create(
    st: &Store,
    scope: String,
    revision: u64,
    mut payload: Value,
    offset: u64,
    limit: u64,
    next_expiry: Option<u64>,
) -> Result<Value, ToolError> {
    let now = crate::model::now();
    let expires_at = next_expiry.unwrap_or(u64::MAX).min(now + TTL_SECONDS);
    let token = st.new_search_cursor()?;
    payload["cursor"] = json!(token);
    payload["cursor_expires_at"] = json!(expires_at);
    let result = page(&payload, offset, limit);
    let bytes = serde_json::to_vec(&payload)
        .map_err(|e| ToolError::invalid(e.to_string()))?
        .len();
    let instant = Instant::now();
    let entry = Entry {
        scope,
        revision,
        payload,
        expires_at,
        deadline: instant + Duration::from_secs(expires_at.saturating_sub(now)),
        created: instant,
        bytes,
    };
    if cache()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(token, entry, now)
    {
        return Ok(result);
    }
    let mut result = result;
    result.as_object_mut().unwrap().remove("cursor");
    result.as_object_mut().unwrap().remove("cursor_expires_at");
    result["cursor_unavailable"] = json!("result_set_too_large_or_expiring");
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(scope: &str, revision: u64, bytes: usize) -> Entry {
        let now = Instant::now();
        Entry {
            scope: scope.into(),
            revision,
            payload: json!({"results":[{"id":"m1","summary":"one"},{"id":"m2","summary":"two"}],"hint":"first page","total_matches":2}),
            expires_at: 100,
            deadline: now + Duration::from_secs(120),
            created: now,
            bytes,
        }
    }
    #[test]
    fn scope_revision_wall_expiry_and_page_disclosure_are_enforced() {
        let mut cache = Cache::default();
        assert!(cache.insert("cursor".into(), entry("alice/db/query", 4, 100), 1));
        let next = cache.get("cursor", "alice/db/query", 4, 1, 1, 1).unwrap();
        assert_eq!(next["results"][0]["id"], "m2");
        assert_eq!(next["returned"], 1);
        assert!(next.get("hint").is_none());
        for (scope, revision, now) in [
            ("bob/db/query", 4, 1),
            ("alice/other/query", 4, 1),
            ("alice/db/other", 4, 1),
            ("alice/db/query", 5, 1),
            ("alice/db/query", 4, 100),
        ] {
            assert!(matches!(
                cache.get("cursor", scope, revision, now, 0, 1),
                Err(ToolError::StaleSearch(_))
            ));
        }
        assert!(cache.entries.is_empty());
    }
    #[test]
    fn monotonic_deadline_and_capacity_keep_cache_bounded() {
        let mut cache = Cache::default();
        let mut expired = entry("scope", 1, 100);
        expired.deadline = Instant::now();
        cache.insert("expired".into(), expired, 1);
        assert!(cache.get("expired", "scope", 1, 1, 0, 1).is_err());
        for index in 0..200 {
            cache.insert(index.to_string(), entry("scope", 1, 100), 1);
        }
        assert_eq!(cache.entries.len(), MAX_ENTRIES);
        cache.insert("large1".into(), entry("scope", 1, MAX_BYTES / 2), 1);
        cache.insert("large2".into(), entry("scope", 1, MAX_BYTES / 2), 1);
        assert!(cache.bytes <= MAX_BYTES);
        let before = cache.entries.len();
        assert!(!cache.insert("oversized".into(), entry("scope", 1, MAX_BYTES + 1), 1));
        assert_eq!(cache.entries.len(), before);
    }
}
