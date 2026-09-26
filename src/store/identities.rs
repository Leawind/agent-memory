//! 身份数据操作：token 生成与哈希、身份 CRUD、token → 请求身份。
//! token 明文只在创建/重置返回值出现一次，库内仅存哈希与尾缀提示。

use crate::sql;
use rusqlite::params;
use serde_json::{json, Value};

use super::{is_unique_violation, Store};

impl Store {
    /// 生成随机 token（64 位十六进制，SQLite PRNG 由系统熵播种）。
    pub fn generate_token(&self) -> Result<String, String> {
        self.conn
            .query_row(sql::TOKEN_GENERATE, [], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())
    }

    /// 是否存在具备 admin 能力的身份（permissions JSON 恒为全键紧凑对象，
    /// LIKE '%"admin":true%' 精确命中）。开启鉴权开关的前置条件。
    pub fn has_admin_identity(&self) -> Result<bool, String> {
        self.conn
            .query_row(sql::IDENTITY_ANY_ADMIN, [], |r| r.get::<_, i64>(0))
            .map(|n| n != 0)
            .map_err(|e| e.to_string())
    }

    /// 新建身份：生成随机 token，返回 (token, 管理视图)。重名由唯一约束显式化。
    /// token 明文只在本返回值出现一次，库内仅存哈希与尾缀提示。
    pub fn identity_create(
        &self,
        name: &str,
        permissions: &crate::auth::Permissions,
    ) -> Result<(String, Value), String> {
        let token = self.generate_token()?;
        let (hash, hint) = token_hash_and_hint(&token);
        let stored = permissions.to_stored_string();
        self.conn
            .execute(
                sql::IDENTITY_INSERT,
                params![name, hash, hint, stored, crate::model::now() as i64],
            )
            .map(|_| ())
            .map_err(|e| {
                if is_unique_violation(&e) {
                    format!("identity '{name}' already exists")
                } else {
                    e.to_string()
                }
            })?;
        let view = self.identity_view(name)?.ok_or_else(|| {
            format!("identity '{name}' vanished right after creation (concurrent modification)")
        })?;
        Ok((token, view))
    }

    /// Bearer token → 请求身份；未知 token 返回 None（调用方决定 401）。
    /// 入参哈希后与库存哈希比对，库内无明文。
    pub fn identity_ctx_by_token(
        &self,
        token: &str,
    ) -> Result<Option<crate::auth::IdentityCtx>, String> {
        let hash = crate::util::sha256_hex(token.as_bytes());
        let row = self.conn.query_row(sql::IDENTITY_BY_TOKEN, [hash], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        });
        match row {
            Ok((_id, name, perms_json)) => {
                let perms = parse_stored_permissions(&perms_json)?;
                Ok(Some(crate::auth::IdentityCtx::new(&name, perms)))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// 管理视图列表。token 只存哈希，这里只给尾缀提示（供辨认，不可复原）。
    pub fn identity_list(&self) -> Result<Vec<Value>, String> {
        let mut st = self
            .conn
            .prepare(sql::IDENTITY_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.map(|row| {
            let (name, token, perms_json, created_at) = row.map_err(|e| e.to_string())?;
            let perms = parse_stored_permissions(&perms_json)?;
            Ok(json!({
                "name": name,
                "token_hint": token,
                "permissions": perms.to_json(),
                "created_at": created_at,
            }))
        })
        .collect()
    }

    /// 单个身份的管理视图（含 token）。
    pub fn identity_view(&self, name: &str) -> Result<Option<Value>, String> {
        self.identity_list()
            .map(|all| all.into_iter().find(|v| v["name"].as_str() == Some(name)))
    }

    pub fn identity_set_permissions(
        &self,
        name: &str,
        permissions: &crate::auth::Permissions,
    ) -> Result<bool, String> {
        let stored = permissions.to_stored_string();
        let n = self
            .conn
            .execute(sql::IDENTITY_UPDATE_PERMISSIONS, params![stored, name])
            .map_err(|e| e.to_string())?;
        Ok(n > 0)
    }

    /// 重置 token（旧 token 立即失效），返回新 token 明文（仅此一次）；
    /// 身份不存在返回 None。
    pub fn identity_reset_token(&self, name: &str) -> Result<Option<String>, String> {
        let token = self.generate_token()?;
        let (hash, hint) = token_hash_and_hint(&token);
        let n = self
            .conn
            .execute(sql::IDENTITY_UPDATE_TOKEN, params![hash, hint, name])
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(None);
        }
        Ok(Some(token))
    }

    pub fn identity_delete(&self, name: &str) -> Result<bool, String> {
        let n = self
            .conn
            .execute(sql::IDENTITY_DELETE, [name])
            .map_err(|e| e.to_string())?;
        Ok(n > 0)
    }
}

/// token → (存储哈希, 尾缀提示)。token 为 64 位 hex，提示取末 4 字符。
fn token_hash_and_hint(token: &str) -> (String, String) {
    let hint: String = token
        .chars()
        .skip(token.chars().count().saturating_sub(4))
        .collect();
    (crate::util::sha256_hex(token.as_bytes()), hint)
}

/// 解析 identities.permissions 列。写入路径已严格校验，这里再防线一次：
/// 损坏的行会让读取报错（身份解析 fail-closed），而不是静默放大权限。
fn parse_stored_permissions(json_text: &str) -> Result<crate::auth::Permissions, String> {
    let v: Value =
        serde_json::from_str(json_text).map_err(|e| format!("corrupt permissions JSON: {e}"))?;
    crate::auth::Permissions::from_json(&v)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::{cleanup, temp_db};
    use super::*;

    #[test]
    fn identity_lifecycle_token_lookup_and_reset() {
        let path = temp_db("identity");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        assert!(st.identity_list().unwrap().is_empty());

        let (token, view) = st
            .identity_create("alice", &crate::auth::Permissions::all())
            .unwrap();
        assert_eq!(token.len(), 64, "token = hex(randomblob(32))");
        assert_eq!(view["name"], "alice");
        assert_eq!(view["permissions"]["admin"], true);
        // 库内无明文：视图只带尾缀提示，且哈希不可逆推出 token
        assert_eq!(view["token_hint"], &token[token.len() - 4..]);
        assert!(serde_json::to_string(&view).unwrap().find(&token).is_none());
        // 重名报错
        assert!(st
            .identity_create("alice", &crate::auth::Permissions::default())
            .is_err());

        // token → 身份上下文
        let ctx = st.identity_ctx_by_token(&token).unwrap().unwrap();
        assert_eq!(ctx.name, "alice");
        assert!(ctx.can(crate::auth::Cap::Admin));
        assert!(!ctx.open_mode);
        // 未知 token → None
        assert!(st.identity_ctx_by_token("nope").unwrap().is_none());

        // 收窄权限后，同一 token 的能力同步收窄
        st.identity_set_permissions("alice", &crate::auth::Permissions::default())
            .unwrap();
        let ctx = st.identity_ctx_by_token(&token).unwrap().unwrap();
        assert!(!ctx.can(crate::auth::Cap::Read));

        // 重置 token：旧 token 立即失效
        let new_token = st.identity_reset_token("alice").unwrap().unwrap();
        assert_ne!(new_token, token);
        assert!(st.identity_ctx_by_token(&token).unwrap().is_none());
        assert!(st.identity_ctx_by_token(&new_token).unwrap().is_some());
        // 不存在的身份重置 → None
        assert!(st.identity_reset_token("nope").unwrap().is_none());

        // 删除后计数归零；再删返回 false
        assert!(st.identity_delete("alice").unwrap());
        assert!(!st.identity_delete("alice").unwrap());
        assert!(st.identity_list().unwrap().is_empty());
        cleanup(&path);
    }
}
