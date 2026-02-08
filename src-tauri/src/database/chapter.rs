//! Chapter CRUD operations

use crate::models::{Chapter, CreateChapterRequest, UpdateChapterRequest};
use rusqlite::params;

use super::macros::add_field;
use super::Database;

impl Database {
    pub fn create_chapter(&self, req: &CreateChapterRequest) -> Result<Chapter, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let position = req.position.unwrap_or_else(|| {
            self.conn
                .query_row(
                    "SELECT COALESCE(MAX(position), 0) + 1 FROM chapters WHERE deleted_at IS NULL",
                    [],
                    |row| row.get(0),
                )
                .unwrap_or(1)
        });

        self.conn
            .execute(
                "INSERT INTO chapters (id, title, summary, status, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'planned', ?4, ?5, ?6)",
                params![id, req.title, req.summary, position, now, now],
            )
            .map_err(|e| e.to_string())?;

        self.get_chapter(&id)
    }

    pub fn get_chapters(&self) -> Result<Vec<Chapter>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, summary, status, notes, position, created_at, updated_at
             FROM chapters WHERE deleted_at IS NULL ORDER BY position",
            )
            .map_err(|e| e.to_string())?;

        let chapters = stmt
            .query_map([], Self::map_chapter)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(chapters)
    }

    pub(crate) fn map_chapter(row: &rusqlite::Row) -> rusqlite::Result<Chapter> {
        Ok(Chapter {
            id: row.get(0)?,
            title: row.get(1)?,
            summary: row.get(2)?,
            status: row.get(3)?,
            notes: row.get(4)?,
            position: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }

    pub fn get_chapter(&self, id: &str) -> Result<Chapter, String> {
        self.conn
            .query_row(
                "SELECT id, title, summary, status, notes, position, created_at, updated_at
             FROM chapters WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                Self::map_chapter,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_chapter(&self, id: &str, req: &UpdateChapterRequest) -> Result<Chapter, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        add_field!(set_clauses, params_vec, req.title, "title");
        add_field!(set_clauses, params_vec, req.summary, "summary");
        add_field!(set_clauses, params_vec, req.status, "status");
        add_field!(set_clauses, params_vec, req.notes, "notes");
        add_field!(set_clauses, params_vec, req.position, "position", int);

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE chapters SET {} WHERE id = ?{}",
                set_clauses.join(", "),
                id_param_idx
            );

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            let mut all_params = params_refs;
            all_params.push(&id);

            self.conn
                .execute(&query, all_params.as_slice())
                .map_err(|e| e.to_string())?;
        }

        self.get_chapter(id)
    }

    pub fn delete_chapter(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        let scene_ids = self.get_chapter_scene_ids(id)?;

        self.run_in_transaction(|| {
            self.cleanup_scene_junctions_batch(&scene_ids)?;

            self.conn
                .execute(
                    "UPDATE chapters SET deleted_at = ?1 WHERE id = ?2",
                    params![now, id],
                )
                .map_err(|e| e.to_string())?;

            // Soft-delete all scenes in this chapter
            self.conn
                .execute(
                    "UPDATE scenes SET deleted_at = ?1 WHERE chapter_id = ?2",
                    params![now, id],
                )
                .map_err(|e| e.to_string())?;

            Ok(())
        })
    }

    fn get_chapter_scene_ids(&self, chapter_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL")
            .map_err(|e| e.to_string())?;
        let ids = stmt
            .query_map(params![chapter_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(ids)
    }

    /// Removes all junction table entries for a batch of scenes (6 queries instead of 6×N).
    fn cleanup_scene_junctions_batch(&self, scene_ids: &[String]) -> Result<(), String> {
        if scene_ids.is_empty() {
            return Ok(());
        }

        const JUNCTION_TABLES: &[&str] = &[
            "canonical_associations",
            "auto_link_dismissed",
            "scene_arcs",
            "event_scenes",
            "issue_scenes",
            "scene_steps",
            "annotations",
        ];

        let placeholders: String = (1..=scene_ids.len())
            .map(|i| format!("?{}", i))
            .collect::<Vec<_>>()
            .join(", ");
        let params: Vec<&dyn rusqlite::ToSql> =
            scene_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();

        for table in JUNCTION_TABLES {
            let table = Self::validate_table_name(table)?;
            self.conn
                .execute(
                    &format!("DELETE FROM {} WHERE scene_id IN ({})", table, placeholders),
                    params.as_slice(),
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Removes all junction table entries for a given scene.
    pub(crate) fn cleanup_scene_junctions(&self, scene_id: &str) -> Result<(), String> {
        const JUNCTION_TABLES: &[&str] = &[
            "canonical_associations",
            "auto_link_dismissed",
            "scene_arcs",
            "event_scenes",
            "issue_scenes",
            "scene_steps",
            "annotations",
        ];
        for table in JUNCTION_TABLES {
            let table = Self::validate_table_name(table)?;
            self.conn
                .execute(
                    &format!("DELETE FROM {} WHERE scene_id = ?1", table),
                    params![scene_id],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn reorder_chapters(&self, ids: &[String]) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        self.run_in_transaction(|| {
            for (i, id) in ids.iter().enumerate() {
                self.conn
                    .execute(
                        "UPDATE chapters SET position = ?1, updated_at = ?2 WHERE id = ?3",
                        params![i as i32, now, id],
                    )
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        })
    }
}
