//! Pure Boolean predicates shared by query syntax and bound rule expressions.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone)]
pub enum Predicate<A> {
    Atom(A),
    Not(Box<Self>),
    All(Box<Self>, Box<Self>),
    Any(Box<Self>, Box<Self>),
    Mutex(Vec<Self>),
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "op", content = "args")]
enum StoredNode<A> {
    Atom(A),
    Not,
    All,
    Any,
    Mutex(usize),
}

// Postfix storage keeps JSON nesting constant even for long valid flat expressions.
impl<A: Serialize> Serialize for Predicate<A> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut pending = vec![(self, false)];
        let mut nodes = Vec::new();
        while let Some((node, visited)) = pending.pop() {
            if visited {
                nodes.push(match node {
                    Self::Atom(a) => StoredNode::Atom(a),
                    Self::Not(_) => StoredNode::Not,
                    Self::All(..) => StoredNode::All,
                    Self::Any(..) => StoredNode::Any,
                    Self::Mutex(args) => StoredNode::Mutex(args.len()),
                });
                continue;
            }
            pending.push((node, true));
            match node {
                Self::Atom(_) => {}
                Self::Not(a) => pending.push((a, false)),
                Self::All(a, b) | Self::Any(a, b) => {
                    pending.push((b, false));
                    pending.push((a, false));
                }
                Self::Mutex(args) => pending.extend(args.iter().rev().map(|arg| (arg, false))),
            }
        }
        nodes.serialize(serializer)
    }
}

impl<'de, A: Deserialize<'de>> Deserialize<'de> for Predicate<A> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let nodes = Vec::<StoredNode<A>>::deserialize(deserializer)?;
        if nodes.is_empty() || nodes.len() > 8192 {
            return Err(D::Error::custom(
                "stored predicate must contain 1..8192 nodes",
            ));
        }
        let mut stack: Vec<(Self, usize)> = Vec::new();
        for node in nodes {
            let (expression, depth) = match node {
                StoredNode::Atom(a) => (Self::Atom(a), 0),
                StoredNode::Not => {
                    let (a, depth) = stack
                        .pop()
                        .ok_or_else(|| D::Error::custom("invalid predicate operand stack"))?;
                    (Self::Not(Box::new(a)), depth + 1)
                }
                StoredNode::All | StoredNode::Any => {
                    let (b, bd) = stack
                        .pop()
                        .ok_or_else(|| D::Error::custom("invalid predicate operand stack"))?;
                    let (a, ad) = stack
                        .pop()
                        .ok_or_else(|| D::Error::custom("invalid predicate operand stack"))?;
                    let expr = if matches!(node, StoredNode::All) {
                        Self::All(Box::new(a), Box::new(b))
                    } else {
                        Self::Any(Box::new(a), Box::new(b))
                    };
                    (expr, ad.max(bd) + 1)
                }
                StoredNode::Mutex(count) => {
                    if count < 2 || count > stack.len() {
                        return Err(D::Error::custom("invalid mutex operand count"));
                    }
                    let operands = stack.split_off(stack.len() - count);
                    let depth = operands.iter().map(|(_, depth)| *depth).max().unwrap_or(0) + 1;
                    (
                        Self::Mutex(operands.into_iter().map(|(expr, _)| expr).collect()),
                        depth,
                    )
                }
            };
            if depth > 128 {
                return Err(D::Error::custom("stored predicate depth exceeds 128"));
            }
            stack.push((expression, depth));
        }
        if stack.len() != 1 {
            return Err(D::Error::custom("invalid predicate operand stack"));
        }
        Ok(stack.pop().expect("validated single expression").0)
    }
}

impl<A> Predicate<A> {
    pub fn try_expand<B, E>(
        self,
        atom: &mut impl FnMut(A) -> Result<Predicate<B>, E>,
    ) -> Result<Predicate<B>, E> {
        Ok(match self {
            Self::Atom(value) => atom(value)?,
            Self::Not(a) => Predicate::Not(Box::new(a.try_expand(atom)?)),
            Self::All(a, b) => {
                Predicate::All(Box::new(a.try_expand(atom)?), Box::new(b.try_expand(atom)?))
            }
            Self::Any(a, b) => {
                Predicate::Any(Box::new(a.try_expand(atom)?), Box::new(b.try_expand(atom)?))
            }
            Self::Mutex(args) => Predicate::Mutex(
                args.into_iter()
                    .map(|a| a.try_expand(atom))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }

    pub fn display(&self, atom: &impl Fn(&A) -> Result<String, String>) -> Result<String, String> {
        self.display_at(atom, 0)
    }

    fn display_at(
        &self,
        atom: &impl Fn(&A) -> Result<String, String>,
        parent: u8,
    ) -> Result<String, String> {
        let precedence = match self {
            Self::Any(..) => 1,
            Self::All(..) => 2,
            Self::Not(..) => 3,
            _ => 4,
        };
        let rendered = match self {
            Self::Atom(a) => atom(a)?,
            Self::Not(a) => format!("!{}", a.display_at(atom, precedence)?),
            Self::All(a, b) => format!(
                "{}&{}",
                a.display_at(atom, precedence)?,
                b.display_at(atom, precedence)?
            ),
            Self::Any(a, b) => format!(
                "{}|{}",
                a.display_at(atom, precedence)?,
                b.display_at(atom, precedence)?
            ),
            Self::Mutex(args) => format!(
                "mutex({})",
                args.iter()
                    .map(|a| a.display(atom))
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
    pub fn eval_with(&self, atom: &impl Fn(&A) -> bool) -> bool {
        match self {
            Self::Atom(value) => atom(value),
            Self::Not(inner) => !inner.eval_with(atom),
            Self::All(a, b) => a.eval_with(atom) && b.eval_with(atom),
            Self::Any(a, b) => a.eval_with(atom) || b.eval_with(atom),
            Self::Mutex(args) => {
                args.iter()
                    .filter(|arg| arg.eval_with(atom))
                    .take(2)
                    .count()
                    <= 1
            }
        }
    }

    pub fn try_map<B, E>(
        self,
        atom: &mut impl FnMut(A) -> Result<B, E>,
    ) -> Result<Predicate<B>, E> {
        Ok(match self {
            Self::Atom(value) => Predicate::Atom(atom(value)?),
            Self::Not(inner) => Predicate::Not(Box::new(inner.try_map(atom)?)),
            Self::All(a, b) => {
                Predicate::All(Box::new(a.try_map(atom)?), Box::new(b.try_map(atom)?))
            }
            Self::Any(a, b) => {
                Predicate::Any(Box::new(a.try_map(atom)?), Box::new(b.try_map(atom)?))
            }
            Self::Mutex(args) => Predicate::Mutex(
                args.into_iter()
                    .map(|arg| arg.try_map(atom))
                    .collect::<Result<_, _>>()?,
            ),
        })
    }

    pub fn visit_atoms<'a>(&'a self, atom: &mut impl FnMut(&'a A)) {
        match self {
            Self::Atom(value) => atom(value),
            Self::Not(inner) => inner.visit_atoms(atom),
            Self::All(a, b) | Self::Any(a, b) => {
                a.visit_atoms(atom);
                b.visit_atoms(atom);
            }
            Self::Mutex(args) => args.iter().for_each(|arg| arg.visit_atoms(atom)),
        }
    }

    pub fn positive_with(&self, atom: &impl Fn(&A) -> bool) -> bool {
        match self {
            Self::Atom(value) => atom(value),
            Self::All(a, b) | Self::Any(a, b) => a.positive_with(atom) && b.positive_with(atom),
            Self::Not(_) | Self::Mutex(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_preserves_boolean_semantics_for_all_tag_subsets() {
        let expression = Predicate::Any(
            Box::new(Predicate::All(
                Box::new(Predicate::Atom("a")),
                Box::new(Predicate::Not(Box::new(Predicate::Atom("b")))),
            )),
            Box::new(Predicate::Mutex(vec![
                Predicate::Atom("b"),
                Predicate::Atom("c"),
            ])),
        );
        let bound = expression
            .clone()
            .try_map(&mut |name| -> Result<usize, ()> {
                Ok(["a", "b", "c"]
                    .iter()
                    .position(|candidate| *candidate == name)
                    .unwrap())
            })
            .unwrap();
        let bound: Predicate<usize> =
            serde_json::from_str(&serde_json::to_string(&bound).unwrap()).unwrap();
        for subset in 0..8 {
            assert_eq!(
                expression.eval_with(&|name| subset
                    & (1 << ["a", "b", "c"]
                        .iter()
                        .position(|candidate| candidate == name)
                        .unwrap())
                    != 0),
                bound.eval_with(&|index| subset & (1 << index) != 0)
            );
        }
        assert!(!expression.positive_with(&|_| true));
        let mut names = Vec::new();
        expression.visit_atoms(&mut |name| names.push(*name));
        assert_eq!(names, ["a", "b", "b", "c"]);
    }

    #[test]
    fn stored_predicates_reject_malformed_stacks_and_excessive_depth() {
        use serde_json::json;
        for value in [
            json!([]),
            json!([{"op":"Not"}]),
            json!([{"op":"Atom","args":1},{"op":"Atom","args":2}]),
            json!([{"op":"Atom","args":1},{"op":"Mutex","args":1}]),
        ] {
            assert!(serde_json::from_value::<Predicate<i64>>(value).is_err());
        }
        let mut nodes = vec![json!({"op":"Atom","args":1})];
        nodes.extend((0..129).map(|_| json!({"op":"Not"})));
        assert!(serde_json::from_value::<Predicate<i64>>(json!(nodes)).is_err());
    }
}
