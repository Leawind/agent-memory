//! Server settings (the settings key-value table) and semantic-search configuration reads.
//! Vector storage lives in `embeddings`; this module only handles config keys and enablement checks.

use crate::sql;
use rusqlite::params;

use super::Store;

impl Store {
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
        self.conn
            .execute(sql::SETTINGS_PUT, params![key, value])
            .map(|_| ())
            .map_err(|e| e.to_string())
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

    pub const SETTING_EMBEDDING_ENABLED: &'static str = "embedding_enabled";
    pub const SETTING_EMBEDDING_BASE_URL: &'static str = "embedding_base_url";
    pub const SETTING_EMBEDDING_MODEL: &'static str = "embedding_model";
    pub const SETTING_EMBEDDING_API_KEY: &'static str = "embedding_api_key";

    /// Semantic search switch (an explicit boolean key of the same kind as auth_required).
    pub fn embedding_enabled(&self) -> Result<bool, String> {
        Ok(self
            .settings_get(Self::SETTING_EMBEDDING_ENABLED)?
            .as_deref()
            == Some("true"))
    }

    /// The effective embedding configuration: usable only when the switch is on and base_url / model are both non-empty —
    /// an incomplete configuration counts as "not configured", and every caller degrades as unavailable instead of erroring.
    pub fn embedding_config(&self) -> Result<Option<crate::embed::EmbedConfig>, String> {
        if !self.embedding_enabled()? {
            return Ok(None);
        }
        let base_url = self
            .settings_get(Self::SETTING_EMBEDDING_BASE_URL)?
            .unwrap_or_default();
        let model = self
            .settings_get(Self::SETTING_EMBEDDING_MODEL)?
            .unwrap_or_default();
        if base_url.trim().is_empty() || model.trim().is_empty() {
            return Ok(None);
        }
        let api_key = self
            .settings_get(Self::SETTING_EMBEDDING_API_KEY)?
            .filter(|s| !s.is_empty());
        Ok(Some(crate::embed::EmbedConfig {
            base_url: base_url.trim().to_string(),
            model: model.trim().to_string(),
            api_key,
        }))
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
