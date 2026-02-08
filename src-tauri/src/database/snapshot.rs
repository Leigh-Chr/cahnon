//! Snapshot operations

use crate::models::{
    Annotation, Arc, BibleEntry, BibleRelationship, Chapter, Event, Issue, IssueJunctionRow,
    JunctionRow, Project, Scene, Snapshot,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use super::Database;

/// All six junction table collections returned from snapshot collection.
type AllJunctions = (
    Vec<JunctionRow>,
    Vec<JunctionRow>,
    Vec<JunctionRow>,
    Vec<JunctionRow>,
    Vec<JunctionRow>,
    Vec<JunctionRow>,
);

/// Extended snapshot entities: bible relationships, issues, issue junctions, annotations.
type ExtendedEntities = (
    Vec<BibleRelationship>,
    Vec<Issue>,
    Vec<IssueJunctionRow>,
    Vec<IssueJunctionRow>,
    Vec<Annotation>,
);

/// Structure for serializing/deserializing snapshot/backup data.
/// All junction table fields use `#[serde(default)]` for backward compatibility
/// with older snapshots that only contain the 6 core entities.
#[derive(Serialize, Deserialize)]
pub struct SnapshotData {
    pub project: Project,
    pub chapters: Vec<Chapter>,
    pub scenes: Vec<Scene>,
    pub bible_entries: Vec<BibleEntry>,
    pub arcs: Vec<Arc>,
    pub events: Vec<Event>,
    // Junction tables (all default to empty Vec for backward compat)
    #[serde(default)]
    pub canonical_associations: Vec<JunctionRow>,
    #[serde(default)]
    pub scene_arcs: Vec<JunctionRow>,
    #[serde(default)]
    pub arc_characters: Vec<JunctionRow>,
    #[serde(default)]
    pub event_scenes: Vec<JunctionRow>,
    #[serde(default)]
    pub event_bible: Vec<JunctionRow>,
    #[serde(default)]
    pub bible_relationships: Vec<BibleRelationship>,
    #[serde(default)]
    pub issues: Vec<Issue>,
    #[serde(default)]
    pub issue_scenes: Vec<IssueJunctionRow>,
    #[serde(default)]
    pub issue_bible: Vec<IssueJunctionRow>,
    #[serde(default)]
    pub scene_steps: Vec<JunctionRow>,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
}

impl Database {
    pub fn create_snapshot(
        &self,
        name: &str,
        description: Option<&str>,
        snapshot_type: &str,
    ) -> Result<Snapshot, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let data = self.collect_snapshot_data(&now)?;

        self.conn
            .execute(
                "INSERT INTO snapshots (id, name, description, snapshot_type, data, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, name, description, snapshot_type, data, now],
            )
            .map_err(|e| e.to_string())?;

        self.get_snapshot(&id)
    }

    fn collect_snapshot_data(&self, timestamp: &str) -> Result<String, String> {
        let data = self.collect_all_snapshot_entities()?;
        self.serialize_snapshot_data(&data, timestamp)
    }

    fn collect_all_snapshot_entities(&self) -> Result<SnapshotData, String> {
        let project = self.get_project()?;
        let chapters = self.get_chapters()?;
        let scenes = self.collect_all_scenes()?;
        let bible_entries = self.get_bible_entries(None)?;
        let arcs = self.get_arcs()?;
        let events = self.get_events()?;
        let junctions = self.collect_all_junction_tables()?;
        let extended = self.collect_extended_snapshot_entities()?;

        Ok(SnapshotData {
            project,
            chapters,
            scenes,
            bible_entries,
            arcs,
            events,
            canonical_associations: junctions.0,
            scene_arcs: junctions.1,
            arc_characters: junctions.2,
            event_scenes: junctions.3,
            event_bible: junctions.4,
            bible_relationships: extended.0,
            issues: extended.1,
            issue_scenes: extended.2,
            issue_bible: extended.3,
            scene_steps: junctions.5,
            annotations: extended.4,
        })
    }

    /// Collects all junction table rows: canonical_associations, scene_arcs,
    /// arc_characters, event_scenes, event_bible, scene_steps.
    fn collect_all_junction_tables(&self) -> Result<AllJunctions, String> {
        Ok((
            self.collect_junction_rows(
                "SELECT id, scene_id, bible_entry_id, created_at FROM canonical_associations",
            )?,
            self.collect_junction_rows("SELECT id, scene_id, arc_id, created_at FROM scene_arcs")?,
            self.collect_junction_rows(
                "SELECT id, arc_id, bible_entry_id, created_at FROM arc_characters",
            )?,
            self.collect_junction_rows(
                "SELECT id, event_id, scene_id, created_at FROM event_scenes",
            )?,
            self.collect_junction_rows(
                "SELECT id, event_id, bible_entry_id, created_at FROM event_bible",
            )?,
            self.collect_junction_rows(
                "SELECT id, scene_id, step_id, created_at FROM scene_steps",
            )?,
        ))
    }

    /// Collects extended entities: bible_relationships, issues, issue junctions, annotations.
    fn collect_extended_snapshot_entities(&self) -> Result<ExtendedEntities, String> {
        Ok((
            self.collect_bible_relationships_for_snapshot()?,
            self.collect_issues_for_snapshot()?,
            self.collect_issue_junction_rows("SELECT id, issue_id, scene_id FROM issue_scenes")?,
            self.collect_issue_junction_rows(
                "SELECT id, issue_id, bible_entry_id FROM issue_bible",
            )?,
            self.collect_annotations_for_snapshot()?,
        ))
    }

    fn serialize_snapshot_data(
        &self,
        data: &SnapshotData,
        timestamp: &str,
    ) -> Result<String, String> {
        let json = serde_json::json!({
            "project": data.project,
            "chapters": data.chapters,
            "scenes": data.scenes,
            "bible_entries": data.bible_entries,
            "arcs": data.arcs,
            "events": data.events,
            "canonical_associations": data.canonical_associations,
            "scene_arcs": data.scene_arcs,
            "arc_characters": data.arc_characters,
            "event_scenes": data.event_scenes,
            "event_bible": data.event_bible,
            "bible_relationships": data.bible_relationships,
            "issues": data.issues,
            "issue_scenes": data.issue_scenes,
            "issue_bible": data.issue_bible,
            "scene_steps": data.scene_steps,
            "annotations": data.annotations,
            "created_at": timestamp,
        });

        Ok(json.to_string())
    }

    fn collect_junction_rows(&self, sql: &str) -> Result<Vec<JunctionRow>, String> {
        let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(JunctionRow {
                    id: row.get(0)?,
                    field_a: row.get(1)?,
                    field_b: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    fn collect_issue_junction_rows(&self, sql: &str) -> Result<Vec<IssueJunctionRow>, String> {
        let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok(IssueJunctionRow {
                    id: row.get(0)?,
                    field_a: row.get(1)?,
                    field_b: row.get(2)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    fn collect_bible_relationships_for_snapshot(&self) -> Result<Vec<BibleRelationship>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, source_id, target_id, relationship_type, note, status, created_at FROM bible_relationships",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], Self::map_bible_relationship_row)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    fn map_bible_relationship_row(row: &rusqlite::Row) -> rusqlite::Result<BibleRelationship> {
        Ok(BibleRelationship {
            id: row.get(0)?,
            source_id: row.get(1)?,
            target_id: row.get(2)?,
            relationship_type: row.get(3)?,
            note: row.get(4)?,
            status: row.get(5)?,
            created_at: row.get(6)?,
        })
    }

    fn collect_issues_for_snapshot(&self) -> Result<Vec<Issue>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, issue_type, title, description, severity, status, resolution_note, created_at, updated_at FROM issues",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], Self::map_issue)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    fn collect_annotations_for_snapshot(&self) -> Result<Vec<Annotation>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, scene_id, start_offset, end_offset, annotation_type, content, status, created_at, updated_at FROM annotations",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], Self::map_annotation)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }

    fn collect_all_scenes(&self) -> Result<Vec<crate::models::Scene>, String> {
        let query = format!(
            "{} FROM scenes WHERE deleted_at IS NULL ORDER BY chapter_id, position",
            crate::database::scene::crud::SCENE_SELECT
        );
        let mut stmt = self.conn.prepare(&query).map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map([], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
    }

    pub fn get_snapshots(&self) -> Result<Vec<Snapshot>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, description, snapshot_type, data, created_at FROM snapshots ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let snapshots = stmt
            .query_map([], Self::map_snapshot)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(snapshots)
    }

    fn map_snapshot(row: &rusqlite::Row) -> rusqlite::Result<Snapshot> {
        Ok(Snapshot {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            snapshot_type: row.get(3)?,
            data: row.get(4)?,
            created_at: row.get(5)?,
        })
    }

    pub fn get_snapshot(&self, id: &str) -> Result<Snapshot, String> {
        self.conn
            .query_row(
                "SELECT id, name, description, snapshot_type, data, created_at FROM snapshots WHERE id = ?1",
                params![id],
                Self::map_snapshot,
            )
            .map_err(|e| e.to_string())
    }

    /// Deletes pre_bulk snapshots older than 30 days per spec 16.1.
    pub fn cleanup_expired_snapshots(&self) -> Result<i32, String> {
        let threshold = (chrono::Utc::now() - chrono::Duration::days(30)).to_rfc3339();
        let deleted = self
            .conn
            .execute(
                "DELETE FROM snapshots WHERE snapshot_type = 'pre_bulk' AND created_at < ?1",
                params![threshold],
            )
            .map_err(|e| e.to_string())?;
        Ok(deleted as i32)
    }

    pub fn delete_snapshot(&self, id: &str) -> Result<(), String> {
        let rows = self
            .conn
            .execute("DELETE FROM snapshots WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err("Snapshot not found".to_string());
        }
        Ok(())
    }

    /// Restores project state from a snapshot.
    /// Creates an automatic backup before restoring.
    pub fn restore_snapshot(&self, id: &str) -> Result<(), String> {
        let snapshot = self.get_snapshot(id)?;

        let data: SnapshotData = serde_json::from_str(&snapshot.data)
            .map_err(|e| format!("Invalid snapshot data: {}", e))?;

        // Create automatic backup before restore
        self.create_snapshot(
            &format!("Auto-backup before restore: {}", snapshot.name),
            Some("Automatic backup created before restoring a snapshot"),
            "pre_restore",
        )?;

        self.run_in_transaction(|| self.restore_all_data(&data))
    }

    /// Imports project data from a JSON backup string.
    /// Creates an automatic backup before importing.
    pub fn import_json_backup(&self, json_data: &str) -> Result<(), String> {
        let data: SnapshotData = serde_json::from_str(json_data)
            .map_err(|e| format!("Invalid JSON backup data: {}", e))?;

        self.create_snapshot(
            "Auto-backup before JSON import",
            Some("Automatic backup created before importing from JSON backup"),
            "pre_import",
        )?;

        self.run_in_transaction(|| self.restore_all_data(&data))
    }

    /// Restores all entities and junction tables from a SnapshotData.
    fn restore_all_data(&self, data: &SnapshotData) -> Result<(), String> {
        self.clear_for_restore()?;
        self.restore_core_entities(data)?;
        self.restore_all_junction_data(data)
    }

    /// Restores the six core entity tables from snapshot data.
    fn restore_core_entities(&self, data: &SnapshotData) -> Result<(), String> {
        self.restore_project(&data.project)?;
        self.restore_chapters(&data.chapters)?;
        self.restore_scenes(&data.scenes)?;
        self.restore_bible_entries(&data.bible_entries)?;
        self.restore_arcs(&data.arcs)?;
        self.restore_events(&data.events)
    }

    /// Restores all junction and extended tables from snapshot data.
    fn restore_all_junction_data(&self, data: &SnapshotData) -> Result<(), String> {
        self.restore_junction_rows(
            "canonical_associations",
            "scene_id",
            "bible_entry_id",
            &data.canonical_associations,
        )?;
        self.restore_junction_rows("scene_arcs", "scene_id", "arc_id", &data.scene_arcs)?;
        self.restore_junction_rows(
            "arc_characters",
            "arc_id",
            "bible_entry_id",
            &data.arc_characters,
        )?;
        self.restore_junction_rows("event_scenes", "event_id", "scene_id", &data.event_scenes)?;
        self.restore_junction_rows(
            "event_bible",
            "event_id",
            "bible_entry_id",
            &data.event_bible,
        )?;
        self.restore_bible_relationships_from_snapshot(&data.bible_relationships)?;
        self.restore_issues_from_snapshot(&data.issues)?;
        self.restore_issue_junction_rows(
            "issue_scenes",
            "issue_id",
            "scene_id",
            &data.issue_scenes,
        )?;
        self.restore_issue_junction_rows(
            "issue_bible",
            "issue_id",
            "bible_entry_id",
            &data.issue_bible,
        )?;
        self.restore_junction_rows("scene_steps", "scene_id", "step_id", &data.scene_steps)?;
        self.restore_annotations_from_snapshot(&data.annotations)
    }

    /// Get scenes stored in a snapshot (for selection UI).
    pub fn get_snapshot_scenes(&self, snapshot_id: &str) -> Result<Vec<Scene>, String> {
        let snapshot = self.get_snapshot(snapshot_id)?;
        let data: SnapshotData = serde_json::from_str(&snapshot.data)
            .map_err(|e| format!("Invalid snapshot data: {}", e))?;
        Ok(data.scenes)
    }

    /// Restore a single scene from a snapshot.
    pub fn restore_scene_from_snapshot(
        &self,
        snapshot_id: &str,
        scene_id: &str,
    ) -> Result<Scene, String> {
        let snapshot = self.get_snapshot(snapshot_id)?;
        let data: SnapshotData = serde_json::from_str(&snapshot.data)
            .map_err(|e| format!("Invalid snapshot data: {}", e))?;

        let scene = data
            .scenes
            .iter()
            .find(|s| s.id == scene_id)
            .ok_or_else(|| format!("Scene {} not found in snapshot", scene_id))?;

        self.apply_scene_snapshot(scene_id, scene)?;
        self.get_scene(scene_id)
    }

    /// Applies a snapshot scene's fields to an existing scene row.
    fn apply_scene_snapshot(&self, scene_id: &str, scene: &Scene) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE scenes SET title = ?1, summary = ?2, text = ?3, status = ?4,
                 pov = ?5, tags = ?6, notes = ?7, todos = ?8, word_target = ?9,
                 time_point = ?10, time_start = ?11, time_end = ?12, on_timeline = ?13,
                 pov_goal = ?14, has_conflict = ?15, has_change = ?16, tension = ?17,
                 setup_for_scene_id = ?18, payoff_of_scene_id = ?19,
                 revision_notes = ?20, revision_checklist = ?21, updated_at = ?22
                 WHERE id = ?23 AND deleted_at IS NULL",
                params![
                    scene.title,
                    scene.summary,
                    scene.text,
                    scene.status,
                    scene.pov,
                    scene.tags,
                    scene.notes,
                    scene.todos,
                    scene.word_target,
                    scene.time_point,
                    scene.time_start,
                    scene.time_end,
                    scene.on_timeline as i32,
                    scene.pov_goal,
                    scene.has_conflict.map(|b| b as i32),
                    scene.has_change.map(|b| b as i32),
                    scene.tension,
                    scene.setup_for_scene_id,
                    scene.payoff_of_scene_id,
                    scene.revision_notes,
                    scene.revision_checklist,
                    chrono::Utc::now().to_rfc3339(),
                    scene_id,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn clear_for_restore(&self) -> Result<(), String> {
        // Junction tables referencing entities we'll restore:
        let tables = [
            "canonical_associations",
            "scene_arcs",
            "arc_characters",
            "event_scenes",
            "event_bible",
            "bible_relationships",
            "issue_scenes",
            "issue_bible",
            "scene_steps",
            "annotations",
            "scene_history",
            // Main tables restored from snapshot
            "scenes",
            "chapters",
            "bible_entries",
            "arcs",
            "events",
            "issues",
        ];
        for table in &tables {
            let table = Self::validate_table_name(table)?;
            self.conn
                .execute(&format!("DELETE FROM {}", table), [])
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn restore_project(&self, project: &Project) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE project SET title = ?1, author = ?2, description = ?3,
                 word_target = ?4, daily_word_target = ?5, updated_at = ?6",
                params![
                    project.title,
                    project.author,
                    project.description,
                    project.word_target,
                    project.daily_word_target,
                    chrono::Utc::now().to_rfc3339()
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn restore_chapters(&self, chapters: &[Chapter]) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO chapters (id, title, summary, status, notes, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        ).map_err(|e| e.to_string())?;
        for chapter in chapters {
            stmt.execute(params![
                chapter.id, chapter.title, chapter.summary, chapter.status,
                chapter.notes, chapter.position, chapter.created_at, chapter.updated_at
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_scenes(&self, scenes: &[Scene]) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO scenes (id, chapter_id, title, summary, text, status, pov, tags, notes, todos,
             word_target, time_point, time_start, time_end, on_timeline, position,
             pov_goal, has_conflict, has_change, tension, setup_for_scene_id, payoff_of_scene_id,
             revision_notes, revision_checklist, word_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)",
        ).map_err(|e| e.to_string())?;
        for scene in scenes {
            stmt.execute(params![
                scene.id, scene.chapter_id, scene.title, scene.summary, scene.text,
                scene.status, scene.pov, scene.tags, scene.notes, scene.todos,
                scene.word_target, scene.time_point, scene.time_start, scene.time_end,
                scene.on_timeline, scene.position, scene.pov_goal, scene.has_conflict,
                scene.has_change, scene.tension, scene.setup_for_scene_id,
                scene.payoff_of_scene_id, scene.revision_notes, scene.revision_checklist,
                scene.word_count, scene.created_at, scene.updated_at
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_bible_entries(&self, entries: &[BibleEntry]) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO bible_entries (id, entry_type, name, aliases, short_description,
             full_description, status, tags, image_path, notes, todos, color, custom_fields,
             created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        ).map_err(|e| e.to_string())?;
        for entry in entries {
            stmt.execute(params![
                entry.id, entry.entry_type, entry.name, entry.aliases,
                entry.short_description, entry.full_description, entry.status, entry.tags,
                entry.image_path, entry.notes, entry.todos, entry.color, entry.custom_fields,
                entry.created_at, entry.updated_at
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_arcs(&self, arcs: &[Arc]) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO arcs (id, name, description, stakes, status, color, position,
             created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        ).map_err(|e| e.to_string())?;
        for arc in arcs {
            stmt.execute(params![
                arc.id, arc.name, arc.description, arc.stakes, arc.status,
                arc.color, arc.position, arc.created_at, arc.updated_at
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_events(&self, events: &[Event]) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO events (id, title, description, time_point, time_start, time_end,
             event_type, importance, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        ).map_err(|e| e.to_string())?;
        for event in events {
            stmt.execute(params![
                event.id, event.title, event.description, event.time_point,
                event.time_start, event.time_end, event.event_type, event.importance,
                event.created_at, event.updated_at
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_junction_rows(
        &self,
        table: &str,
        col_a: &str,
        col_b: &str,
        rows: &[JunctionRow],
    ) -> Result<(), String> {
        if rows.is_empty() {
            return Ok(());
        }
        let sql = format!(
            "INSERT OR IGNORE INTO {} (id, {}, {}, created_at) VALUES (?1, ?2, ?3, ?4)",
            table, col_a, col_b
        );
        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;
        for row in rows {
            stmt.execute(params![row.id, row.field_a, row.field_b, row.created_at])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_issue_junction_rows(
        &self,
        table: &str,
        col_a: &str,
        col_b: &str,
        rows: &[IssueJunctionRow],
    ) -> Result<(), String> {
        if rows.is_empty() {
            return Ok(());
        }
        let sql = format!(
            "INSERT OR IGNORE INTO {} (id, {}, {}) VALUES (?1, ?2, ?3)",
            table, col_a, col_b
        );
        let mut stmt = self.conn.prepare(&sql).map_err(|e| e.to_string())?;
        for row in rows {
            stmt.execute(params![row.id, row.field_a, row.field_b])
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_bible_relationships_from_snapshot(
        &self,
        relationships: &[BibleRelationship],
    ) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "INSERT OR IGNORE INTO bible_relationships (id, source_id, target_id, relationship_type, note, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        ).map_err(|e| e.to_string())?;
        for rel in relationships {
            stmt.execute(params![
                rel.id, rel.source_id, rel.target_id, rel.relationship_type,
                rel.note, rel.status, rel.created_at
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_issues_from_snapshot(&self, issues: &[Issue]) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO issues (id, issue_type, title, description, severity, status, resolution_note, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        ).map_err(|e| e.to_string())?;
        for issue in issues {
            stmt.execute(params![
                issue.id, issue.issue_type, issue.title, issue.description,
                issue.severity, issue.status, issue.resolution_note,
                issue.created_at, issue.updated_at
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_annotations_from_snapshot(&self, annotations: &[Annotation]) -> Result<(), String> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO annotations (id, scene_id, start_offset, end_offset, annotation_type, content, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        ).map_err(|e| e.to_string())?;
        for ann in annotations {
            stmt.execute(params![
                ann.id, ann.scene_id, ann.start_offset, ann.end_offset,
                ann.annotation_type, ann.content, ann.status, ann.created_at, ann.updated_at
            ]).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
