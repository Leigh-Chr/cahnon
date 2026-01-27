//! Fact / revelation operations.
//!
//! Facts track narrative revelations — things the reader or characters learn
//! throughout the story. Each fact can be linked to characters who know it,
//! along with the scene where they learned it. This enables detecting
//! knowledge inconsistencies (a character referencing something they shouldn't
//! know yet based on scene ordering).

use crate::models::{CreateFactRequest, Fact, FactCharacter, UpdateFactRequest};
use rusqlite::params;

use super::Database;

impl Database {
    // ========================================================================
    // Fact CRUD
    // ========================================================================

    /// Creates a new fact.
    pub fn create_fact(&self, req: &CreateFactRequest) -> Result<Fact, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO facts (id, content, category, revealed_in_scene_id, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    req.content,
                    req.category.as_deref().unwrap_or("plot"),
                    req.revealed_in_scene_id,
                    req.status.as_deref().unwrap_or("secret"),
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_fact(&id)
    }

    /// Gets a fact by ID.
    pub fn get_fact(&self, id: &str) -> Result<Fact, String> {
        self.conn
            .query_row(
                "SELECT id, content, category, revealed_in_scene_id, status, created_at, updated_at
                 FROM facts WHERE id = ?1",
                params![id],
                Self::map_fact,
            )
            .map_err(|e| e.to_string())
    }

    /// Lists all facts, ordered by creation date.
    pub fn get_facts(&self) -> Result<Vec<Fact>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, content, category, revealed_in_scene_id, status, created_at, updated_at
                 FROM facts ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;

        let facts = stmt
            .query_map([], Self::map_fact)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(facts)
    }

    /// Updates a fact.
    pub fn update_fact(&self, id: &str, req: &UpdateFactRequest) -> Result<Fact, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        macro_rules! add_field {
            ($field:expr, $column:literal) => {
                if let Some(val) = &$field {
                    set_clauses.push(format!("{} = ?{}", $column, params_vec.len() + 1));
                    params_vec.push(Box::new(val.clone()));
                }
            };
        }

        add_field!(req.content, "content");
        add_field!(req.category, "category");
        add_field!(req.revealed_in_scene_id, "revealed_in_scene_id");
        add_field!(req.status, "status");

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE facts SET {} WHERE id = ?{}",
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

        self.get_fact(id)
    }

    /// Deletes a fact and cascades to fact_characters.
    pub fn delete_fact(&self, id: &str) -> Result<(), String> {
        // Clean up junction table first
        self.conn
            .execute(
                "DELETE FROM fact_characters WHERE fact_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        let rows = self
            .conn
            .execute("DELETE FROM facts WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err("Fact not found".to_string());
        }
        Ok(())
    }

    fn map_fact(row: &rusqlite::Row) -> rusqlite::Result<Fact> {
        Ok(Fact {
            id: row.get(0)?,
            content: row.get(1)?,
            category: row.get(2)?,
            revealed_in_scene_id: row.get(3)?,
            status: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }

    // ========================================================================
    // Fact-Scene queries
    // ========================================================================

    /// Gets all facts revealed in a specific scene.
    pub fn get_facts_for_scene(&self, scene_id: &str) -> Result<Vec<Fact>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, content, category, revealed_in_scene_id, status, created_at, updated_at
                 FROM facts WHERE revealed_in_scene_id = ?1
                 ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;

        let facts = stmt
            .query_map(params![scene_id], Self::map_fact)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(facts)
    }

    // ========================================================================
    // Fact-Character operations
    // ========================================================================

    /// Links a character (bible entry) to a fact, recording when they learned it.
    pub fn link_character_to_fact(
        &self,
        fact_id: &str,
        bible_entry_id: &str,
        learned_in_scene_id: Option<&str>,
    ) -> Result<FactCharacter, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT OR IGNORE INTO fact_characters (id, fact_id, bible_entry_id, learned_in_scene_id, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, fact_id, bible_entry_id, learned_in_scene_id, now],
            )
            .map_err(|e| e.to_string())?;

        self.conn
            .query_row(
                "SELECT id, fact_id, bible_entry_id, learned_in_scene_id, created_at
                 FROM fact_characters WHERE fact_id = ?1 AND bible_entry_id = ?2",
                params![fact_id, bible_entry_id],
                Self::map_fact_character,
            )
            .map_err(|e| e.to_string())
    }

    /// Removes a character's knowledge link to a fact.
    pub fn unlink_character_from_fact(
        &self,
        fact_id: &str,
        bible_entry_id: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM fact_characters WHERE fact_id = ?1 AND bible_entry_id = ?2",
                params![fact_id, bible_entry_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Gets all characters who know a specific fact.
    pub fn get_fact_characters(&self, fact_id: &str) -> Result<Vec<FactCharacter>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, fact_id, bible_entry_id, learned_in_scene_id, created_at
                 FROM fact_characters WHERE fact_id = ?1
                 ORDER BY created_at",
            )
            .map_err(|e| e.to_string())?;

        let characters = stmt
            .query_map(params![fact_id], Self::map_fact_character)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(characters)
    }

    fn map_fact_character(row: &rusqlite::Row) -> rusqlite::Result<FactCharacter> {
        Ok(FactCharacter {
            id: row.get(0)?,
            fact_id: row.get(1)?,
            bible_entry_id: row.get(2)?,
            learned_in_scene_id: row.get(3)?,
            created_at: row.get(4)?,
        })
    }

    // ========================================================================
    // Knowledge queries
    // ========================================================================

    /// Returns all facts a character knows by the time a given scene occurs.
    ///
    /// A character knows a fact if they have a `fact_characters` entry whose
    /// `learned_in_scene_id` refers to a scene positioned at or before the
    /// target scene in manuscript order (chapter position, then scene position
    /// within chapter). Facts with no `learned_in_scene_id` are also included
    /// (assumed to be prior knowledge).
    pub fn get_character_knowledge_at_scene(
        &self,
        bible_entry_id: &str,
        scene_id: &str,
    ) -> Result<Vec<Fact>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT f.id, f.content, f.category, f.revealed_in_scene_id, f.status, f.created_at, f.updated_at
                 FROM facts f
                 JOIN fact_characters fc ON f.id = fc.fact_id
                 WHERE fc.bible_entry_id = ?1
                   AND (
                     fc.learned_in_scene_id IS NULL
                     OR (
                       SELECT c.position * 1000000 + s.position
                       FROM scenes s
                       JOIN chapters c ON s.chapter_id = c.id
                       WHERE s.id = fc.learned_in_scene_id
                         AND s.deleted_at IS NULL
                         AND c.deleted_at IS NULL
                     ) <= (
                       SELECT c2.position * 1000000 + s2.position
                       FROM scenes s2
                       JOIN chapters c2 ON s2.chapter_id = c2.id
                       WHERE s2.id = ?2
                         AND s2.deleted_at IS NULL
                         AND c2.deleted_at IS NULL
                     )
                   )
                 ORDER BY f.created_at",
            )
            .map_err(|e| e.to_string())?;

        let facts = stmt
            .query_map(params![bible_entry_id, scene_id], Self::map_fact)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(facts)
    }

    /// Detects knowledge inconsistencies across the manuscript.
    ///
    /// Returns tuples of `(fact_id, character_name, scene_title)` where a
    /// character is associated with a scene (via canonical associations) that
    /// occurs *before* the scene where they learned the fact. This indicates
    /// the character might reference knowledge they shouldn't have yet.
    ///
    /// The check works by:
    /// 1. Finding all fact-character links that have a `learned_in_scene_id`
    /// 2. Finding scenes where the character appears (via canonical associations)
    ///    that are ordered *before* the learning scene
    /// 3. Checking if the fact's `revealed_in_scene_id` matches one of those
    ///    earlier scenes — meaning the fact is mentioned in a scene the character
    ///    is in, before they should know it
    pub fn detect_knowledge_inconsistencies(
        &self,
    ) -> Result<Vec<(String, String, String)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT fc.fact_id, be.name, s.title
                 FROM fact_characters fc
                 JOIN bible_entries be ON fc.bible_entry_id = be.id
                 JOIN canonical_associations ca ON ca.bible_entry_id = fc.bible_entry_id
                 JOIN scenes s ON ca.scene_id = s.id
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE fc.learned_in_scene_id IS NOT NULL
                   AND s.deleted_at IS NULL
                   AND c.deleted_at IS NULL
                   AND be.deleted_at IS NULL
                   AND (c.position * 1000000 + s.position) < (
                     SELECT c2.position * 1000000 + s2.position
                     FROM scenes s2
                     JOIN chapters c2 ON s2.chapter_id = c2.id
                     WHERE s2.id = fc.learned_in_scene_id
                       AND s2.deleted_at IS NULL
                       AND c2.deleted_at IS NULL
                   )
                   AND EXISTS (
                     SELECT 1 FROM facts f
                     WHERE f.id = fc.fact_id
                       AND f.revealed_in_scene_id = s.id
                   )
                 ORDER BY c.position, s.position",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(results)
    }
}
