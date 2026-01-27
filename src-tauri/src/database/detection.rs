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
            Self::detect_isolated_scenes,
            Self::detect_arc_inconsistencies,
            Self::detect_narrative_patterns,
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
    // 4. Isolated scenes (no links, no POV, no tags)
    // ========================================================================

    fn detect_isolated_scenes(&self) -> Result<Vec<DetectedIssue>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title
                 FROM scenes s
                 WHERE s.deleted_at IS NULL
                   AND (s.pov IS NULL OR s.pov = '')
                   AND (s.tags IS NULL OR s.tags = '')
                   AND NOT EXISTS (
                       SELECT 1 FROM canonical_associations ca WHERE ca.scene_id = s.id
                   )
                   AND NOT EXISTS (
                       SELECT 1 FROM scene_arcs sa
                       JOIN arcs a ON sa.arc_id = a.id
                       WHERE sa.scene_id = s.id AND a.deleted_at IS NULL
                   )
                   AND NOT EXISTS (
                       SELECT 1 FROM event_scenes es
                       JOIN events ev ON es.event_id = ev.id
                       WHERE es.scene_id = s.id AND ev.deleted_at IS NULL
                   )
                 ORDER BY s.position",
            )
            .map_err(|e| e.to_string())?;

        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let issues: Vec<DetectedIssue> = rows
            .into_iter()
            .map(|(id, title)| DetectedIssue {
                issue_type: "isolated_scene".to_string(),
                title: format!("Isolated scene \"{}\"", title),
                description: format!(
                    "Scene \"{}\" has no associations, no POV character, and no tags. \
                     Consider linking it to bible entries, arcs, or events.",
                    title
                ),
                severity: "info".to_string(),
                scene_ids: vec![id],
                bible_entry_ids: vec![],
            })
            .collect();

        Ok(issues)
    }

    // ========================================================================
    // 5. Arc inconsistencies
    // ========================================================================

    fn detect_arc_inconsistencies(&self) -> Result<Vec<DetectedIssue>, String> {
        let mut issues = Vec::new();

        // 5a. Arcs marked 'complete' that still have scenes in 'draft'
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.name, s.id, s.title
                 FROM arcs a
                 JOIN scene_arcs sa ON a.id = sa.arc_id
                 JOIN scenes s ON sa.scene_id = s.id
                 WHERE a.deleted_at IS NULL
                   AND s.deleted_at IS NULL
                   AND a.status = 'complete'
                   AND s.status = 'draft'
                 ORDER BY a.name, s.position",
            )
            .map_err(|e| e.to_string())?;

        let draft_in_complete: Vec<(String, String, String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        // Group by arc
        let mut arc_draft_scenes: std::collections::HashMap<
            (String, String),
            Vec<(String, String)>,
        > = std::collections::HashMap::new();
        for (arc_id, arc_name, scene_id, scene_title) in draft_in_complete {
            arc_draft_scenes
                .entry((arc_id, arc_name))
                .or_default()
                .push((scene_id, scene_title));
        }

        for ((_arc_id, arc_name), scenes) in &arc_draft_scenes {
            let scene_names: Vec<&str> = scenes.iter().map(|(_, t)| t.as_str()).collect();
            let scene_ids: Vec<String> = scenes.iter().map(|(id, _)| id.clone()).collect();
            issues.push(DetectedIssue {
                issue_type: "arc_inconsistency".to_string(),
                title: format!("Complete arc \"{}\" has draft scenes", arc_name),
                description: format!(
                    "Arc \"{}\" is marked as complete, but {} scene(s) are still in draft: {}",
                    arc_name,
                    scenes.len(),
                    scene_names.join(", ")
                ),
                severity: "warning".to_string(),
                scene_ids,
                bible_entry_ids: vec![],
            });
        }

        // 5b. Arcs with 0 scenes
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.name
                 FROM arcs a
                 WHERE a.deleted_at IS NULL
                   AND NOT EXISTS (
                       SELECT 1 FROM scene_arcs sa
                       JOIN scenes s ON sa.scene_id = s.id
                       WHERE sa.arc_id = a.id AND s.deleted_at IS NULL
                   )",
            )
            .map_err(|e| e.to_string())?;

        let empty_arcs: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (_arc_id, arc_name) in empty_arcs {
            issues.push(DetectedIssue {
                issue_type: "arc_inconsistency".to_string(),
                title: format!("Arc \"{}\" has no scenes", arc_name),
                description: format!(
                    "Arc \"{}\" is not linked to any scenes. Consider adding scenes or removing the arc.",
                    arc_name
                ),
                severity: "info".to_string(),
                scene_ids: vec![],
                bible_entry_ids: vec![],
            });
        }

        // 5c. Arcs where all scenes are in the same chapter
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.name, COUNT(DISTINCT s.chapter_id) as chapter_count,
                        MIN(c.title) as chapter_title, COUNT(sa.id) as scene_count
                 FROM arcs a
                 JOIN scene_arcs sa ON a.id = sa.arc_id
                 JOIN scenes s ON sa.scene_id = s.id
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE a.deleted_at IS NULL AND s.deleted_at IS NULL AND c.deleted_at IS NULL
                 GROUP BY a.id, a.name
                 HAVING chapter_count = 1 AND scene_count > 1",
            )
            .map_err(|e| e.to_string())?;

        let single_chapter_arcs: Vec<(String, String, i32, String, i32)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (_arc_id, arc_name, _, chapter_title, scene_count) in single_chapter_arcs {
            issues.push(DetectedIssue {
                issue_type: "arc_inconsistency".to_string(),
                title: format!("Arc \"{}\" spans only one chapter", arc_name),
                description: format!(
                    "All {} scenes of arc \"{}\" are in chapter \"{}\". \
                     Arcs typically span multiple chapters.",
                    scene_count, arc_name, chapter_title
                ),
                severity: "info".to_string(),
                scene_ids: vec![],
                bible_entry_ids: vec![],
            });
        }

        Ok(issues)
    }

    // ========================================================================
    // 6. Narrative patterns
    // ========================================================================

    fn detect_narrative_patterns(&self) -> Result<Vec<DetectedIssue>, String> {
        let mut issues = Vec::new();

        // Load all non-deleted scenes in manuscript order
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title, s.pov, s.has_conflict, s.tension, c.position as ch_pos
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                 ORDER BY c.position, s.position",
            )
            .map_err(|e| e.to_string())?;

        #[allow(clippy::type_complexity)]
        let scenes: Vec<(
            String,
            String,
            Option<String>,
            Option<bool>,
            Option<String>,
            i32,
        )> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get::<_, Option<i32>>(3)?.map(|v| v != 0),
                    row.get(4)?,
                    row.get(5)?,
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        if scenes.is_empty() {
            return Ok(issues);
        }

        // 6a. 5+ consecutive scenes without conflict
        let mut no_conflict_run: Vec<&str> = Vec::new();
        let mut no_conflict_ids: Vec<String> = Vec::new();

        for (id, title, _, has_conflict, _, _) in &scenes {
            if has_conflict == &Some(true) {
                if no_conflict_run.len() >= 5 {
                    issues.push(DetectedIssue {
                        issue_type: "narrative_pattern".to_string(),
                        title: format!(
                            "{} consecutive scenes without conflict",
                            no_conflict_run.len()
                        ),
                        description: format!(
                            "Scenes {} through {} ({} scenes) have no conflict flagged. \
                             Consider adding tension or conflict to maintain reader engagement.",
                            no_conflict_run.first().unwrap_or(&""),
                            no_conflict_run.last().unwrap_or(&""),
                            no_conflict_run.len()
                        ),
                        severity: "info".to_string(),
                        scene_ids: no_conflict_ids.clone(),
                        bible_entry_ids: vec![],
                    });
                }
                no_conflict_run.clear();
                no_conflict_ids.clear();
            } else {
                no_conflict_run.push(title.as_str());
                no_conflict_ids.push(id.clone());
            }
        }
        // Check trailing run
        if no_conflict_run.len() >= 5 {
            issues.push(DetectedIssue {
                issue_type: "narrative_pattern".to_string(),
                title: format!(
                    "{} consecutive scenes without conflict",
                    no_conflict_run.len()
                ),
                description: format!(
                    "Scenes \"{}\" through \"{}\" ({} scenes) have no conflict flagged. \
                     Consider adding tension or conflict to maintain reader engagement.",
                    no_conflict_run.first().unwrap_or(&""),
                    no_conflict_run.last().unwrap_or(&""),
                    no_conflict_run.len()
                ),
                severity: "info".to_string(),
                scene_ids: no_conflict_ids,
                bible_entry_ids: vec![],
            });
        }

        // 6b. 4+ consecutive scenes with same POV
        let mut same_pov_run: Vec<&str> = Vec::new();
        let mut same_pov_ids: Vec<String> = Vec::new();
        let mut current_pov: Option<&str> = None;

        for (id, title, pov, _, _, _) in &scenes {
            let pov_str = pov.as_deref().unwrap_or("");
            if pov_str.is_empty() {
                // No POV set -- reset run
                if same_pov_run.len() >= 4 {
                    if let Some(pov_name) = current_pov {
                        issues.push(DetectedIssue {
                            issue_type: "narrative_pattern".to_string(),
                            title: format!(
                                "{} consecutive scenes with same POV ({})",
                                same_pov_run.len(),
                                pov_name
                            ),
                            description: format!(
                                "Scenes \"{}\" through \"{}\" ({} scenes) all share the same \
                                 POV character \"{}\". Consider varying perspectives.",
                                same_pov_run.first().unwrap_or(&""),
                                same_pov_run.last().unwrap_or(&""),
                                same_pov_run.len(),
                                pov_name
                            ),
                            severity: "info".to_string(),
                            scene_ids: same_pov_ids.clone(),
                            bible_entry_ids: vec![],
                        });
                    }
                }
                same_pov_run.clear();
                same_pov_ids.clear();
                current_pov = None;
            } else if current_pov == Some(pov_str) {
                same_pov_run.push(title.as_str());
                same_pov_ids.push(id.clone());
            } else {
                // POV changed
                if same_pov_run.len() >= 4 {
                    if let Some(pov_name) = current_pov {
                        issues.push(DetectedIssue {
                            issue_type: "narrative_pattern".to_string(),
                            title: format!(
                                "{} consecutive scenes with same POV ({})",
                                same_pov_run.len(),
                                pov_name
                            ),
                            description: format!(
                                "Scenes \"{}\" through \"{}\" ({} scenes) all share the same \
                                 POV character \"{}\". Consider varying perspectives.",
                                same_pov_run.first().unwrap_or(&""),
                                same_pov_run.last().unwrap_or(&""),
                                same_pov_run.len(),
                                pov_name
                            ),
                            severity: "info".to_string(),
                            scene_ids: same_pov_ids.clone(),
                            bible_entry_ids: vec![],
                        });
                    }
                }
                same_pov_run.clear();
                same_pov_ids.clear();
                same_pov_run.push(title.as_str());
                same_pov_ids.push(id.clone());
                current_pov = Some(pov_str);
            }
        }
        // Check trailing run
        if same_pov_run.len() >= 4 {
            if let Some(pov_name) = current_pov {
                issues.push(DetectedIssue {
                    issue_type: "narrative_pattern".to_string(),
                    title: format!(
                        "{} consecutive scenes with same POV ({})",
                        same_pov_run.len(),
                        pov_name
                    ),
                    description: format!(
                        "Scenes \"{}\" through \"{}\" ({} scenes) all share the same \
                         POV character \"{}\". Consider varying perspectives.",
                        same_pov_run.first().unwrap_or(&""),
                        same_pov_run.last().unwrap_or(&""),
                        same_pov_run.len(),
                        pov_name
                    ),
                    severity: "info".to_string(),
                    scene_ids: same_pov_ids,
                    bible_entry_ids: vec![],
                });
            }
        }

        // 6c. Tension plateau: same tension value for 5+ consecutive scenes
        let mut same_tension_run: Vec<&str> = Vec::new();
        let mut same_tension_ids: Vec<String> = Vec::new();
        let mut current_tension: Option<&str> = None;

        for (id, title, _, _, tension, _) in &scenes {
            let tension_str = tension.as_deref().unwrap_or("");
            if tension_str.is_empty() {
                // No tension set -- reset run
                if same_tension_run.len() >= 5 {
                    if let Some(tension_val) = current_tension {
                        issues.push(DetectedIssue {
                            issue_type: "narrative_pattern".to_string(),
                            title: format!(
                                "Tension plateau ({}) for {} consecutive scenes",
                                tension_val,
                                same_tension_run.len()
                            ),
                            description: format!(
                                "Scenes \"{}\" through \"{}\" ({} scenes) all have the same \
                                 tension level \"{}\". Consider varying the tension to create \
                                 a more dynamic narrative arc.",
                                same_tension_run.first().unwrap_or(&""),
                                same_tension_run.last().unwrap_or(&""),
                                same_tension_run.len(),
                                tension_val
                            ),
                            severity: "info".to_string(),
                            scene_ids: same_tension_ids.clone(),
                            bible_entry_ids: vec![],
                        });
                    }
                }
                same_tension_run.clear();
                same_tension_ids.clear();
                current_tension = None;
            } else if current_tension == Some(tension_str) {
                same_tension_run.push(title.as_str());
                same_tension_ids.push(id.clone());
            } else {
                // Tension changed
                if same_tension_run.len() >= 5 {
                    if let Some(tension_val) = current_tension {
                        issues.push(DetectedIssue {
                            issue_type: "narrative_pattern".to_string(),
                            title: format!(
                                "Tension plateau ({}) for {} consecutive scenes",
                                tension_val,
                                same_tension_run.len()
                            ),
                            description: format!(
                                "Scenes \"{}\" through \"{}\" ({} scenes) all have the same \
                                 tension level \"{}\". Consider varying the tension to create \
                                 a more dynamic narrative arc.",
                                same_tension_run.first().unwrap_or(&""),
                                same_tension_run.last().unwrap_or(&""),
                                same_tension_run.len(),
                                tension_val
                            ),
                            severity: "info".to_string(),
                            scene_ids: same_tension_ids.clone(),
                            bible_entry_ids: vec![],
                        });
                    }
                }
                same_tension_run.clear();
                same_tension_ids.clear();
                same_tension_run.push(title.as_str());
                same_tension_ids.push(id.clone());
                current_tension = Some(tension_str);
            }
        }
        // Check trailing run
        if same_tension_run.len() >= 5 {
            if let Some(tension_val) = current_tension {
                issues.push(DetectedIssue {
                    issue_type: "narrative_pattern".to_string(),
                    title: format!(
                        "Tension plateau ({}) for {} consecutive scenes",
                        tension_val,
                        same_tension_run.len()
                    ),
                    description: format!(
                        "Scenes \"{}\" through \"{}\" ({} scenes) all have the same \
                         tension level \"{}\". Consider varying the tension to create \
                         a more dynamic narrative arc.",
                        same_tension_run.first().unwrap_or(&""),
                        same_tension_run.last().unwrap_or(&""),
                        same_tension_run.len(),
                        tension_val
                    ),
                    severity: "info".to_string(),
                    scene_ids: same_tension_ids,
                    bible_entry_ids: vec![],
                });
            }
        }

        Ok(issues)
    }

    // ========================================================================
    // 7. Referential integrity
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
