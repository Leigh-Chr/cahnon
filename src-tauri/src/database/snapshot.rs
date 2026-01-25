//! Snapshot operations

use crate::models::Snapshot;
use rusqlite::params;

use super::Database;

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
}
