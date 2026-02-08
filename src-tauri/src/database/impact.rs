//! Impact Awareness (delete preview).
//!
//! Before deleting a scene, chapter, or bible entry, shows
//! what would be affected: broken links, orphaned entities,
//! lost associations, etc.

use crate::models::{ImpactItem, ImpactPreview};
use rusqlite::params;

use super::Database;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Query a list of `(id, name)` pairs from the database.
fn query_id_name_pairs(
    conn: &rusqlite::Connection,
    sql: &str,
    param: &str,
) -> Result<Vec<(String, String)>, String> {
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let result = stmt
        .query_map(params![param], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string());
    result
}

/// Count rows matching a single-parameter query, returning 0 on error.
fn count_rows(conn: &rusqlite::Connection, sql: &str, param: &str) -> i32 {
    conn.query_row(sql, params![param], |row| row.get(0))
        .unwrap_or(0)
}

/// Build an `ImpactItem` for each `(id, name)` pair.
fn items_from_pairs(
    pairs: &[(String, String)],
    impact_type: &str,
    description_fn: impl Fn(&str) -> String,
) -> Vec<ImpactItem> {
    pairs
        .iter()
        .map(|(id, name)| ImpactItem {
            impact_type: impact_type.to_string(),
            description: description_fn(name),
            entity_id: Some(id.clone()),
            entity_name: Some(name.clone()),
        })
        .collect()
}

/// Build a single summary `ImpactItem` when a count is > 0.
fn summary_item(count: i32, impact_type: &str, description: String) -> Option<ImpactItem> {
    if count > 0 {
        Some(ImpactItem {
            impact_type: impact_type.to_string(),
            description,
            entity_id: None,
            entity_name: None,
        })
    } else {
        None
    }
}

/// Wrap a finished `Vec<ImpactItem>` into an `ImpactPreview`.
fn into_preview(items: Vec<ImpactItem>) -> ImpactPreview {
    let total_count = items.len() as i32;
    ImpactPreview { items, total_count }
}

// ---------------------------------------------------------------------------
// Scene-impact helpers
// ---------------------------------------------------------------------------

impl Database {
    /// Arcs that would lose a scene point.
    fn scene_impact_arcs(&self, scene_id: &str) -> Result<Vec<ImpactItem>, String> {
        let pairs = query_id_name_pairs(
            &self.conn,
            "SELECT a.id, a.name
             FROM scene_arcs sa
             JOIN arcs a ON sa.arc_id = a.id
             WHERE sa.scene_id = ?1 AND a.deleted_at IS NULL",
            scene_id,
        )?;
        Ok(items_from_pairs(&pairs, "arc_loses_scene", |name| {
            format!("Arc \"{}\" will lose this scene point", name)
        }))
    }

    /// Events that would become orphaned (only linked to this scene).
    fn scene_impact_orphan_events(&self, scene_id: &str) -> Result<Vec<ImpactItem>, String> {
        let pairs = query_id_name_pairs(
            &self.conn,
            "SELECT e.id, e.title
             FROM event_scenes es
             JOIN events e ON es.event_id = e.id
             WHERE es.scene_id = ?1 AND e.deleted_at IS NULL
               AND (SELECT COUNT(*) FROM event_scenes es2 WHERE es2.event_id = e.id) = 1",
            scene_id,
        )?;
        Ok(items_from_pairs(&pairs, "orphan_event", |name| {
            format!("Event \"{}\" will have no linked scenes", name)
        }))
    }

    /// Scenes that have setup/payoff links pointing to this scene.
    fn scene_impact_setup_payoff(&self, scene_id: &str) -> Result<Vec<ImpactItem>, String> {
        let pairs = query_id_name_pairs(
            &self.conn,
            "SELECT id, title FROM scenes
             WHERE deleted_at IS NULL
               AND (setup_for_scene_id = ?1 OR payoff_of_scene_id = ?1)",
            scene_id,
        )?;
        Ok(items_from_pairs(&pairs, "broken_setup_payoff", |name| {
            format!("Scene \"{}\" has a setup/payoff link to this scene", name)
        }))
    }

    /// Canonical associations that would be removed.
    fn scene_impact_associations(&self, scene_id: &str) -> Option<ImpactItem> {
        let count = count_rows(
            &self.conn,
            "SELECT COUNT(*) FROM canonical_associations WHERE scene_id = ?1",
            scene_id,
        );
        summary_item(
            count,
            "associations_lost",
            format!("{} canonical association(s) will be removed", count),
        )
    }

    /// Issues linked to this scene.
    fn scene_impact_issues(&self, scene_id: &str) -> Option<ImpactItem> {
        let count = count_rows(
            &self.conn,
            "SELECT COUNT(*) FROM issue_scenes WHERE scene_id = ?1",
            scene_id,
        );
        summary_item(
            count,
            "issues_unlinked",
            format!("{} issue(s) linked to this scene will be unlinked", count),
        )
    }
}

// ---------------------------------------------------------------------------
// Bible-entry-impact helpers
// ---------------------------------------------------------------------------

impl Database {
    /// Scenes that reference this bible entry via canonical associations.
    fn bible_impact_scenes(&self, bible_entry_id: &str) -> Result<Option<ImpactItem>, String> {
        let pairs = query_id_name_pairs(
            &self.conn,
            "SELECT s.id, s.title
             FROM canonical_associations ca
             JOIN scenes s ON ca.scene_id = s.id
             WHERE ca.bible_entry_id = ?1 AND s.deleted_at IS NULL",
            bible_entry_id,
        )?;
        Ok(summary_item(
            pairs.len() as i32,
            "scenes_lose_association",
            format!(
                "{} scene(s) reference this entry and will lose the association",
                pairs.len()
            ),
        ))
    }

    /// Relationships that will be broken.
    fn bible_impact_relationships(&self, bible_entry_id: &str) -> Option<ImpactItem> {
        let count = count_rows(
            &self.conn,
            "SELECT COUNT(*) FROM bible_relationships WHERE source_id = ?1 OR target_id = ?1",
            bible_entry_id,
        );
        summary_item(
            count,
            "relationships_broken",
            format!(
                "{} relationship(s) involving this entry will be broken",
                count
            ),
        )
    }

    /// Arc character links.
    fn bible_impact_arc_characters(&self, bible_entry_id: &str) -> Option<ImpactItem> {
        let count = count_rows(
            &self.conn,
            "SELECT COUNT(*) FROM arc_characters WHERE bible_entry_id = ?1",
            bible_entry_id,
        );
        summary_item(
            count,
            "arc_character_lost",
            format!("Character will be removed from {} arc(s)", count),
        )
    }

    /// Issues linked to this bible entry.
    fn bible_impact_issues(&self, bible_entry_id: &str) -> Option<ImpactItem> {
        let count = count_rows(
            &self.conn,
            "SELECT COUNT(*) FROM issue_bible WHERE bible_entry_id = ?1",
            bible_entry_id,
        );
        summary_item(
            count,
            "issues_unlinked",
            format!("{} issue(s) will lose their link to this entry", count),
        )
    }
}

// ---------------------------------------------------------------------------
// Chapter-impact helpers
// ---------------------------------------------------------------------------

impl Database {
    /// Collect scene IDs belonging to a chapter.
    fn chapter_scene_ids(&self, chapter_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL")
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_map(params![chapter_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string());
        result
    }

    /// Aggregate per-scene impacts into chapter-level summary items using batch queries.
    fn aggregate_scene_impacts(&self, scene_ids: &[String]) -> Result<Vec<ImpactItem>, String> {
        if scene_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = (1..=scene_ids.len()).map(|i| format!("?{}", i)).collect();
        let in_clause = placeholders.join(", ");
        let params_refs: Vec<&dyn rusqlite::ToSql> = scene_ids
            .iter()
            .map(|s| s as &dyn rusqlite::ToSql)
            .collect();

        // Batch: distinct arcs affected
        let arc_count: i32 = {
            let sql = format!(
                "SELECT COUNT(DISTINCT sa.arc_id) FROM scene_arcs sa
                 JOIN arcs a ON sa.arc_id = a.id
                 WHERE sa.scene_id IN ({}) AND a.deleted_at IS NULL",
                in_clause
            );
            self.conn
                .query_row(&sql, params_refs.as_slice(), |row| row.get(0))
                .unwrap_or(0)
        };

        // Batch: orphan events (only linked to scenes in this set)
        let orphan_event_count: i32 = {
            let sql = format!(
                "SELECT COUNT(*) FROM events e
                 WHERE e.deleted_at IS NULL
                   AND EXISTS (SELECT 1 FROM event_scenes es WHERE es.event_id = e.id AND es.scene_id IN ({}))
                   AND NOT EXISTS (SELECT 1 FROM event_scenes es2 WHERE es2.event_id = e.id AND es2.scene_id NOT IN ({}))",
                in_clause, in_clause
            );
            // Need to pass params twice for the two IN clauses
            let mut double_params: Vec<&dyn rusqlite::ToSql> =
                Vec::with_capacity(scene_ids.len() * 2);
            double_params.extend(params_refs.iter());
            double_params.extend(params_refs.iter());
            self.conn
                .query_row(&sql, double_params.as_slice(), |row| row.get(0))
                .unwrap_or(0)
        };

        // Batch: broken setup/payoff links
        let setup_payoff_count: i32 = {
            let sql = format!(
                "SELECT COUNT(*) FROM scenes
                 WHERE deleted_at IS NULL
                   AND (setup_for_scene_id IN ({in_c}) OR payoff_of_scene_id IN ({in_c}))
                   AND id NOT IN ({in_c})",
                in_c = in_clause
            );
            let mut triple_params: Vec<&dyn rusqlite::ToSql> =
                Vec::with_capacity(scene_ids.len() * 3);
            triple_params.extend(params_refs.iter());
            triple_params.extend(params_refs.iter());
            triple_params.extend(params_refs.iter());
            self.conn
                .query_row(&sql, triple_params.as_slice(), |row| row.get(0))
                .unwrap_or(0)
        };

        // Batch: associations count
        let associations_count: i32 = {
            let sql = format!(
                "SELECT COUNT(*) FROM canonical_associations WHERE scene_id IN ({})",
                in_clause
            );
            self.conn
                .query_row(&sql, params_refs.as_slice(), |row| row.get(0))
                .unwrap_or(0)
        };

        let mut items = Vec::new();
        if let Some(item) = summary_item(
            arc_count,
            "arcs_affected",
            format!("{} arc(s) will lose scene points", arc_count),
        ) {
            items.push(item);
        }
        if let Some(item) = summary_item(
            orphan_event_count,
            "orphan_events",
            format!("{} event(s) will have no linked scenes", orphan_event_count),
        ) {
            items.push(item);
        }
        if let Some(item) = summary_item(
            setup_payoff_count,
            "broken_setup_payoff",
            format!("{} setup/payoff link(s) will break", setup_payoff_count),
        ) {
            items.push(item);
        }
        if let Some(item) = summary_item(
            associations_count,
            "associations_lost",
            format!(
                "Canonical associations from {} scene(s) will be removed",
                associations_count
            ),
        ) {
            items.push(item);
        }
        Ok(items)
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

impl Database {
    /// Preview impact of deleting a scene.
    pub fn preview_delete_scene_impact(&self, scene_id: &str) -> Result<ImpactPreview, String> {
        let mut items = Vec::new();

        items.extend(self.scene_impact_arcs(scene_id)?);
        items.extend(self.scene_impact_orphan_events(scene_id)?);
        items.extend(self.scene_impact_setup_payoff(scene_id)?);
        items.extend(self.scene_impact_associations(scene_id));
        items.extend(self.scene_impact_issues(scene_id));

        Ok(into_preview(items))
    }

    /// Preview impact of deleting a bible entry.
    pub fn preview_delete_bible_entry_impact(
        &self,
        bible_entry_id: &str,
    ) -> Result<ImpactPreview, String> {
        let mut items = Vec::new();

        items.extend(self.bible_impact_scenes(bible_entry_id)?);
        items.extend(self.bible_impact_relationships(bible_entry_id));
        items.extend(self.bible_impact_arc_characters(bible_entry_id));
        items.extend(self.bible_impact_issues(bible_entry_id));

        Ok(into_preview(items))
    }

    /// Preview impact of deleting a chapter (cumulates all scene impacts).
    pub fn preview_delete_chapter_impact(&self, chapter_id: &str) -> Result<ImpactPreview, String> {
        let scene_ids = self.chapter_scene_ids(chapter_id)?;

        let mut items = Vec::new();

        if let Some(item) = summary_item(
            scene_ids.len() as i32,
            "scenes_deleted",
            format!("{} scene(s) will be deleted", scene_ids.len()),
        ) {
            items.push(item);
        }

        items.extend(self.aggregate_scene_impacts(&scene_ids)?);

        Ok(into_preview(items))
    }
}
