//! World State at Scene N.
//!
//! For a given scene, computes the complete narrative world state:
//! character presences, knowledge states, open setups, active arcs,
//! dramatic irony, and location history. All queries use manuscript
//! ordering: `chapter.position * 1000000 + scene.position`.

use crate::models::{
    ActiveArcState, CharacterKnowledgeState, CharacterPresence, DramaticIronyItem,
    LocationHistoryItem, OpenSetup, WorldState,
};
use rusqlite::params;

use super::Database;

impl Database {
    /// Returns the complete world state at the given scene point.
    pub fn get_world_state_at_scene(&self, scene_id: &str) -> Result<WorldState, String> {
        // Get the manuscript order position of the target scene
        let scene_order: i64 = self
            .conn
            .query_row(
                "SELECT c.position * 1000000 + s.position
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.id = ?1 AND s.deleted_at IS NULL AND c.deleted_at IS NULL",
                params![scene_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Scene not found: {}", e))?;

        let character_presences = self.ws_character_presences(scene_id, scene_order)?;
        let character_knowledge = self.ws_character_knowledge(scene_id, scene_order)?;
        let open_setups = self.ws_open_setups(scene_order)?;
        let active_arcs = self.ws_active_arcs(scene_id, scene_order)?;
        let dramatic_irony = self.ws_dramatic_irony(scene_id, scene_order)?;
        let location_history = self.ws_location_history(scene_id)?;

        Ok(WorldState {
            scene_id: scene_id.to_string(),
            character_presences,
            character_knowledge,
            open_setups,
            active_arcs,
            dramatic_irony,
            location_history,
        })
    }

    /// Characters that have appeared up to this scene point.
    fn ws_character_presences(
        &self,
        scene_id: &str,
        scene_order: i64,
    ) -> Result<Vec<CharacterPresence>, String> {
        // Count all scene positions for gap calculation
        let total_scenes_before: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                   AND (c.position * 1000000 + s.position) <= ?1",
                params![scene_order],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let mut stmt = self
            .conn
            .prepare(
                "SELECT be.id, be.name,
                        COUNT(DISTINCT ca.scene_id) as appearance_count,
                        MAX(CASE WHEN ca.scene_id = ?1 THEN 1 ELSE 0 END) as present_here
                 FROM bible_entries be
                 JOIN canonical_associations ca ON ca.bible_entry_id = be.id
                 JOIN scenes s ON ca.scene_id = s.id
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE be.entry_type = 'character'
                   AND be.deleted_at IS NULL
                   AND s.deleted_at IS NULL
                   AND c.deleted_at IS NULL
                   AND (c.position * 1000000 + s.position) <= ?2
                 GROUP BY be.id
                 ORDER BY appearance_count DESC",
            )
            .map_err(|e| e.to_string())?;

        let characters: Vec<(String, String, i32, bool)> = stmt
            .query_map(params![scene_id, scene_order], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut presences = Vec::with_capacity(characters.len());

        for (be_id, name, count, present) in characters {
            // Get last scene for this character before or at current point
            let last_scene: (String, String, i32) = self
                .conn
                .query_row(
                    "SELECT s.id, s.title,
                            (SELECT COUNT(*)
                             FROM scenes s3
                             JOIN chapters c3 ON s3.chapter_id = c3.id
                             WHERE s3.deleted_at IS NULL AND c3.deleted_at IS NULL
                               AND (c3.position * 1000000 + s3.position) > (c2.position * 1000000 + s2.position)
                               AND (c3.position * 1000000 + s3.position) <= ?3
                            ) as gap
                     FROM canonical_associations ca2
                     JOIN scenes s2 ON ca2.scene_id = s2.id
                     JOIN chapters c2 ON s2.chapter_id = c2.id
                     JOIN scenes s ON s.id = s2.id
                     WHERE ca2.bible_entry_id = ?1
                       AND s2.deleted_at IS NULL AND c2.deleted_at IS NULL
                       AND (c2.position * 1000000 + s2.position) <= ?2
                     ORDER BY (c2.position * 1000000 + s2.position) DESC
                     LIMIT 1",
                    params![be_id, scene_order, scene_order],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .unwrap_or_else(|_| ("".to_string(), "".to_string(), total_scenes_before));

            presences.push(CharacterPresence {
                bible_entry_id: be_id,
                name,
                appearance_count: count,
                last_scene_id: last_scene.0,
                last_scene_title: last_scene.1,
                gap_scenes: last_scene.2,
                present_here: present,
            });
        }

        Ok(presences)
    }

    /// Facts known by characters present in this scene.
    fn ws_character_knowledge(
        &self,
        scene_id: &str,
        scene_order: i64,
    ) -> Result<Vec<CharacterKnowledgeState>, String> {
        // Get characters present in this scene
        let mut stmt = self
            .conn
            .prepare(
                "SELECT be.id, be.name
                 FROM canonical_associations ca
                 JOIN bible_entries be ON ca.bible_entry_id = be.id
                 WHERE ca.scene_id = ?1
                   AND be.entry_type = 'character'
                   AND be.deleted_at IS NULL",
            )
            .map_err(|e| e.to_string())?;

        let characters: Vec<(String, String)> = stmt
            .query_map(params![scene_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut knowledge_states = Vec::with_capacity(characters.len());

        for (be_id, name) in characters {
            // Facts this character knows at this point
            let mut fact_stmt = self
                .conn
                .prepare(
                    "SELECT f.content
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
                             AND s.deleted_at IS NULL AND c.deleted_at IS NULL
                         ) <= ?2
                       )",
                )
                .map_err(|e| e.to_string())?;

            let facts: Vec<String> = fact_stmt
                .query_map(params![be_id, scene_order], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            knowledge_states.push(CharacterKnowledgeState {
                bible_entry_id: be_id,
                name,
                known_facts: facts,
            });
        }

        Ok(knowledge_states)
    }

    /// Scenes that set up something but whose payoff hasn't occurred yet.
    fn ws_open_setups(&self, scene_order: i64) -> Result<Vec<OpenSetup>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title, s.setup_for_scene_id
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                   AND s.setup_for_scene_id IS NOT NULL
                   AND s.setup_for_scene_id != ''
                   AND (c.position * 1000000 + s.position) <= ?1
                   AND NOT EXISTS (
                     SELECT 1 FROM scenes s2
                     JOIN chapters c2 ON s2.chapter_id = c2.id
                     WHERE s2.payoff_of_scene_id = s.id
                       AND s2.deleted_at IS NULL AND c2.deleted_at IS NULL
                       AND (c2.position * 1000000 + s2.position) <= ?1
                   )
                 ORDER BY c.position, s.position",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![scene_order], |row| {
                Ok(OpenSetup {
                    scene_id: row.get(0)?,
                    scene_title: row.get(1)?,
                    setup_for_scene_id: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(results)
    }

    /// Arcs that have at least one scene before this point.
    fn ws_active_arcs(
        &self,
        scene_id: &str,
        scene_order: i64,
    ) -> Result<Vec<ActiveArcState>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.name, a.color,
                        (SELECT COUNT(*)
                         FROM scene_arcs sa2
                         JOIN scenes s2 ON sa2.scene_id = s2.id
                         JOIN chapters c2 ON s2.chapter_id = c2.id
                         WHERE sa2.arc_id = a.id
                           AND s2.deleted_at IS NULL AND c2.deleted_at IS NULL
                           AND (c2.position * 1000000 + s2.position) <= ?2
                        ) as scenes_before,
                        (SELECT COUNT(*)
                         FROM scene_arcs sa3
                         JOIN scenes s3 ON sa3.scene_id = s3.id
                         WHERE sa3.arc_id = a.id AND s3.deleted_at IS NULL
                        ) as scenes_total,
                        (SELECT s4.title
                         FROM scene_arcs sa4
                         JOIN scenes s4 ON sa4.scene_id = s4.id
                         JOIN chapters c4 ON s4.chapter_id = c4.id
                         WHERE sa4.arc_id = a.id
                           AND s4.deleted_at IS NULL AND c4.deleted_at IS NULL
                           AND (c4.position * 1000000 + s4.position) <= ?2
                         ORDER BY (c4.position * 1000000 + s4.position) DESC
                         LIMIT 1
                        ) as last_scene_title
                 FROM arcs a
                 WHERE a.deleted_at IS NULL
                   AND EXISTS (
                     SELECT 1 FROM scene_arcs sa
                     JOIN scenes s ON sa.scene_id = s.id
                     JOIN chapters c ON s.chapter_id = c.id
                     WHERE sa.arc_id = a.id
                       AND s.deleted_at IS NULL AND c.deleted_at IS NULL
                       AND (c.position * 1000000 + s.position) <= ?2
                   )
                 ORDER BY a.position",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![scene_id, scene_order], |row| {
                Ok(ActiveArcState {
                    arc_id: row.get(0)?,
                    arc_name: row.get(1)?,
                    color: row.get(2)?,
                    scenes_before: row.get(3)?,
                    scenes_total: row.get(4)?,
                    last_scene_title: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(results)
    }

    /// Facts the reader knows but characters present in this scene don't.
    fn ws_dramatic_irony(
        &self,
        scene_id: &str,
        scene_order: i64,
    ) -> Result<Vec<DramaticIronyItem>, String> {
        // Facts revealed to reader (via scenes) before or at this point,
        // but NOT known by characters present in this scene
        let mut stmt = self
            .conn
            .prepare(
                "SELECT f.content, be.name, rev_s.title
                 FROM facts f
                 JOIN scenes rev_s ON f.revealed_in_scene_id = rev_s.id
                 JOIN chapters rev_c ON rev_s.chapter_id = rev_c.id
                 -- Characters present in the target scene
                 JOIN canonical_associations ca ON ca.scene_id = ?1
                 JOIN bible_entries be ON ca.bible_entry_id = be.id AND be.entry_type = 'character'
                 WHERE f.revealed_in_scene_id IS NOT NULL
                   AND rev_s.deleted_at IS NULL AND rev_c.deleted_at IS NULL
                   AND be.deleted_at IS NULL
                   -- Fact was revealed before or at this scene
                   AND (rev_c.position * 1000000 + rev_s.position) <= ?2
                   -- Character does NOT know this fact yet
                   AND NOT EXISTS (
                     SELECT 1 FROM fact_characters fc
                     WHERE fc.fact_id = f.id
                       AND fc.bible_entry_id = be.id
                       AND (
                         fc.learned_in_scene_id IS NULL
                         OR (
                           SELECT c3.position * 1000000 + s3.position
                           FROM scenes s3
                           JOIN chapters c3 ON s3.chapter_id = c3.id
                           WHERE s3.id = fc.learned_in_scene_id
                             AND s3.deleted_at IS NULL AND c3.deleted_at IS NULL
                         ) <= ?2
                       )
                   )
                 ORDER BY (rev_c.position * 1000000 + rev_s.position)",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![scene_id, scene_order], |row| {
                Ok(DramaticIronyItem {
                    fact_content: row.get(0)?,
                    character_name: row.get(1)?,
                    revealed_in_scene_title: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(results)
    }

    /// Previous scenes at the same location as this scene.
    fn ws_location_history(&self, scene_id: &str) -> Result<Vec<LocationHistoryItem>, String> {
        // Find location associations for this scene, then find other scenes at same locations
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT s2.id, s2.title, ch2.title
                 FROM canonical_associations ca1
                 JOIN bible_entries be ON ca1.bible_entry_id = be.id
                 JOIN canonical_associations ca2 ON ca2.bible_entry_id = be.id
                 JOIN scenes s2 ON ca2.scene_id = s2.id
                 JOIN chapters ch2 ON s2.chapter_id = ch2.id
                 JOIN scenes s1 ON ca1.scene_id = s1.id
                 JOIN chapters ch1 ON s1.chapter_id = ch1.id
                 WHERE ca1.scene_id = ?1
                   AND be.entry_type = 'location'
                   AND be.deleted_at IS NULL
                   AND s2.id != ?1
                   AND s2.deleted_at IS NULL AND ch2.deleted_at IS NULL
                   AND (ch2.position * 1000000 + s2.position) < (ch1.position * 1000000 + s1.position)
                 ORDER BY ch2.position, s2.position",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![scene_id], |row| {
                Ok(LocationHistoryItem {
                    scene_id: row.get(0)?,
                    scene_title: row.get(1)?,
                    chapter_title: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(results)
    }
}
