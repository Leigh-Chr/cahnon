//! Scene row mapping helpers.

use crate::models::Scene;

use super::super::Database;

impl Database {
    pub(crate) fn map_scene(row: &rusqlite::Row) -> rusqlite::Result<Scene> {
        Ok(Scene {
            id: row.get(0)?,
            chapter_id: row.get(1)?,
            title: row.get(2)?,
            summary: row.get(3)?,
            text: row.get(4)?,
            status: row.get(5)?,
            pov: row.get(6)?,
            tags: row.get(7)?,
            notes: row.get(8)?,
            todos: row.get(9)?,
            word_target: row.get(10)?,
            time_point: row.get(11)?,
            time_start: row.get(12)?,
            time_end: row.get(13)?,
            on_timeline: row.get::<_, i32>(14)? != 0,
            position: row.get(15)?,
            pov_goal: row.get(16)?,
            has_dramatic_conflict: row.get::<_, Option<i32>>(17)?.map(|v| v != 0),
            has_change: row.get::<_, Option<i32>>(18)?.map(|v| v != 0),
            tension: row.get(19)?,
            setup_for_scene_id: row.get(20)?,
            payoff_of_scene_id: row.get(21)?,
            revision_notes: row.get(22)?,
            revision_checklist: row.get(23)?,
            word_count: row.get(24)?,
            created_at: row.get(25)?,
            updated_at: row.get(26)?,
        })
    }
}
