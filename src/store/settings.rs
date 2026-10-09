//! Server settings (the settings key-value table) and semantic-search configuration reads.
//! Vector storage lives in `embeddings`; this module only handles config keys and enablement checks.

use crate::sql;
use rusqlite::params;

use super::Store;

impl Store {
    pub fn search_revision(&self) -> Result<u64, String> {
        self.settings_get("search_revision")?.map_or(Ok(0), |raw| {
            raw.parse().map_err(|_| "corrupt search revision".into())
        })
    }

    pub fn bump_search_revision(&self) -> Result<(), String> {
        self.conn
            .execute(sql::SEARCH_REVISION_BUMP, [])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn new_search_cursor(&self) -> Result<String, String> {
        self.conn
            .query_row(sql::SEARCH_CURSOR_GENERATE, [], |row| row.get(0))
            .map_err(|e| e.to_string())
    }
    pub const SETTING_SEARCH_LIMITS: &'static str = "search_limits";
    pub const SETTING_LIFECYCLE_POLICY: &'static str = "lifecycle_policy";

    pub fn lifecycle_policy(&self) -> Result<crate::lifecycle::Policy, String> {
        let registry = self.predicate_registry("global")?;
        let tags = self.tag_id_names()?;
        self.stored_lifecycle_policy()?
            .try_map(&mut |expr| registry.display(expr, &tags))
    }

    pub fn stored_lifecycle_policy(
        &self,
    ) -> Result<crate::lifecycle::Policy<crate::named_predicates::BoundExpr>, String> {
        match self.settings_get(Self::SETTING_LIFECYCLE_POLICY)? {
            Some(raw) => serde_json::from_str(&raw)
                .map_err(|e| format!("corrupt lifecycle_policy JSON: {e}")),
            None => Ok(crate::lifecycle::Policy::default()),
        }
    }

    pub fn lifecycle_policy_compiled(
        &self,
    ) -> Result<crate::lifecycle::Policy<crate::tag_expr::TagExpr>, String> {
        let registry = self.predicate_registry("global")?;
        let tags = self.tag_id_names()?;
        self.stored_lifecycle_policy()?.try_map(&mut |expr| {
            registry.resolve(expr)?.try_map(&mut |id| {
                tags.get(&id)
                    .cloned()
                    .map(crate::tag_expr::TagAtom::Tag)
                    .ok_or_else(|| format!("missing tag id {id}"))
            })
        })
    }

    pub fn search_limits(&self) -> Result<crate::search::Limits, String> {
        match self.settings_get(Self::SETTING_SEARCH_LIMITS)? {
            Some(raw) => {
                let value = serde_json::from_str(&raw)
                    .map_err(|e| format!("corrupt search_limits JSON: {e}"))?;
                crate::search::Limits::parse(&value)
            }
            None => Ok(crate::search::Limits::default()),
        }
    }

    /// Read a server setting; returns None when the key does not exist.
    pub fn settings_get(&self, key: &str) -> Result<Option<String>, String> {
        match self
            .conn
            .query_row(sql::SETTINGS_GET, [key], |r| r.get::<_, String>(0))
        {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn settings_put(&self, key: &str, value: &str) -> Result<(), String> {
        let policy_raw;
        let value = if key == Self::SETTING_LIFECYCLE_POLICY {
            let parsed = serde_json::from_str(value)
                .map_err(|e| format!("invalid lifecycle_policy JSON: {e}"))?;
            let policy = crate::lifecycle::Policy::parse(&parsed)?;
            let registry = self.predicate_registry("global")?;
            let tags = self
                .tag_id_names()?
                .into_iter()
                .map(|(id, name)| (name, id))
                .collect();
            let bound = policy
                .try_map(&mut |raw| registry.bind(crate::tag_expr::parse_rule(raw)?, &tags))?;
            for rule in &bound.rules {
                registry.resolve(&rule.predicate)?;
            }
            policy_raw = serde_json::to_string(&bound).map_err(|e| e.to_string())?;
            policy_raw.as_str()
        } else {
            value
        };
        if key == Self::SETTING_EMBEDDING_MODELS {
            if let Ok(serde_json::Value::Array(items)) = serde_json::from_str(value) {
                for entry in items.iter().filter_map(crate::embed::EmbedEntry::from_json) {
                    if let Some(fingerprint) = entry.fingerprint() {
                        if !entry.id.is_empty()
                            && self
                                .settings_get(&format!("embedding_cache:{}", entry.id))?
                                .as_deref()
                                != Some(&fingerprint)
                        {
                            self.embedding_delete_model(&entry.id)?;
                        }
                    }
                }
            }
        }
        self.conn
            .execute(sql::SETTINGS_PUT, params![key, value])
            .map_err(|e| e.to_string())?;
        if [
            Self::SETTING_SEARCH_LIMITS,
            Self::SETTING_LIFECYCLE_POLICY,
            Self::SETTING_TAG_RULES,
            Self::SETTING_EMBEDDING_MODELS,
            Self::SETTING_RERANK_MODELS,
        ]
        .contains(&key)
        {
            self.bump_search_revision()?;
        }
        Ok(())
    }

    /// Token auth switch (a settings key, explicit and persisted): whether enforcement happens at runtime depends solely on this value,
    /// defaulting to off (a fresh database = open mode with zero configuration). **Enabling** has a precondition — at least one
    /// admin identity must exist (guarded at the api layer, see `has_admin_identity`) — so the admin surface cannot
    /// end up locked with no one holding a token after the switch is flipped.
    pub const SETTING_AUTH_REQUIRED: &'static str = "auth_required";

    pub fn auth_required(&self) -> Result<bool, String> {
        Ok(self.settings_get(Self::SETTING_AUTH_REQUIRED)?.as_deref() == Some("true"))
    }

    pub fn set_auth_required(&self, on: bool) -> Result<(), String> {
        self.settings_put(
            Self::SETTING_AUTH_REQUIRED,
            if on { "true" } else { "false" },
        )
    }

    /// The anonymous identity's capability set (a settings key holding full-key boolean JSON, same shape as identities.permissions):
    /// once auth is enabled, tokenless requests resolve to the anonymous identity through it; unset or capability-less =
    /// anonymous access rejected wholesale. Ineffective while auth is off (open mode always has full capabilities).
    pub const SETTING_ANONYMOUS_PERMISSIONS: &'static str = "anonymous_permissions";

    /// Read the anonymous capability set; None = unset (anonymous rejected). Corrupt JSON in the database errors out rather
    /// than silently widening permissions — the HTTP layer's parsing is fail-closed accordingly (surfacing as 401).
    pub fn anonymous_permissions(&self) -> Result<Option<crate::auth::Permissions>, String> {
        let Some(raw) = self.settings_get(Self::SETTING_ANONYMOUS_PERMISSIONS)? else {
            return Ok(None);
        };
        let v: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|e| format!("corrupt anonymous_permissions JSON: {e}"))?;
        crate::auth::Permissions::from_json(&v).map(Some)
    }

    /// Write/clear the anonymous capability set. Some stores full-key compact JSON; None = delete the key (anonymous rejected).
    pub fn set_anonymous_permissions(
        &self,
        value: Option<&crate::auth::Permissions>,
    ) -> Result<(), String> {
        match value {
            Some(p) => {
                self.settings_put(Self::SETTING_ANONYMOUS_PERMISSIONS, &p.to_stored_string())
            }
            None => self
                .conn
                .execute(sql::SETTINGS_DELETE, [Self::SETTING_ANONYMOUS_PERMISSIONS])
                .map(|_| ())
                .map_err(|e| e.to_string()),
        }
    }

    /// The ordered embedding candidate list (one JSON array, canonical form written by the REST
    /// layer via `EmbedEntry::to_json`). Array order is the failover priority; each entry carries
    /// its own enabled switch, endpoint credentials, instruction prefixes and cosine floor.
    pub const SETTING_EMBEDDING_MODELS: &'static str = "embedding_models";

    /// All configured embedding candidates, in priority order (disabled ones included — they are
    /// listed and manageable in the admin UI, just never selected at runtime).
    pub fn embedding_entries(&self) -> Result<Vec<crate::embed::EmbedEntry>, String> {
        let Some(raw) = self.settings_get(Self::SETTING_EMBEDDING_MODELS)? else {
            return Ok(Vec::new());
        };
        let value: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|e| format!("corrupt embedding_models JSON: {e}"))?;
        let arr = value
            .as_array()
            .ok_or("corrupt embedding_models: not a JSON array")?;
        Ok(arr
            .iter()
            .filter_map(crate::embed::EmbedEntry::from_json)
            .collect())
    }

    /// The runtime selection pool: enabled, fully specified candidates in priority order. An
    /// empty list means semantic search is unconfigured — every caller degrades as unavailable
    /// instead of erroring.
    pub fn embedding_configs(&self) -> Result<Vec<crate::embed::EmbedConfig>, String> {
        Ok(self
            .embedding_entries()?
            .iter()
            .filter_map(|e| e.usable())
            .collect())
    }

    /// The ordered reranker candidate list (same shape and discipline as `embedding_models`).
    pub const SETTING_RERANK_MODELS: &'static str = "rerank_models";

    /// All configured reranker candidates, in priority order (disabled ones included).
    pub fn rerank_entries(&self) -> Result<Vec<crate::rerank::RerankEntry>, String> {
        let Some(raw) = self.settings_get(Self::SETTING_RERANK_MODELS)? else {
            return Ok(Vec::new());
        };
        let value: serde_json::Value =
            serde_json::from_str(&raw).map_err(|e| format!("corrupt rerank_models JSON: {e}"))?;
        let arr = value
            .as_array()
            .ok_or("corrupt rerank_models: not a JSON array")?;
        Ok(arr
            .iter()
            .filter_map(crate::rerank::RerankEntry::from_json)
            .collect())
    }

    /// The runtime reranker selection pool: enabled, fully specified candidates in priority order.
    /// An empty list means reranking is off — search results keep their fused order.
    pub fn rerank_configs(&self) -> Result<Vec<crate::rerank::RerankConfig>, String> {
        Ok(self
            .rerank_entries()?
            .iter()
            .filter_map(|e| e.usable())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    #[test]
    fn settings_roundtrip_and_upsert() {
        let path = temp_db("settings");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        assert_eq!(st.settings_get("instructions").unwrap(), None);
        st.settings_put("instructions", "v1").unwrap();
        st.settings_put("instructions", "v2").unwrap();
        assert_eq!(
            st.settings_get("instructions").unwrap().as_deref(),
            Some("v2")
        );
        // Other keys do not affect each other
        st.settings_put("other", "x").unwrap();
        assert_eq!(
            st.settings_get("instructions").unwrap().as_deref(),
            Some("v2")
        );
        cleanup(&path);
    }

    /// The candidate list read: prefixes read back verbatim (a trailing space is part of an E5
    /// instruction), per-entry floors apply with the built-in default, and only enabled +
    /// complete entries enter the runtime pool while all entries stay listed.
    #[test]
    fn embedding_entries_parse_and_filter() {
        let path = temp_db("embedding-entries");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        // Unset key = unconfigured
        assert!(st.embedding_entries().unwrap().is_empty());
        assert!(st.embedding_configs().unwrap().is_empty());

        st.settings_put(
            Store::SETTING_EMBEDDING_MODELS,
            r#"[
                {"base_url": "http://x/v1", "id": "e5", "name": "e5", "model": "e5", "query_prefix": "query: ", "passage_prefix": "passage: ", "min_similarity": 0.45},
                {"base_url": "http://y/v1", "id": "m3", "name": "m3", "model": "m3", "enabled": false, "min_similarity": 0},
                {"base_url": "", "id": "draft", "name": "draft", "model": "draft", "min_similarity": 0.2}
            ]"#,
        )
        .unwrap();
        let entries = st.embedding_entries().unwrap();
        assert_eq!(
            entries.len(),
            3,
            "disabled and incomplete entries stay listed"
        );

        let e5 = entries[0].usable().unwrap();
        // Trailing space preserved; empty string = unset
        assert_eq!(e5.query_prefix.as_deref(), Some("query: "));
        assert_eq!(e5.passage_prefix.as_deref(), Some("passage: "));
        assert!((e5.min_similarity - 0.45).abs() < 1e-6);
        assert_eq!(e5.vector_key(), "e5");

        let m3 = entries[1].usable();
        assert!(m3.is_none(), "disabled = not operational");
        assert_eq!(entries[1].vector_key(), "m3", "identity exists regardless");

        assert!(entries[2].usable().is_none(), "blank base_url = not usable");

        // Runtime pool: only the one usable candidate, in priority order
        let configs = st.embedding_configs().unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].model, "e5");

        // "0" floor semantics moved into the entry: 0.0 means the floor is off
        assert_eq!(entries[1].min_similarity, Some(0.0));

        // Corrupt JSON surfaces as an error (the admin UI's problem to fix, not silently ignored)
        st.settings_put(Store::SETTING_EMBEDDING_MODELS, "{oops")
            .unwrap();
        assert!(st.embedding_entries().is_err());
        cleanup(&path);
    }

    #[test]
    fn anonymous_permissions_roundtrip_and_clear() {
        let path = temp_db("anon-perms");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        // Unset = None (anonymous rejected)
        assert_eq!(st.anonymous_permissions().unwrap(), None);

        let read_only =
            crate::auth::Permissions::from_json(&serde_json::json!({ "read": true })).unwrap();
        st.set_anonymous_permissions(Some(&read_only)).unwrap();
        let got = st.anonymous_permissions().unwrap().unwrap();
        assert!(got.has(crate::auth::Cap::Read));
        assert!(!got.has(crate::auth::Cap::Admin));

        // Overwrite and clear
        st.set_anonymous_permissions(Some(&crate::auth::Permissions::all()))
            .unwrap();
        assert!(st
            .anonymous_permissions()
            .unwrap()
            .unwrap()
            .has(crate::auth::Cap::Admin));
        st.set_anonymous_permissions(None).unwrap();
        assert_eq!(st.anonymous_permissions().unwrap(), None);
        cleanup(&path);
    }
}
