//! Identity data operations: token generation and hashing, identity CRUD, token → request identity.
//! Token plaintext appears only once, in the create/reset return value; the database stores just a hash and a suffix hint.

use crate::sql;
use rusqlite::params;
use serde_json::{json, Value};

use super::{is_unique_violation, Store};

impl Store {
    /// Generate a random token: `sk_` prefix + 62 lowercase hex characters (65 characters in total).
    /// All randomness comes from SQLite randomblob (its PRNG is seeded from system entropy);
    /// SQLite's hex() outputs uppercase, so lower() normalizes to lowercase.
    pub fn generate_token(&self) -> Result<String, String> {
        self.conn
            .query_row(sql::TOKEN_GENERATE, [], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())
    }

    /// Whether any identity holds the admin capability (the permissions JSON is always a full-key compact object,
    /// so LIKE '%"admin":true%' matches precisely). Precondition for enabling the auth switch.
    pub fn has_admin_identity(&self) -> Result<bool, String> {
        self.conn
            .query_row(sql::IDENTITY_ANY_ADMIN, [], |r| r.get::<_, i64>(0))
            .map(|n| n != 0)
            .map_err(|e| e.to_string())
    }

    /// Create a new identity: generate a random token and return (token, admin view). Duplicate names surface via the unique constraint.
    /// Token plaintext appears only once, in this return value; the database stores just a hash and a suffix hint.
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

    /// Bearer token → request identity; unknown tokens return None (the caller decides on 401).
    /// The input is hashed and compared against the stored hash; no plaintext exists in the database.
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

    /// List of admin views. Tokens are stored only as hashes, so this offers just the suffix hint (for recognition, not recovery).
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

    /// Admin view of a single identity (including the token).
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

    /// Reset the token (the old one is invalidated immediately) and return the new token plaintext (this one time only);
    /// returns None when the identity does not exist.
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

/// Token → (stored hash, suffix hint). A token looks like `sk_<62 lowercase hex>`; the hint is its last 4 characters.
fn token_hash_and_hint(token: &str) -> (String, String) {
    let hint: String = token
        .chars()
        .skip(token.chars().count().saturating_sub(4))
        .collect();
    (crate::util::sha256_hex(token.as_bytes()), hint)
}

/// Parse the identities.permissions column. The write path already validates strictly; this is one more line of defense:
/// corrupt rows make reads error out (identity resolution is fail-closed) instead of silently widening permissions.
fn parse_stored_permissions(json_text: &str) -> Result<crate::auth::Permissions, String> {
    let v: Value =
        serde_json::from_str(json_text).map_err(|e| format!("corrupt permissions JSON: {e}"))?;
    crate::auth::Permissions::from_json(&v)
}

/// Token contract: `sk_` prefix + 62 lowercase hex characters (65 characters in total).
#[cfg(test)]
fn assert_token_format(token: &str) {
    assert!(
        token.starts_with("sk_"),
        "token must start with sk_: {token}"
    );
    let hex_part = &token["sk_".len()..];
    assert_eq!(hex_part.len(), 62, "token = {token}");
    assert!(
        hex_part
            .chars()
            .all(|c| c.is_ascii_digit() || matches!(c, 'a'..='f')),
        "token must be lowercase hex after the prefix: {token}"
    );
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
        assert_token_format(&token);
        assert_eq!(view["name"], "alice");
        assert_eq!(view["permissions"]["admin"], true);
        // No plaintext in the database: the view carries only the suffix hint, and the hash cannot be reversed into the token
        assert_eq!(view["token_hint"], &token[token.len() - 4..]);
        assert!(serde_json::to_string(&view).unwrap().find(&token).is_none());
        // Duplicate names error out
        assert!(st
            .identity_create("alice", &crate::auth::Permissions::default())
            .is_err());

        // token → identity context
        let ctx = st.identity_ctx_by_token(&token).unwrap().unwrap();
        assert_eq!(ctx.name, "alice");
        assert!(ctx.can(crate::auth::Cap::Admin));
        assert_eq!(ctx.mode, crate::auth::Mode::Token);
        // Unknown token → None
        assert!(st.identity_ctx_by_token("nope").unwrap().is_none());

        // After narrowing permissions, the same token's capabilities narrow in step
        st.identity_set_permissions("alice", &crate::auth::Permissions::default())
            .unwrap();
        let ctx = st.identity_ctx_by_token(&token).unwrap().unwrap();
        assert!(!ctx.can(crate::auth::Cap::Read));

        // Reset token: the old one is invalidated immediately
        let new_token = st.identity_reset_token("alice").unwrap().unwrap();
        assert_token_format(&new_token);
        assert_ne!(new_token, token);
        assert!(st.identity_ctx_by_token(&token).unwrap().is_none());
        assert!(st.identity_ctx_by_token(&new_token).unwrap().is_some());
        // Resetting a nonexistent identity → None
        assert!(st.identity_reset_token("nope").unwrap().is_none());

        // After deletion the count drops to zero; deleting again returns false
        assert!(st.identity_delete("alice").unwrap());
        assert!(!st.identity_delete("alice").unwrap());
        assert!(st.identity_list().unwrap().is_empty());
        cleanup(&path);
    }

    /// Token format contract: repeated samples all satisfy the sk_ prefix + 62 lowercase hex, and are all distinct.
    #[test]
    fn generated_tokens_are_sk_prefixed_lowercase_hex() {
        let path = temp_db("token-format");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        let mut drawn = std::collections::HashSet::new();
        for _ in 0..8 {
            let token = st.generate_token().unwrap();
            assert_token_format(&token);
            drawn.insert(token);
        }
        assert_eq!(drawn.len(), 8, "randomblob draws must not repeat");
        cleanup(&path);
    }
}
