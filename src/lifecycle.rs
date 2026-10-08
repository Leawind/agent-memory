//! Content lifecycle is independent of relevance and access reinforcement.

use serde::{Deserialize, Serialize};

pub const MAX_TIMESTAMP: u64 = 253_402_300_799;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    #[default]
    Fact,
    Preference,
    Procedure,
    Context,
    Event,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Metadata {
    pub kind: Kind,
    pub expires_at: Option<u64>,
    pub archived_at: Option<u64>,
    pub pinned: bool,
}

impl Metadata {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .expires_at
            .into_iter()
            .chain(self.archived_at)
            .any(|t| t > MAX_TIMESTAMP)
        {
            return Err(format!(
                "lifecycle timestamps must be epoch seconds <= {MAX_TIMESTAMP}"
            ));
        }
        Ok(())
    }

    pub fn state(&self, now: u64) -> &'static str {
        if self.archived_at.is_some() {
            "archived"
        } else if self.expires_at.is_some_and(|deadline| deadline <= now) {
            "expired"
        } else {
            "active"
        }
    }

    pub fn matches(&self, state: &str, now: u64) -> bool {
        state == "all" || self.state(now) == state
    }

    pub fn view(&self, now: u64) -> serde_json::Value {
        let mut value = serde_json::to_value(self).expect("lifecycle serializes");
        value["state"] = serde_json::json!(self.state(now));
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expiry_is_hard_even_for_pins_and_archive_is_reversible() {
        let mut data = Metadata {
            expires_at: Some(100),
            pinned: true,
            ..Default::default()
        };
        assert_eq!(data.state(99), "active");
        assert_eq!(data.state(100), "expired");
        data.archived_at = Some(50);
        assert_eq!(data.state(100), "archived");
        data.archived_at = None;
        assert_eq!(data.state(100), "expired");
        data.expires_at = None;
        assert_eq!(data.state(100), "active");
    }
}
