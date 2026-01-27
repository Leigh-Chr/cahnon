//! Scene health scoring — detects real structural problems only.
//!
//! Computes a health score (0.0–1.0) based on 3 problem checks:
//! open issues, TBD/TODO markers in done scenes, and broken setup/payoff links.
//! Metadata completeness (POV, arcs, associations, etc.) is intentionally excluded
//! to avoid penalising creative choices.

use crate::models::{HealthCheck, SceneHealth};
use rusqlite::params;

use super::Database;

/// Internal struct for scene data needed by health checks.
struct SceneRow {
    id: String,
    status: String,
    text: String,
    setup_for_scene_id: Option<String>,
    payoff_of_scene_id: Option<String>,
    todos: Option<String>,
}

impl Database {
    /// Computes health scores for all non-deleted scenes in the project.
    pub fn get_scene_health_batch(&self) -> Result<Vec<SceneHealth>, String> {
        let scenes = self.fetch_scene_rows_for_health()?;
        let scenes_with_open_issues = self.health_scenes_with_open_issues()?;

        let results = scenes
            .iter()
            .map(|scene| {
                let checks = Self::compute_health_checks(
                    scene,
                    scenes_with_open_issues.contains(&scene.id),
                    self.health_check_setup_payoff_intact(scene),
                );
                let score = Self::compute_health_score(&checks);
                SceneHealth {
                    scene_id: scene.id.clone(),
                    score,
                    checks,
                }
            })
            .collect();

        Ok(results)
    }

    fn fetch_scene_rows_for_health(&self) -> Result<Vec<SceneRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.status, s.text,
                        s.setup_for_scene_id, s.payoff_of_scene_id, s.todos
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                 ORDER BY c.position, s.position",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SceneRow {
                    id: row.get(0)?,
                    status: row.get(1)?,
                    text: row.get(2)?,
                    setup_for_scene_id: row.get(3)?,
                    payoff_of_scene_id: row.get(4)?,
                    todos: row.get(5)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    fn compute_health_score(checks: &[HealthCheck]) -> f64 {
        let total_weight: f64 = checks.iter().map(|c| c.weight).sum();
        if total_weight > 0.0 {
            checks
                .iter()
                .map(|c| if c.passed { c.weight } else { 0.0 })
                .sum::<f64>()
                / total_weight
        } else {
            1.0
        }
    }

    fn compute_health_checks(
        scene: &SceneRow,
        has_open_issues: bool,
        setup_payoff_ok: bool,
    ) -> Vec<HealthCheck> {
        let mut checks = Vec::with_capacity(3);

        // 1. no_open_issues — actual problems flagged on this scene
        checks.push(HealthCheck {
            name: "no_open_issues".to_string(),
            passed: !has_open_issues,
            weight: 1.0,
            label: "No open issues".to_string(),
        });

        // 2. no_tbd_markers — only relevant for done scenes
        let has_tbd = if scene.status == "done" {
            let text_lower = scene.text.to_lowercase();
            text_lower.contains("tbd")
                || text_lower.contains("todo")
                || text_lower.contains("xxx")
                || text_lower.contains("??")
        } else {
            false
        };
        let has_todo_items = if scene.status == "done" {
            scene.todos.as_ref().is_some_and(|t| !t.trim().is_empty())
        } else {
            false
        };
        checks.push(HealthCheck {
            name: "no_tbd_markers".to_string(),
            passed: !has_tbd && !has_todo_items,
            weight: 1.0,
            label: "No TBD/TODO markers".to_string(),
        });

        // 3. setup_payoff_intact — broken structural links
        checks.push(HealthCheck {
            name: "setup_payoff_intact".to_string(),
            passed: setup_payoff_ok,
            weight: 1.0,
            label: "Setup/payoff links valid".to_string(),
        });

        checks
    }

    /// Check that setup/payoff scene references still exist.
    fn health_check_setup_payoff_intact(&self, scene: &SceneRow) -> bool {
        self.scene_ref_exists(&scene.setup_for_scene_id)
            && self.scene_ref_exists(&scene.payoff_of_scene_id)
    }

    /// Returns true if the optional scene ID is None, empty, or points to an existing scene.
    fn scene_ref_exists(&self, scene_id: &Option<String>) -> bool {
        match scene_id {
            Some(id) if !id.is_empty() => self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM scenes WHERE id = ?1 AND deleted_at IS NULL",
                    params![id],
                    |row| row.get(0),
                )
                .unwrap_or(false),
            _ => true,
        }
    }

    fn health_scenes_with_open_issues(&self) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT isc.scene_id
                 FROM issue_scenes isc
                 JOIN issues i ON isc.issue_id = i.id
                 WHERE i.status = 'open'",
            )
            .map_err(|e| e.to_string())?;

        let ids = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(ids)
    }
}
