//! Persistent constraints bind stable tag ids, while the public form uses current names.

use crate::tag_expr::{self, TagExpr};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleSpec {
    pub name: String,
    pub expression: String,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleSpecs {
    pub constraints: Vec<RuleSpec>,
    #[serde(default)]
    pub derivations: Vec<RuleSpec>,
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub constraints: Vec<Constraint>,
    #[serde(default)]
    pub derivations: Vec<Derivation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Derivation {
    pub name: String,
    pub left: Expr,
    pub right: Expr,
    pub bidirectional: bool,
}

pub struct Closure {
    pub effective: Vec<i64>,
    pub derived: BTreeMap<i64, Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Constraint {
    pub name: String,
    pub expression: Expr,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "op", content = "args")]
pub enum Expr {
    Tag(i64),
    Not(Box<Expr>),
    All(Box<Expr>, Box<Expr>),
    Any(Box<Expr>, Box<Expr>),
    Mutex(Vec<Expr>),
}

impl Expr {
    fn positive(&self) -> bool {
        match self {
            Self::Tag(_) => true,
            Self::All(a, b) | Self::Any(a, b) => a.positive() && b.positive(),
            _ => false,
        }
    }

    fn heads(&self) -> Result<Vec<i64>, String> {
        match self {
            Self::Tag(id) => Ok(vec![*id]),
            Self::All(a, b) => {
                let mut ids = a.heads()?;
                ids.extend(b.heads()?);
                ids.sort_unstable();
                ids.dedup();
                Ok(ids)
            }
            _ => Err("derivation conclusions must be tag names joined by &".into()),
        }
    }

    fn tag_ids(&self, ids: &mut HashSet<i64>) {
        match self {
            Self::Tag(id) => {
                ids.insert(*id);
            }
            Self::All(a, b) | Self::Any(a, b) => {
                a.tag_ids(ids);
                b.tag_ids(ids);
            }
            Self::Not(a) => a.tag_ids(ids),
            Self::Mutex(args) => {
                for a in args {
                    a.tag_ids(ids);
                }
            }
        }
    }
    fn bind(expr: TagExpr, tags: &HashMap<String, i64>) -> Result<Self, String> {
        Ok(match expr {
            TagExpr::Tag(name) => Self::Tag(*tags.get(&name).ok_or_else(|| {
                format!("unknown tag '{name}' in rule; create the tag first")
            })?),
            TagExpr::Regex(_) => return Err("persistent rules require literal tag names; regex atoms are only supported in queries".into()),
            TagExpr::Not(a) => Self::Not(Box::new(Self::bind(*a, tags)?)),
            TagExpr::All(a, b) => Self::All(Box::new(Self::bind(*a, tags)?), Box::new(Self::bind(*b, tags)?)),
            TagExpr::Any(a, b) => Self::Any(Box::new(Self::bind(*a, tags)?), Box::new(Self::bind(*b, tags)?)),
            TagExpr::Mutex(args) => Self::Mutex(args.into_iter().map(|a| Self::bind(a, tags)).collect::<Result<_, _>>()?),
        })
    }

    pub fn eval(&self, tags: &[i64]) -> bool {
        match self {
            Self::Tag(id) => tags.contains(id),
            Self::Not(a) => !a.eval(tags),
            Self::All(a, b) => a.eval(tags) && b.eval(tags),
            Self::Any(a, b) => a.eval(tags) || b.eval(tags),
            Self::Mutex(args) => args.iter().filter(|a| a.eval(tags)).take(2).count() <= 1,
        }
    }

    pub fn references(&self, id: i64) -> bool {
        match self {
            Self::Tag(tag) => *tag == id,
            Self::Not(a) => a.references(id),
            Self::All(a, b) | Self::Any(a, b) => a.references(id) || b.references(id),
            Self::Mutex(args) => args.iter().any(|a| a.references(id)),
        }
    }

    fn display(&self, tags: &HashMap<i64, String>) -> Result<String, String> {
        self.display_with_precedence(tags, 0)
    }

    fn display_with_precedence(
        &self,
        tags: &HashMap<i64, String>,
        parent: u8,
    ) -> Result<String, String> {
        let precedence = match self {
            Self::Any(..) => 1,
            Self::All(..) => 2,
            Self::Not(..) => 3,
            _ => 4,
        };
        let rendered = match self {
            Self::Tag(id) => tags
                .get(id)
                .ok_or_else(|| format!("rule references missing tag id {id}"))?
                .clone(),
            Self::Not(a) => format!("!{}", a.display_with_precedence(tags, precedence)?),
            Self::All(a, b) => format!(
                "{}&{}",
                a.display_with_precedence(tags, precedence)?,
                b.display_with_precedence(tags, precedence)?
            ),
            Self::Any(a, b) => format!(
                "{}|{}",
                a.display_with_precedence(tags, precedence)?,
                b.display_with_precedence(tags, precedence)?
            ),
            Self::Mutex(args) => format!(
                "mutex({})",
                args.iter()
                    .map(|a| a.display(tags))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(",")
            ),
        };
        Ok(if precedence < parent {
            format!("({rendered})")
        } else {
            rendered
        })
    }
}

impl Rules {
    pub fn compile(specs: RuleSpecs, tags: &HashMap<String, i64>) -> Result<Self, String> {
        if specs.constraints.len() > 128 || specs.derivations.len() > 128 {
            return Err("at most 128 constraints and 128 derivations are allowed".into());
        }
        let mut names = HashSet::new();
        let mut constraints = Vec::new();
        for spec in specs.constraints {
            let name = spec.name.trim().to_string();
            if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
                return Err("rule name must be 1-100 characters without control characters".into());
            }
            if !names.insert(name.clone()) {
                return Err(format!("duplicate rule name '{name}'"));
            }
            let expression = Expr::bind(tag_expr::parse_rule(&spec.expression)?, tags)
                .map_err(|e| format!("rule '{name}': {e}"))?;
            constraints.push(Constraint { name, expression });
        }
        let mut derivations = Vec::new();
        for spec in specs.derivations {
            let name = spec.name.trim().to_string();
            if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
                return Err("rule name must be 1-100 characters without control characters".into());
            }
            if !names.insert(name.clone()) {
                return Err(format!("duplicate rule name '{name}'"));
            }
            if spec.expression.chars().count() > 32768 {
                return Err("derivation exceeds 32768 characters".into());
            }
            let bidirectional = spec.expression.contains("<=>");
            let arrow = if bidirectional { "<=>" } else { "=>" };
            let (left, right) = spec
                .expression
                .split_once(arrow)
                .ok_or("derivation requires => or <=>")?;
            if right.contains("=>") {
                return Err("derivation must contain exactly one arrow".into());
            }
            let left = Expr::bind(tag_expr::parse_rule(left)?, tags)?;
            let right = Expr::bind(tag_expr::parse_rule(right)?, tags)?;
            if !left.positive() || !right.positive() {
                return Err(
                    "derivations must be positive: negation and mutex are not allowed".into(),
                );
            }
            let mut heads = right.heads()?;
            if bidirectional {
                heads.extend(left.heads()?);
            }
            if tags
                .get(crate::model::RESERVED_TAG)
                .is_some_and(|id| heads.contains(id))
            {
                return Err("the reserved convention tag cannot be derived".into());
            }
            derivations.push(Derivation {
                name,
                left,
                right,
                bidirectional,
            });
        }
        Ok(Self {
            constraints,
            derivations,
        })
    }

    pub fn specs(&self, tags: &HashMap<i64, String>) -> Result<RuleSpecs, String> {
        Ok(RuleSpecs {
            derivations: self
                .derivations
                .iter()
                .map(|rule| {
                    Ok(RuleSpec {
                        name: rule.name.clone(),
                        expression: format!(
                            "{} {} {}",
                            rule.left.display(tags)?,
                            if rule.bidirectional { "<=>" } else { "=>" },
                            rule.right.display(tags)?
                        ),
                    })
                })
                .collect::<Result<_, String>>()?,
            constraints: self
                .constraints
                .iter()
                .map(|rule| {
                    Ok(RuleSpec {
                        name: rule.name.clone(),
                        expression: rule.expression.display(tags)?,
                    })
                })
                .collect::<Result<_, String>>()?,
        })
    }

    pub fn violations<'a>(&'a self, tags: &[i64]) -> Vec<&'a str> {
        self.constraints
            .iter()
            .filter(|rule| !rule.expression.eval(tags))
            .map(|rule| rule.name.as_str())
            .collect()
    }

    /// Positive rules are monotone on a finite set of tag ids, including cycles. Starting
    /// from original tags on every recomputation prevents unsupported cycles surviving deletion.
    pub fn closure(&self, original: &[i64]) -> Result<Closure, String> {
        for rule in &self.derivations {
            if !rule.left.positive() || !rule.right.positive() {
                return Err("stored derivations must be positive".into());
            }
        }
        let mut effective = original.to_vec();
        effective.sort_unstable();
        effective.dedup();
        let mut derived: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        loop {
            let mut changed = false;
            for rule in &self.derivations {
                let directions = [(&rule.left, &rule.right), (&rule.right, &rule.left)];
                for (premise, conclusion) in
                    directions
                        .into_iter()
                        .take(if rule.bidirectional { 2 } else { 1 })
                {
                    if !premise.eval(&effective) {
                        continue;
                    }
                    for id in conclusion.heads()? {
                        let sources = derived.entry(id).or_default();
                        if !sources.contains(&rule.name) {
                            sources.push(rule.name.clone());
                        }
                        if !effective.contains(&id) {
                            effective.push(id);
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
            }
        }
        effective.sort_unstable();
        for sources in derived.values_mut() {
            sources.sort();
        }
        Ok(Closure { effective, derived })
    }

    /// Kahn's algorithm detects dependency cycles without recursion. Positive cycles are
    /// informational, because their least fixed point remains well-defined.
    pub fn has_positive_cycles(&self) -> Result<bool, String> {
        let mut edges: HashMap<i64, HashSet<i64>> = HashMap::new();
        for rule in &self.derivations {
            for (premise, conclusion) in [(&rule.left, &rule.right), (&rule.right, &rule.left)]
                .into_iter()
                .take(if rule.bidirectional { 2 } else { 1 })
            {
                let mut inputs = HashSet::new();
                premise.tag_ids(&mut inputs);
                for input in inputs {
                    edges.entry(input).or_default().extend(conclusion.heads()?);
                }
            }
        }
        let mut degree: HashMap<i64, usize> = HashMap::new();
        for (input, outputs) in &edges {
            degree.entry(*input).or_default();
            for output in outputs {
                *degree.entry(*output).or_default() += 1;
            }
        }
        let mut ready = degree
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        let mut removed = 0;
        while let Some(id) = ready.pop() {
            removed += 1;
            if let Some(outputs) = edges.get(&id) {
                for output in outputs {
                    let n = degree.get_mut(output).expect("edge target exists");
                    *n -= 1;
                    if *n == 0 {
                        ready.push(*output);
                    }
                }
            }
        }
        Ok(removed < degree.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn derivations(expressions: &[&str]) -> Rules {
        let tags = HashMap::from([
            ("a".into(), 1),
            ("b".into(), 2),
            ("c".into(), 3),
            ("convention".into(), 4),
        ]);
        let specs = serde_json::from_value(json!({"constraints": [], "derivations": expressions.iter().enumerate().map(|(n, e)| json!({"name": format!("r{n}"), "expression": e})).collect::<Vec<_>>()})).unwrap();
        Rules::compile(specs, &tags).unwrap()
    }

    #[test]
    fn positive_cycles_have_a_rooted_least_fixed_point() {
        let rules = derivations(&["a => b", "b => c", "c => a"]);
        assert!(rules.has_positive_cycles().unwrap());
        let closure = rules.closure(&[1]).unwrap();
        assert_eq!(closure.effective, vec![1, 2, 3]);
        assert_eq!(closure.derived[&1], vec!["r2"]);
        assert!(rules.closure(&[]).unwrap().effective.is_empty());
        assert!(!derivations(&["a => b", "b => c"])
            .has_positive_cycles()
            .unwrap());
    }

    #[test]
    fn equivalence_expands_conjunction_in_both_directions() {
        let rules = derivations(&["a <=> b&c"]);
        assert_eq!(rules.closure(&[1]).unwrap().effective, vec![1, 2, 3]);
        assert_eq!(rules.closure(&[2, 3]).unwrap().effective, vec![1, 2, 3]);
        assert_eq!(rules.closure(&[2]).unwrap().effective, vec![2]);
        assert_eq!(
            derivations(&["a|b => c"]).closure(&[2]).unwrap().effective,
            vec![2, 3]
        );
    }

    #[test]
    fn derivation_validation_rejects_nonmonotone_ambiguous_and_reserved_outputs() {
        let tags = HashMap::from([("a".into(), 1), ("b".into(), 2), ("convention".into(), 3)]);
        for expression in [
            "!a => b",
            "mutex(a,b) => b",
            "a => a|b",
            "a|b <=> a",
            "a => convention",
            "convention <=> a",
            "a => b => a",
            "a",
        ] {
            let specs = serde_json::from_value(json!({"constraints": [], "derivations": [{"name": "invalid", "expression": expression}]})).unwrap();
            assert!(Rules::compile(specs, &tags).is_err(), "{expression}");
        }
    }

    #[test]
    fn complex_constraints_bind_ids_and_follow_renames() {
        let tags = HashMap::from([("a".into(), 1), ("b".into(), 2), ("c".into(), 3)]);
        let specs = serde_json::from_value(json!({"constraints": [{"name": "only the triple is forbidden", "expression": "!(a&b&c)"}]})).unwrap();
        let rules = Rules::compile(specs, &tags).unwrap();
        assert!(rules.violations(&[1, 2]).is_empty());
        assert_eq!(
            rules.violations(&[1, 2, 3]),
            vec!["only the triple is forbidden"]
        );
        let renamed = HashMap::from([(1, "alpha".into()), (2, "b".into()), (3, "c".into())]);
        assert!(rules.specs(&renamed).unwrap().constraints[0]
            .expression
            .contains("alpha"));
        let roundtrip: Rules =
            serde_json::from_str(&serde_json::to_string(&rules).unwrap()).unwrap();
        assert_eq!(
            roundtrip.violations(&[1, 2, 3]),
            rules.violations(&[1, 2, 3])
        );
    }

    #[test]
    fn maximum_flat_rule_survives_rendering_and_long_tag_rename() {
        let tags = HashMap::from([("a".into(), 1)]);
        let expression = vec!["a"; 128].join("&");
        let specs = serde_json::from_value(
            json!({"constraints": [{"name": "flat", "expression": expression}]}),
        )
        .unwrap();
        let rules = Rules::compile(specs, &tags).unwrap();
        let long_name = "a".repeat(100);
        let names = HashMap::from([(1, long_name.clone())]);
        let renamed = rules.specs(&names).unwrap();
        let rebound = Rules::compile(renamed, &HashMap::from([(long_name, 1)])).unwrap();
        assert!(rebound.violations(&[1]).is_empty());
        assert_eq!(rebound.violations(&[]), vec!["flat"]);
    }
}
