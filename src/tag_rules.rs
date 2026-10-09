//! Persistent constraints bind stable tag ids, while the public form uses current names.

use crate::tag_expr::{self, TagAtom, TagExpr};
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
    pub derivations: Vec<DerivationSpec>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DerivationSpec {
    Expression(RuleSpec),
    Predicate(DerivationParts),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DerivationParts {
    pub name: String,
    pub predicate: String,
    pub derived: Vec<String>,
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
    pub directions: Vec<DerivedRule>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DerivedRule {
    pub predicate: Expr,
    pub derived: Vec<i64>,
}

impl Derivation {
    pub fn references(&self, id: i64) -> bool {
        self.directions
            .iter()
            .any(|r| r.predicate.references(id) || r.derived.contains(&id))
    }
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

pub type Expr = crate::predicate::Predicate<i64>;

impl Expr {
    fn positive(&self) -> bool {
        self.positive_with(&|_| true)
    }

    fn tag_ids(&self, ids: &mut HashSet<i64>) {
        self.visit_atoms(&mut |id| {
            ids.insert(*id);
        });
    }
    fn bind(expr: TagExpr, tags: &HashMap<String, i64>) -> Result<Self, String> {
        expr.try_map(&mut |atom| match atom {
            TagAtom::Tag(name) => Ok(*tags.get(&name).ok_or_else(|| {
                format!("unknown tag '{name}' in rule; create the tag first")
            })?),
            TagAtom::Regex(_) => Err("persistent rules require literal tag names; regex atoms are only supported in queries".into()),
        })
    }

    pub fn eval(&self, tags: &[i64]) -> bool {
        self.eval_with(&|id| tags.contains(id))
    }

    pub fn references(&self, id: i64) -> bool {
        let mut found = false;
        self.visit_atoms(&mut |tag| found |= *tag == id);
        found
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
            Self::Atom(id) => tags
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
            let (name, directions) = match spec {
                DerivationSpec::Predicate(parts) => (
                    parts.name,
                    vec![DerivedRule {
                        predicate: Expr::bind(tag_expr::parse_rule(&parts.predicate)?, tags)?,
                        derived: bind_set(parts.derived, tags)?,
                    }],
                ),
                DerivationSpec::Expression(spec) => {
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
                    let right = parse_set(right, tags)?;
                    let directions = if bidirectional {
                        let left = parse_set(left, tags)?;
                        vec![
                            DerivedRule {
                                predicate: conjunction(&left),
                                derived: right.clone(),
                            },
                            DerivedRule {
                                predicate: conjunction(&right),
                                derived: left,
                            },
                        ]
                    } else {
                        vec![DerivedRule {
                            predicate: Expr::bind(tag_expr::parse_rule(left)?, tags)?,
                            derived: right,
                        }]
                    };
                    (spec.name, directions)
                }
            };
            let name = name.trim().to_string();
            if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
                return Err("rule name must be 1-100 characters without control characters".into());
            }
            if !names.insert(name.clone()) {
                return Err(format!("duplicate rule name '{name}'"));
            }
            if directions.iter().any(|r| !r.predicate.positive()) {
                return Err(
                    "derivations must be positive: negation and mutex are not allowed".into(),
                );
            }
            if tags
                .get(crate::model::RESERVED_TAG)
                .is_some_and(|id| directions.iter().any(|r| r.derived.contains(id)))
            {
                return Err("the reserved convention tag cannot be derived".into());
            }
            derivations.push(Derivation { name, directions });
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
                    let expression = if rule.directions.len() == 2 {
                        format!(
                            "{} <=> {}",
                            display_set(&rule.directions[1].derived, tags)?,
                            display_set(&rule.directions[0].derived, tags)?
                        )
                    } else {
                        format!(
                            "{} => {}",
                            rule.directions[0].predicate.display(tags)?,
                            display_set(&rule.directions[0].derived, tags)?
                        )
                    };
                    Ok(DerivationSpec::Expression(RuleSpec {
                        name: rule.name.clone(),
                        expression,
                    }))
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
            if rule.directions.iter().any(|r| !r.predicate.positive()) {
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
                for direction in &rule.directions {
                    if !direction.predicate.eval(&effective) {
                        continue;
                    }
                    for id in direction.derived.iter().copied() {
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
            for direction in &rule.directions {
                let mut inputs = HashSet::new();
                direction.predicate.tag_ids(&mut inputs);
                for input in inputs {
                    edges
                        .entry(input)
                        .or_default()
                        .extend(direction.derived.iter().copied());
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

fn bind_set(names: Vec<String>, tags: &HashMap<String, i64>) -> Result<Vec<i64>, String> {
    if names.is_empty() || names.len() > 128 {
        return Err("tag set must contain 1..128 tags".into());
    }
    let mut ids = Vec::new();
    for name in names {
        let name = crate::model::normalize_tag_name(&name)?;
        let id = *tags
            .get(&name)
            .ok_or_else(|| format!("unknown tag '{name}'; create the tag first"))?;
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    Ok(ids)
}

fn parse_set(raw: &str, tags: &HashMap<String, i64>) -> Result<Vec<i64>, String> {
    bind_set(raw.split(',').map(str::to_string).collect(), tags)
}

fn conjunction(ids: &[i64]) -> Expr {
    ids.iter()
        .copied()
        .map(Expr::Atom)
        .reduce(|a, b| Expr::All(Box::new(a), Box::new(b)))
        .expect("validated nonempty set")
}

fn display_set(ids: &[i64], tags: &HashMap<i64, String>) -> Result<String, String> {
    ids.iter()
        .map(|id| {
            tags.get(id)
                .cloned()
                .ok_or_else(|| format!("missing tag id {id}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|names| names.join(", "))
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
        let rules = derivations(&["a <=> b, c"]);
        assert_eq!(rules.closure(&[1]).unwrap().effective, vec![1, 2, 3]);
        assert_eq!(rules.closure(&[2, 3]).unwrap().effective, vec![1, 2, 3]);
        assert_eq!(rules.closure(&[2]).unwrap().effective, vec![2]);
        assert_eq!(
            derivations(&["a|b => c"]).closure(&[2]).unwrap().effective,
            vec![2, 3]
        );
    }

    #[test]
    fn predicate_and_output_set_are_independent_and_roundtrip() {
        let tags = HashMap::from([
            ("a".into(), 1),
            ("b".into(), 2),
            ("c".into(), 3),
            ("d".into(), 4),
            ("e".into(), 5),
        ]);
        for spec in [
            json!({"name":"multi", "expression":"a&b => c, d, e"}),
            json!({"name":"multi", "predicate":"a&b", "derived":["c","d","e","c"]}),
        ] {
            let rules = Rules::compile(
                serde_json::from_value(json!({"constraints":[],"derivations":[spec]})).unwrap(),
                &tags,
            )
            .unwrap();
            assert_eq!(rules.closure(&[1]).unwrap().effective, vec![1]);
            assert_eq!(
                rules.closure(&[1, 2]).unwrap().effective,
                vec![1, 2, 3, 4, 5]
            );
            let names = tags.iter().map(|(name, id)| (*id, name.clone())).collect();
            let rebound = Rules::compile(rules.specs(&names).unwrap(), &tags).unwrap();
            assert_eq!(
                rebound.closure(&[1, 2]).unwrap().effective,
                vec![1, 2, 3, 4, 5]
            );
        }
        let rules = Rules::compile(serde_json::from_value(json!({"constraints":[], "derivations":[{"name":"dual", "expression":"a, b <=> c, d"}]})).unwrap(), &tags).unwrap();
        assert_eq!(rules.closure(&[1]).unwrap().effective, vec![1]);
        assert_eq!(rules.closure(&[1, 2]).unwrap().effective, vec![1, 2, 3, 4]);
        assert_eq!(rules.closure(&[3, 4]).unwrap().effective, vec![1, 2, 3, 4]);
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
