//! Scene CRUD operations.

use crate::models::{CreateSceneRequest, Scene, UpdateSceneRequest};
use rusqlite::params;

use crate::database::macros::add_field;

use super::super::Database;

/// SQL query for selecting all scene fields.
pub(crate) const SCENE_SELECT: &str =
    "SELECT id, chapter_id, title, summary, text, status, pov, tags, notes, todos,
        word_target, time_point, time_start, time_end, on_timeline, position,
        pov_goal, has_dramatic_conflict, has_change, tension, setup_for_scene_id, payoff_of_scene_id, revision_notes, revision_checklist,
        word_count, created_at, updated_at";

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
             VALUES (?1, ?2, ?3, ?4, '', 'to_write', ?5, 1, ?6, ?7)",
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

        let mut set_clauses = Vec::new();
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        Self::collect_scene_content_fields(req, &mut set_clauses, &mut params);
        Self::collect_scene_meta_fields(req, &mut set_clauses, &mut params);
        Self::collect_scene_revision_fields(req, &mut set_clauses, &mut params);

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

    fn collect_scene_content_fields(
        req: &UpdateSceneRequest,
        set_clauses: &mut Vec<String>,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
    ) {
        add_field!(set_clauses, params, req.title, "title");
        add_field!(set_clauses, params, req.summary, "summary");
        add_field!(set_clauses, params, req.text, "text");

        // Cache word count when text changes
        if let Some(text) = &req.text {
            let plain = crate::database::HTML_TAG_REGEX.replace_all(text, " ");
            let wc = plain.split_whitespace().count() as i32;
            set_clauses.push(format!("word_count = ?{}", params.len() + 1));
            params.push(Box::new(wc));
        }
    }

    fn collect_scene_meta_fields(
        req: &UpdateSceneRequest,
        set_clauses: &mut Vec<String>,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
    ) {
        add_field!(set_clauses, params, req.status, "status");
        add_field!(set_clauses, params, req.pov, "pov");
        add_field!(set_clauses, params, req.tags, "tags");
        add_field!(set_clauses, params, req.notes, "notes");
        add_field!(set_clauses, params, req.todos, "todos");
        add_field!(set_clauses, params, req.word_target, "word_target", int);
        add_field!(set_clauses, params, req.time_point, "time_point");
        add_field!(set_clauses, params, req.time_start, "time_start");
        add_field!(set_clauses, params, req.time_end, "time_end");
        add_field!(set_clauses, params, req.on_timeline, "on_timeline", bool);
        add_field!(set_clauses, params, req.position, "position", int);
    }

    fn collect_scene_revision_fields(
        req: &UpdateSceneRequest,
        set_clauses: &mut Vec<String>,
        params: &mut Vec<Box<dyn rusqlite::ToSql>>,
    ) {
        add_field!(set_clauses, params, req.pov_goal, "pov_goal");
        add_field!(
            set_clauses,
            params,
            req.has_dramatic_conflict,
            "has_dramatic_conflict",
            bool
        );
        add_field!(set_clauses, params, req.has_change, "has_change", bool);
        add_field!(set_clauses, params, req.tension, "tension");
        add_field!(
            set_clauses,
            params,
            req.setup_for_scene_id,
            "setup_for_scene_id"
        );
        add_field!(
            set_clauses,
            params,
            req.payoff_of_scene_id,
            "payoff_of_scene_id"
        );
        add_field!(set_clauses, params, req.revision_notes, "revision_notes");
        add_field!(
            set_clauses,
            params,
            req.revision_checklist,
            "revision_checklist"
        );
    }

    pub fn delete_scene(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        self.run_in_transaction(|| {
            // Clean up junction tables to avoid orphaned records
            self.cleanup_scene_junctions(id)?;

            // Soft-delete the scene itself
            self.conn
                .execute(
                    "UPDATE scenes SET deleted_at = ?1 WHERE id = ?2",
                    params![now, id],
                )
                .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn reorder_scenes(&self, chapter_id: &str, ids: &[String]) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        self.run_in_transaction(|| {
            for (i, id) in ids.iter().enumerate() {
                self.conn
                    .execute(
                        "UPDATE scenes SET position = ?1, updated_at = ?2 WHERE id = ?3 AND chapter_id = ?4",
                        params![i as i32, now, id, chapter_id],
                    )
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        })
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
