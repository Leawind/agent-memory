use super::Store;
use crate::{
    named_predicates::{Atom, Definition, Registry},
    sql,
    tag_expr::{self, TagAtom, TagExpr},
};
use rusqlite::params;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

impl Store {
    pub fn predicate_rows(&self) -> Result<Vec<Definition>, String> {
        let mut statement = self
            .conn
            .prepare(sql::PREDICATE_ALL)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.map(|r| {
            let (id, name, raw, description) = r.map_err(|e| e.to_string())?;
            Ok(Definition {
                id,
                name,
                predicate: serde_json::from_str(&raw)
                    .map_err(|e| format!("corrupt named predicate: {e}"))?,
                description,
            })
        })
        .collect()
    }

    pub fn predicate_registry(&self) -> Result<Registry, String> {
        Registry::new(self.predicate_rows()?)
    }

    pub fn predicates_view(&self) -> Result<Value, String> {
        let registry = self.predicate_registry()?;
        let tags = self.tag_id_names()?;
        let mut rows = registry.definitions.values().collect::<Vec<_>>();
        rows.sort_by(|a, b| a.name.cmp(&b.name));
        rows.into_iter().map(|d| Ok(json!({"name":d.name,"predicate":registry.display(&d.predicate,&tags)?,"description":d.description}))).collect::<Result<Vec<_>,String>>().map(Value::Array)
    }

    pub fn import_predicates(&self, value: &Value) -> Result<(), String> {
        let rows = value.as_array().ok_or("predicates must be an array")?;
        if rows.len() > 128 {
            return Err("at most 128 named predicates are allowed".into());
        }
        let tags = self.tag_id_names()?;
        let names = tags.iter().map(|(id, name)| (name.clone(), *id)).collect();
        let placeholder = serde_json::to_string(&crate::named_predicates::BoundExpr::Atom(
            Atom::Tag(*tags.keys().next().ok_or("missing tags")?),
        ))
        .map_err(|e| e.to_string())?;
        let mut inputs = Vec::new();
        let mut seen = HashSet::new();
        for row in rows {
            let obj = row
                .as_object()
                .ok_or("predicate definition must be an object")?;
            if obj
                .keys()
                .any(|k| !["name", "predicate", "description"].contains(&k.as_str()))
            {
                return Err("unknown predicate definition field".into());
            }
            let name = crate::model::normalize_tag_name(
                row["name"].as_str().ok_or("predicate requires name")?,
            )?;
            if !seen.insert(name.clone()) {
                return Err("duplicate predicate name".into());
            }
            let predicate = row["predicate"]
                .as_str()
                .ok_or("predicate requires expression")?
                .to_string();
            let description = row["description"]
                .as_str()
                .ok_or("predicate requires description")?
                .to_string();
            if description.chars().count() > 512 {
                return Err("predicate description exceeds 512 characters".into());
            }
            self.conn
                .execute(sql::PREDICATE_PUT, params![name, placeholder, description])
                .map_err(|e| e.to_string())?;
            inputs.push((name, predicate, description));
        }
        let registry = self.predicate_registry()?;
        for (name, predicate, description) in inputs {
            let bound = registry.bind(tag_expr::parse_rule(&predicate)?, &names)?;
            self.conn
                .execute(
                    sql::PREDICATE_PUT,
                    params![
                        name,
                        serde_json::to_string(&bound).map_err(|e| e.to_string())?,
                        description
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        self.validate_predicates()
    }

    pub fn predicate_list(&self) -> Result<Value, String> {
        Ok(json!({"predicates": self.predicates_view()?}))
    }

    pub fn predicate_set(
        &self,
        name: &str,
        predicate: &str,
        description: &str,
    ) -> Result<Value, String> {
        let rows = self.predicate_rows()?;
        if !rows.iter().any(|d| d.name == name) && rows.len() >= 128 {
            return Err("at most 128 named predicates are allowed".into());
        }
        let tags = self.tag_id_names()?;
        let names: HashMap<String, i64> =
            tags.iter().map(|(id, name)| (name.clone(), *id)).collect();
        // Predeclare a stable ID so self references are diagnosed as cycles rather than unknown names.
        let placeholder = crate::named_predicates::BoundExpr::Atom(Atom::Tag(
            *names
                .get(crate::model::RESERVED_TAG)
                .ok_or("missing reserved tag")?,
        ));
        self.conn
            .execute(
                sql::PREDICATE_PUT,
                params![
                    name,
                    serde_json::to_string(&placeholder).map_err(|e| e.to_string())?,
                    description
                ],
            )
            .map_err(|e| e.to_string())?;
        let registry = self.predicate_registry()?;
        let bound = registry.bind(tag_expr::parse_rule(predicate)?, &names)?;
        self.conn
            .execute(
                sql::PREDICATE_PUT,
                params![
                    name,
                    serde_json::to_string(&bound).map_err(|e| e.to_string())?,
                    description
                ],
            )
            .map_err(|e| e.to_string())?;
        self.validate_predicates()?;
        Ok(json!({"saved":true}))
    }

    pub fn validate_predicates(&self) -> Result<(), String> {
        self.predicate_registry().map(|_| ())
    }

    pub fn predicate_delete(&self, name: &str) -> Result<Value, String> {
        let rows = self.predicate_rows()?;
        let id = rows
            .iter()
            .find(|d| d.name == name)
            .ok_or("named predicate not found")?
            .id;
        if rows.iter().any(|d| d.predicate.references_named(id)) {
            return Err("predicate is still referenced; remove dependent predicates first".into());
        }
        self.conn
            .execute(sql::PREDICATE_DELETE, [id])
            .map_err(|e| e.to_string())?;
        Ok(json!({"deleted":true}))
    }

    pub fn resolve_named_query(&self, expr: TagExpr) -> Result<TagExpr, String> {
        let mut named = false;
        expr.visit_atoms(&mut |a| named |= matches!(a, TagAtom::Named(_)));
        if !named {
            return Ok(expr);
        }
        self.expand_named_refs(expr)
    }

    /// Expand `@named` references into literal tag leaves. An expression without named
    /// references passes through untouched.
    pub fn expand_named_refs(&self, expr: TagExpr) -> Result<TagExpr, String> {
        let mut named = false;
        expr.visit_atoms(&mut |a| named |= matches!(a, TagAtom::Named(_)));
        if !named {
            return Ok(expr);
        }
        let registry = self.predicate_registry()?;
        let tags = self.tag_id_names()?;
        let mut atoms = 0usize;
        let expanded = expr.try_expand(&mut |atom| match atom {
            TagAtom::Named(name) => {
                let predicate = registry
                    .expanded
                    .get(&registry.lookup(&name)?)
                    .ok_or("unresolved predicate")?;
                predicate.visit_atoms(&mut |_| atoms += 1);
                if atoms > 4096 {
                    return Err("expanded query exceeds 4096 atoms".to_string());
                }
                predicate.clone().try_map(&mut |id| {
                    tags.get(&id)
                        .cloned()
                        .map(TagAtom::Tag)
                        .ok_or_else(|| format!("predicate references missing tag id {id}"))
                })
            }
            atom => {
                atoms += 1;
                if atoms > 4096 {
                    return Err("expanded query exceeds 4096 atoms".to_string());
                }
                Ok(TagExpr::Atom(atom))
            }
        })?;
        crate::named_predicates::validate_expansion(&expanded)?;
        Ok(expanded)
    }
}
