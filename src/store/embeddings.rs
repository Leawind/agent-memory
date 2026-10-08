//! Storage for semantic-search vectors (derived data): upsert / delete / full load / backfill queue.

use crate::sql;
use rusqlite::params;
use std::collections::HashMap;

use super::Store;

impl Store {
    /// Derived, per-cache metadata lives beside settings so no schema migration is needed.
    /// A missing or mismatched fingerprint makes all vectors pending under the same ID.
    fn embedding_cache_key(
        &self,
        model: &str,
        fingerprint: Option<&str>,
    ) -> Result<String, String> {
        match fingerprint {
            Some(expected)
                if self
                    .settings_get(&format!("embedding_cache:{model}"))?
                    .as_deref()
                    != Some(expected) =>
            {
                Ok(String::new())
            }
            _ => Ok(model.to_string()),
        }
    }

    /// Store one vector (upsert: re-embedding the same memory overwrites the old row).
    pub fn embedding_put(
        &self,
        id: i64,
        model: &str,
        fingerprint: &str,
        vec: &[f32],
    ) -> Result<(), String> {
        if self
            .embedding_cache_key(model, Some(fingerprint))?
            .is_empty()
        {
            self.embedding_delete_model(model)?;
            self.settings_put(&format!("embedding_cache:{model}"), fingerprint)?;
        }
        self.conn
            .execute(
                sql::EMBEDDING_PUT,
                params![
                    id,
                    model,
                    vec.len() as i64,
                    crate::embed::vec_to_blob(vec),
                    crate::model::now() as i64
                ],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Delete one memory's vector (used when an update invalidates it; silent if absent).
    pub fn embedding_delete(&self, id: i64) -> Result<(), String> {
        self.conn
            .execute(sql::EMBEDDING_DELETE, [id])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// All vectors for the current model, for the in-memory cosine pass of hybrid search.
    pub fn embeddings_active(
        &self,
        model: &str,
        fingerprint: &str,
    ) -> Result<HashMap<i64, Vec<f32>>, String> {
        let model = self.embedding_cache_key(model, Some(fingerprint))?;
        let mut st = self
            .conn
            .prepare(sql::EMBEDDING_ACTIVE_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([&model], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let mut out = HashMap::new();
        for row in rows {
            let (id, blob) = row.map_err(|e| e.to_string())?;
            out.insert(id, crate::embed::blob_to_vec(&blob));
        }
        Ok(out)
    }

    /// Vector identities that actually have cached rows, with row counts (cache-management listing;
    /// identities configured but never backfilled are not in the table — the caller unions them).
    pub fn embedding_cached_models(&self) -> Result<Vec<(String, usize)>, String> {
        let mut st = self
            .conn
            .prepare(sql::EMBEDDING_MODELS_LIST)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Delete every cached vector of one identity (per-model cache management); returns the row count.
    pub fn embedding_delete_model(&self, model: &str) -> Result<usize, String> {
        let deleted = self
            .conn
            .execute(sql::EMBEDDING_DELETE_MODEL, [model])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(sql::SETTINGS_DELETE, [format!("embedding_cache:{model}")])
            .map_err(|e| e.to_string())?;
        Ok(deleted)
    }

    /// One pending backfill batch: memories lacking vectors for the current model, id ascending (with title and content for embedding).
    pub fn embedding_pending_batch(
        &self,
        model: &str,
        fingerprint: &str,
        limit: usize,
    ) -> Result<Vec<(i64, String, String)>, String> {
        let model = self.embedding_cache_key(model, Some(fingerprint))?;
        let mut st = self
            .conn
            .prepare(sql::EMBEDDING_PENDING_BATCH)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map(params![model, limit as i64], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// One specific memory's pending row for the current model (`None` once its vector is stored):
    /// lets the create-time dedup hint embed exactly the new memory instead of depending on the
    /// asynchronous backfill having reached it.
    pub fn embedding_pending_for(
        &self,
        model: &str,
        fingerprint: &str,
        id: i64,
    ) -> Result<Option<(i64, String, String)>, String> {
        let model = self.embedding_cache_key(model, Some(fingerprint))?;
        let mut st = self
            .conn
            .prepare(sql::EMBEDDING_PENDING_FOR)
            .map_err(|e| e.to_string())?;
        let mut rows = st
            .query_map(params![model, id], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;
        match rows.next() {
            Some(row) => row.map(Some).map_err(|e| e.to_string()),
            None => Ok(None),
        }
    }

    pub fn embedding_pending_count(
        &self,
        model: &str,
        fingerprint: Option<&str>,
    ) -> Result<usize, String> {
        let model = self.embedding_cache_key(model, fingerprint)?;
        self.conn
            .query_row(sql::EMBEDDING_PENDING_COUNT, [&model], |r| {
                r.get::<_, i64>(0)
            })
            .map(|n| n as usize)
            .map_err(|e| e.to_string())
    }

    pub fn embedding_embedded_count(
        &self,
        model: &str,
        fingerprint: Option<&str>,
    ) -> Result<usize, String> {
        let model = self.embedding_cache_key(model, fingerprint)?;
        self.conn
            .query_row(sql::EMBEDDING_EMBEDDED_COUNT, [&model], |r| {
                r.get::<_, i64>(0)
            })
            .map(|n| n as usize)
            .map_err(|e| e.to_string())
    }
}
