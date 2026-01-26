//! Saved filter operations

use crate::models::{CreateSavedFilterRequest, SavedFilter, UpdateSavedFilterRequest};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn create_saved_filter(
        &self,
        req: &CreateSavedFilterRequest,
    ) -> Result<SavedFilter, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO saved_filters (id, name, filter_type, filter_data, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, req.name, req.filter_type, req.filter_data, now, now],
            )
            .map_err(|e| e.to_string())?;

        self.get_saved_filter(&id)
    }

    pub fn get_saved_filters(&self, filter_type: Option<&str>) -> Result<Vec<SavedFilter>, String> {
        let (query, param_value);
        if let Some(ft) = filter_type {
            query = "SELECT id, name, filter_type, filter_data, created_at, updated_at
                     FROM saved_filters WHERE filter_type = ?1 ORDER BY name";
            param_value = Some(ft.to_string());
        } else {
            query = "SELECT id, name, filter_type, filter_data, created_at, updated_at
                     FROM saved_filters ORDER BY name";
            param_value = None;
        }

        let mut stmt = self.conn.prepare(query).map_err(|e| e.to_string())?;

        let filters = if let Some(ref ftype) = param_value {
            stmt.query_map(params![ftype], Self::map_saved_filter)
        } else {
            stmt.query_map([], Self::map_saved_filter)
        }
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(filters)
    }

    fn map_saved_filter(row: &rusqlite::Row) -> rusqlite::Result<SavedFilter> {
        Ok(SavedFilter {
            id: row.get(0)?,
            name: row.get(1)?,
            filter_type: row.get(2)?,
            filter_data: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }

    pub fn get_saved_filter(&self, id: &str) -> Result<SavedFilter, String> {
        self.conn
            .query_row(
                "SELECT id, name, filter_type, filter_data, created_at, updated_at
                 FROM saved_filters WHERE id = ?1",
                params![id],
                Self::map_saved_filter,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_saved_filter(
        &self,
        id: &str,
        req: &UpdateSavedFilterRequest,
    ) -> Result<SavedFilter, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref name) = req.name {
            set_clauses.push(format!("name = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(name.clone()));
        }
        if let Some(ref data) = req.filter_data {
            set_clauses.push(format!("filter_data = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(data.clone()));
        }

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE saved_filters SET {} WHERE id = ?{}",
                set_clauses.join(", "),
                id_idx
            );

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            let mut all_params = params_refs;
            all_params.push(&id);

            self.conn
                .execute(&query, all_params.as_slice())
                .map_err(|e| e.to_string())?;
        }

        self.get_saved_filter(id)
    }

    pub fn delete_saved_filter(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM saved_filters WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
