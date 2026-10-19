//! Content lifecycle is independent of relevance and access reinforcement.

use crate::tag_expr::{TagAtom, TagExpr};
use serde::{Deserialize, Serialize};

pub const MAX_TIMESTAMP: u64 = 253_402_300_799;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct Metadata {
    pub expires_at: Option<u64>,
    pub pinned: bool,
}

impl Metadata {
    pub fn validate(&self) -> Result<(), String> {
        if self.expires_at.is_some_and(|t| t > MAX_TIMESTAMP) {
            return Err(format!(
                "lifecycle timestamps must be epoch seconds <= {MAX_TIMESTAMP}"
            ));
        }
        Ok(())
    }

    pub fn state(&self, now: u64) -> &'static str {
        if self.expires_at.is_some_and(|deadline| deadline <= now) {
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

/// One decay override: memories whose tags satisfy `predicate` decay with `half_life_days`. A
/// predicate is a tag-set boolean expression — bare tag names, `/regex/` and `@named` references
/// (expanded against the global namespace); a leaf matches a memory's full tag names and the key
/// halves of valued tags. Rules apply in order and the first match wins, so listing order is
/// part of the policy.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecayRule {
    pub predicate: String,
    pub half_life_days: Option<f64>,
}

/// Policy rules compiled for evaluation: each rule's predicate parsed with named references
/// expanded; None = the rule cannot resolve (deleted tag/predicate) and never matches — decay
/// is a soft ranking signal, so it degrades instead of failing the search.
pub type DecayPlan = Vec<(Option<TagExpr>, Option<f64>)>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Policy {
    pub rules: Vec<DecayRule>,
    pub default_half_life_days: Option<f64>,
    pub freshness_weight: f64,
    pub reinforcement_weight: f64,
}

impl Default for Policy {
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
            let expr = crate::tag_expr::parse_rule(&rule.predicate)
                .map_err(|e| format!("invalid decay rule {:?}: {e}", rule.predicate))?;
            let mut keyed = false;
            expr.visit_atoms(&mut |atom| keyed |= matches!(atom, TagAtom::KeyEq { .. }));
            if keyed {
                return Err(format!(
                    "invalid decay rule {:?}: key=value comparisons are query-only; use tag names, /regex/ or @named references",
                    rule.predicate
                ));
            }
        }
        Ok(policy)
    }

    pub fn freshness(
        &self,
        plan: &DecayPlan,
        meta: &Metadata,
        tags: &[String],
        updated_at: u64,
        now: u64,
    ) -> f64 {
        if meta.pinned {
            return 1.0;
        }
        let half_life = plan
            .iter()
            .find_map(|(expr, days)| expr.as_ref().filter(|e| e.eval(tags)).map(|_| *days))
            .unwrap_or(self.default_half_life_days);
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
    policy: &Policy,
    plan: &DecayPlan,
    now: u64,
) -> [Vec<crate::search::Hit>; 2] {
    let default_meta = Metadata::default();
    let mut channels = [Vec::new(), Vec::new()];
    for idx in eligible {
        let memory = &memories[*idx];
        let id = crate::store::Store::parse_id(&memory.id).unwrap_or(0);
        // Rule leaves match full tag names and the key halves of valued tags alike
        let mut names: Vec<String> = memory.tags.clone();
        for tag in &memory.tags {
            if let Some((key, _)) = tag.split_once(':') {
                names.push(key.to_string());
            }
        }
        let freshness = policy.freshness(
            plan,
            lifecycle.get(&id).unwrap_or(&default_meta),
            &names,
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
    fn expiry_is_hard_even_for_pins() {
        let mut data = Metadata {
            expires_at: Some(100),
            pinned: true,
        };
        assert_eq!(data.state(99), "active");
        assert_eq!(data.state(100), "expired");
        data.expires_at = None;
        assert_eq!(data.state(100), "active");
    }

    #[test]
    fn ordered_predicate_rules_support_overrides_fallback_and_pins() {
        let policy = Policy::parse(&serde_json::json!({"rules":[{"predicate":"durable", "half_life_days":null},{"predicate":"temp", "half_life_days":7}], "default_half_life_days":30})).unwrap();
        let rule = |raw: &str| Some(crate::tag_expr::parse(raw).unwrap());
        let plan: DecayPlan = vec![(rule("durable"), None), (rule("temp"), Some(7.0))];
        let mut meta = Metadata::default();
        // A leaf matches bare tags and the key halves of valued tags alike
        let durable = vec!["durable".to_string()];
        let keyed = vec!["temp:new".to_string(), "temp".to_string()];
        assert_eq!(policy.freshness(&plan, &meta, &durable, 0, 30 * 86400), 1.0);
        assert!((policy.freshness(&plan, &meta, &keyed, 0, 7 * 86400) - 0.5).abs() < 1e-12);
        assert!((policy.freshness(&plan, &meta, &[], 0, 30 * 86400) - 0.5).abs() < 1e-12);
        // Dead rules (unresolvable references) never match; first live match wins
        let dead_first: DecayPlan = vec![(None, Some(3.0)), (rule("temp"), Some(7.0))];
        assert!((policy.freshness(&dead_first, &meta, &keyed, 0, 7 * 86400) - 0.5).abs() < 1e-12);
        meta.pinned = true;
        meta.expires_at = Some(100);
        assert_eq!(policy.freshness(&plan, &meta, &[], 0, 1000), 1.0);
        assert!(!meta.matches("active", 1000));
        for invalid in [
            serde_json::json!({"default_half_life_days":0}),
            serde_json::json!({"rules":[{"predicate":"temp","half_life_days":-1}]}),
            serde_json::json!({"freshness_weight":-1}),
            serde_json::json!({"rules":[{"predicate":"","half_life_days":7}]}),
            serde_json::json!({"rules":[{"predicate":"a=b","half_life_days":7}]}),
            serde_json::json!({"typo":1}),
        ] {
            assert!(Policy::parse(&invalid).is_err());
        }
    }
}
