//! Timeline conflict detection

use crate::models::{Scene, TimelineConflict};

use super::Database;

/// Represents a time range for conflict checking
struct TimeRange {
    start: String,
    end: String,
}

impl TimeRange {
    /// Create a TimeRange from a scene's time fields
    fn from_scene(scene: &Scene) -> Option<Self> {
        // If time_point is set, treat it as both start and end
        if let Some(point) = &scene.time_point {
            return Some(TimeRange {
                start: point.clone(),
                end: point.clone(),
            });
        }

        // If time_start is set, use it with optional time_end
        if let Some(start) = &scene.time_start {
            return Some(TimeRange {
                start: start.clone(),
                end: scene.time_end.clone().unwrap_or_else(|| start.clone()),
            });
        }

        None
    }

    /// Check if this range overlaps with another
    /// Uses string comparison which works for ISO dates, numbers, and many other formats
    fn overlaps(&self, other: &TimeRange) -> bool {
        // Two ranges [a1, a2] and [b1, b2] overlap if a1 <= b2 AND b1 <= a2
        self.start <= other.end && other.start <= self.end
    }

    /// Get a display string for this range
    fn display(&self) -> String {
        if self.start == self.end {
            self.start.clone()
        } else {
            format!("{} - {}", self.start, self.end)
        }
    }
}

impl Database {
    pub fn detect_timeline_conflicts(&self) -> Result<Vec<TimelineConflict>, String> {
        let scenes = self.get_all_scenes_for_timeline()?;
        let mut conflicts = Vec::new();

        conflicts.extend(self.detect_overlapping_time_conflicts(&scenes));
        conflicts.extend(Self::detect_missing_time_conflicts(&scenes));

        Ok(conflicts)
    }

    fn detect_overlapping_time_conflicts(&self, scenes: &[Scene]) -> Vec<TimelineConflict> {
        let mut conflicts = Vec::new();
        let scenes_by_pov = Self::group_scenes_by_pov(scenes);

        for (pov, pov_scenes) in &scenes_by_pov {
            conflicts.extend(Self::check_pov_time_conflicts(pov, pov_scenes));
        }
        conflicts
    }

    fn group_scenes_by_pov(scenes: &[Scene]) -> std::collections::HashMap<String, Vec<&Scene>> {
        let mut scenes_by_pov = std::collections::HashMap::new();
        for scene in scenes {
            if let Some(pov) = &scene.pov {
                scenes_by_pov
                    .entry(pov.clone())
                    .or_insert_with(Vec::new)
                    .push(scene);
            }
        }
        scenes_by_pov
    }

    fn check_pov_time_conflicts(pov: &str, pov_scenes: &[&Scene]) -> Vec<TimelineConflict> {
        let mut conflicts = Vec::new();
        if pov_scenes.len() < 2 {
            return conflicts;
        }

        for i in 0..pov_scenes.len() {
            for j in (i + 1)..pov_scenes.len() {
                if let Some(conflict) =
                    Self::check_scene_pair_conflict(pov, pov_scenes[i], pov_scenes[j])
                {
                    conflicts.push(conflict);
                }
            }
        }
        conflicts
    }

    fn check_scene_pair_conflict(
        pov: &str,
        scene_a: &Scene,
        scene_b: &Scene,
    ) -> Option<TimelineConflict> {
        let range_a = TimeRange::from_scene(scene_a)?;
        let range_b = TimeRange::from_scene(scene_b)?;

        if range_a.overlaps(&range_b) {
            let conflict_type = if range_a.start == range_a.end
                && range_b.start == range_b.end
                && range_a.start == range_b.start
            {
                "same_time"
            } else {
                "overlapping_time"
            };

            Some(TimelineConflict {
                conflict_type: conflict_type.to_string(),
                description: format!(
                    "{} appears in overlapping scenes: '{}' ({}) and '{}' ({})",
                    pov,
                    scene_a.title,
                    range_a.display(),
                    scene_b.title,
                    range_b.display()
                ),
                scene_ids: vec![scene_a.id.clone(), scene_b.id.clone()],
                character_id: None,
                character_name: Some(pov.to_string()),
                time_point: Some(range_a.start.clone()),
            })
        } else {
            None
        }
    }

    fn detect_missing_time_conflicts(scenes: &[Scene]) -> Vec<TimelineConflict> {
        scenes
            .iter()
            .filter(|s| s.pov.is_some() && s.time_point.is_none() && s.time_start.is_none())
            .map(|scene| TimelineConflict {
                conflict_type: "missing_time".to_string(),
                description: format!(
                    "Scene '{}' has a POV but no timeline placement",
                    scene.title
                ),
                scene_ids: vec![scene.id.clone()],
                character_id: None,
                character_name: scene.pov.clone(),
                time_point: None,
            })
            .collect()
    }
}
