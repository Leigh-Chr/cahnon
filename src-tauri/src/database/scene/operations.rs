//! Scene split, merge, and duplicate operations.

use crate::models::Scene;
use rusqlite::params;

use super::super::Database;

impl Database {
    /// Splits a scene at the specified cursor position.
    /// Returns both the original (first part) and the new scene (second part).
    pub fn split_scene(
        &self,
        id: &str,
        split_position: i32,
        new_scene_title: Option<&str>,
    ) -> Result<(Scene, Scene), String> {
        if split_position < 0 {
            return Err("Split position cannot be negative".to_string());
        }

        let original = self.get_scene(id)?;
        let (first_part, second_part) = Self::split_text_at_char(&original.text, split_position)?;

        let now = chrono::Utc::now().to_rfc3339();
        let new_id = uuid::Uuid::new_v4().to_string();
        let new_title = new_scene_title
            .unwrap_or(&format!("{} (continued)", original.title))
            .to_string();

        self.run_in_transaction(|| {
            self.update_original_scene_text(id, &first_part, &now)?;
            self.shift_scene_positions(&original.chapter_id, original.position, &now)?;
            self.insert_split_scene(&new_id, &original, &new_title, &second_part, &now)?;
            self.copy_associations(id, &new_id, &now)?;
            Ok(())
        })?;

        let updated_original = self.get_scene(id)?;
        let new_scene = self.get_scene(&new_id)?;

        Ok((updated_original, new_scene))
    }

    fn split_text_at_char(text: &str, split_position: i32) -> Result<(String, String), String> {
        let split_pos = split_position as usize;
        let char_count = text.chars().count();
        if split_pos > char_count {
            return Err("Split position is beyond text length".to_string());
        }

        let byte_pos = text
            .char_indices()
            .nth(split_pos)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        Ok((text[..byte_pos].to_string(), text[byte_pos..].to_string()))
    }

    fn update_original_scene_text(&self, id: &str, text: &str, now: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scenes SET text = ?1, updated_at = ?2 WHERE id = ?3",
                params![text, now, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn shift_scene_positions(
        &self,
        chapter_id: &str,
        after_position: i32,
        now: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scenes SET position = position + 1, updated_at = ?1 WHERE chapter_id = ?2 AND position > ?3 AND deleted_at IS NULL",
                params![now, chapter_id, after_position],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn insert_split_scene(
        &self,
        new_id: &str,
        original: &Scene,
        title: &str,
        text: &str,
        now: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO scenes (id, chapter_id, title, summary, text, status, pov, tags, notes, todos, word_target, time_point, time_start, time_end, on_timeline, position, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    new_id,
                    original.chapter_id,
                    title,
                    Option::<String>::None,
                    text,
                    original.status,
                    original.pov,
                    original.tags,
                    Option::<String>::None,
                    Option::<String>::None,
                    Option::<i32>::None,
                    Option::<String>::None,
                    Option::<String>::None,
                    Option::<String>::None,
                    original.on_timeline as i32,
                    original.position + 1,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn copy_associations(
        &self,
        source_scene_id: &str,
        target_scene_id: &str,
        now: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO canonical_associations (id, scene_id, bible_entry_id, created_at)
                 SELECT ?1 || '-' || bible_entry_id, ?2, bible_entry_id, ?3
                 FROM canonical_associations WHERE scene_id = ?4",
                params![target_scene_id, target_scene_id, now, source_scene_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Merges multiple scenes into the first scene.
    /// The scenes must be in the same chapter and are merged in order of their positions.
    /// Returns the merged scene (all other scenes are deleted).
    pub fn merge_scenes(&self, scene_ids: &[String]) -> Result<Scene, String> {
        if scene_ids.len() < 2 {
            return Err("At least two scenes are required to merge".to_string());
        }

        let scenes = self.collect_and_validate_scenes(scene_ids)?;
        let now = chrono::Utc::now().to_rfc3339();
        let target_scene = &scenes[0];

        let combined_text = Self::combine_scene_texts(&scenes);
        let merged_notes = Self::combine_scene_notes(&scenes);

        self.run_in_transaction(|| {
            self.update_merged_scene(target_scene, &combined_text, &merged_notes, &now)?;
            self.transfer_associations_to_target(target_scene, &scenes[1..], &now)?;
            self.soft_delete_merged_scenes(&scenes[1..], &now)?;
            Ok(())
        })?;

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
        if sources.is_empty() {
            return Ok(());
        }
        // Build a single INSERT...SELECT with IN clause for all source scene IDs
        let placeholders: Vec<String> = (0..sources.len()).map(|i| format!("?{}", i + 4)).collect();
        let sql = format!(
            "INSERT OR IGNORE INTO canonical_associations (id, scene_id, bible_entry_id, created_at)
             SELECT ?1 || '-' || scene_id || '-' || bible_entry_id, ?2, bible_entry_id, ?3
             FROM canonical_associations WHERE scene_id IN ({})",
            placeholders.join(", ")
        );
        let uuid_prefix = uuid::Uuid::new_v4().to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::with_capacity(3 + sources.len());
        params_vec.push(Box::new(uuid_prefix));
        params_vec.push(Box::new(target.id.clone()));
        params_vec.push(Box::new(now.to_string()));
        params_vec.extend(
            sources
                .iter()
                .map(|s| Box::new(s.id.clone()) as Box<dyn rusqlite::ToSql>),
        );
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        self.conn
            .execute(&sql, params_refs.as_slice())
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn soft_delete_merged_scenes(&self, scenes: &[Scene], now: &str) -> Result<(), String> {
        if scenes.is_empty() {
            return Ok(());
        }
        let placeholders: String = (0..scenes.len())
            .map(|i| format!("?{}", i + 2))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "UPDATE scenes SET deleted_at = ?1 WHERE id IN ({})",
            placeholders
        );
        let mut params_all: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(1 + scenes.len());
        params_all.push(&now as &dyn rusqlite::ToSql);
        params_all.extend(scenes.iter().map(|s| &s.id as &dyn rusqlite::ToSql));
        self.conn
            .execute(&sql, params_all.as_slice())
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Duplicate a scene.
    /// If `structure_only` is true, only metadata (title, summary, status, POV, etc.) is copied, not the text content.
    pub fn duplicate_scene(&self, id: &str, structure_only: bool) -> Result<Scene, String> {
        let original = self.get_scene(id)?;
        let new_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let new_title = format!("{} (copy)", original.title);

        let text = if structure_only {
            String::new()
        } else {
            original.text.clone()
        };

        self.run_in_transaction(|| {
            self.shift_scene_positions(&original.chapter_id, original.position, &now)?;
            self.insert_duplicated_scene(&new_id, &original, &new_title, &text, &now)?;
            Ok(())
        })?;

        self.get_scene(&new_id)
    }

    fn insert_duplicated_scene(
        &self,
        new_id: &str,
        original: &Scene,
        title: &str,
        text: &str,
        now: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO scenes (id, chapter_id, title, summary, text, status, pov, tags, notes, todos, word_target, time_point, time_start, time_end, on_timeline, position, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    new_id,
                    original.chapter_id,
                    title,
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
        Ok(())
    }
}
