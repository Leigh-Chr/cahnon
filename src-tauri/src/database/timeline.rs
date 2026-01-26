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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_scene(
        id: &str,
        title: &str,
        pov: Option<&str>,
        time_point: Option<&str>,
        time_start: Option<&str>,
        time_end: Option<&str>,
    ) -> Scene {
        Scene {
            id: id.to_string(),
            chapter_id: "ch1".to_string(),
            title: title.to_string(),
            summary: None,
            text: String::new(),
            status: "draft".to_string(),
            pov: pov.map(|s| s.to_string()),
            tags: None,
            notes: None,
            todos: None,
            word_target: None,
            time_point: time_point.map(|s| s.to_string()),
            time_start: time_start.map(|s| s.to_string()),
            time_end: time_end.map(|s| s.to_string()),
            on_timeline: true,
            position: 0,
            pov_goal: None,
            has_conflict: None,
            has_change: None,
            tension: None,
            setup_for_scene_id: None,
            payoff_of_scene_id: None,
            revision_notes: None,
            revision_checklist: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    // --- TimeRange::from_scene ---

    #[test]
    fn test_time_range_from_scene_with_time_point() {
        let scene = make_scene("s1", "S1", Some("Alice"), Some("1200"), None, None);
        let range = TimeRange::from_scene(&scene).unwrap();
        assert_eq!(range.start, "1200");
        assert_eq!(range.end, "1200");
    }

    #[test]
    fn test_time_range_from_scene_with_start_and_end() {
        let scene = make_scene("s1", "S1", Some("Alice"), None, Some("1000"), Some("1200"));
        let range = TimeRange::from_scene(&scene).unwrap();
        assert_eq!(range.start, "1000");
        assert_eq!(range.end, "1200");
    }

    #[test]
    fn test_time_range_from_scene_with_start_only() {
        let scene = make_scene("s1", "S1", Some("Alice"), None, Some("1000"), None);
        let range = TimeRange::from_scene(&scene).unwrap();
        assert_eq!(range.start, "1000");
        assert_eq!(range.end, "1000"); // Falls back to start
    }

    #[test]
    fn test_time_range_from_scene_no_time() {
        let scene = make_scene("s1", "S1", Some("Alice"), None, None, None);
        assert!(TimeRange::from_scene(&scene).is_none());
    }

    #[test]
    fn test_time_range_from_scene_time_point_takes_priority() {
        // If both time_point and time_start are set, time_point wins
        let scene = make_scene(
            "s1",
            "S1",
            Some("Alice"),
            Some("1200"),
            Some("1000"),
            Some("1100"),
        );
        let range = TimeRange::from_scene(&scene).unwrap();
        assert_eq!(range.start, "1200");
        assert_eq!(range.end, "1200");
    }

    // --- TimeRange::overlaps ---

    #[test]
    fn test_overlaps_same_range() {
        let a = TimeRange {
            start: "100".to_string(),
            end: "200".to_string(),
        };
        let b = TimeRange {
            start: "100".to_string(),
            end: "200".to_string(),
        };
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_overlaps_partial_overlap() {
        let a = TimeRange {
            start: "100".to_string(),
            end: "200".to_string(),
        };
        let b = TimeRange {
            start: "150".to_string(),
            end: "250".to_string(),
        };
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a)); // Symmetric
    }

    #[test]
    fn test_overlaps_contained() {
        let a = TimeRange {
            start: "100".to_string(),
            end: "300".to_string(),
        };
        let b = TimeRange {
            start: "150".to_string(),
            end: "200".to_string(),
        };
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn test_overlaps_touching_boundaries() {
        // [100, 200] and [200, 300] — touching at 200 counts as overlap
        let a = TimeRange {
            start: "100".to_string(),
            end: "200".to_string(),
        };
        let b = TimeRange {
            start: "200".to_string(),
            end: "300".to_string(),
        };
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_overlaps_disjoint() {
        let a = TimeRange {
            start: "100".to_string(),
            end: "200".to_string(),
        };
        let b = TimeRange {
            start: "300".to_string(),
            end: "400".to_string(),
        };
        assert!(!a.overlaps(&b));
        assert!(!b.overlaps(&a));
    }

    #[test]
    fn test_overlaps_point_ranges_same() {
        let a = TimeRange {
            start: "100".to_string(),
            end: "100".to_string(),
        };
        let b = TimeRange {
            start: "100".to_string(),
            end: "100".to_string(),
        };
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_overlaps_point_ranges_different() {
        let a = TimeRange {
            start: "100".to_string(),
            end: "100".to_string(),
        };
        let b = TimeRange {
            start: "200".to_string(),
            end: "200".to_string(),
        };
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn test_overlaps_point_within_range() {
        let a = TimeRange {
            start: "100".to_string(),
            end: "300".to_string(),
        };
        let b = TimeRange {
            start: "200".to_string(),
            end: "200".to_string(),
        };
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_overlaps_iso_dates() {
        // String comparison works for ISO dates
        let a = TimeRange {
            start: "2024-01-01".to_string(),
            end: "2024-06-30".to_string(),
        };
        let b = TimeRange {
            start: "2024-03-15".to_string(),
            end: "2024-09-01".to_string(),
        };
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_overlaps_iso_dates_disjoint() {
        let a = TimeRange {
            start: "2024-01-01".to_string(),
            end: "2024-03-01".to_string(),
        };
        let b = TimeRange {
            start: "2024-06-01".to_string(),
            end: "2024-09-01".to_string(),
        };
        assert!(!a.overlaps(&b));
    }

    // --- TimeRange::display ---

    #[test]
    fn test_display_point() {
        let r = TimeRange {
            start: "1200".to_string(),
            end: "1200".to_string(),
        };
        assert_eq!(r.display(), "1200");
    }

    #[test]
    fn test_display_range() {
        let r = TimeRange {
            start: "1000".to_string(),
            end: "1200".to_string(),
        };
        assert_eq!(r.display(), "1000 - 1200");
    }

    // --- group_scenes_by_pov ---

    #[test]
    fn test_group_scenes_by_pov_empty() {
        let scenes: Vec<Scene> = vec![];
        let groups = Database::group_scenes_by_pov(&scenes);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_group_scenes_by_pov_no_pov() {
        let scenes = vec![make_scene("s1", "S1", None, Some("100"), None, None)];
        let groups = Database::group_scenes_by_pov(&scenes);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_group_scenes_by_pov_multiple() {
        let scenes = vec![
            make_scene("s1", "S1", Some("Alice"), Some("100"), None, None),
            make_scene("s2", "S2", Some("Bob"), Some("200"), None, None),
            make_scene("s3", "S3", Some("Alice"), Some("300"), None, None),
        ];
        let groups = Database::group_scenes_by_pov(&scenes);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups["Alice"].len(), 2);
        assert_eq!(groups["Bob"].len(), 1);
    }

    // --- check_pov_time_conflicts ---

    #[test]
    fn test_check_pov_time_conflicts_single_scene() {
        let scene = make_scene("s1", "S1", Some("Alice"), Some("100"), None, None);
        let scenes_refs: Vec<&Scene> = vec![&scene];
        let conflicts = Database::check_pov_time_conflicts("Alice", &scenes_refs);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_check_pov_time_conflicts_overlapping_pair() {
        let s1 = make_scene("s1", "S1", Some("Alice"), None, Some("100"), Some("200"));
        let s2 = make_scene("s2", "S2", Some("Alice"), None, Some("150"), Some("250"));
        let scenes_refs: Vec<&Scene> = vec![&s1, &s2];
        let conflicts = Database::check_pov_time_conflicts("Alice", &scenes_refs);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, "overlapping_time");
    }

    #[test]
    fn test_check_pov_time_conflicts_same_point() {
        let s1 = make_scene("s1", "S1", Some("Alice"), Some("100"), None, None);
        let s2 = make_scene("s2", "S2", Some("Alice"), Some("100"), None, None);
        let scenes_refs: Vec<&Scene> = vec![&s1, &s2];
        let conflicts = Database::check_pov_time_conflicts("Alice", &scenes_refs);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, "same_time");
    }

    #[test]
    fn test_check_pov_time_conflicts_no_time_data() {
        let s1 = make_scene("s1", "S1", Some("Alice"), None, None, None);
        let s2 = make_scene("s2", "S2", Some("Alice"), None, None, None);
        let scenes_refs: Vec<&Scene> = vec![&s1, &s2];
        let conflicts = Database::check_pov_time_conflicts("Alice", &scenes_refs);
        assert!(conflicts.is_empty()); // No TimeRange → no conflict
    }

    #[test]
    fn test_check_pov_three_scenes_multiple_conflicts() {
        let s1 = make_scene("s1", "S1", Some("A"), None, Some("100"), Some("300"));
        let s2 = make_scene("s2", "S2", Some("A"), None, Some("200"), Some("400"));
        let s3 = make_scene("s3", "S3", Some("A"), None, Some("250"), Some("350"));
        let scenes_refs: Vec<&Scene> = vec![&s1, &s2, &s3];
        let conflicts = Database::check_pov_time_conflicts("A", &scenes_refs);
        // s1 overlaps s2, s1 overlaps s3, s2 overlaps s3 → 3 conflicts
        assert_eq!(conflicts.len(), 3);
    }

    // --- detect_missing_time_conflicts ---

    #[test]
    fn test_detect_missing_time_no_pov() {
        let scenes = vec![make_scene("s1", "S1", None, None, None, None)];
        let conflicts = Database::detect_missing_time_conflicts(&scenes);
        assert!(conflicts.is_empty()); // No POV → not a missing_time issue
    }

    #[test]
    fn test_detect_missing_time_with_pov_and_time() {
        let scenes = vec![make_scene(
            "s1",
            "S1",
            Some("Alice"),
            Some("100"),
            None,
            None,
        )];
        let conflicts = Database::detect_missing_time_conflicts(&scenes);
        assert!(conflicts.is_empty()); // Has time → no issue
    }

    #[test]
    fn test_detect_missing_time_with_pov_no_time() {
        let scenes = vec![make_scene("s1", "S1", Some("Alice"), None, None, None)];
        let conflicts = Database::detect_missing_time_conflicts(&scenes);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, "missing_time");
        assert_eq!(conflicts[0].character_name, Some("Alice".to_string()));
    }

    #[test]
    fn test_detect_missing_time_with_start_only() {
        let scenes = vec![make_scene(
            "s1",
            "S1",
            Some("Alice"),
            None,
            Some("100"),
            None,
        )];
        let conflicts = Database::detect_missing_time_conflicts(&scenes);
        assert!(conflicts.is_empty()); // Has time_start → not missing
    }
}
