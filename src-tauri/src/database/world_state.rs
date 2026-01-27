//! World State at Scene N.
//!
//! For a given scene, computes the narrative world state:
//! character presences, open setups, and active arcs.
//! All queries use manuscript ordering: `chapter.position * 1000000 + scene.position`.

use crate::models::{ActiveArcState, CharacterPresence, OpenSetup, WorldState};
use rusqlite::params;

use super::Database;

/// Intermediate row returned by the character aggregation query.
struct CharacterRow {
    bible_entry_id: String,
    name: String,
    appearance_count: i32,
    present_here: bool,
}

/// Intermediate row for a character's last-scene lookup.
#[derive(Clone)]
struct LastSceneInfo {
    scene_id: String,
    scene_title: String,
    gap: i32,
}

/// Raw row from the character thread scene query.
struct ThreadSceneRow {
    scene_id: String,
    scene_title: String,
    chapter_title: String,
    chapter_id: String,
    pov: Option<String>,
    tension: Option<String>,
    summary: Option<String>,
    scene_order: i64,
}

impl Database {
    /// Returns the world state at the given scene point.
    pub fn get_world_state_at_scene(&self, scene_id: &str) -> Result<WorldState, String> {
        let scene_order = self.ws_scene_order(scene_id)?;

        let character_presences = self.ws_character_presences(scene_id, scene_order)?;
        let open_setups = self.ws_open_setups(scene_order)?;
        let active_arcs = self.ws_active_arcs(scene_id, scene_order)?;

        Ok(WorldState {
            scene_id: scene_id.to_string(),
            character_presences,
            open_setups,
            active_arcs,
        })
    }

    /// Resolves the manuscript-order position of a scene.
    fn ws_scene_order(&self, scene_id: &str) -> Result<i64, String> {
        self.conn
            .query_row(
                "SELECT c.position * 1000000 + s.position
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.id = ?1 AND s.deleted_at IS NULL AND c.deleted_at IS NULL",
                params![scene_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Scene not found: {}", e))
    }

    // ── Character Presences ──────────────────────────────────────────────

    /// Characters that have appeared up to this scene point.
    fn ws_character_presences(
        &self,
        scene_id: &str,
        scene_order: i64,
    ) -> Result<Vec<CharacterPresence>, String> {
        let characters = self.ws_character_rows(scene_id, scene_order)?;
        let last_scenes = self.ws_last_scenes_batch(scene_order)?;

        let presences = characters
            .into_iter()
            .map(|ch| {
                let last = last_scenes
                    .get(&ch.bible_entry_id)
                    .cloned()
                    .unwrap_or(LastSceneInfo {
                        scene_id: String::new(),
                        scene_title: String::new(),
                        gap: 0,
                    });
                CharacterPresence {
                    bible_entry_id: ch.bible_entry_id,
                    name: ch.name,
                    appearance_count: ch.appearance_count,
                    last_scene_id: last.scene_id,
                    last_scene_title: last.scene_title,
                    gap_scenes: last.gap,
                    present_here: ch.present_here,
                }
            })
            .collect();

        Ok(presences)
    }

    /// Aggregated character rows: id, name, appearance count, present-here flag.
    fn ws_character_rows(
        &self,
        scene_id: &str,
        scene_order: i64,
    ) -> Result<Vec<CharacterRow>, String> {
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

        let rows = stmt
            .query_map(params![scene_id, scene_order], |row| {
                Ok(CharacterRow {
                    bible_entry_id: row.get(0)?,
                    name: row.get(1)?,
                    appearance_count: row.get(2)?,
                    present_here: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    /// Batch-load the most recent scene for every character at or before `scene_order`.
    /// Uses a window function to find the latest scene per character in a single query,
    /// and computes the gap (scenes between last appearance and scene_order) in SQL.
    fn ws_last_scenes_batch(
        &self,
        scene_order: i64,
    ) -> Result<std::collections::HashMap<String, LastSceneInfo>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "WITH ranked AS (
                    SELECT ca.bible_entry_id,
                           s.id AS scene_id,
                           s.title AS scene_title,
                           c.position * 1000000 + s.position AS scene_ord,
                           ROW_NUMBER() OVER (PARTITION BY ca.bible_entry_id
                                              ORDER BY c.position * 1000000 + s.position DESC) AS rn
                    FROM canonical_associations ca
                    JOIN scenes s ON ca.scene_id = s.id
                    JOIN chapters c ON s.chapter_id = c.id
                    WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                      AND (c.position * 1000000 + s.position) <= ?1
                )
                SELECT r.bible_entry_id, r.scene_id, r.scene_title,
                       (SELECT COUNT(*)
                        FROM scenes s3
                        JOIN chapters c3 ON s3.chapter_id = c3.id
                        WHERE s3.deleted_at IS NULL AND c3.deleted_at IS NULL
                          AND (c3.position * 1000000 + s3.position) > r.scene_ord
                          AND (c3.position * 1000000 + s3.position) <= ?1
                       ) AS gap
                FROM ranked r
                WHERE r.rn = 1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![scene_order], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    LastSceneInfo {
                        scene_id: row.get(1)?,
                        scene_title: row.get(2)?,
                        gap: row.get(3)?,
                    },
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().collect())
    }

    // ── Open Setups ──────────────────────────────────────────────────────

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

        let rows = stmt
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
        Ok(rows)
    }

    // ── Active Arcs ──────────────────────────────────────────────────────

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

        let rows = stmt
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
        Ok(rows)
    }

    // ── Character Thread ─────────────────────────────────────────────────

    /// Returns the character thread: all scenes where a character appears, in order.
    pub fn get_character_thread(
        &self,
        bible_entry_id: &str,
    ) -> Result<crate::models::CharacterThread, String> {
        let character_name = self.ct_character_name(bible_entry_id)?;
        let rows = self.ct_scene_rows(bible_entry_id)?;
        let all_positions = self.ct_all_scene_positions()?;
        let scenes = self.ct_build_scenes(bible_entry_id, &rows, &all_positions)?;

        Ok(crate::models::CharacterThread {
            bible_entry_id: bible_entry_id.to_string(),
            character_name,
            scenes,
        })
    }

    /// Fetch the character's display name.
    fn ct_character_name(&self, bible_entry_id: &str) -> Result<String, String> {
        self.conn
            .query_row(
                "SELECT name FROM bible_entries WHERE id = ?1 AND deleted_at IS NULL",
                params![bible_entry_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Character not found: {}", e))
    }

    /// All scenes where the character appears, in manuscript order.
    fn ct_scene_rows(&self, bible_entry_id: &str) -> Result<Vec<ThreadSceneRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title, c.title, c.id, s.pov, s.tension, s.summary,
                        c.position * 1000000 + s.position as scene_order
                 FROM canonical_associations ca
                 JOIN scenes s ON ca.scene_id = s.id
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE ca.bible_entry_id = ?1
                   AND s.deleted_at IS NULL AND c.deleted_at IS NULL
                 ORDER BY scene_order",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![bible_entry_id], |row| {
                Ok(ThreadSceneRow {
                    scene_id: row.get(0)?,
                    scene_title: row.get(1)?,
                    chapter_title: row.get(2)?,
                    chapter_id: row.get(3)?,
                    pov: row.get(4)?,
                    tension: row.get(5)?,
                    summary: row.get(6)?,
                    scene_order: row.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    /// Ordered list of all (non-deleted) scene positions in the manuscript.
    fn ct_all_scene_positions(&self) -> Result<Vec<i64>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.position * 1000000 + s.position as scene_order
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                 ORDER BY scene_order",
            )
            .map_err(|e| e.to_string())?;

        let positions = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(positions)
    }

    /// Assemble `CharacterThreadScene` entries from raw rows.
    fn ct_build_scenes(
        &self,
        bible_entry_id: &str,
        rows: &[ThreadSceneRow],
        all_positions: &[i64],
    ) -> Result<Vec<crate::models::CharacterThreadScene>, String> {
        let mut scenes = Vec::with_capacity(rows.len());
        let mut prev_order: Option<i64> = None;

        for (i, row) in rows.iter().enumerate() {
            let gap = Self::ct_gap(prev_order, row.scene_order, all_positions);
            prev_order = Some(row.scene_order);

            let other_characters = self.ct_other_characters(&row.scene_id, bible_entry_id)?;

            scenes.push(crate::models::CharacterThreadScene {
                scene_id: row.scene_id.clone(),
                scene_title: row.scene_title.clone(),
                chapter_title: row.chapter_title.clone(),
                chapter_id: row.chapter_id.clone(),
                position_index: i as i32,
                pov: row.pov.clone(),
                tension: row.tension.clone(),
                summary: row.summary.clone(),
                other_characters,
                gap_from_previous: gap,
            });
        }

        Ok(scenes)
    }

    /// Number of scenes between the previous appearance and the current one.
    fn ct_gap(prev_order: Option<i64>, current_order: i64, all_positions: &[i64]) -> i32 {
        match prev_order {
            Some(prev) => {
                let prev_idx = all_positions.iter().position(|&p| p == prev).unwrap_or(0);
                let curr_idx = all_positions
                    .iter()
                    .position(|&p| p == current_order)
                    .unwrap_or(0);
                curr_idx.saturating_sub(prev_idx) as i32
            }
            None => 0,
        }
    }

    /// Other characters appearing in the same scene (excluding the target character).
    fn ct_other_characters(
        &self,
        scene_id: &str,
        exclude_bible_entry_id: &str,
    ) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT be.name
                 FROM canonical_associations ca
                 JOIN bible_entries be ON ca.bible_entry_id = be.id
                 WHERE ca.scene_id = ?1
                   AND be.entry_type = 'character'
                   AND be.id != ?2
                   AND be.deleted_at IS NULL
                 ORDER BY be.name",
            )
            .map_err(|e| e.to_string())?;

        let names = stmt
            .query_map(params![scene_id, exclude_bible_entry_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(names)
    }
}
