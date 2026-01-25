//! Deleted items operations (Trash)

use crate::models::{Chapter, Scene};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn get_deleted_scenes(&self) -> Result<Vec<Scene>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, chapter_id, title, summary, text, status, pov, tags, notes, todos,
                    word_target, time_point, time_start, time_end, on_timeline, position,
                    pov_goal, has_conflict, has_change, tension, setup_for_scene_id, payoff_of_scene_id, revision_notes, revision_checklist,
                    created_at, updated_at
             FROM scenes WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map([], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
    }

    pub fn restore_scene(&self, id: &str) -> Result<Scene, String> {
        self.conn
            .execute(
                "UPDATE scenes SET deleted_at = NULL WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        self.get_scene(id)
    }

    pub fn get_deleted_chapters(&self) -> Result<Vec<Chapter>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, summary, status, notes, position, created_at, updated_at
             FROM chapters WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let chapters = stmt
            .query_map([], Self::map_chapter)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(chapters)
    }

    pub fn restore_chapter(&self, id: &str) -> Result<Chapter, String> {
        self.conn
            .execute(
                "UPDATE chapters SET deleted_at = NULL WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "UPDATE scenes SET deleted_at = NULL WHERE chapter_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        self.get_chapter(id)
    }
}
