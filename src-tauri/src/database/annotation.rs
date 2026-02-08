//! Annotation operations

use crate::models::{
    Annotation, AnnotationOffsetUpdate, CreateAnnotationRequest, UpdateAnnotationRequest,
};
use rusqlite::params;

use super::Database;

/// Column list for annotation SELECT queries (11 columns).
const ANNOTATION_COLUMNS: &str = "id, scene_id, start_offset, end_offset, annotation_type, content, status, annotated_text, orphaned, created_at, updated_at";

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
        let annotated_text = req.annotated_text.as_deref().unwrap_or("");

        self.conn
            .execute(
                "INSERT INTO annotations (id, scene_id, start_offset, end_offset, annotation_type, content, status, annotated_text, orphaned, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    id,
                    req.scene_id,
                    req.start_offset,
                    req.end_offset,
                    req.annotation_type.as_deref().unwrap_or("comment"),
                    req.content,
                    "open",
                    annotated_text,
                    0,
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
            .prepare(&format!(
                "SELECT {} FROM annotations WHERE scene_id = ?1 ORDER BY start_offset",
                ANNOTATION_COLUMNS
            ))
            .map_err(|e| e.to_string())?;

        let annotations = stmt
            .query_map(params![scene_id], Self::map_annotation)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(annotations)
    }

    pub(crate) fn map_annotation(row: &rusqlite::Row) -> rusqlite::Result<Annotation> {
        Ok(Annotation {
            id: row.get(0)?,
            scene_id: row.get(1)?,
            start_offset: row.get(2)?,
            end_offset: row.get(3)?,
            annotation_type: row.get(4)?,
            content: row.get(5)?,
            status: row.get(6)?,
            annotated_text: row.get(7)?,
            orphaned: row.get::<_, i32>(8)? != 0,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
        })
    }

    pub fn get_annotation(&self, id: &str) -> Result<Annotation, String> {
        self.conn
            .query_row(
                &format!(
                    "SELECT {} FROM annotations WHERE id = ?1",
                    ANNOTATION_COLUMNS
                ),
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
        if let Some(start_offset) = req.start_offset {
            set_clauses.push(format!("start_offset = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(start_offset));
        }
        if let Some(end_offset) = req.end_offset {
            set_clauses.push(format!("end_offset = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(end_offset));
        }
        if let Some(orphaned) = req.orphaned {
            set_clauses.push(format!("orphaned = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(orphaned as i32));
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

    pub fn batch_update_annotation_offsets(
        &self,
        updates: &[AnnotationOffsetUpdate],
    ) -> Result<(), String> {
        if updates.is_empty() {
            return Ok(());
        }
        let now = chrono::Utc::now().to_rfc3339();
        let mut stmt = self
            .conn
            .prepare(
                "UPDATE annotations SET start_offset = ?1, end_offset = ?2, annotated_text = ?3, orphaned = 0, updated_at = ?4 WHERE id = ?5",
            )
            .map_err(|e| e.to_string())?;
        for update in updates {
            stmt.execute(params![
                update.start_offset,
                update.end_offset,
                update.annotated_text,
                now,
                update.id
            ])
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn delete_annotation(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM annotations WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
