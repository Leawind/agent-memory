use super::Store;
use crate::{lifecycle::Metadata, sql};
use rusqlite::params;
use std::collections::HashMap;

fn decode(row: &rusqlite::Row<'_>, start: usize) -> rusqlite::Result<Metadata> {
    Ok(Metadata {
        expires_at: row.get::<_, Option<i64>>(start)?.map(|t| t as u64),
        archived_at: row.get::<_, Option<i64>>(start + 1)?.map(|t| t as u64),
        pinned: row.get(start + 2)?,
    })
}

impl Store {
    pub fn lifecycle_get(&self, id: i64) -> Result<Metadata, String> {
        match self
            .conn
            .query_row(sql::LIFECYCLE_GET, [id], |row| decode(row, 0))
        {
            Ok(meta) => Ok(meta),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Metadata::default()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn lifecycle_all(&self) -> Result<HashMap<i64, Metadata>, String> {
        let mut statement = self
            .conn
            .prepare(sql::LIFECYCLE_ALL)
            .map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, decode(row, 1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
    }

    pub fn lifecycle_put(&self, id: i64, meta: &Metadata) -> Result<(), String> {
        meta.validate()?;
        self.conn
            .execute(
                sql::LIFECYCLE_PUT,
                params![
                    id,
                    meta.expires_at.map(|t| t as i64),
                    meta.archived_at.map(|t| t as i64),
                    meta.pinned
                ],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}
