use super::Store;
use crate::{
    auth::IdentityCtx,
    named_predicates::{Atom, Definition, Registry},
    sql,
    tag_expr::{self, TagAtom, TagExpr},
};
use rusqlite::params;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};

impl Store {
    pub fn predicate_rows(&self, namespace: Option<&str>) -> Result<Vec<Definition>, String> {
        let mut statement = self
            .conn
            .prepare(sql::PREDICATE_ALL)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([namespace], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.map(|r| {
            let (id, namespace, name, raw, description) = r.map_err(|e| e.to_string())?;
            Ok(Definition {
                id,
                namespace,
                name,
                predicate: serde_json::from_str(&raw)
                    .map_err(|e| format!("corrupt named predicate: {e}"))?,
                description,
            })
        })
        .collect()
    }

    pub fn predicate_registry(&self, namespace: &str) -> Result<Registry, String> {
        Registry::new(self.predicate_rows(Some(namespace))?, namespace.into())
    }

    pub fn global_predicates_view(&self) -> Result<Value, String> {
        let registry = self.predicate_registry("global")?;
        let tags = self.tag_id_names()?;
        let mut rows = registry.definitions.values().collect::<Vec<_>>();
        rows.sort_by(|a, b| a.name.cmp(&b.name));
        rows.into_iter().map(|d| Ok(json!({"name":d.name,"predicate":registry.display(&d.predicate,&tags)?,"description":d.description}))).collect::<Result<Vec<_>,String>>().map(Value::Array)
    }

    pub fn import_global_predicates(&self, value: &Value) -> Result<(), String> {
        let rows = value.as_array().ok_or("predicates must be an array")?;
        if rows.len() > 128 {
            return Err("at most 128 global predicates are allowed".into());
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
                .execute(
                    sql::PREDICATE_PUT,
                    params![
                        "global",
                        Option::<i64>::None,
                        name,
                        placeholder,
                        description
                    ],
                )
                .map_err(|e| e.to_string())?;
            inputs.push((name, predicate, description));
        }
        let registry = self.predicate_registry("global")?;
        for (name, predicate, description) in inputs {
            let bound = registry.bind(tag_expr::parse_rule(&predicate)?, &names)?;
            self.conn
                .execute(
                    sql::PREDICATE_PUT,
                    params![
                        "global",
                        Option::<i64>::None,
                        name,
                        serde_json::to_string(&bound).map_err(|e| e.to_string())?,
                        description
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        self.validate_all_predicate_scopes()
    }

    pub fn predicate_list(&self, ctx: &IdentityCtx) -> Result<Value, String> {
        let namespace = ctx.predicate_namespace()?;
        let registry = self.predicate_registry(&namespace)?;
        let tags = self.tag_id_names()?;
        let mut definitions = registry.definitions.values().collect::<Vec<_>>();
        definitions.sort_by(|a, b| (&a.namespace, &a.name).cmp(&(&b.namespace, &b.name)));
        let rows = definitions.into_iter().map(|d| Ok(json!({"name":d.name,"scope":if d.namespace=="global" {"global"} else {"user"},"predicate":registry.display(&d.predicate,&tags)?,"description":d.description}))).collect::<Result<Vec<_>,String>>()?;
        Ok(json!({"predicates":rows}))
    }

    pub fn predicate_set(
        &self,
        ctx: &IdentityCtx,
        scope: &str,
        name: &str,
        predicate: &str,
        description: &str,
    ) -> Result<Value, String> {
        let namespace = if scope == "global" {
            "global".to_string()
        } else {
            ctx.predicate_namespace()?
        };
        let rows = self.predicate_rows(Some(&namespace))?;
        if !rows
            .iter()
            .any(|d| d.namespace == namespace && d.name == name)
            && rows.iter().filter(|d| d.namespace == namespace).count() >= 128
        {
            return Err("at most 128 named predicates per scope are allowed".into());
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
        let owner = if scope == "global" {
            None
        } else {
            ctx.identity_id
        };
        self.conn
            .execute(
                sql::PREDICATE_PUT,
                params![
                    namespace,
                    owner,
                    name,
                    serde_json::to_string(&placeholder).map_err(|e| e.to_string())?,
                    description
                ],
            )
            .map_err(|e| e.to_string())?;
        let registry = self.predicate_registry(&namespace)?;
        let bound = registry.bind(tag_expr::parse_rule(predicate)?, &names)?;
        self.conn
            .execute(
                sql::PREDICATE_PUT,
                params![
                    namespace,
                    owner,
                    name,
                    serde_json::to_string(&bound).map_err(|e| e.to_string())?,
                    description
                ],
            )
            .map_err(|e| e.to_string())?;
        self.validate_all_predicate_scopes()?;
        if scope == "global" {
            self.recompute_tag_rules()?;
        }
        Ok(json!({"saved":true}))
    }

    pub fn validate_all_predicate_scopes(&self) -> Result<(), String> {
        let rows = self.predicate_rows(None)?;
        let mut scopes: HashSet<String> = rows.iter().map(|d| d.namespace.clone()).collect();
        scopes.insert("global".into());
        for scope in scopes {
            Registry::new(
                rows.iter()
                    .filter(|d| d.namespace == "global" || d.namespace == scope)
                    .cloned()
                    .collect(),
                scope,
            )?;
        }
        Ok(())
    }

    pub fn predicate_delete(
        &self,
        ctx: &IdentityCtx,
        scope: &str,
        name: &str,
    ) -> Result<Value, String> {
        let namespace = if scope == "global" {
            "global".to_string()
        } else {
            ctx.predicate_namespace()?
        };
        let rows = self.predicate_rows(None)?;
        let id = rows
            .iter()
            .find(|d| d.namespace == namespace && d.name == name)
            .ok_or("named predicate not found")?
            .id;
        let rules = self.stored_tag_rules()?;
        if rows.iter().any(|d| d.predicate.references_named(id))
            || rules
                .constraints
                .iter()
                .any(|r| r.expression.references_named(id))
            || rules
                .derivations
                .iter()
                .flat_map(|r| &r.directions)
                .any(|r| r.predicate.references_named(id))
        {
            return Err(
                "predicate is still referenced; remove dependent predicates and rules first".into(),
            );
        }
        self.conn
            .execute(sql::PREDICATE_DELETE, [id])
            .map_err(|e| e.to_string())?;
        Ok(json!({"deleted":true}))
    }

    pub fn resolve_named_query(&self, ctx: &IdentityCtx, expr: TagExpr) -> Result<TagExpr, String> {
        let mut named = false;
        expr.visit_atoms(&mut |a| named |= matches!(a, TagAtom::Named(_)));
        if !named {
            return Ok(expr);
        }
        let namespace = ctx.predicate_namespace()?;
        let registry = self.predicate_registry(&namespace)?;
        let tags = self.tag_id_names()?;
        let expanded = expr.try_expand(&mut |atom| match atom {
            TagAtom::Named(name) => registry
                .expanded
                .get(&registry.lookup(&name)?)
                .ok_or("unresolved predicate")?
                .clone()
                .try_map(&mut |id| {
                    tags.get(&id)
                        .cloned()
                        .map(TagAtom::Tag)
                        .ok_or_else(|| format!("predicate references missing tag id {id}"))
                }),
            atom => Ok(TagExpr::Atom(atom)),
        })?;
        crate::named_predicates::validate_expansion(&expanded)?;
        Ok(expanded)
    }
}
