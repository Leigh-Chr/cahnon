//! Timeline conflict detection

use crate::models::{Scene, TimelineConflict};

use super::Database;

impl Database {
    pub fn detect_timeline_conflicts(&self) -> Result<Vec<TimelineConflict>, String> {
        let scenes = self.get_all_scenes_for_timeline()?;
        let mut conflicts = Vec::new();

        conflicts.extend(self.detect_same_time_conflicts(&scenes));
        conflicts.extend(Self::detect_missing_time_conflicts(&scenes));

        Ok(conflicts)
    }

    fn detect_same_time_conflicts(&self, scenes: &[Scene]) -> Vec<TimelineConflict> {
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
        let time_a = scene_a.time_point.as_ref().or(scene_a.time_start.as_ref());
        let time_b = scene_b.time_point.as_ref().or(scene_b.time_start.as_ref());

        match (time_a, time_b) {
            (Some(ta), Some(tb)) if ta == tb => Some(TimelineConflict {
                conflict_type: "same_time".to_string(),
                description: format!(
                    "{} appears in multiple scenes at the same time: '{}'",
                    pov, ta
                ),
                scene_ids: vec![scene_a.id.clone(), scene_b.id.clone()],
                character_id: None,
                character_name: Some(pov.to_string()),
                time_point: Some(ta.clone()),
            }),
            _ => None,
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
