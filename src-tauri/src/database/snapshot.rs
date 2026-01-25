//! Snapshot operations

use crate::models::{Arc, BibleEntry, Chapter, Event, Project, Scene, Snapshot};
use rusqlite::params;
use serde::Deserialize;

use super::Database;

/// Structure for deserializing snapshot/backup data
#[derive(Deserialize)]
pub struct SnapshotData {
    project: Project,
    chapters: Vec<Chapter>,
    scenes: Vec<Scene>,
    bible_entries: Vec<BibleEntry>,
    arcs: Vec<Arc>,
    events: Vec<Event>,
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
        let project = self.get_project()?;
        let chapters = self.get_chapters()?;
        let all_scenes = self.collect_all_scenes(&chapters)?;
        let bible_entries = self.get_bible_entries(None)?;
        let arcs = self.get_arcs()?;
        let events = self.get_events()?;

        let data = serde_json::json!({
            "project": project,
            "chapters": chapters,
            "scenes": all_scenes,
            "bible_entries": bible_entries,
            "arcs": arcs,
            "events": events,
            "created_at": timestamp,
        });

        Ok(data.to_string())
    }

    fn collect_all_scenes(
        &self,
        chapters: &[crate::models::Chapter],
    ) -> Result<Vec<crate::models::Scene>, String> {
        let mut all_scenes = Vec::new();
        for chapter in chapters {
            let scenes = self.get_scenes(&chapter.id)?;
            all_scenes.extend(scenes);
        }
        Ok(all_scenes)
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
        // Get the snapshot
        let snapshot = self.get_snapshot(id)?;

        // Parse the snapshot data
        let data: SnapshotData = serde_json::from_str(&snapshot.data)
            .map_err(|e| format!("Invalid snapshot data: {}", e))?;

        // Create automatic backup before restore
        self.create_snapshot(
            &format!("Auto-backup before restore: {}", snapshot.name),
            Some("Automatic backup created before restoring a snapshot"),
            "pre_restore",
        )?;

        // Clear existing data (order matters for foreign keys)
        self.clear_for_restore()?;

        // Restore data
        self.restore_project(&data.project)?;
        self.restore_chapters(&data.chapters)?;
        self.restore_scenes(&data.scenes)?;
        self.restore_bible_entries(&data.bible_entries)?;
        self.restore_arcs(&data.arcs)?;
        self.restore_events(&data.events)?;

        Ok(())
    }

    /// Imports project data from a JSON backup string.
    /// Creates an automatic backup before importing.
    pub fn import_json_backup(&self, json_data: &str) -> Result<(), String> {
        // Parse the JSON data
        let data: SnapshotData = serde_json::from_str(json_data)
            .map_err(|e| format!("Invalid JSON backup data: {}", e))?;

        // Create automatic backup before import
        self.create_snapshot(
            "Auto-backup before JSON import",
            Some("Automatic backup created before importing from JSON backup"),
            "pre_import",
        )?;

        // Clear existing data (order matters for foreign keys)
        self.clear_for_restore()?;

        // Restore data from the imported JSON
        self.restore_project(&data.project)?;
        self.restore_chapters(&data.chapters)?;
        self.restore_scenes(&data.scenes)?;
        self.restore_bible_entries(&data.bible_entries)?;
        self.restore_arcs(&data.arcs)?;
        self.restore_events(&data.events)?;

        Ok(())
    }

    fn clear_for_restore(&self) -> Result<(), String> {
        // Delete in order to respect foreign key constraints
        // Junction tables first
        self.conn
            .execute("DELETE FROM canonical_associations", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM scene_arcs", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM event_scenes", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM event_bible", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM bible_relationships", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM scene_steps", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM issue_scenes", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM issue_bible", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM annotations", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM scene_history", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM cuts", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM name_mentions", [])
            .map_err(|e| e.to_string())?;

        // Then main tables
        self.conn
            .execute("DELETE FROM scenes", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM chapters", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM bible_entries", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM arcs", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM events", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM issues", [])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute("DELETE FROM name_registry", [])
            .map_err(|e| e.to_string())?;

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
        for chapter in chapters {
            self.conn
                .execute(
                    "INSERT INTO chapters (id, title, summary, status, notes, position, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        chapter.id,
                        chapter.title,
                        chapter.summary,
                        chapter.status,
                        chapter.notes,
                        chapter.position,
                        chapter.created_at,
                        chapter.updated_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_scenes(&self, scenes: &[Scene]) -> Result<(), String> {
        for scene in scenes {
            self.conn
                .execute(
                    "INSERT INTO scenes (id, chapter_id, title, summary, text, status, pov, tags, notes, todos,
                     word_target, time_point, time_start, time_end, on_timeline, position,
                     pov_goal, has_conflict, has_change, tension, setup_for_scene_id, payoff_of_scene_id,
                     revision_notes, revision_checklist, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                             ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)",
                    params![
                        scene.id,
                        scene.chapter_id,
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
                        scene.on_timeline,
                        scene.position,
                        scene.pov_goal,
                        scene.has_conflict,
                        scene.has_change,
                        scene.tension,
                        scene.setup_for_scene_id,
                        scene.payoff_of_scene_id,
                        scene.revision_notes,
                        scene.revision_checklist,
                        scene.created_at,
                        scene.updated_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_bible_entries(&self, entries: &[BibleEntry]) -> Result<(), String> {
        for entry in entries {
            self.conn
                .execute(
                    "INSERT INTO bible_entries (id, entry_type, name, aliases, short_description,
                     full_description, status, tags, image_path, notes, todos, color, custom_fields,
                     created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                    params![
                        entry.id,
                        entry.entry_type,
                        entry.name,
                        entry.aliases,
                        entry.short_description,
                        entry.full_description,
                        entry.status,
                        entry.tags,
                        entry.image_path,
                        entry.notes,
                        entry.todos,
                        entry.color,
                        entry.custom_fields,
                        entry.created_at,
                        entry.updated_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_arcs(&self, arcs: &[Arc]) -> Result<(), String> {
        for arc in arcs {
            self.conn
                .execute(
                    "INSERT INTO arcs (id, name, description, stakes, characters, status, color, position,
                     created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        arc.id,
                        arc.name,
                        arc.description,
                        arc.stakes,
                        arc.characters,
                        arc.status,
                        arc.color,
                        arc.position,
                        arc.created_at,
                        arc.updated_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn restore_events(&self, events: &[Event]) -> Result<(), String> {
        for event in events {
            self.conn
                .execute(
                    "INSERT INTO events (id, title, description, time_point, time_start, time_end,
                     event_type, importance, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        event.id,
                        event.title,
                        event.description,
                        event.time_point,
                        event.time_start,
                        event.time_end,
                        event.event_type,
                        event.importance,
                        event.created_at,
                        event.updated_at
                    ],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
