//! Project analysis detectors.
//!
//! This module implements automated detection of potential issues in a project,
//! including narrative inconsistencies, orphaned data, broken references, and
//! structural patterns that may warrant the writer's attention.

use crate::models::DetectedIssue;
use rusqlite::params;

use super::Database;

/// A scene reference used for setup/payoff validation.
struct SceneRef<'a> {
    id: &'a str,
    title: &'a str,
    chapter_id: &'a str,
    position: i32,
}

/// Placeholder markers to look for in scene text.
const UPPER_MARKERS: &[&str] = &["TBD", "TODO", "XXX"];
const PLAIN_MARKERS: &[&str] = &["??"];

/// Returns placeholder markers found in the given text.
///
/// Checks for `TBD`, `TODO`, `XXX` (case-insensitive) and `??` (literal).
fn find_placeholder_markers(text: &str) -> Vec<&'static str> {
    let plain = super::HTML_TAG_REGEX.replace_all(text, " ");
    let upper = plain.to_uppercase();

    let mut found: Vec<&'static str> = UPPER_MARKERS
        .iter()
        .filter(|m| upper.contains(**m))
        .copied()
        .collect();

    found.extend(
        PLAIN_MARKERS
            .iter()
            .filter(|m| plain.contains(**m))
            .copied(),
    );

    found
}

/// Describes a scene reference direction (setup or payoff) for reuse across checks.
struct RefDirection {
    /// Whether the scene must come *before* its target (`true` for setup).
    must_come_before: bool,
    /// Generates the title when the target scene is missing.
    missing_title: fn(&str) -> String,
    /// Generates the description when the target scene is missing.
    missing_desc: fn(&str) -> String,
    /// Generates the title when ordering is wrong.
    ordering_title: fn(&str) -> String,
    /// Generates the description when ordering is wrong.
    ordering_desc: fn(&str) -> String,
}

const SETUP_DIRECTION: RefDirection = RefDirection {
    must_come_before: true,
    missing_title: |t| format!("Setup target missing for scene \"{}\"", t),
    missing_desc: |t| {
        format!(
            "Scene \"{}\" references a setup target scene that no longer exists.",
            t
        )
    },
    ordering_title: |t| format!("Setup scene \"{}\" comes after its target", t),
    ordering_desc: |t| {
        format!(
            "Scene \"{}\" is marked as setup for another scene, but it \
             appears at the same position or later in the manuscript. \
             Setup scenes should come before their target.",
            t
        )
    },
};

const PAYOFF_DIRECTION: RefDirection = RefDirection {
    must_come_before: false,
    missing_title: |t| format!("Payoff source missing for scene \"{}\"", t),
    missing_desc: |t| {
        format!(
            "Scene \"{}\" references a payoff source scene that no longer exists.",
            t
        )
    },
    ordering_title: |t| format!("Payoff scene \"{}\" comes before its source", t),
    ordering_desc: |t| {
        format!(
            "Scene \"{}\" is marked as payoff of another scene, but it \
             appears at the same position or earlier in the manuscript. \
             Payoff scenes should come after the scene they pay off.",
            t
        )
    },
};

/// Computes a global ordering value from chapter position and scene position.
fn global_position(chapter_pos: i32, scene_pos: i32) -> i64 {
    chapter_pos as i64 * 1_000_000 + scene_pos as i64
}

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

        let issues = rows
            .into_iter()
            .filter_map(|(id, title, text)| {
                let markers = find_placeholder_markers(&text);
                if markers.is_empty() {
                    return None;
                }
                Some(DetectedIssue {
                    issue_type: "tbd_in_done".to_string(),
                    title: format!("Placeholder text in completed scene \"{}\"", title),
                    description: format!(
                        "Scene \"{}\" is marked as done but still contains placeholder markers: {}",
                        title,
                        markers.join(", ")
                    ),
                    severity: "warning".to_string(),
                    scene_ids: vec![id],
                    bible_entry_ids: vec![],
                })
            })
            .collect();

        Ok(issues)
    }

    // ========================================================================
    // 2. Broken setup/payoff references
    // ========================================================================

    fn detect_broken_setup_payoff(&self) -> Result<Vec<DetectedIssue>, String> {
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

        let mut issues = Vec::new();

        for (scene_id, scene_title, scene_chapter_id, scene_position, setup_for, payoff_of) in &rows
        {
            let scene_ref = SceneRef {
                id: scene_id,
                title: scene_title,
                chapter_id: scene_chapter_id,
                position: *scene_position,
            };

            if let Some(target_id) = setup_for {
                self.check_scene_ref(&mut issues, &scene_ref, target_id, &SETUP_DIRECTION);
            }

            if let Some(target_id) = payoff_of {
                self.check_scene_ref(&mut issues, &scene_ref, target_id, &PAYOFF_DIRECTION);
            }
        }

        Ok(issues)
    }

    /// Validates a single setup/payoff reference and appends any issues found.
    fn check_scene_ref(
        &self,
        issues: &mut Vec<DetectedIssue>,
        scene: &SceneRef,
        target_id: &str,
        dir: &RefDirection,
    ) {
        let target_info = match self.get_scene_position_info(target_id) {
            Ok(info) => info,
            Err(_) => return,
        };

        match target_info {
            Some((_target_chapter_id, target_position, target_chapter_pos)) => {
                let scene_chapter_pos = self.get_chapter_position(scene.chapter_id).unwrap_or(0);
                let scene_global = global_position(scene_chapter_pos, scene.position);
                let target_global = global_position(target_chapter_pos, target_position);

                let ordering_violated = if dir.must_come_before {
                    scene_global >= target_global
                } else {
                    scene_global <= target_global
                };

                if ordering_violated {
                    issues.push(DetectedIssue {
                        issue_type: "broken_setup_payoff".to_string(),
                        title: (dir.ordering_title)(scene.title),
                        description: (dir.ordering_desc)(scene.title),
                        severity: "warning".to_string(),
                        scene_ids: vec![scene.id.to_string(), target_id.to_string()],
                        bible_entry_ids: vec![],
                    });
                }
            }
            None => {
                issues.push(DetectedIssue {
                    issue_type: "broken_setup_payoff".to_string(),
                    title: (dir.missing_title)(scene.title),
                    description: (dir.missing_desc)(scene.title),
                    severity: "error".to_string(),
                    scene_ids: vec![scene.id.to_string()],
                    bible_entry_ids: vec![],
                });
            }
        }
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
        self.detect_broken_scene_refs(&mut issues, "setup_for_scene_id", "setup")?;
        self.detect_broken_scene_refs(&mut issues, "payoff_of_scene_id", "payoff")?;
        self.detect_broken_bible_relationships(&mut issues)?;
        Ok(issues)
    }

    /// Detects scenes whose `ref_column` points to a deleted or missing scene.
    fn detect_broken_scene_refs(
        &self,
        issues: &mut Vec<DetectedIssue>,
        ref_column: &str,
        label: &str,
    ) -> Result<(), String> {
        // ref_column is always a compile-time literal, safe to interpolate.
        let sql = format!(
            "SELECT s.id, s.title, s.{col}
             FROM scenes s
             WHERE s.deleted_at IS NULL
               AND s.{col} IS NOT NULL
               AND NOT EXISTS (
                   SELECT 1 FROM scenes target
                   WHERE target.id = s.{col} AND target.deleted_at IS NULL
               )",
            col = ref_column
        );

        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;

        let rows: Vec<(String, String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (scene_id, scene_title, _target_id) in rows {
            issues.push(DetectedIssue {
                issue_type: "referential_integrity".to_string(),
                title: format!("Broken {} reference in scene \"{}\"", label, scene_title),
                description: format!(
                    "Scene \"{}\" has a {} that points to a deleted or \
                     non-existent scene. The reference should be cleared.",
                    scene_title, ref_column
                ),
                severity: "error".to_string(),
                scene_ids: vec![scene_id],
                bible_entry_ids: vec![],
            });
        }

        Ok(())
    }

    /// Detects bible relationships where one or both entries have been deleted.
    fn detect_broken_bible_relationships(
        &self,
        issues: &mut Vec<DetectedIssue>,
    ) -> Result<(), String> {
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

        let rows: Vec<(String, String, String, String, String, String)> = stmt
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

        for (_rel_id, source_id, target_id, rel_type, source_name, target_name) in rows {
            let entry_ids = [(&source_name, &source_id), (&target_name, &target_id)]
                .iter()
                .filter(|(name, _)| name.as_str() != "[deleted]")
                .map(|(_, id)| (*id).clone())
                .collect();

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

        Ok(())
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
