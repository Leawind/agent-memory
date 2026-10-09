//! Persisted references bind stable IDs; expansion is bounded and rejects cycles.
use crate::{
    predicate::Predicate,
    tag_expr::{TagAtom, TagExpr},
    tag_rules::Expr,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Atom {
    Tag(i64),
    Named(i64),
}
pub type BoundExpr = Predicate<Atom>;

#[derive(Clone, Debug)]
pub struct Definition {
    pub id: i64,
    pub namespace: String,
    pub name: String,
    pub predicate: BoundExpr,
    pub description: String,
}

#[derive(Default)]
pub struct Registry {
    pub definitions: HashMap<i64, Definition>,
    pub expanded: HashMap<i64, Expr>,
    namespace: String,
    heights: HashMap<i64, usize>,
}

impl Registry {
    pub fn new(rows: Vec<Definition>, namespace: String) -> Result<Self, String> {
        let mut registry = Self {
            definitions: rows.into_iter().map(|d| (d.id, d)).collect(),
            namespace,
            ..Self::default()
        };
        let ids = registry.definitions.keys().copied().collect::<Vec<_>>();
        for id in ids {
            registry.expand_definition(id, &mut HashSet::new(), 0)?;
        }
        Ok(registry)
    }

    fn expand_definition(
        &mut self,
        id: i64,
        path: &mut HashSet<i64>,
        depth: usize,
    ) -> Result<Expr, String> {
        if depth > 64 {
            return Err("named predicate dependency depth exceeds 64".into());
        }
        if let Some(expr) = self.expanded.get(&id) {
            if depth + self.heights[&id] > 64 {
                return Err("named predicate dependency depth exceeds 64".into());
            }
            return Ok(expr.clone());
        }
        if !path.insert(id) {
            return Err("named predicate reference cycle detected".into());
        }
        let definition = self
            .definitions
            .get(&id)
            .ok_or("missing named predicate reference")?
            .clone();
        let mut refs = Vec::new();
        definition.predicate.visit_atoms(&mut |atom| {
            if let Atom::Named(id) = atom {
                refs.push(*id);
            }
        });
        let mut height = 0;
        for reference in refs {
            let target = self
                .definitions
                .get(&reference)
                .ok_or("missing named predicate reference")?;
            if target.namespace != "global" && target.namespace != definition.namespace {
                return Err("predicates cannot reference another user's scope".into());
            }
            self.expand_definition(reference, path, depth + 1)?;
            height = height.max(self.heights[&reference] + 1);
        }
        let expanded = self.resolve(&definition.predicate)?;
        path.remove(&id);
        self.expanded.insert(id, expanded.clone());
        self.heights.insert(id, height);
        Ok(expanded)
    }

    pub fn lookup(&self, raw: &str) -> Result<i64, String> {
        let (scope, name) = match raw.split_once("::") {
            Some((scope, name)) => (Some(scope), name),
            None => (None, raw),
        };
        let name = crate::model::normalize_tag_name(name)?;
        let scopes = match scope {
            None => vec![self.namespace.as_str(), "global"],
            Some("global") => vec!["global"],
            Some("user") if self.namespace != "global" => vec![self.namespace.as_str()],
            _ => return Err("predicate reference scope must be global or user; global rules cannot reference user predicates".into()),
        };
        for namespace in scopes {
            if let Some(d) = self
                .definitions
                .values()
                .find(|d| d.namespace == namespace && d.name == name)
            {
                return Ok(d.id);
            }
        }
        Err(format!("unknown named predicate '@{raw}'"))
    }

    pub fn bind(&self, expr: TagExpr, tags: &HashMap<String, i64>) -> Result<BoundExpr, String> {
        expr.try_map(&mut |atom| match atom {
            TagAtom::Tag(name) => {
                let name = crate::model::normalize_tag_name(&name)?;
                tags.get(&name)
                    .copied()
                    .map(Atom::Tag)
                    .ok_or_else(|| format!("unknown tag '{name}'; create the tag first"))
            }
            TagAtom::Named(name) => self.lookup(&name).map(Atom::Named),
            TagAtom::Regex(_) => {
                Err("persisted predicates require literal tags; regex atoms are query-only".into())
            }
        })
    }

    pub fn resolve(&self, expr: &BoundExpr) -> Result<Expr, String> {
        let mut nodes = 0usize;
        let resolved = expr.clone().try_expand(&mut |atom| {
            let expr = match atom {
                Atom::Tag(id) => Expr::Atom(id),
                Atom::Named(id) => self
                    .expanded
                    .get(&id)
                    .cloned()
                    .ok_or("unresolved named predicate")?,
            };
            expr.visit_atoms(&mut |_| nodes += 1);
            if nodes > 4096 {
                return Err("expanded predicate exceeds 4096 atoms".to_string());
            }
            Ok(expr)
        })?;
        validate_expansion(&resolved)?;
        Ok(resolved)
    }

    pub fn display(&self, expr: &BoundExpr, tags: &HashMap<i64, String>) -> Result<String, String> {
        expr.display(&|atom| match atom {
            Atom::Tag(id) => tags
                .get(id)
                .cloned()
                .ok_or_else(|| format!("missing tag id {id}")),
            Atom::Named(id) => {
                let d = self.definitions.get(id).ok_or("missing named predicate")?;
                Ok(format!(
                    "@{}::{}",
                    if d.namespace == "global" {
                        "global"
                    } else {
                        "user"
                    },
                    d.name
                ))
            }
        })
    }
}

pub fn validate_expansion<A>(expr: &Predicate<A>) -> Result<(), String> {
    let mut pending = vec![(expr, 0usize)];
    let mut nodes = 0;
    while let Some((node, depth)) = pending.pop() {
        nodes += 1;
        if nodes > 8192 || depth > 128 {
            return Err("expanded predicate exceeds 8192 nodes or depth 128".into());
        }
        match node {
            Predicate::Atom(_) => {}
            Predicate::Not(a) => pending.push((a, depth + 1)),
            Predicate::All(a, b) | Predicate::Any(a, b) => {
                pending.push((a, depth + 1));
                pending.push((b, depth + 1));
            }
            Predicate::Mutex(args) => pending.extend(args.iter().map(|a| (a, depth + 1))),
        }
    }
    Ok(())
}

impl BoundExpr {
    pub fn references_named(&self, id: i64) -> bool {
        let mut found = false;
        self.visit_atoms(&mut |a| found |= matches!(a, Atom::Named(reference) if *reference == id));
        found
    }
    pub fn references_tag(&self, id: i64) -> bool {
        let mut found = false;
        self.visit_atoms(&mut |a| found |= matches!(a, Atom::Tag(reference) if *reference == id));
        found
    }
}
