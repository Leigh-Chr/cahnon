//! Chapter CRUD operations

use crate::models::{Chapter, CreateChapterRequest, UpdateChapterRequest};
use rusqlite::params;

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
        let (id, title, summary, status) = Self::map_chapter_core(row)?;
        let (notes, position, created_at, updated_at) = Self::map_chapter_meta(row)?;
        Ok(Chapter {
            id,
            title,
            summary,
            status,
            notes,
            position,
            created_at,
            updated_at,
        })
    }

    fn map_chapter_core(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, Option<String>, String)> {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }

    fn map_chapter_meta(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(Option<String>, i32, String, String)> {
        Ok((row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?))
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

        macro_rules! add_field {
            ($field:expr, $column:literal) => {
                if let Some(val) = &$field {
                    set_clauses.push(format!("{} = ?{}", $column, params_vec.len() + 1));
                    params_vec.push(Box::new(val.clone()));
                }
            };
            ($field:expr, $column:literal, int) => {
                if let Some(val) = $field {
                    set_clauses.push(format!("{} = ?{}", $column, params_vec.len() + 1));
                    params_vec.push(Box::new(val));
                }
            };
        }

        add_field!(req.title, "title");
        add_field!(req.summary, "summary");
        add_field!(req.status, "status");
        add_field!(req.notes, "notes");
        add_field!(req.position, "position", int);

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

        // Collect scene IDs in this chapter for junction cleanup
        let scene_ids: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT id FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL")
                .map_err(|e| e.to_string())?;
            let result = stmt
                .query_map(params![id], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            result
        };

        // Clean up junction tables for all scenes in this chapter
        for scene_id in &scene_ids {
            self.conn
                .execute(
                    "DELETE FROM canonical_associations WHERE scene_id = ?1",
                    params![scene_id],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM scene_arcs WHERE scene_id = ?1",
                    params![scene_id],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM event_scenes WHERE scene_id = ?1",
                    params![scene_id],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM issue_scenes WHERE scene_id = ?1",
                    params![scene_id],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM scene_steps WHERE scene_id = ?1",
                    params![scene_id],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM annotations WHERE scene_id = ?1",
                    params![scene_id],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM name_mentions WHERE scene_id = ?1",
                    params![scene_id],
                )
                .map_err(|e| e.to_string())?;
        }

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
    }

    pub fn reorder_chapters(&self, ids: &[String]) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute("BEGIN TRANSACTION", [])
            .map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            for (i, id) in ids.iter().enumerate() {
                self.conn
                    .execute(
                        "UPDATE chapters SET position = ?1, updated_at = ?2 WHERE id = ?3",
                        params![i as i32, now, id],
                    )
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }
}
