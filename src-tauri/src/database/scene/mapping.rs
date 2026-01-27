//! Scene row mapping helpers.

use crate::models::Scene;

use super::super::Database;

impl Database {
    pub(crate) fn map_scene(row: &rusqlite::Row) -> rusqlite::Result<Scene> {
        let core = Self::map_scene_core(row)?;
        let meta = Self::map_scene_meta(row)?;
        let revision = Self::map_scene_revision(row)?;

        Ok(Scene {
            id: core.0,
            chapter_id: core.1,
            title: core.2,
            summary: core.3,
            text: core.4,
            status: meta.0,
            pov: meta.1,
            tags: meta.2,
            notes: meta.3,
            todos: meta.4,
            word_target: meta.5,
            time_point: meta.6,
            time_start: meta.7,
            time_end: meta.8,
            on_timeline: meta.9,
            position: meta.10,
            pov_goal: revision.0,
            has_conflict: revision.1,
            has_change: revision.2,
            tension: revision.3,
            setup_for_scene_id: revision.4,
            payoff_of_scene_id: revision.5,
            revision_notes: revision.6,
            revision_checklist: revision.7,
            word_count: revision.8,
            created_at: revision.9,
            updated_at: revision.10,
        })
    }

    #[allow(clippy::type_complexity)]
    fn map_scene_core(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, String, Option<String>, String)> {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn map_scene_meta(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<String>,
        Option<String>,
        Option<String>,
        bool,
        i32,
    )> {
        let meta1 = Self::map_scene_meta_part1(row)?;
        let meta2 = Self::map_scene_meta_part2(row)?;
        Ok((
            meta1.0, meta1.1, meta1.2, meta1.3, meta1.4, meta1.5, meta2.0, meta2.1, meta2.2,
            meta2.3, meta2.4,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn map_scene_meta_part1(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<i32>,
    )> {
        Ok((
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            row.get(8)?,
            row.get(9)?,
            row.get(10)?,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn map_scene_meta_part2(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(Option<String>, Option<String>, Option<String>, bool, i32)> {
        Ok((
            row.get(11)?,
            row.get(12)?,
            row.get(13)?,
            row.get::<_, i32>(14)? != 0,
            row.get(15)?,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn map_scene_revision(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        i32,
        String,
        String,
    )> {
        let rev1 = Self::map_scene_revision_part1(row)?;
        let rev2 = Self::map_scene_revision_part2(row)?;
        Ok((
            rev1.0, rev1.1, rev1.2, rev1.3, rev1.4, rev2.0, rev2.1, rev2.2, rev2.3, rev2.4, rev2.5,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn map_scene_revision_part1(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        Option<String>,
        Option<bool>,
        Option<bool>,
        Option<String>,
        Option<String>,
    )> {
        Ok((
            row.get(16)?,
            row.get::<_, Option<i32>>(17)?.map(|v| v != 0),
            row.get::<_, Option<i32>>(18)?.map(|v| v != 0),
            row.get(19)?,
            row.get(20)?,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn map_scene_revision_part2(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        Option<String>,
        Option<String>,
        Option<String>,
        i32,
        String,
        String,
    )> {
        Ok((
            row.get(21)?,
            row.get(22)?,
            row.get(23)?,
            row.get(24)?,
            row.get(25)?,
            row.get(26)?,
        ))
    }
}
