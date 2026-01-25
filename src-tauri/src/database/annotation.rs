//! Annotation operations

use crate::models::{Annotation, CreateAnnotationRequest, UpdateAnnotationRequest};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn create_annotation(&self, req: &CreateAnnotationRequest) -> Result<Annotation, String> {
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

    fn map_annotation(row: &rusqlite::Row) -> rusqlite::Result<Annotation> {
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

        if let Some(content) = &req.content {
            self.conn
                .execute(
                    "UPDATE annotations SET content = ?1, updated_at = ?2 WHERE id = ?3",
                    params![content, now, id],
                )
                .map_err(|e| e.to_string())?;
        }
        if let Some(status) = &req.status {
            self.conn
                .execute(
                    "UPDATE annotations SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![status, now, id],
                )
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
