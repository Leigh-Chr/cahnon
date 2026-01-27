//! Annotation operations

use crate::models::{Annotation, CreateAnnotationRequest, UpdateAnnotationRequest};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn create_annotation(&self, req: &CreateAnnotationRequest) -> Result<Annotation, String> {
        if req.start_offset < 0 {
            return Err("Start offset cannot be negative".to_string());
        }
        if req.end_offset < 0 {
            return Err("End offset cannot be negative".to_string());
        }
        if req.end_offset <= req.start_offset {
            return Err("End offset must be greater than start offset".to_string());
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO annotations (id, scene_id, start_offset, end_offset, annotation_type, content, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    id,
                    req.scene_id,
                    req.start_offset,
                    req.end_offset,
                    req.annotation_type.as_deref().unwrap_or("comment"),
                    req.content,
                    "open",
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_annotation(&id)
    }

    pub fn get_annotations(&self, scene_id: &str) -> Result<Vec<Annotation>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, scene_id, start_offset, end_offset, annotation_type, content, status, created_at, updated_at
             FROM annotations WHERE scene_id = ?1 ORDER BY start_offset",
            )
            .map_err(|e| e.to_string())?;

        let annotations = stmt
            .query_map(params![scene_id], Self::map_annotation)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(annotations)
    }

    pub(crate) fn map_annotation(row: &rusqlite::Row) -> rusqlite::Result<Annotation> {
        let (id, scene_id, start_offset, end_offset) = Self::map_annotation_location(row)?;
        let (annotation_type, content, status, created_at, updated_at) =
            Self::map_annotation_content(row)?;
        Ok(Annotation {
            id,
            scene_id,
            start_offset,
            end_offset,
            annotation_type,
            content,
            status,
            created_at,
            updated_at,
        })
    }

    fn map_annotation_location(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, i32, i32)> {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }

    #[allow(clippy::type_complexity)]
    fn map_annotation_content(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, String, String, String)> {
        Ok((
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            row.get(8)?,
        ))
    }

    pub fn get_annotation(&self, id: &str) -> Result<Annotation, String> {
        self.conn
            .query_row(
                "SELECT id, scene_id, start_offset, end_offset, annotation_type, content, status, created_at, updated_at
             FROM annotations WHERE id = ?1",
                params![id],
                Self::map_annotation,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_annotation(
        &self,
        id: &str,
        req: &UpdateAnnotationRequest,
    ) -> Result<Annotation, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(ref content) = req.content {
            set_clauses.push(format!("content = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(content.clone()));
        }
        if let Some(ref status) = req.status {
            set_clauses.push(format!("status = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(status.clone()));
        }

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE annotations SET {} WHERE id = ?{}",
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

        self.get_annotation(id)
    }

    pub fn delete_annotation(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM annotations WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
