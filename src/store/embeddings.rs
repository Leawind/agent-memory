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

    /// Store one memory's vectors (upsert: re-embedding replaces the previous row). The chunks of
    /// one memory always land together, and the memory's rows are cleared first, so a re-embed
    /// with fewer chunks cannot leave stale ones behind.
    pub fn embedding_put(
        &self,
        id: i64,
        model: &str,
        fingerprint: &str,
        vectors: &[Vec<f32>],
    ) -> Result<(), String> {
        if self
            .embedding_cache_key(model, Some(fingerprint))?
            .is_empty()
        {
            self.embedding_delete_model(model)?;
            self.settings_put(&format!("embedding_cache:{model}"), fingerprint)?;
        }
        if vectors.is_empty() {
            return Ok(());
        }
        self.conn
            .execute(sql::EMBEDDING_DELETE_CHUNKS, params![id, model])
            .map_err(|e| e.to_string())?;
        let now = crate::model::now() as i64;
        for (chunk, vec) in vectors.iter().enumerate() {
            self.conn
                .execute(
                    sql::EMBEDDING_PUT,
                    params![
                        id,
                        model,
                        chunk as i64,
                        vec.len() as i64,
                        crate::embed::vec_to_blob(vec),
                        now
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Delete one memory's vector (used when an update invalidates it; silent if absent).
    pub fn embedding_delete(&self, id: i64) -> Result<(), String> {
        self.conn
            .execute(sql::EMBEDDING_DELETE, [id])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// All vectors for the current model, grouped per memory in chunk order, for the in-memory
    /// cosine pass of hybrid search (a memory's score is its best chunk).
    pub fn embeddings_active(
        &self,
        model: &str,
        fingerprint: &str,
    ) -> Result<HashMap<i64, Vec<Vec<f32>>>, String> {
        let model = self.embedding_cache_key(model, Some(fingerprint))?;
        let mut st = self
            .conn
            .prepare_cached(sql::EMBEDDING_ACTIVE_ALL)
            .map_err(|e| e.to_string())?;
        let rows = st
            .query_map([&model], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?))
            })
            .map_err(|e| e.to_string())?;
        let mut out: HashMap<i64, Vec<Vec<f32>>> = HashMap::new();
        for row in rows {
            let (id, blob) = row.map_err(|e| e.to_string())?;
            out.entry(id)
                .or_default()
                .push(crate::embed::blob_to_vec(&blob));
        }
        Ok(out)
    }

    /// Vector identities that actually have cached rows, with row counts (cache-management listing;
    /// identities configured but never backfilled are not in the table — the caller unions them).
    pub fn embedding_cached_models(&self) -> Result<Vec<(String, usize)>, String> {
        let mut st = self
            .conn
            .prepare_cached(sql::EMBEDDING_MODELS_LIST)
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
            .prepare_cached(sql::EMBEDDING_PENDING_BATCH)
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
            .prepare_cached(sql::EMBEDDING_PENDING_FOR)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::test_support::{cleanup, temp_db};

    /// Chunks of one memory round-trip in order, count as one memory for the cache listings, and a
    /// re-embed with fewer chunks replaces the set instead of leaving stale rows behind.
    #[test]
    fn chunks_roundtrip_and_replace_wholesale() {
        let path = temp_db("embedding-chunks");
        cleanup(&path);
        let st = Store::open(&path).unwrap();
        st.insert_memory("s", "body", &[], 1, 1).unwrap();

        st.embedding_put(1, "m", "fp", &[vec![1.0, 0.0], vec![0.0, 1.0]])
            .unwrap();
        let table = st.embeddings_active("m", "fp").unwrap();
        assert_eq!(table[&1], vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
        assert_eq!(st.embedding_embedded_count("m", Some("fp")).unwrap(), 1);
        assert_eq!(st.embedding_pending_count("m", Some("fp")).unwrap(), 0);
        assert_eq!(st.embedding_cached_models().unwrap(), vec![("m".into(), 1)]);

        st.embedding_put(1, "m", "fp", &[vec![0.5, 0.5]]).unwrap();
        assert_eq!(
            st.embeddings_active("m", "fp").unwrap()[&1],
            vec![vec![0.5, 0.5]],
            "the second chunk must not survive the shorter re-embed"
        );
        assert_eq!(st.embedding_embedded_count("m", Some("fp")).unwrap(), 1);

        // A fingerprint mismatch makes the whole model pending again (cache identity unchanged)
        assert_eq!(st.embedding_pending_count("m", Some("other")).unwrap(), 1);
        cleanup(&path);
    }
}
