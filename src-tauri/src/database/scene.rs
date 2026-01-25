//! Scene CRUD operations

use crate::models::{CreateSceneRequest, Scene, SceneHistoryEntry, UpdateSceneRequest};
use rusqlite::params;

use super::Database;

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
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, chapter_id, title, summary, text, status, pov, tags, notes, todos,
                    word_target, time_point, time_start, time_end, on_timeline, position,
                    pov_goal, has_conflict, has_change, tension, setup_for_scene_id, payoff_of_scene_id, revision_notes, revision_checklist,
                    created_at, updated_at
             FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL ORDER BY position",
            )
            .map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map(params![chapter_id], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
    }

    pub fn get_scene(&self, id: &str) -> Result<Scene, String> {
        self.conn
            .query_row(
                "SELECT id, chapter_id, title, summary, text, status, pov, tags, notes, todos,
                    word_target, time_point, time_start, time_end, on_timeline, position,
                    pov_goal, has_conflict, has_change, tension, setup_for_scene_id, payoff_of_scene_id, revision_notes, revision_checklist,
                    created_at, updated_at
             FROM scenes WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                Self::map_scene,
            )
            .map_err(|e| e.to_string())
    }

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
            created_at: revision.8,
            updated_at: revision.9,
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
        String,
        String,
    )> {
        let rev1 = Self::map_scene_revision_part1(row)?;
        let rev2 = Self::map_scene_revision_part2(row)?;
        Ok((
            rev1.0, rev1.1, rev1.2, rev1.3, rev1.4, rev2.0, rev2.1, rev2.2, rev2.3, rev2.4,
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
        String,
        String,
    )> {
        Ok((
            row.get(21)?,
            row.get(22)?,
            row.get(23)?,
            row.get(24)?,
            row.get(25)?,
        ))
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

    /// Save current scene text to history before update
    fn save_scene_to_history(&self, id: &str, now: &str) -> Result<(), String> {
        if let Ok(scene) = self.get_scene(id) {
            let history_id = uuid::Uuid::new_v4().to_string();
            let _ = self.conn.execute(
                "INSERT INTO scene_history (id, scene_id, text, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![history_id, id, scene.text, now],
            );
        }
        Ok(())
    }

    pub fn delete_scene(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
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
        for (i, id) in ids.iter().enumerate() {
            self.conn
                .execute(
                    "UPDATE scenes SET position = ?1, updated_at = ?2 WHERE id = ?3 AND chapter_id = ?4",
                    params![i as i32, now, id, chapter_id],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
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

    // ========================================================================
    // Scene History operations
    // ========================================================================

    pub fn get_scene_history(&self, scene_id: &str) -> Result<Vec<SceneHistoryEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, scene_id, text, created_at FROM scene_history WHERE scene_id = ?1 ORDER BY created_at DESC LIMIT 100",
            )
            .map_err(|e| e.to_string())?;

        let entries = stmt
            .query_map(params![scene_id], |row| {
                Ok(SceneHistoryEntry {
                    id: row.get(0)?,
                    scene_id: row.get(1)?,
                    text: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(entries)
    }

    pub fn restore_scene_version(&self, scene_id: &str, history_id: &str) -> Result<Scene, String> {
        let history_text: String = self
            .conn
            .query_row(
                "SELECT text FROM scene_history WHERE id = ?1 AND scene_id = ?2",
                params![history_id, scene_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        self.update_scene(
            scene_id,
            &UpdateSceneRequest {
                text: Some(history_text),
                ..Default::default()
            },
        )
    }

    pub fn compare_scene_versions(
        &self,
        scene_id: &str,
        version_id_a: &str,
        version_id_b: &str,
    ) -> Result<(String, String), String> {
        let text_a: String = self
            .conn
            .query_row(
                "SELECT text FROM scene_history WHERE id = ?1 AND scene_id = ?2",
                params![version_id_a, scene_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let text_b: String = self
            .conn
            .query_row(
                "SELECT text FROM scene_history WHERE id = ?1 AND scene_id = ?2",
                params![version_id_b, scene_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        Ok((text_a, text_b))
    }

    // ========================================================================
    // All scenes for timeline
    // ========================================================================

    pub fn get_all_scenes_for_timeline(&self) -> Result<Vec<Scene>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, chapter_id, title, summary, text, status, pov, tags, notes, todos,
                    word_target, time_point, time_start, time_end, on_timeline, position,
                    pov_goal, has_conflict, has_change, tension, setup_for_scene_id, payoff_of_scene_id, revision_notes, revision_checklist,
                    created_at, updated_at
             FROM scenes WHERE deleted_at IS NULL AND on_timeline = 1 AND (time_point IS NOT NULL OR time_start IS NOT NULL)
             ORDER BY COALESCE(time_point, time_start)",
            )
            .map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map([], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
    }

    // ========================================================================
    // Scene split and merge operations
    // ========================================================================

    /// Splits a scene at the specified cursor position
    /// Returns both the original (first part) and the new scene (second part)
    pub fn split_scene(
        &self,
        id: &str,
        split_position: i32,
        new_scene_title: Option<&str>,
    ) -> Result<(Scene, Scene), String> {
        let original = self.get_scene(id)?;
        let text = &original.text;

        // Split the text at the given position
        let split_pos = split_position as usize;
        if split_pos > text.len() {
            return Err("Split position is beyond text length".to_string());
        }

        let first_part = &text[..split_pos];
        let second_part = &text[split_pos..];

        let now = chrono::Utc::now().to_rfc3339();
        let new_id = uuid::Uuid::new_v4().to_string();
        let new_title = new_scene_title
            .unwrap_or(&format!("{} (continued)", original.title))
            .to_string();

        // Update original scene with first part of text
        self.conn
            .execute(
                "UPDATE scenes SET text = ?1, updated_at = ?2 WHERE id = ?3",
                params![first_part, now, id],
            )
            .map_err(|e| e.to_string())?;

        // Shift positions of scenes after the original
        self.conn
            .execute(
                "UPDATE scenes SET position = position + 1, updated_at = ?1 WHERE chapter_id = ?2 AND position > ?3 AND deleted_at IS NULL",
                params![now, original.chapter_id, original.position],
            )
            .map_err(|e| e.to_string())?;

        // Create new scene with second part of text
        self.conn
            .execute(
                "INSERT INTO scenes (id, chapter_id, title, summary, text, status, pov, tags, notes, todos, word_target, time_point, time_start, time_end, on_timeline, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    new_id,
                    original.chapter_id,
                    new_title,
                    Option::<String>::None, // New scene starts without summary
                    second_part,
                    original.status,
                    original.pov,
                    original.tags,
                    Option::<String>::None, // New scene starts without notes
                    Option::<String>::None, // New scene starts without todos
                    Option::<i32>::None,    // No word target for new scene
                    Option::<String>::None, // No time_point
                    Option::<String>::None, // No time_start
                    Option::<String>::None, // No time_end
                    original.on_timeline as i32,
                    original.position + 1,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        // Copy associations to the new scene (they might apply to both)
        self.conn
            .execute(
                "INSERT OR IGNORE INTO canonical_associations (id, scene_id, bible_entry_id, created_at)
             SELECT ?1 || '-' || bible_entry_id, ?2, bible_entry_id, ?3
             FROM canonical_associations WHERE scene_id = ?4",
                params![new_id, new_id, now, id],
            )
            .map_err(|e| e.to_string())?;

        let updated_original = self.get_scene(id)?;
        let new_scene = self.get_scene(&new_id)?;

        Ok((updated_original, new_scene))
    }

    /// Merges multiple scenes into the first scene
    /// The scenes must be in the same chapter and are merged in order of their positions
    /// Returns the merged scene (all other scenes are deleted)
    pub fn merge_scenes(&self, scene_ids: &[String]) -> Result<Scene, String> {
        if scene_ids.len() < 2 {
            return Err("At least two scenes are required to merge".to_string());
        }

        let scenes = self.collect_and_validate_scenes(scene_ids)?;
        let now = chrono::Utc::now().to_rfc3339();
        let target_scene = &scenes[0];

        let combined_text = Self::combine_scene_texts(&scenes);
        let merged_notes = Self::combine_scene_notes(&scenes);

        self.update_merged_scene(target_scene, &combined_text, &merged_notes, &now)?;
        self.transfer_associations_to_target(target_scene, &scenes[1..], &now)?;
        self.soft_delete_merged_scenes(&scenes[1..], &now)?;

        self.get_scene(&target_scene.id)
    }

    fn collect_and_validate_scenes(&self, scene_ids: &[String]) -> Result<Vec<Scene>, String> {
        let mut scenes: Vec<Scene> = Vec::new();
        for id in scene_ids {
            scenes.push(self.get_scene(id)?);
        }

        let chapter_id = &scenes[0].chapter_id;
        if scenes.iter().any(|s| &s.chapter_id != chapter_id) {
            return Err("All scenes must be in the same chapter to merge".to_string());
        }

        scenes.sort_by_key(|s| s.position);
        Ok(scenes)
    }

    fn combine_scene_texts(scenes: &[Scene]) -> String {
        let mut combined = scenes[0].text.clone();
        for scene in scenes.iter().skip(1) {
            if !combined.is_empty() && !scene.text.is_empty() {
                combined.push_str("\n\n");
            }
            combined.push_str(&scene.text);
        }
        combined
    }

    fn combine_scene_notes(scenes: &[Scene]) -> Option<String> {
        let notes: Vec<String> = scenes
            .iter()
            .filter_map(|s| s.notes.clone())
            .filter(|n| !n.is_empty())
            .collect();
        if notes.is_empty() {
            None
        } else {
            Some(notes.join("\n---\n"))
        }
    }

    fn update_merged_scene(
        &self,
        target: &Scene,
        text: &str,
        notes: &Option<String>,
        now: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scenes SET text = ?1, notes = ?2, updated_at = ?3 WHERE id = ?4",
                params![text, notes, now, target.id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn transfer_associations_to_target(
        &self,
        target: &Scene,
        sources: &[Scene],
        now: &str,
    ) -> Result<(), String> {
        for scene in sources {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO canonical_associations (id, scene_id, bible_entry_id, created_at)
                 SELECT ?1 || '-' || bible_entry_id, ?2, bible_entry_id, ?3
                 FROM canonical_associations WHERE scene_id = ?4",
                    params![uuid::Uuid::new_v4().to_string(), target.id, now, scene.id],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn soft_delete_merged_scenes(&self, scenes: &[Scene], now: &str) -> Result<(), String> {
        for scene in scenes {
            self.conn
                .execute(
                    "UPDATE scenes SET deleted_at = ?1 WHERE id = ?2",
                    params![now, scene.id],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Duplicate a scene.
    /// If `structure_only` is true, only metadata (title, summary, status, POV, etc.) is copied, not the text content.
    pub fn duplicate_scene(&self, id: &str, structure_only: bool) -> Result<Scene, String> {
        let original = self.get_scene(id)?;
        let new_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let new_title = format!("{} (copy)", original.title);

        // If structure_only, use empty text; otherwise copy the original text
        let text = if structure_only {
            String::new()
        } else {
            original.text
        };

        self.conn
            .execute(
                "INSERT INTO scenes (id, chapter_id, title, summary, text, status, pov, tags, notes, todos, word_target, time_point, time_start, time_end, on_timeline, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    new_id,
                    original.chapter_id,
                    new_title,
                    original.summary,
                    text,
                    original.status,
                    original.pov,
                    original.tags,
                    original.notes,
                    original.todos,
                    original.word_target,
                    original.time_point,
                    original.time_start,
                    original.time_end,
                    original.on_timeline as i32,
                    original.position + 1,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_scene(&new_id)
    }
}
