//! Scene CRUD operations.

use crate::models::{CreateSceneRequest, Scene, UpdateSceneRequest};
use rusqlite::params;

use super::super::Database;

/// SQL query for selecting all scene fields.
pub(crate) const SCENE_SELECT: &str =
    "SELECT id, chapter_id, title, summary, text, status, pov, tags, notes, todos,
        word_target, time_point, time_start, time_end, on_timeline, position,
        pov_goal, has_conflict, has_change, tension, setup_for_scene_id, payoff_of_scene_id, revision_notes, revision_checklist,
        created_at, updated_at";

impl Database {
    pub fn create_scene(&self, req: &CreateSceneRequest) -> Result<Scene, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let position = req.position.unwrap_or_else(|| {
            self.conn
                .query_row(
                    "SELECT COALESCE(MAX(position), 0) + 1 FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL",
                    params![req.chapter_id],
                    |row| row.get(0),
                )
                .unwrap_or(1)
        });

        self.conn
            .execute(
                "INSERT INTO scenes (id, chapter_id, title, summary, text, status, position, on_timeline, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, '', 'to write', ?5, 1, ?6, ?7)",
                params![id, req.chapter_id, req.title, req.summary, position, now, now],
            )
            .map_err(|e| e.to_string())?;

        self.get_scene(&id)
    }

    pub fn get_scenes(&self, chapter_id: &str) -> Result<Vec<Scene>, String> {
        let query = format!(
            "{} FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL ORDER BY position",
            SCENE_SELECT
        );
        let mut stmt = self.conn.prepare(&query).map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map(params![chapter_id], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
    }

    pub fn get_scene(&self, id: &str) -> Result<Scene, String> {
        let query = format!(
            "{} FROM scenes WHERE id = ?1 AND deleted_at IS NULL",
            SCENE_SELECT
        );
        self.conn
            .query_row(&query, params![id], Self::map_scene)
            .map_err(|e| e.to_string())
    }

    pub fn update_scene(&self, id: &str, req: &UpdateSceneRequest) -> Result<Scene, String> {
        let now = chrono::Utc::now().to_rfc3339();

        // Save to history before updating text
        if req.text.is_some() {
            self.save_scene_to_history(id, &now)?;
        }

        // Build dynamic update query
        let mut set_clauses = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Helper macro to add optional field updates
        macro_rules! add_field {
            ($field:expr, $column:literal) => {
                if let Some(val) = &$field {
                    set_clauses.push(format!("{} = ?{}", $column, params.len() + 1));
                    params.push(Box::new(val.clone()));
                }
            };
            ($field:expr, $column:literal, bool) => {
                if let Some(val) = $field {
                    set_clauses.push(format!("{} = ?{}", $column, params.len() + 1));
                    params.push(Box::new(val as i32));
                }
            };
            ($field:expr, $column:literal, int) => {
                if let Some(val) = $field {
                    set_clauses.push(format!("{} = ?{}", $column, params.len() + 1));
                    params.push(Box::new(val));
                }
            };
        }

        // Add all optional fields
        add_field!(req.title, "title");
        add_field!(req.summary, "summary");
        add_field!(req.text, "text");
        add_field!(req.status, "status");
        add_field!(req.pov, "pov");
        add_field!(req.tags, "tags");
        add_field!(req.notes, "notes");
        add_field!(req.todos, "todos");
        add_field!(req.word_target, "word_target", int);
        add_field!(req.time_point, "time_point");
        add_field!(req.time_start, "time_start");
        add_field!(req.time_end, "time_end");
        add_field!(req.on_timeline, "on_timeline", bool);
        add_field!(req.position, "position", int);
        // Revision fields
        add_field!(req.pov_goal, "pov_goal");
        add_field!(req.has_conflict, "has_conflict", bool);
        add_field!(req.has_change, "has_change", bool);
        add_field!(req.tension, "tension");
        add_field!(req.setup_for_scene_id, "setup_for_scene_id");
        add_field!(req.payoff_of_scene_id, "payoff_of_scene_id");
        add_field!(req.revision_notes, "revision_notes");
        add_field!(req.revision_checklist, "revision_checklist");

        // Only execute if there are fields to update
        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params.len() + 1));
            params.push(Box::new(now));

            let id_param_idx = params.len() + 1;
            let query = format!(
                "UPDATE scenes SET {} WHERE id = ?{}",
                set_clauses.join(", "),
                id_param_idx
            );

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params.iter().map(|p| p.as_ref()).collect();
            let mut all_params = params_refs;
            all_params.push(&id);

            self.conn
                .execute(&query, all_params.as_slice())
                .map_err(|e| e.to_string())?;
        }

        self.get_scene(id)
    }

    pub fn delete_scene(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        // Clean up junction tables to avoid orphaned records
        self.conn
            .execute(
                "DELETE FROM canonical_associations WHERE scene_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM scene_arcs WHERE scene_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM event_scenes WHERE scene_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM issue_scenes WHERE scene_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM scene_steps WHERE scene_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM annotations WHERE scene_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM name_mentions WHERE scene_id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        // Soft-delete the scene itself
        self.conn
            .execute(
                "UPDATE scenes SET deleted_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reorder_scenes(&self, chapter_id: &str, ids: &[String]) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute("BEGIN TRANSACTION", [])
            .map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            for (i, id) in ids.iter().enumerate() {
                self.conn
                    .execute(
                        "UPDATE scenes SET position = ?1, updated_at = ?2 WHERE id = ?3 AND chapter_id = ?4",
                        params![i as i32, now, id, chapter_id],
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

    pub fn move_scene_to_chapter(
        &self,
        scene_id: &str,
        target_chapter_id: &str,
        position: i32,
    ) -> Result<Scene, String> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE scenes SET chapter_id = ?1, position = ?2, updated_at = ?3 WHERE id = ?4",
                params![target_chapter_id, position, now, scene_id],
            )
            .map_err(|e| e.to_string())?;
        self.get_scene(scene_id)
    }

    pub fn get_all_scenes_for_timeline(&self) -> Result<Vec<Scene>, String> {
        let query = format!(
            "{} FROM scenes WHERE deleted_at IS NULL AND on_timeline = 1 AND (time_point IS NOT NULL OR time_start IS NOT NULL) ORDER BY COALESCE(time_point, time_start)",
            SCENE_SELECT
        );
        let mut stmt = self.conn.prepare(&query).map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map([], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
    }
}
