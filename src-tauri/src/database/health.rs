//! Narrative health scoring for scenes.
//!
//! Computes a health score (0.0–1.0) for each scene based on weighted checks:
//! POV assigned, associations, timeline info, arc links, open issues,
//! TBD markers in done scenes, setup/payoff integrity, and tension level.

use crate::models::{HealthCheck, SceneHealth};
use rusqlite::params;

use super::Database;

/// Internal struct for scene data needed by health checks.
struct SceneRow {
    id: String,
    pov: Option<String>,
    status: String,
    text: String,
    tension: Option<String>,
    on_timeline: bool,
    time_point: Option<String>,
    time_start: Option<String>,
    setup_for_scene_id: Option<String>,
    payoff_of_scene_id: Option<String>,
    todos: Option<String>,
}

impl Database {
    /// Computes health scores for all non-deleted scenes in the project.
    pub fn get_scene_health_batch(&self) -> Result<Vec<SceneHealth>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.pov, s.status, s.text, s.tension,
                        s.on_timeline, s.time_point, s.time_start,
                        s.setup_for_scene_id, s.payoff_of_scene_id, s.todos
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                 ORDER BY c.position, s.position",
            )
            .map_err(|e| e.to_string())?;

        let scenes: Vec<SceneRow> = stmt
            .query_map([], |row| {
                Ok(SceneRow {
                    id: row.get(0)?,
                    pov: row.get(1)?,
                    status: row.get(2)?,
                    text: row.get(3)?,
                    tension: row.get(4)?,
                    on_timeline: row.get(5)?,
                    time_point: row.get(6)?,
                    time_start: row.get(7)?,
                    setup_for_scene_id: row.get(8)?,
                    payoff_of_scene_id: row.get(9)?,
                    todos: row.get(10)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        // Pre-fetch per-scene data in bulk for efficiency
        let scenes_with_associations = self.health_scenes_with_associations()?;
        let scenes_with_arcs = self.health_scenes_with_arcs()?;
        let scenes_with_open_issues = self.health_scenes_with_open_issues()?;

        let mut results = Vec::with_capacity(scenes.len());

        for scene in &scenes {
            let checks = Self::compute_health_checks(
                scene,
                scenes_with_associations.contains(&scene.id),
                scenes_with_arcs.contains(&scene.id),
                scenes_with_open_issues.contains(&scene.id),
                self.health_check_setup_payoff_intact(scene),
            );

            let total_weight: f64 = checks.iter().map(|c| c.weight).sum();
            let score = if total_weight > 0.0 {
                checks
                    .iter()
                    .map(|c| if c.passed { c.weight } else { 0.0 })
                    .sum::<f64>()
                    / total_weight
            } else {
                0.0
            };

            results.push(SceneHealth {
                scene_id: scene.id.clone(),
                score,
                checks,
            });
        }

        Ok(results)
    }

    fn compute_health_checks(
        scene: &SceneRow,
        has_associations: bool,
        has_arc: bool,
        has_open_issues: bool,
        setup_payoff_ok: bool,
    ) -> Vec<HealthCheck> {
        let mut checks = Vec::with_capacity(8);

        // 1. has_pov (0.15)
        checks.push(HealthCheck {
            name: "has_pov".to_string(),
            passed: scene.pov.as_ref().is_some_and(|p| !p.trim().is_empty()),
            weight: 0.15,
            label: "POV assigned".to_string(),
        });

        // 2. has_associations (0.15)
        checks.push(HealthCheck {
            name: "has_associations".to_string(),
            passed: has_associations,
            weight: 0.15,
            label: "Has canonical associations".to_string(),
        });

        // 3. has_timeline (0.10)
        let has_timeline = !scene.on_timeline
            || scene.time_point.as_ref().is_some_and(|t| !t.is_empty())
            || scene.time_start.as_ref().is_some_and(|t| !t.is_empty());
        checks.push(HealthCheck {
            name: "has_timeline".to_string(),
            passed: has_timeline,
            weight: 0.10,
            label: "Timeline info set".to_string(),
        });

        // 4. has_arc (0.15)
        checks.push(HealthCheck {
            name: "has_arc".to_string(),
            passed: has_arc,
            weight: 0.15,
            label: "Linked to an arc".to_string(),
        });

        // 5. no_open_issues (0.15)
        checks.push(HealthCheck {
            name: "no_open_issues".to_string(),
            passed: !has_open_issues,
            weight: 0.15,
            label: "No open issues".to_string(),
        });

        // 6. no_tbd_markers (0.15) — only relevant for done scenes
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
            weight: 0.15,
            label: "No TBD/TODO markers".to_string(),
        });

        // 7. setup_payoff_intact (0.10)
        checks.push(HealthCheck {
            name: "setup_payoff_intact".to_string(),
            passed: setup_payoff_ok,
            weight: 0.10,
            label: "Setup/payoff links valid".to_string(),
        });

        // 8. has_tension (0.05)
        checks.push(HealthCheck {
            name: "has_tension".to_string(),
            passed: scene.tension.as_ref().is_some_and(|t| !t.is_empty()),
            weight: 0.05,
            label: "Tension level defined".to_string(),
        });

        checks
    }

    /// Check that setup/payoff scene references still exist.
    fn health_check_setup_payoff_intact(&self, scene: &SceneRow) -> bool {
        if let Some(ref setup_id) = scene.setup_for_scene_id {
            if !setup_id.is_empty() {
                let exists: bool = self
                    .conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM scenes WHERE id = ?1 AND deleted_at IS NULL",
                        params![setup_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);
                if !exists {
                    return false;
                }
            }
        }
        if let Some(ref payoff_id) = scene.payoff_of_scene_id {
            if !payoff_id.is_empty() {
                let exists: bool = self
                    .conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM scenes WHERE id = ?1 AND deleted_at IS NULL",
                        params![payoff_id],
                        |row| row.get(0),
                    )
                    .unwrap_or(false);
                if !exists {
                    return false;
                }
            }
        }
        true
    }

    fn health_scenes_with_associations(&self) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT ca.scene_id
                 FROM canonical_associations ca
                 JOIN scenes s ON ca.scene_id = s.id
                 WHERE s.deleted_at IS NULL",
            )
            .map_err(|e| e.to_string())?;

        let ids = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(ids)
    }

    fn health_scenes_with_arcs(&self) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT sa.scene_id
                 FROM scene_arcs sa
                 JOIN scenes s ON sa.scene_id = s.id
                 WHERE s.deleted_at IS NULL",
            )
            .map_err(|e| e.to_string())?;

        let ids = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(ids)
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
