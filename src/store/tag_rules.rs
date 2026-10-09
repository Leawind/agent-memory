//! Atomic publication and write validation for administrator-managed tag constraints.

use super::Store;
use crate::{
    sql,
    tag_rules::{RuleSpecs, Rules},
};
use rusqlite::params;
use serde_json::{json, Value};
use std::collections::HashMap;

impl Store {
    pub const SETTING_TAG_RULES: &'static str = "tag_rules";

    pub fn tag_id_names(&self) -> Result<HashMap<i64, String>, String> {
        let mut statement = self
            .conn
            .prepare(sql::TAG_EXPORT_ALL)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn stored_tag_rules(&self) -> Result<Rules<crate::named_predicates::Atom>, String> {
        match self.settings_get(Self::SETTING_TAG_RULES)? {
            None => Ok(Rules {
                constraints: Vec::new(),
                derivations: Vec::new(),
            }),
            Some(raw) => serde_json::from_str(&raw).map_err(|e| format!("corrupt tag rules: {e}")),
        }
    }

    pub fn tag_rules(&self) -> Result<Rules, String> {
        let registry = self.predicate_registry("global")?;
        let rules = self
            .stored_tag_rules()?
            .try_map(&mut |expr| registry.resolve(expr))?;
        for direction in rules.derivations.iter().flat_map(|r| &r.directions) {
            if !direction.predicate.positive_with(&|_| true) {
                return Err(
                    "derivations require positive monotone predicates, including named references"
                        .into(),
                );
            }
        }
        Ok(rules)
    }

    pub fn tag_rules_view(&self) -> Result<Value, String> {
        let registry = self.predicate_registry("global")?;
        let tags = self.tag_id_names()?;
        serde_json::to_value(
            self.stored_tag_rules()?
                .specs_using(&tags, &|expr| registry.display(expr, &tags))?,
        )
        .map_err(|e| e.to_string())
    }

    pub fn compile_tag_rules(
        &self,
        value: &Value,
    ) -> Result<Rules<crate::named_predicates::Atom>, String> {
        let specs: RuleSpecs =
            serde_json::from_value(value.clone()).map_err(|e| format!("invalid tag rules: {e}"))?;
        let tags = self
            .tag_id_names()?
            .into_iter()
            .map(|(id, name)| (name, id))
            .collect();
        let registry = self.predicate_registry("global")?;
        Rules::compile_using(
            specs,
            &tags,
            &|expr| registry.bind(expr, &tags),
            &crate::named_predicates::Atom::Tag,
            &|expr| Ok(registry.resolve(expr)?.positive_with(&|_| true)),
        )
    }

    /// The caller holds an IMMEDIATE transaction for validation and publication together.
    pub fn replace_tag_rules(&self, value: &Value, preview: bool) -> Result<Value, String> {
        let stored = self.compile_tag_rules(value)?;
        let registry = self.predicate_registry("global")?;
        let rules = stored.try_map(&mut |expr| registry.resolve(expr))?;
        let result = self.apply_tag_rules(&rules, preview)?;
        if !preview {
            self.settings_put(
                Self::SETTING_TAG_RULES,
                &serde_json::to_string(&stored).map_err(|e| e.to_string())?,
            )?;
        }
        Ok(result)
    }

    pub fn recompute_tag_rules(&self) -> Result<(), String> {
        self.apply_tag_rules(&self.tag_rules()?, false).map(|_| ())
    }

    fn apply_tag_rules(&self, rules: &Rules, preview: bool) -> Result<Value, String> {
        let mut violations = Vec::new();
        let mut count = 0;
        let mut recomputed = Vec::new();
        for memory in self.all_memories()? {
            let id = Self::parse_id(&memory.id).ok_or("invalid stored memory id")?;
            let original = self.original_tag_ids(id)?;
            let closure = rules.closure(&original)?;
            let failed = rules.violations(&closure.effective);
            if !failed.is_empty() {
                count += 1;
                if violations.len() < 100 {
                    violations
                        .push(json!({"id": memory.id, "summary": memory.summary, "rules": failed}));
                }
            }
            recomputed.push((id, original, closure));
        }
        if preview {
            return Ok(
                json!({"valid": count == 0, "total_violations": count, "violations": violations, "positive_cycles": rules.has_positive_cycles()?}),
            );
        }
        if count != 0 {
            let ids = violations
                .iter()
                .take(10)
                .filter_map(|v| v["id"].as_str())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!("tag rules rejected: {count} existing memories violate constraints ({ids}); preview the rules for details"));
        }
        for (id, original, closure) in recomputed {
            self.replace_effective_tags(id, &original, &closure)?;
        }
        Ok(json!({"saved": true}))
    }

    pub fn validate_tag_constraints(&self, ids: &[i64]) -> Result<(), String> {
        let rules = self.tag_rules()?;
        let failed = rules.violations(ids);
        if failed.is_empty() {
            Ok(())
        } else {
            Err(format!("tag constraints violated: {}", failed.join(", ")))
        }
    }

    fn origin_rows(&self, id: i64) -> Result<Vec<(i64, String, bool, bool)>, String> {
        let mut statement = self
            .conn
            .prepare(sql::MEMORY_TAG_ORIGINS)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn original_tag_ids(&self, id: i64) -> Result<Vec<i64>, String> {
        Ok(self
            .origin_rows(id)?
            .into_iter()
            .filter(|(_, _, original, _)| *original)
            .map(|(id, _, _, _)| id)
            .collect())
    }

    pub fn memory_tag_provenance(&self, id: i64) -> Result<Value, String> {
        let rows = self.origin_rows(id)?;
        let original = rows
            .iter()
            .filter(|(_, _, original, _)| *original)
            .map(|(id, _, _, _)| *id)
            .collect::<Vec<_>>();
        let closure = self.tag_rules()?.closure(&original)?;
        let originals = rows
            .iter()
            .filter(|(_, _, original, _)| *original)
            .map(|(_, name, _, _)| name)
            .collect::<Vec<_>>();
        let derived = rows.iter().filter(|(_, _, _, derived)| *derived).map(|(id, name, _, _)| json!({"tag": name, "rules": closure.derived.get(id).cloned().unwrap_or_default()})).collect::<Vec<_>>();
        Ok(json!({"original_tags": originals, "derived_tags": derived}))
    }

    pub fn replace_effective_tags(
        &self,
        id: i64,
        original: &[i64],
        closure: &crate::tag_rules::Closure,
    ) -> Result<(), String> {
        self.conn
            .execute(sql::MEMORY_CLEAR_TAGS, [id])
            .map_err(|e| e.to_string())?;
        for tag_id in &closure.effective {
            self.conn
                .execute(
                    sql::MEMORY_LINK_TAG,
                    params![
                        id,
                        tag_id,
                        original.contains(tag_id),
                        closure.derived.contains_key(tag_id)
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn assert_tag_deletable(&self, name: &str) -> Result<(), String> {
        let ids = self.tag_ids_for_names(&[name.to_string()])?;
        if let Some(id) = ids.first() {
            if self
                .stored_lifecycle_policy()?
                .rules
                .iter()
                .any(|r| r.predicate.references_tag(*id))
            {
                return Err(format!(
                    "tag '{name}' is referenced by a decay policy; remove that reference first"
                ));
            }
            if self
                .predicate_rows(None)?
                .iter()
                .any(|d| d.predicate.references_tag(*id))
            {
                return Err(format!(
                    "tag '{name}' is referenced by a named predicate; remove that reference first"
                ));
            }
            let rules = self.tag_rules()?;
            let mut refs = rules
                .constraints
                .iter()
                .filter(|r| r.expression.references(*id))
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>();
            refs.extend(
                rules
                    .derivations
                    .iter()
                    .filter(|r| r.references(*id))
                    .map(|r| r.name.as_str()),
            );
            if !refs.is_empty() {
                return Err(format!("tag '{name}' is referenced by rules: {}; remove those rules before deleting the tag", refs.join(", ")));
            }
        }
        Ok(())
    }
}
