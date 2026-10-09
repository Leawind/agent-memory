//! Pure Boolean predicates shared by query syntax and bound rule expressions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "op", content = "args")]
pub enum Predicate<A> {
    Atom(A),
    Not(Box<Self>),
    All(Box<Self>, Box<Self>),
    Any(Box<Self>, Box<Self>),
    Mutex(Vec<Self>),
}

impl<A> Predicate<A> {
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
}
