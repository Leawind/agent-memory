//! Content lifecycle is independent of relevance and access reinforcement.

use serde::{Deserialize, Serialize};

pub const MAX_TIMESTAMP: u64 = 253_402_300_799;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Metadata {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecayRule<P = String> {
    pub predicate: P,
    pub half_life_days: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(
    default,
    deny_unknown_fields,
    bound(deserialize = "P: Deserialize<'de>")
)]
pub struct Policy<P = String> {
    pub rules: Vec<DecayRule<P>>,
    pub default_half_life_days: Option<f64>,
    pub freshness_weight: f64,
    pub reinforcement_weight: f64,
}

impl<P> Default for Policy<P> {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            default_half_life_days: None,
            freshness_weight: 0.2,
            reinforcement_weight: 0.1,
        }
    }
}

impl Policy {
    pub fn parse(value: &serde_json::Value) -> Result<Self, String> {
        let policy: Self = serde_json::from_value(value.clone())
            .map_err(|e| format!("invalid lifecycle_policy: {e}"))?;
        if policy.rules.len() > 128 {
            return Err("at most 128 decay rules are allowed".into());
        }
        if policy
            .rules
            .iter()
            .map(|r| r.half_life_days)
            .chain([policy.default_half_life_days])
            .flatten()
            .any(|days| !days.is_finite() || !(1.0..=36500.0).contains(&days))
        {
            return Err("half-life must be null (no decay) or 1..36500 days".into());
        }
        if [policy.freshness_weight, policy.reinforcement_weight]
            .into_iter()
            .any(|weight| !weight.is_finite() || !(0.0..=5.0).contains(&weight))
        {
            return Err("lifecycle RRF weights must be between 0 and 5".into());
        }
        for rule in &policy.rules {
            crate::tag_expr::parse_rule(&rule.predicate)?;
        }
        Ok(policy)
    }
}

impl<P> Policy<P> {
    pub fn try_map<B>(
        &self,
        bind: &mut impl FnMut(&P) -> Result<B, String>,
    ) -> Result<Policy<B>, String> {
        Ok(Policy {
            rules: self
                .rules
                .iter()
                .map(|r| {
                    Ok(DecayRule {
                        predicate: bind(&r.predicate)?,
                        half_life_days: r.half_life_days,
                    })
                })
                .collect::<Result<_, String>>()?,
            default_half_life_days: self.default_half_life_days,
            freshness_weight: self.freshness_weight,
            reinforcement_weight: self.reinforcement_weight,
        })
    }
}

impl Policy<crate::tag_expr::TagExpr> {
    pub fn freshness(&self, meta: &Metadata, tags: &[String], updated_at: u64, now: u64) -> f64 {
        if meta.pinned {
            return 1.0;
        }
        let half_life = self
            .rules
            .iter()
            .find(|r| r.predicate.eval(tags))
            .map_or(self.default_half_life_days, |r| r.half_life_days);
        half_life.map_or(1.0, |days| {
            (-std::f64::consts::LN_2 * now.saturating_sub(updated_at) as f64 / (days * 86400.0))
                .exp()
        })
    }
}

/// Soft priors rank only the relevance candidate union; they cannot recall memories.
pub fn rank_channels(
    memories: &[crate::model::Memory],
    eligible: &std::collections::HashSet<usize>,
    lifecycle: &std::collections::HashMap<i64, Metadata>,
    usage: &std::collections::HashMap<i64, crate::access::Projection>,
    policy: &Policy<crate::tag_expr::TagExpr>,
    now: u64,
) -> [Vec<crate::search::Hit>; 2] {
    let default_meta = Metadata::default();
    let mut channels = [Vec::new(), Vec::new()];
    for idx in eligible {
        let memory = &memories[*idx];
        let id = crate::store::Store::parse_id(&memory.id).unwrap_or(0);
        let freshness = policy.freshness(
            lifecycle.get(&id).unwrap_or(&default_meta),
            &memory.tags,
            memory.updated_at,
            now,
        );
        for (channel, score) in [
            freshness,
            usage.get(&id).map_or(0.0, |state| state.value(now)),
        ]
        .into_iter()
        .enumerate()
        {
            if score > 0.0 {
                channels[channel].push(crate::search::Hit {
                    idx: *idx,
                    score: score.to_bits() as i64,
                    snippet: String::new(),
                });
            }
        }
    }
    for channel in &mut channels {
        channel.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.idx.cmp(&b.idx)));
    }
    channels
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

    #[test]
    fn ordered_predicate_rules_support_overrides_fallback_and_pins() {
        let policy = Policy::parse(&serde_json::json!({"rules":[{"predicate":"durable", "half_life_days":null},{"predicate":"temporary|event", "half_life_days":7}], "default_half_life_days":30})).unwrap();
        let policy = policy
            .try_map(&mut |raw| crate::tag_expr::parse_rule(raw))
            .unwrap();
        let mut meta = Metadata::default();
        assert_eq!(
            policy.freshness(
                &meta,
                &["durable".into(), "temporary".into()],
                0,
                30 * 86400
            ),
            1.0
        );
        assert!((policy.freshness(&meta, &["temporary".into()], 0, 7 * 86400) - 0.5).abs() < 1e-12);
        assert!((policy.freshness(&meta, &[], 0, 30 * 86400) - 0.5).abs() < 1e-12);
        meta.pinned = true;
        meta.expires_at = Some(100);
        assert_eq!(policy.freshness(&meta, &[], 0, 1000), 1.0);
        assert!(!meta.matches("active", 1000));
        for invalid in [
            serde_json::json!({"default_half_life_days":0}),
            serde_json::json!({"rules":[{"predicate":"a","half_life_days":-1}]}),
            serde_json::json!({"freshness_weight":-1}),
            serde_json::json!({"typo":1}),
        ] {
            assert!(Policy::parse(&invalid).is_err());
        }
    }
}
