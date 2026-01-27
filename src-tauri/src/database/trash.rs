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
                    word_count, created_at, updated_at
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

    /// Purges items that have been in trash for more than 30 days.
    /// Returns (scenes_purged, chapters_purged).
    pub fn purge_expired_trash(&self) -> Result<(usize, usize), String> {
        let threshold = (chrono::Utc::now() - chrono::Duration::days(30)).to_rfc3339();

        // Get expired scene IDs for junction cleanup
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM scenes WHERE deleted_at IS NOT NULL AND deleted_at < ?1")
            .map_err(|e| e.to_string())?;
        let expired_scene_ids: Vec<String> = stmt
            .query_map(params![threshold], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        drop(stmt);

        // Clean junction tables for expired scenes
        for sid in &expired_scene_ids {
            let _ = self.conn.execute(
                "DELETE FROM canonical_associations WHERE scene_id = ?1",
                params![sid],
            );
            let _ = self
                .conn
                .execute("DELETE FROM scene_arcs WHERE scene_id = ?1", params![sid]);
            let _ = self
                .conn
                .execute("DELETE FROM event_scenes WHERE scene_id = ?1", params![sid]);
            let _ = self
                .conn
                .execute("DELETE FROM issue_scenes WHERE scene_id = ?1", params![sid]);
            let _ = self
                .conn
                .execute("DELETE FROM scene_steps WHERE scene_id = ?1", params![sid]);
            let _ = self
                .conn
                .execute("DELETE FROM annotations WHERE scene_id = ?1", params![sid]);
            let _ = self.conn.execute(
                "DELETE FROM scene_history WHERE scene_id = ?1",
                params![sid],
            );
        }

        let scenes_purged = self
            .conn
            .execute(
                "DELETE FROM scenes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
                params![threshold],
            )
            .map_err(|e| e.to_string())?;

        let chapters_purged = self
            .conn
            .execute(
                "DELETE FROM chapters WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
                params![threshold],
            )
            .map_err(|e| e.to_string())?;

        Ok((scenes_purged, chapters_purged))
    }

    pub fn restore_chapter(&self, id: &str) -> Result<Chapter, String> {
        // Get the chapter's deleted_at timestamp to only restore scenes deleted at the same time
        let chapter_deleted_at: String = self
            .conn
            .query_row(
                "SELECT deleted_at FROM chapters WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        self.conn
            .execute(
                "UPDATE chapters SET deleted_at = NULL WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        // Only restore scenes that were deleted at the same time as the chapter
        // (not scenes that were individually deleted before the chapter)
        self.conn
            .execute(
                "UPDATE scenes SET deleted_at = NULL WHERE chapter_id = ?1 AND deleted_at = ?2",
                params![id, chapter_deleted_at],
            )
            .map_err(|e| e.to_string())?;
        self.get_chapter(id)
    }
}
