//! Storage for semantic-search vectors (derived data): upsert / delete / full load / backfill queue.

use crate::sql;
use rusqlite::params;
use std::collections::HashMap;

use super::Store;

impl Store {
    /// Store one vector (upsert: re-embedding the same memory overwrites the old row).
    pub fn embedding_put(&self, id: i64, model: &str, vec: &[f32]) -> Result<(), String> {
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
    pub fn embeddings_active(&self, model: &str) -> Result<HashMap<i64, Vec<f32>>, String> {
        let mut st = self
            .conn
            .prepare(sql::EMBEDDING_ACTIVE_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([model], |r| {
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

    /// One pending backfill batch: memories lacking vectors for the current model, id ascending (with title and content for embedding).
    pub fn embedding_pending_batch(
        &self,
        model: &str,
        limit: usize,
    ) -> Result<Vec<(i64, String, String)>, String> {
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

    pub fn embedding_pending_count(&self, model: &str) -> Result<usize, String> {
        self.conn
            .query_row(sql::EMBEDDING_PENDING_COUNT, [model], |r| {
                r.get::<_, i64>(0)
            })
            .map(|n| n as usize)
            .map_err(|e| e.to_string())
    }

    pub fn embedding_embedded_count(&self, model: &str) -> Result<usize, String> {
        self.conn
            .query_row(sql::EMBEDDING_EMBEDDED_COUNT, [model], |r| {
                r.get::<_, i64>(0)
            })
            .map(|n| n as usize)
            .map_err(|e| e.to_string())
    }
}
