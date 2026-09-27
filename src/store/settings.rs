//! 服务器设置（settings 键值表）与语义搜索的配置读取。
//! 向量的存取在 `embeddings`，这里只管配置键与生效判定。

use crate::sql;
use rusqlite::params;

use super::Store;

impl Store {
    /// 读取服务器设置；键不存在返回 None。
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

    /// token 鉴权开关（settings 键，显式且持久化）：运行时强制与否只看这个值，
    /// 缺省视为关闭（全新库 = 开放模式零配置）。**开启**有前置条件——至少存在
    /// 一个 admin 身份（api 层把守，见 `has_admin_identity`），防止开关翻上后
    /// 无人持有 token、管理面整体锁死。
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

    /// 匿名身份的能力集（settings 键，存全键布尔 JSON，与 identities.permissions
    /// 同形态）：鉴权开启后无 token 请求按它解析为匿名身份；未设置或全无能力 =
    /// 匿名被整体拒绝。鉴权关闭时不生效（开放模式恒全能力）。
    pub const SETTING_ANONYMOUS_PERMISSIONS: &'static str = "anonymous_permissions";

    /// 读取匿名能力集；None = 未设置（匿名被拒绝）。库中 JSON 损坏时报错而非
    /// 静默放大权限——HTTP 层的解析随之 fail-closed（表现为 401）。
    pub fn anonymous_permissions(&self) -> Result<Option<crate::auth::Permissions>, String> {
        let Some(raw) = self.settings_get(Self::SETTING_ANONYMOUS_PERMISSIONS)? else {
            return Ok(None);
        };
        let v: serde_json::Value = serde_json::from_str(&raw)
            .map_err(|e| format!("corrupt anonymous_permissions JSON: {e}"))?;
        crate::auth::Permissions::from_json(&v).map(Some)
    }

    /// 写入/清除匿名能力集。Some 存全键紧凑 JSON；None = 删除键（匿名被拒绝）。
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

    /// 语义搜索开关（与 auth_required 同型的显式布尔键）。
    pub fn embedding_enabled(&self) -> Result<bool, String> {
        Ok(self
            .settings_get(Self::SETTING_EMBEDDING_ENABLED)?
            .as_deref()
            == Some("true"))
    }

    /// 生效的 embedding 配置：开关开启且 base_url / model 均非空才可用——
    /// 配置不完整视为"未配置"，所有调用方按不可用降级，不报错。
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
        // 其他键互不影响
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
        // 未设置 = None（匿名被拒绝）
        assert_eq!(st.anonymous_permissions().unwrap(), None);

        let read_only =
            crate::auth::Permissions::from_json(&serde_json::json!({ "read": true })).unwrap();
        st.set_anonymous_permissions(Some(&read_only)).unwrap();
        let got = st.anonymous_permissions().unwrap().unwrap();
        assert!(got.has(crate::auth::Cap::Read));
        assert!(!got.has(crate::auth::Cap::Admin));

        // 覆盖写入与清除
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
