//! Arc operations

use crate::models::{Arc, CreateArcRequest, Scene, UpdateArcRequest};
use rusqlite::params;

use super::macros::add_field;
use super::Database;

impl Database {
    pub fn create_arc(&self, req: &CreateArcRequest) -> Result<Arc, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let position = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), 0) + 1 FROM arcs WHERE deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        self.conn
            .execute(
                "INSERT INTO arcs (id, name, description, stakes, status, color, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    id,
                    req.name,
                    req.description,
                    req.stakes,
                    req.status.as_deref().unwrap_or("setup"),
                    req.color,
                    position,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_arc(&id)
    }

    pub fn get_arcs(&self) -> Result<Vec<Arc>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, description, stakes, status, color, position, created_at, updated_at
             FROM arcs WHERE deleted_at IS NULL ORDER BY position",
            )
            .map_err(|e| e.to_string())?;

        let mut arcs: Vec<Arc> = stmt
            .query_map([], Self::map_arc_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        // Batch-load all arc characters in a single query
        let char_map = self.get_all_arc_characters_batch()?;
        for arc in &mut arcs {
            if let Some(chars) = char_map.get(&arc.id) {
                arc.characters = chars.clone();
            }
        }

        Ok(arcs)
    }

    /// Loads all arc-character associations in one query, grouped by arc_id.
    fn get_all_arc_characters_batch(
        &self,
    ) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT arc_id, bible_entry_id FROM arc_characters")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (arc_id, bible_entry_id) in rows {
            map.entry(arc_id).or_default().push(bible_entry_id);
        }
        Ok(map)
    }

    /// Maps a row (without characters) to an Arc with empty characters vec.
    fn map_arc_row(row: &rusqlite::Row) -> rusqlite::Result<Arc> {
        Ok(Arc {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            stakes: row.get(3)?,
            characters: Vec::new(), // loaded separately
            status: row.get(4)?,
            color: row.get(5)?,
            position: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }

    pub fn get_arc(&self, id: &str) -> Result<Arc, String> {
        let mut arc = self
            .conn
            .query_row(
                "SELECT id, name, description, stakes, status, color, position, created_at, updated_at
             FROM arcs WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                Self::map_arc_row,
            )
            .map_err(|e| e.to_string())?;

        arc.characters = self.get_arc_characters(id)?;
        Ok(arc)
    }

    pub fn update_arc(&self, id: &str, req: &UpdateArcRequest) -> Result<Arc, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        add_field!(set_clauses, params_vec, req.name, "name");
        add_field!(set_clauses, params_vec, req.description, "description");
        add_field!(set_clauses, params_vec, req.stakes, "stakes");
        add_field!(set_clauses, params_vec, req.status, "status");
        add_field!(set_clauses, params_vec, req.color, "color");

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE arcs SET {} WHERE id = ?{}",
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

        self.get_arc(id)
    }

    pub fn delete_arc(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        self.run_in_transaction(|| {
            // Clean up junction tables to avoid orphaned records
            self.conn
                .execute("DELETE FROM scene_arcs WHERE arc_id = ?1", params![id])
                .map_err(|e| e.to_string())?;
            self.conn
                .execute("DELETE FROM arc_characters WHERE arc_id = ?1", params![id])
                .map_err(|e| e.to_string())?;

            self.conn
                .execute(
                    "UPDATE arcs SET deleted_at = ?1 WHERE id = ?2",
                    params![now, id],
                )
                .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    pub fn link_scene_to_arc(&self, scene_id: &str, arc_id: &str) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO scene_arcs (id, scene_id, arc_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, scene_id, arc_id, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn unlink_scene_from_arc(&self, scene_id: &str, arc_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM scene_arcs WHERE scene_id = ?1 AND arc_id = ?2",
                params![scene_id, arc_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_scene_arcs(&self, scene_id: &str) -> Result<Vec<Arc>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.name, a.description, a.stakes, a.status, a.color, a.position, a.created_at, a.updated_at
             FROM arcs a
             JOIN scene_arcs sa ON a.id = sa.arc_id
             WHERE sa.scene_id = ?1 AND a.deleted_at IS NULL
             ORDER BY a.position",
            )
            .map_err(|e| e.to_string())?;

        let mut arcs: Vec<Arc> = stmt
            .query_map(params![scene_id], Self::map_arc_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let char_map = self.get_all_arc_characters_batch()?;
        for arc in &mut arcs {
            if let Some(chars) = char_map.get(&arc.id) {
                arc.characters = chars.clone();
            }
        }

        Ok(arcs)
    }

    // ========================================================================
    // Arc-Character operations
    // ========================================================================

    pub fn get_arc_characters(&self, arc_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT bible_entry_id FROM arc_characters WHERE arc_id = ?1")
            .map_err(|e| e.to_string())?;

        let ids = stmt
            .query_map(params![arc_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(ids)
    }

    pub fn set_arc_characters(
        &self,
        arc_id: &str,
        character_ids: &[String],
    ) -> Result<Vec<String>, String> {
        self.run_in_transaction(|| {
            // Remove all existing links
            self.conn
                .execute(
                    "DELETE FROM arc_characters WHERE arc_id = ?1",
                    params![arc_id],
                )
                .map_err(|e| e.to_string())?;

            // Insert new links
            let now = chrono::Utc::now().to_rfc3339();
            for char_id in character_ids {
                let id = uuid::Uuid::new_v4().to_string();
                self.conn
                    .execute(
                        "INSERT OR IGNORE INTO arc_characters (id, arc_id, bible_entry_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                        params![id, arc_id, char_id, now],
                    )
                    .map_err(|e| e.to_string())?;
            }

            Ok(())
        })?;

        self.get_arc_characters(arc_id)
    }

    pub fn get_arc_scenes(&self, arc_id: &str) -> Result<Vec<Scene>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.chapter_id, s.title, s.summary, s.text, s.status, s.pov, s.tags,
                    s.notes, s.todos, s.word_target, s.time_point, s.time_start, s.time_end,
                    s.on_timeline, s.position, s.pov_goal, s.has_conflict, s.has_change, s.tension,
                    s.setup_for_scene_id, s.payoff_of_scene_id, s.revision_notes, s.revision_checklist,
                    s.word_count, s.created_at, s.updated_at
             FROM scenes s
             JOIN scene_arcs sa ON s.id = sa.scene_id
             WHERE sa.arc_id = ?1 AND s.deleted_at IS NULL
             ORDER BY s.position",
            )
            .map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map(params![arc_id], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
    }

    pub fn get_character_arcs(&self, bible_entry_id: &str) -> Result<Vec<Arc>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.name, a.description, a.stakes, a.status, a.color, a.position, a.created_at, a.updated_at
             FROM arcs a
             JOIN arc_characters ac ON a.id = ac.arc_id
             WHERE ac.bible_entry_id = ?1 AND a.deleted_at IS NULL
             ORDER BY a.position",
            )
            .map_err(|e| e.to_string())?;

        let mut arcs: Vec<Arc> = stmt
            .query_map(params![bible_entry_id], Self::map_arc_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let char_map = self.get_all_arc_characters_batch()?;
        for arc in &mut arcs {
            if let Some(chars) = char_map.get(&arc.id) {
                arc.characters = chars.clone();
            }
        }

        Ok(arcs)
    }
}
