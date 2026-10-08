//! Persistent constraints bind stable tag ids, while the public form uses current names.

use crate::tag_expr::{self, TagExpr};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
}

#[derive(Default, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub constraints: Vec<Constraint>,
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
        Ok(match self {
            Self::Tag(id) => tags
                .get(id)
                .ok_or_else(|| format!("rule references missing tag id {id}"))?
                .clone(),
            Self::Not(a) => format!("!({})", a.display(tags)?),
            Self::All(a, b) => format!("({}&{})", a.display(tags)?, b.display(tags)?),
            Self::Any(a, b) => format!("({}|{})", a.display(tags)?, b.display(tags)?),
            Self::Mutex(args) => format!(
                "mutex({})",
                args.iter()
                    .map(|a| a.display(tags))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(",")
            ),
        })
    }
}

impl Rules {
    pub fn compile(specs: RuleSpecs, tags: &HashMap<String, i64>) -> Result<Self, String> {
        if specs.constraints.len() > 128 {
            return Err("at most 128 tag constraints are allowed".into());
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
            let expression = Expr::bind(tag_expr::parse(&spec.expression)?, tags)
                .map_err(|e| format!("rule '{name}': {e}"))?;
            constraints.push(Constraint { name, expression });
        }
        Ok(Self { constraints })
    }

    pub fn specs(&self, tags: &HashMap<i64, String>) -> Result<RuleSpecs, String> {
        Ok(RuleSpecs {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
