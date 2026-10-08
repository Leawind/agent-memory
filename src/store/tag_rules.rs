//! Atomic publication and write validation for administrator-managed tag constraints.

use super::Store;
use crate::{
    sql,
    tag_rules::{RuleSpecs, Rules},
};
use serde_json::{json, Value};
use std::collections::HashMap;

impl Store {
    pub const SETTING_TAG_RULES: &'static str = "tag_rules";

    fn tag_id_names(&self) -> Result<HashMap<i64, String>, String> {
        let mut statement = self
            .conn
            .prepare(sql::TAG_EXPORT_ALL)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn tag_rules(&self) -> Result<Rules, String> {
        match self.settings_get(Self::SETTING_TAG_RULES)? {
            None => Ok(Rules::default()),
            Some(raw) => serde_json::from_str(&raw).map_err(|e| format!("corrupt tag rules: {e}")),
        }
    }

    pub fn tag_rules_view(&self) -> Result<Value, String> {
        serde_json::to_value(self.tag_rules()?.specs(&self.tag_id_names()?)?)
            .map_err(|e| e.to_string())
    }

    pub fn compile_tag_rules(&self, value: &Value) -> Result<Rules, String> {
        let specs: RuleSpecs =
            serde_json::from_value(value.clone()).map_err(|e| format!("invalid tag rules: {e}"))?;
        let tags = self
            .tag_id_names()?
            .into_iter()
            .map(|(id, name)| (name, id))
            .collect();
        Rules::compile(specs, &tags)
    }

    /// The caller holds an IMMEDIATE transaction for validation and publication together.
    pub fn replace_tag_rules(&self, value: &Value, preview: bool) -> Result<Value, String> {
        let rules = self.compile_tag_rules(value)?;
        let tags: HashMap<_, _> = self
            .tag_id_names()?
            .into_iter()
            .map(|(id, name)| (name, id))
            .collect();
        let mut violations = Vec::new();
        let mut count = 0;
        for memory in self.all_memories()? {
            let ids = memory
                .tags
                .iter()
                .map(|name| tags[name])
                .collect::<Vec<_>>();
            let failed = rules.violations(&ids);
            if !failed.is_empty() {
                count += 1;
                if violations.len() < 100 {
                    violations
                        .push(json!({"id": memory.id, "summary": memory.summary, "rules": failed}));
                }
            }
        }
        if preview {
            return Ok(
                json!({"valid": count == 0, "total_violations": count, "violations": violations}),
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
        self.settings_put(
            Self::SETTING_TAG_RULES,
            &serde_json::to_string(&rules).map_err(|e| e.to_string())?,
        )?;
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

    pub fn assert_tag_deletable(&self, name: &str) -> Result<(), String> {
        let ids = self.tag_ids_for_names(&[name.to_string()])?;
        if let Some(id) = ids.first() {
            let rules = self.tag_rules()?;
            let refs = rules
                .constraints
                .iter()
                .filter(|r| r.expression.references(*id))
                .map(|r| r.name.as_str())
                .collect::<Vec<_>>();
            if !refs.is_empty() {
                return Err(format!("tag '{name}' is referenced by rules: {}; remove those rules before deleting the tag", refs.join(", ")));
            }
        }
        Ok(())
    }
}
