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
        let text = &original.text;

        // Split the text at the given character position (not byte position)
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
        let first_part = &text[..byte_pos];
        let second_part = &text[byte_pos..];

        let now = chrono::Utc::now().to_rfc3339();
        let new_id = uuid::Uuid::new_v4().to_string();
        let new_title = new_scene_title
            .unwrap_or(&format!("{} (continued)", original.title))
            .to_string();

        // Use a transaction to ensure atomicity
        self.conn
            .execute("BEGIN TRANSACTION", [])
            .map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
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

            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            }
            Err(e) => {
                if let Err(rollback_err) = self.conn.execute("ROLLBACK", []) {
                    eprintln!("Failed to rollback transaction: {}", rollback_err);
                }
                return Err(e);
            }
        }

        let updated_original = self.get_scene(id)?;
        let new_scene = self.get_scene(&new_id)?;

        Ok((updated_original, new_scene))
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

        // Use a transaction to ensure atomicity
        self.conn
            .execute("BEGIN TRANSACTION", [])
            .map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            self.update_merged_scene(target_scene, &combined_text, &merged_notes, &now)?;
            self.transfer_associations_to_target(target_scene, &scenes[1..], &now)?;
            self.soft_delete_merged_scenes(&scenes[1..], &now)?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
            }
            Err(e) => {
                if let Err(rollback_err) = self.conn.execute("ROLLBACK", []) {
                    eprintln!("Failed to rollback transaction: {}", rollback_err);
                }
                return Err(e);
            }
        }

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

        self.conn.execute("BEGIN", []).map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            // Shift positions of scenes after the original to make room
            self.conn
                .execute(
                    "UPDATE scenes SET position = position + 1, updated_at = ?1 WHERE chapter_id = ?2 AND position > ?3 AND deleted_at IS NULL",
                    params![now, original.chapter_id, original.position],
                )
                .map_err(|e| e.to_string())?;

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

            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
                self.get_scene(&new_id)
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }
}
