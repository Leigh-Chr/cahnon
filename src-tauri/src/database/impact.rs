//! Impact Awareness (delete preview).
//!
//! Before deleting a scene, chapter, or bible entry, shows
//! what would be affected: broken links, orphaned entities,
//! lost associations, etc.

use crate::models::{ImpactItem, ImpactPreview};
use rusqlite::params;

use super::Database;

impl Database {
    /// Preview impact of deleting a scene.
    pub fn preview_delete_scene_impact(&self, scene_id: &str) -> Result<ImpactPreview, String> {
        let mut items = Vec::new();

        // 1. Arcs that would lose a scene point
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.name
                 FROM scene_arcs sa
                 JOIN arcs a ON sa.arc_id = a.id
                 WHERE sa.scene_id = ?1 AND a.deleted_at IS NULL",
            )
            .map_err(|e| e.to_string())?;

        let arcs: Vec<(String, String)> = stmt
            .query_map(params![scene_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (arc_id, arc_name) in &arcs {
            items.push(ImpactItem {
                impact_type: "arc_loses_scene".to_string(),
                description: format!("Arc \"{}\" will lose this scene point", arc_name),
                entity_id: Some(arc_id.clone()),
                entity_name: Some(arc_name.clone()),
            });
        }

        // 2. Events that would become orphaned (only linked to this scene)
        let mut stmt = self
            .conn
            .prepare(
                "SELECT e.id, e.title
                 FROM event_scenes es
                 JOIN events e ON es.event_id = e.id
                 WHERE es.scene_id = ?1 AND e.deleted_at IS NULL
                   AND (SELECT COUNT(*) FROM event_scenes es2 WHERE es2.event_id = e.id) = 1",
            )
            .map_err(|e| e.to_string())?;

        let orphan_events: Vec<(String, String)> = stmt
            .query_map(params![scene_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (event_id, event_title) in &orphan_events {
            items.push(ImpactItem {
                impact_type: "orphan_event".to_string(),
                description: format!("Event \"{}\" will have no linked scenes", event_title),
                entity_id: Some(event_id.clone()),
                entity_name: Some(event_title.clone()),
            });
        }

        // 3. Setup/payoff links that reference this scene
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title FROM scenes
                 WHERE deleted_at IS NULL
                   AND (setup_for_scene_id = ?1 OR payoff_of_scene_id = ?1)",
            )
            .map_err(|e| e.to_string())?;

        let linked_scenes: Vec<(String, String)> = stmt
            .query_map(params![scene_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        for (linked_id, linked_title) in &linked_scenes {
            items.push(ImpactItem {
                impact_type: "broken_setup_payoff".to_string(),
                description: format!(
                    "Scene \"{}\" has a setup/payoff link to this scene",
                    linked_title
                ),
                entity_id: Some(linked_id.clone()),
                entity_name: Some(linked_title.clone()),
            });
        }

        // 4. Canonical associations
        let assoc_count: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM canonical_associations WHERE scene_id = ?1",
                params![scene_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if assoc_count > 0 {
            items.push(ImpactItem {
                impact_type: "associations_lost".to_string(),
                description: format!("{} canonical association(s) will be removed", assoc_count),
                entity_id: None,
                entity_name: None,
            });
        }

        // 6. Issues linked to this scene
        let issue_count: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM issue_scenes WHERE scene_id = ?1",
                params![scene_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if issue_count > 0 {
            items.push(ImpactItem {
                impact_type: "issues_unlinked".to_string(),
                description: format!(
                    "{} issue(s) linked to this scene will be unlinked",
                    issue_count
                ),
                entity_id: None,
                entity_name: None,
            });
        }

        let total_count = items.len() as i32;
        Ok(ImpactPreview { items, total_count })
    }

    /// Preview impact of deleting a bible entry.
    pub fn preview_delete_bible_entry_impact(
        &self,
        bible_entry_id: &str,
    ) -> Result<ImpactPreview, String> {
        let mut items = Vec::new();

        // 1. Scenes that reference this entry via canonical associations
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title
                 FROM canonical_associations ca
                 JOIN scenes s ON ca.scene_id = s.id
                 WHERE ca.bible_entry_id = ?1 AND s.deleted_at IS NULL",
            )
            .map_err(|e| e.to_string())?;

        let scenes: Vec<(String, String)> = stmt
            .query_map(params![bible_entry_id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        if !scenes.is_empty() {
            items.push(ImpactItem {
                impact_type: "scenes_lose_association".to_string(),
                description: format!(
                    "{} scene(s) reference this entry and will lose the association",
                    scenes.len()
                ),
                entity_id: None,
                entity_name: None,
            });
        }

        // 2. Relationships that will be broken
        let rel_count: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM bible_relationships
                 WHERE source_id = ?1 OR target_id = ?1",
                params![bible_entry_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if rel_count > 0 {
            items.push(ImpactItem {
                impact_type: "relationships_broken".to_string(),
                description: format!(
                    "{} relationship(s) involving this entry will be broken",
                    rel_count
                ),
                entity_id: None,
                entity_name: None,
            });
        }

        // 3. Arc character links
        let arc_count: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM arc_characters
                 WHERE bible_entry_id = ?1",
                params![bible_entry_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if arc_count > 0 {
            items.push(ImpactItem {
                impact_type: "arc_character_lost".to_string(),
                description: format!("Character will be removed from {} arc(s)", arc_count),
                entity_id: None,
                entity_name: None,
            });
        }

        // 4. Issues linked to this bible entry
        let issue_count: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM issue_bible WHERE bible_entry_id = ?1",
                params![bible_entry_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if issue_count > 0 {
            items.push(ImpactItem {
                impact_type: "issues_unlinked".to_string(),
                description: format!(
                    "{} issue(s) will lose their link to this entry",
                    issue_count
                ),
                entity_id: None,
                entity_name: None,
            });
        }

        let total_count = items.len() as i32;
        Ok(ImpactPreview { items, total_count })
    }

    /// Preview impact of deleting a chapter (cumulates all scene impacts).
    pub fn preview_delete_chapter_impact(&self, chapter_id: &str) -> Result<ImpactPreview, String> {
        // Get all non-deleted scenes in this chapter
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL")
            .map_err(|e| e.to_string())?;

        let scene_ids: Vec<String> = stmt
            .query_map(params![chapter_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut all_items = Vec::new();

        if !scene_ids.is_empty() {
            all_items.push(ImpactItem {
                impact_type: "scenes_deleted".to_string(),
                description: format!("{} scene(s) will be deleted", scene_ids.len()),
                entity_id: None,
                entity_name: None,
            });
        }

        // Aggregate scene impacts
        let mut arc_ids = std::collections::HashSet::new();
        let mut orphan_event_count = 0;
        let mut setup_payoff_count = 0;
        let mut total_associations = 0;

        for sid in &scene_ids {
            let impact = self.preview_delete_scene_impact(sid)?;
            for item in &impact.items {
                match item.impact_type.as_str() {
                    "arc_loses_scene" => {
                        if let Some(ref eid) = item.entity_id {
                            arc_ids.insert(eid.clone());
                        }
                    }
                    "orphan_event" => orphan_event_count += 1,
                    "broken_setup_payoff" => setup_payoff_count += 1,
                    "associations_lost" => total_associations += 1,
                    _ => {}
                }
            }
        }

        if !arc_ids.is_empty() {
            all_items.push(ImpactItem {
                impact_type: "arcs_affected".to_string(),
                description: format!("{} arc(s) will lose scene points", arc_ids.len()),
                entity_id: None,
                entity_name: None,
            });
        }
        if orphan_event_count > 0 {
            all_items.push(ImpactItem {
                impact_type: "orphan_events".to_string(),
                description: format!("{} event(s) will have no linked scenes", orphan_event_count),
                entity_id: None,
                entity_name: None,
            });
        }
        if setup_payoff_count > 0 {
            all_items.push(ImpactItem {
                impact_type: "broken_setup_payoff".to_string(),
                description: format!("{} setup/payoff link(s) will break", setup_payoff_count),
                entity_id: None,
                entity_name: None,
            });
        }
        if total_associations > 0 {
            all_items.push(ImpactItem {
                impact_type: "associations_lost".to_string(),
                description: format!(
                    "Canonical associations from {} scene(s) will be removed",
                    total_associations
                ),
                entity_id: None,
                entity_name: None,
            });
        }

        let total_count = all_items.len() as i32;
        Ok(ImpactPreview {
            items: all_items,
            total_count,
        })
    }
}
