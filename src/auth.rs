//! 身份与能力模型：token 鉴权的数据层（无账号——token 即身份，无注册/登录）。
//!
//! [`Cap`] 是权限的唯一登记表：`identities.permissions` 列存 JSON（键 = 能力
//! 名），写入与读取都经过本模块的严格校验，未知键拒绝。工具对能力的要求见
//! `tools::defs::required_cap`，执行点在 `tools::execute` 统一把守；REST 侧
//! 的非工具端点（身份/设置/运维）在 `api` 层各自要求能力。
//!
//! 开放模式（库中无任何身份）下请求免鉴权，[`IdentityCtx::open_mode`] 视为
//! 全能力——这也是"启用鉴权"的入口：开放模式下即可创建第一个身份。鉴权开启
//! 后，无 token 请求按服务器设置 `anonymous_permissions` 解析为匿名身份
//! （[`IdentityCtx::anonymous`]，能力可配置；未配置则拒绝）。

use crate::tools::ToolError;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

/// 能力集合；`as_str` 同时是 permissions JSON 的键名。
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

/// 一份身份的能力集合。JSON 形态恒为全键对象（如 {"read":true,"create":false,…}），
/// 便于 UI 渲染成复选框、也避免"缺键"与"显式 false"的歧义。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Permissions {
    caps: BTreeSet<Cap>,
}

impl Permissions {
    /// 从 JSON 解析：必须是对象、键必须都在登记表内、值必须是布尔。
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

    /// UI 复选框 / initialize 摘要用的全键对象。
    pub fn to_json(&self) -> Value {
        let mut obj = Map::new();
        for cap in Cap::ALL {
            obj.insert(cap.as_str().to_string(), json!(self.caps.contains(&cap)));
        }
        Value::Object(obj)
    }

    /// 入库形态（serde_json 默认按键排序，序列化稳定）。
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

    /// 是否一个能力都没有（匿名能力集为空 = 匿名访问被整体拒绝）。
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

/// 请求的鉴权形态：开放模式（免鉴权、全能力）、匿名身份（配置的能力集）、
/// token 身份（ identities 表中登记的身份）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Open,
    Anonymous,
    Token,
}

/// 请求级身份上下文：HTTP 层解析 Bearer token / 匿名设置后构造，贯穿工具与 API 处理器。
#[derive(Debug, Clone)]
pub struct IdentityCtx {
    pub name: String,
    pub permissions: Permissions,
    pub mode: Mode,
}

impl IdentityCtx {
    /// 开放模式上下文：视为全能力（开放模式本身就信任全部访问者，
    /// 由此 UI 才能在开放模式下创建第一个身份以启用鉴权）。
    pub fn open_mode() -> Self {
        IdentityCtx {
            name: "local".into(),
            permissions: Permissions::all(),
            mode: Mode::Open,
        }
    }

    /// 匿名身份上下文：鉴权开启后无 token 请求按 settings 的
    /// `anonymous_permissions` 解析而来，能力集由操作者配置。
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

    /// initialize 回传给 agent 的自我认知摘要。
    pub fn summary(&self) -> Value {
        json!({
            "name": self.name,
            "mode": self.mode.as_str(),
            "permissions": self.permissions.to_json(),
        })
    }

    /// 附加到 initialize instructions 的人类可读一行。
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
        // 未知键拒绝（打错能力名不允许静默生效）
        let err = Permissions::from_json(&json!({"read": true, "riter": false})).unwrap_err();
        assert!(err.contains("unknown permission 'riter'"), "got: {err}");
        // 非布尔值拒绝
        let err = Permissions::from_json(&json!({"read": "yes"})).unwrap_err();
        assert!(err.contains("must be a boolean"), "got: {err}");
        // 非对象拒绝
        assert!(Permissions::from_json(&json!(["read"])).is_err());
        // 空对象 = 全无能力
        let none = Permissions::from_json(&json!({})).unwrap();
        for cap in Cap::ALL {
            assert!(!none.has(cap));
        }
        // 往返：to_json 输出恒为全键对象
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
        // 开放模式全能力
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

        // whoami 摘要带匿名 mode 与全键能力对象
        let summary = anon.summary();
        assert_eq!(summary["name"], "anonymous");
        assert_eq!(summary["mode"], "anonymous");
        assert_eq!(summary["permissions"]["read"], true);
        assert_eq!(summary["permissions"]["admin"], false);

        // initialize 身份行
        let line = anon.describe_line();
        assert!(line.contains("anonymous"), "got: {line}");
        assert!(line.contains("read"), "got: {line}");

        // 空能力集可被 is_empty 识别（HTTP 层据此整体拒绝匿名）；
        // 显式 false 同样不授予能力
        assert!(Permissions::default().is_empty());
        assert!(Permissions::from_json(&json!({ "read": false }))
            .map(|p| p.is_empty())
            .unwrap());
        // 能力全无时 describe_line 显示 (none)
        let none = IdentityCtx::anonymous(Permissions::default());
        assert!(none.describe_line().contains("(none)"));
    }
}
