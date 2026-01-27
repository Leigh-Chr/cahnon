//! Project analysis detectors.
//!
//! This module implements automated detection of potential issues in a project,
//! including narrative inconsistencies, orphaned data, broken references, and
//! structural patterns that may warrant the writer's attention.

use crate::models::DetectedIssue;
use rusqlite::params;

use super::Database;

impl Database {
    /// Runs all detectors and collects results into a single list.
    #[allow(clippy::type_complexity)]
    pub fn run_all_detections(&self) -> Result<Vec<DetectedIssue>, String> {
        let mut issues = Vec::new();

        let detectors: Vec<fn(&Database) -> Result<Vec<DetectedIssue>, String>> = vec![
            Self::detect_tbd_in_done,
            Self::detect_broken_setup_payoff,
            Self::detect_orphan_bible_entries,
            Self::detect_referential_integrity,
        ];

        for detector in detectors {
            match detector(self) {
                Ok(mut detected) => issues.append(&mut detected),
                Err(e) => {
                    // Log but don't abort: one failing detector shouldn't block the rest.
                    eprintln!("Detection error: {}", e);
                }
            }
        }

        Ok(issues)
    }

    // ========================================================================
    // 1. TBD / TODO / XXX / ?? in scenes marked as "done"
    // ========================================================================

    fn detect_tbd_in_done(&self) -> Result<Vec<DetectedIssue>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, text FROM scenes
                 WHERE status = 'done' AND deleted_at IS NULL AND text != ''",
            )
            .map_err(|e| e.to_string())?;

        let rows: Vec<(String, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut issues = Vec::new();

        for (id, title, text) in rows {
            let plain = super::HTML_TAG_REGEX.replace_all(&text, " ");
            let upper = plain.to_uppercase();

            let mut markers_found = Vec::new();
            if upper.contains("TBD") {
                markers_found.push("TBD");
            }
            if upper.contains("TODO") {
                markers_found.push("TODO");
            }
            if upper.contains("XXX") {
                markers_found.push("XXX");
            }
            if plain.contains("??") {
                markers_found.push("??");
            }

            if !markers_found.is_empty() {
                issues.push(DetectedIssue {
                    issue_type: "tbd_in_done".to_string(),
                    title: format!("Placeholder text in completed scene \"{}\"", title),
                    description: format!(
                        "Scene \"{}\" is marked as done but still contains placeholder markers: {}",
                        title,
                        markers_found.join(", ")
                    ),
                    severity: "warning".to_string(),
                    scene_ids: vec![id],
                    bible_entry_ids: vec![],
                });
            }
        }

        Ok(issues)
    }

    // ========================================================================
    // 2. Broken setup/payoff references
    // ========================================================================

    fn detect_broken_setup_payoff(&self) -> Result<Vec<DetectedIssue>, String> {
        let mut issues = Vec::new();

        // Fetch scenes with setup or payoff references
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title, s.chapter_id, s.position,
                        s.setup_for_scene_id, s.payoff_of_scene_id
                 FROM scenes s
                 WHERE s.deleted_at IS NULL
                   AND (s.setup_for_scene_id IS NOT NULL OR s.payoff_of_scene_id IS NOT NULL)",
            )
            .map_err(|e| e.to_string())?;

        #[allow(clippy::type_complexity)]
        let rows: Vec<(String, String, String, i32, Option<String>, Option<String>)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (scene_id, scene_title, scene_chapter_id, scene_position, setup_for, payoff_of) in &rows
        {
            // Check setup_for_scene_id
            if let Some(target_id) = setup_for {
                match self.get_scene_position_info(target_id) {
                    Ok(Some((_target_chapter_id, target_position, target_chapter_pos))) => {
                        // Setup scene should come BEFORE its target
                        let scene_chapter_pos =
                            self.get_chapter_position(scene_chapter_id).unwrap_or(0);
                        let scene_global =
                            scene_chapter_pos as i64 * 100_000 + *scene_position as i64;
                        let target_global =
                            target_chapter_pos as i64 * 100_000 + target_position as i64;

                        if scene_global >= target_global {
                            issues.push(DetectedIssue {
                                issue_type: "broken_setup_payoff".to_string(),
                                title: format!(
                                    "Setup scene \"{}\" comes after its target",
                                    scene_title
                                ),
                                description: format!(
                                    "Scene \"{}\" is marked as setup for another scene, but it \
                                     appears at the same position or later in the manuscript. \
                                     Setup scenes should come before their target.",
                                    scene_title
                                ),
                                severity: "warning".to_string(),
                                scene_ids: vec![scene_id.clone(), target_id.clone()],
                                bible_entry_ids: vec![],
                            });
                        }
                    }
                    Ok(None) => {
                        // Target scene doesn't exist or is deleted
                        issues.push(DetectedIssue {
                            issue_type: "broken_setup_payoff".to_string(),
                            title: format!(
                                "Setup target missing for scene \"{}\"",
                                scene_title
                            ),
                            description: format!(
                                "Scene \"{}\" references a setup target scene that no longer exists.",
                                scene_title
                            ),
                            severity: "error".to_string(),
                            scene_ids: vec![scene_id.clone()],
                            bible_entry_ids: vec![],
                        });
                    }
                    Err(_) => {}
                }
            }

            // Check payoff_of_scene_id
            if let Some(target_id) = payoff_of {
                match self.get_scene_position_info(target_id) {
                    Ok(Some((_target_chapter_id, target_position, target_chapter_pos))) => {
                        // Payoff scene should come AFTER its target
                        let scene_chapter_pos =
                            self.get_chapter_position(scene_chapter_id).unwrap_or(0);
                        let scene_global =
                            scene_chapter_pos as i64 * 100_000 + *scene_position as i64;
                        let target_global =
                            target_chapter_pos as i64 * 100_000 + target_position as i64;

                        if scene_global <= target_global {
                            issues.push(DetectedIssue {
                                issue_type: "broken_setup_payoff".to_string(),
                                title: format!(
                                    "Payoff scene \"{}\" comes before its source",
                                    scene_title
                                ),
                                description: format!(
                                    "Scene \"{}\" is marked as payoff of another scene, but it \
                                     appears at the same position or earlier in the manuscript. \
                                     Payoff scenes should come after the scene they pay off.",
                                    scene_title
                                ),
                                severity: "warning".to_string(),
                                scene_ids: vec![scene_id.clone(), target_id.clone()],
                                bible_entry_ids: vec![],
                            });
                        }
                    }
                    Ok(None) => {
                        issues.push(DetectedIssue {
                            issue_type: "broken_setup_payoff".to_string(),
                            title: format!(
                                "Payoff source missing for scene \"{}\"",
                                scene_title
                            ),
                            description: format!(
                                "Scene \"{}\" references a payoff source scene that no longer exists.",
                                scene_title
                            ),
                            severity: "error".to_string(),
                            scene_ids: vec![scene_id.clone()],
                            bible_entry_ids: vec![],
                        });
                    }
                    Err(_) => {}
                }
            }
        }

        Ok(issues)
    }

    /// Returns (chapter_id, scene_position, chapter_position) for a scene, or None if deleted/missing.
    fn get_scene_position_info(
        &self,
        scene_id: &str,
    ) -> Result<Option<(String, i32, i32)>, String> {
        let result = self.conn.query_row(
            "SELECT s.chapter_id, s.position, c.position
             FROM scenes s
             JOIN chapters c ON s.chapter_id = c.id
             WHERE s.id = ?1 AND s.deleted_at IS NULL AND c.deleted_at IS NULL",
            params![scene_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        );

        match result {
            Ok(info) => Ok(Some(info)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Returns the position of a chapter, or 0 if not found.
    fn get_chapter_position(&self, chapter_id: &str) -> Result<i32, String> {
        self.conn
            .query_row(
                "SELECT position FROM chapters WHERE id = ?1 AND deleted_at IS NULL",
                params![chapter_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())
    }

    // ========================================================================
    // 3. Orphan bible entries (no links anywhere)
    // ========================================================================

    fn detect_orphan_bible_entries(&self) -> Result<Vec<DetectedIssue>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT b.id, b.name, b.entry_type
                 FROM bible_entries b
                 WHERE b.deleted_at IS NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM canonical_associations ca
                       WHERE ca.bible_entry_id = b.id
                   )
                   AND NOT EXISTS (
                       SELECT 1 FROM arc_characters ac
                       JOIN arcs a ON ac.arc_id = a.id
                       WHERE ac.bible_entry_id = b.id AND a.deleted_at IS NULL
                   )
                   AND NOT EXISTS (
                       SELECT 1 FROM event_bible eb
                       JOIN events ev ON eb.event_id = ev.id
                       WHERE eb.bible_entry_id = b.id AND ev.deleted_at IS NULL
                   )
                   AND NOT EXISTS (
                       SELECT 1 FROM bible_relationships br
                       WHERE br.source_id = b.id OR br.target_id = b.id
                   )
                 ORDER BY b.entry_type, b.name",
            )
            .map_err(|e| e.to_string())?;

        let rows: Vec<(String, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let issues: Vec<DetectedIssue> = rows
            .into_iter()
            .map(|(id, name, entry_type)| DetectedIssue {
                issue_type: "orphan_bible_entry".to_string(),
                title: format!("Unlinked {} \"{}\"", entry_type, name),
                description: format!(
                    "{} \"{}\" has no associations with any scenes, arcs, events, or other bible entries.",
                    capitalize_first(&entry_type),
                    name
                ),
                severity: "info".to_string(),
                scene_ids: vec![],
                bible_entry_ids: vec![id],
            })
            .collect();

        Ok(issues)
    }

    // ========================================================================
    // 4. Referential integrity
    // ========================================================================

    fn detect_referential_integrity(&self) -> Result<Vec<DetectedIssue>, String> {
        let mut issues = Vec::new();

        // 7a. setup_for_scene_id pointing to deleted scenes
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title, s.setup_for_scene_id
                 FROM scenes s
                 WHERE s.deleted_at IS NULL
                   AND s.setup_for_scene_id IS NOT NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM scenes target
                       WHERE target.id = s.setup_for_scene_id AND target.deleted_at IS NULL
                   )",
            )
            .map_err(|e| e.to_string())?;

        let broken_setups: Vec<(String, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (scene_id, scene_title, _target_id) in broken_setups {
            issues.push(DetectedIssue {
                issue_type: "referential_integrity".to_string(),
                title: format!("Broken setup reference in scene \"{}\"", scene_title),
                description: format!(
                    "Scene \"{}\" has a setup_for_scene_id that points to a deleted or \
                     non-existent scene. The reference should be cleared.",
                    scene_title
                ),
                severity: "error".to_string(),
                scene_ids: vec![scene_id],
                bible_entry_ids: vec![],
            });
        }

        // 7b. payoff_of_scene_id pointing to deleted scenes
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title, s.payoff_of_scene_id
                 FROM scenes s
                 WHERE s.deleted_at IS NULL
                   AND s.payoff_of_scene_id IS NOT NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM scenes target
                       WHERE target.id = s.payoff_of_scene_id AND target.deleted_at IS NULL
                   )",
            )
            .map_err(|e| e.to_string())?;

        let broken_payoffs: Vec<(String, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (scene_id, scene_title, _target_id) in broken_payoffs {
            issues.push(DetectedIssue {
                issue_type: "referential_integrity".to_string(),
                title: format!("Broken payoff reference in scene \"{}\"", scene_title),
                description: format!(
                    "Scene \"{}\" has a payoff_of_scene_id that points to a deleted or \
                     non-existent scene. The reference should be cleared.",
                    scene_title
                ),
                severity: "error".to_string(),
                scene_ids: vec![scene_id],
                bible_entry_ids: vec![],
            });
        }

        // 7c. bible_relationships pointing to deleted entries
        let mut stmt = self
            .conn
            .prepare(
                "SELECT br.id, br.source_id, br.target_id, br.relationship_type,
                        COALESCE(src.name, '[deleted]') as source_name,
                        COALESCE(tgt.name, '[deleted]') as target_name
                 FROM bible_relationships br
                 LEFT JOIN bible_entries src ON br.source_id = src.id AND src.deleted_at IS NULL
                 LEFT JOIN bible_entries tgt ON br.target_id = tgt.id AND tgt.deleted_at IS NULL
                 WHERE src.id IS NULL OR tgt.id IS NULL",
            )
            .map_err(|e| e.to_string())?;

        let broken_rels: Vec<(String, String, String, String, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (_rel_id, source_id, target_id, rel_type, source_name, target_name) in broken_rels {
            let mut entry_ids = Vec::new();
            if source_name != "[deleted]" {
                entry_ids.push(source_id.clone());
            }
            if target_name != "[deleted]" {
                entry_ids.push(target_id.clone());
            }

            issues.push(DetectedIssue {
                issue_type: "referential_integrity".to_string(),
                title: format!(
                    "Broken relationship: {} -> {} ({})",
                    source_name, target_name, rel_type
                ),
                description: format!(
                    "Bible relationship \"{}\" between \"{}\" and \"{}\" references a \
                     deleted entry. The relationship should be removed.",
                    rel_type, source_name, target_name
                ),
                severity: "warning".to_string(),
                scene_ids: vec![],
                bible_entry_ids: entry_ids,
            });
        }

        Ok(issues)
    }
}

/// Capitalizes the first character of a string.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
