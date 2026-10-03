//! Identity and capability model: the data layer behind token auth (no accounts — the token is the identity; no registration/login).
//!
//! [`Cap`] is the single registry of capabilities: the `identities.permissions` column stores JSON (keys = capability
//! names), and both writes and reads pass through this module's strict validation, which rejects unknown keys. Tool capability
//! requirements live in `tools::defs::required_cap` and are enforced centrally at `tools::execute`; on the REST side,
//! the non-tool endpoints (identities/settings/ops) each require capabilities at the `api` layer.
//!
//! In open mode (no identities in the database) requests skip auth, and [`IdentityCtx::open_mode`] counts as
//! fully capable — which is also the gateway for enabling auth: the first identity is created while still in open mode. Once auth is
//! enabled, tokenless requests resolve to the anonymous identity according to the server setting `anonymous_permissions`
//! ([`IdentityCtx::anonymous`], capabilities configurable; rejected when unconfigured).

use crate::tools::ToolError;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

/// Capability set; `as_str` doubles as the permissions JSON key name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cap {
    Read,
    Create,
    Update,
    Delete,
    TagManage,
    Admin,
}

impl Cap {
    pub const ALL: [Cap; 6] = [
        Cap::Read,
        Cap::Create,
        Cap::Update,
        Cap::Delete,
        Cap::TagManage,
        Cap::Admin,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Cap::Read => "read",
            Cap::Create => "create",
            Cap::Update => "update",
            Cap::Delete => "delete",
            Cap::TagManage => "tag_manage",
            Cap::Admin => "admin",
        }
    }

    pub fn parse(s: &str) -> Option<Cap> {
        Cap::ALL.into_iter().find(|c| c.as_str() == s)
    }
}

/// An identity's capability set. The JSON form is always a full-key object (e.g. {"read":true,"create":false,…}),
/// which makes checkbox rendering easy in the UI and removes the ambiguity between a missing key and an explicit false.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Permissions {
    caps: BTreeSet<Cap>,
}

impl Permissions {
    /// Parse from JSON: must be an object, every key must be in the registry, and every value must be a boolean.
    pub fn from_json(v: &Value) -> Result<Permissions, String> {
        let obj = v
            .as_object()
            .ok_or_else(|| format!("permissions must be a JSON object, got: {v}"))?;
        let mut caps = BTreeSet::new();
        for (k, val) in obj {
            let cap = Cap::parse(k)
                .ok_or_else(|| format!("unknown permission '{k}' (valid: {})", valid_cap_list()))?;
            let b = val
                .as_bool()
                .ok_or_else(|| format!("permission '{k}' must be a boolean"))?;
            if b {
                caps.insert(cap);
            }
        }
        Ok(Permissions { caps })
    }

    pub fn all() -> Permissions {
        Permissions {
            caps: Cap::ALL.into_iter().collect(),
        }
    }

    pub fn has(&self, cap: Cap) -> bool {
        self.caps.contains(&cap)
    }

    /// Full-key object used by UI checkboxes / the initialize summary.
    pub fn to_json(&self) -> Value {
        let mut obj = Map::new();
        for cap in Cap::ALL {
            obj.insert(cap.as_str().to_string(), json!(self.caps.contains(&cap)));
        }
        Value::Object(obj)
    }

    /// Storage form (serde_json sorts keys by default, so serialization is stable).
    pub fn to_stored_string(&self) -> String {
        serde_json::to_string(&self.to_json()).unwrap_or_default()
    }

    pub fn names(&self) -> Vec<&'static str> {
        Cap::ALL
            .into_iter()
            .filter(|c| self.caps.contains(c))
            .map(|c| c.as_str())
            .collect()
    }

    /// Whether there is not a single capability (an empty anonymous capability set = anonymous access is rejected wholesale).
    pub fn is_empty(&self) -> bool {
        self.caps.is_empty()
    }
}

fn valid_cap_list() -> String {
    Cap::ALL
        .iter()
        .map(|c| c.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// A request's auth shape: open mode (no auth, full capabilities), anonymous identity (configured capability set),
/// or token identity (an identity registered in the identities table).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Open,
    Anonymous,
    Token,
}

/// Request-scoped identity context: built by the HTTP layer after resolving the Bearer token / anonymous settings, and threaded through tool and API handlers.
#[derive(Debug, Clone)]
pub struct IdentityCtx {
    pub name: String,
    pub permissions: Permissions,
    pub mode: Mode,
}

impl IdentityCtx {
    /// Open-mode context: counts as fully capable (open mode inherently trusts all callers,
    /// which is how the UI can create the first identity and enable auth).
    pub fn open_mode() -> Self {
        IdentityCtx {
            name: "local".into(),
            permissions: Permissions::all(),
            mode: Mode::Open,
        }
    }

    /// Anonymous identity context: what tokenless requests resolve to once auth is enabled,
    /// based on the settings' `anonymous_permissions`; the capability set is configured by the operator.
    pub fn anonymous(permissions: Permissions) -> Self {
        IdentityCtx {
            name: "anonymous".into(),
            permissions,
            mode: Mode::Anonymous,
        }
    }

    pub fn new(name: &str, permissions: Permissions) -> Self {
        IdentityCtx {
            name: name.to_string(),
            permissions,
            mode: Mode::Token,
        }
    }

    pub fn can(&self, cap: Cap) -> bool {
        self.permissions.has(cap)
    }

    pub fn require(&self, cap: Cap) -> Result<(), ToolError> {
        if self.can(cap) {
            Ok(())
        } else {
            Err(ToolError::forbidden(format!(
                "identity '{}' lacks the '{}' permission (ask the server operator)",
                self.name,
                cap.as_str()
            )))
        }
    }

    /// Every listed capability must be present; the error names all missing ones at once so the
    /// caller does not have to retry one permission at a time.
    pub fn require_all(&self, caps: &[Cap]) -> Result<(), ToolError> {
        let missing: Vec<&str> = caps
            .iter()
            .filter(|c| !self.can(**c))
            .map(|c| c.as_str())
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        Err(ToolError::forbidden(format!(
            "identity '{}' lacks the '{}' permission{} (ask the server operator)",
            self.name,
            missing.join("', '"),
            if missing.len() > 1 { "s" } else { "" },
        )))
    }

    /// The self-description summary returned to the agent in initialize.
    pub fn summary(&self) -> Value {
        json!({
            "name": self.name,
            "mode": self.mode.as_str(),
            "permissions": self.permissions.to_json(),
        })
    }

    /// A human-readable line appended to the initialize instructions.
    pub fn describe_line(&self) -> String {
        let perms = self.permissions.names();
        let list = if perms.is_empty() {
            "(none)".to_string()
        } else {
            perms.join(", ")
        };
        match self.mode {
            Mode::Open => {
                "Access mode: open (no identities configured); all permissions granted.".to_string()
            }
            Mode::Anonymous => format!("Access mode: anonymous (no token); permissions: {list}."),
            Mode::Token => format!("Caller identity: {}; permissions: {list}.", self.name),
        }
    }
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Open => "open",
            Mode::Anonymous => "anonymous",
            Mode::Token => "token",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cap_roundtrip_and_reject_unknown() {
        for cap in Cap::ALL {
            assert_eq!(Cap::parse(cap.as_str()), Some(cap));
        }
        assert_eq!(Cap::parse("nope"), None);
        assert_eq!(Cap::parse(""), None);
    }

    #[test]
    fn permissions_json_is_strict() {
        // Unknown keys rejected (a mistyped capability name must not silently take effect)
        let err = Permissions::from_json(&json!({"read": true, "riter": false})).unwrap_err();
        assert!(err.contains("unknown permission 'riter'"), "got: {err}");
        // Non-boolean values rejected
        let err = Permissions::from_json(&json!({"read": "yes"})).unwrap_err();
        assert!(err.contains("must be a boolean"), "got: {err}");
        // Non-objects rejected
        assert!(Permissions::from_json(&json!(["read"])).is_err());
        // Empty object = no capabilities at all
        let none = Permissions::from_json(&json!({})).unwrap();
        for cap in Cap::ALL {
            assert!(!none.has(cap));
        }
        // Round-trip: to_json output is always a full-key object
        let p = Permissions::from_json(&json!({ "read": true, "admin": true })).unwrap();
        let parsed = Permissions::from_json(&p.to_json()).unwrap();
        assert_eq!(p, parsed);
        assert_eq!(parsed.names(), vec!["read", "admin"]);
        assert_eq!(Permissions::all().names().len(), 6);
    }

    #[test]
    fn require_produces_forbidden_with_identity_name() {
        let perms = Permissions::from_json(&json!({ "read": true })).unwrap();
        let ctx = IdentityCtx::new("alice", perms);
        assert!(ctx.require(Cap::Read).is_ok());
        let err = ctx.require(Cap::Delete).unwrap_err();
        match err {
            ToolError::Forbidden(msg) => {
                assert!(
                    msg.contains("alice") && msg.contains("delete"),
                    "got: {msg}"
                );
            }
            other => panic!("expected Forbidden, got {other:?}"),
        }
        // Open mode = full capabilities
        let open = IdentityCtx::open_mode();
        for cap in Cap::ALL {
            assert!(open.can(cap));
        }
        assert_eq!(open.mode, Mode::Open);
    }

    #[test]
    fn anonymous_ctx_reports_mode_and_capabilities() {
        let perms = Permissions::from_json(&json!({ "read": true })).unwrap();
        let anon = IdentityCtx::anonymous(perms);
        assert_eq!(anon.name, "anonymous");
        assert_eq!(anon.mode, Mode::Anonymous);
        assert!(anon.can(Cap::Read));
        assert!(!anon.can(Cap::Create));

        // whoami summary carries the anonymous mode and the full-key capability object
        let summary = anon.summary();
        assert_eq!(summary["name"], "anonymous");
        assert_eq!(summary["mode"], "anonymous");
        assert_eq!(summary["permissions"]["read"], true);
        assert_eq!(summary["permissions"]["admin"], false);

        // initialize identity line
        let line = anon.describe_line();
        assert!(line.contains("anonymous"), "got: {line}");
        assert!(line.contains("read"), "got: {line}");

        // An empty capability set is detected by is_empty (the HTTP layer rejects anonymous access wholesale on that basis);
        // an explicit false grants no capability either
        assert!(Permissions::default().is_empty());
        assert!(Permissions::from_json(&json!({ "read": false }))
            .map(|p| p.is_empty())
            .unwrap());
        // describe_line shows (none) when there are no capabilities at all
        let none = IdentityCtx::anonymous(Permissions::default());
        assert!(none.describe_line().contains("(none)"));
    }
}
