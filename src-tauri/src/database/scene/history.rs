//! Scene history operations.

use crate::models::{SceneHistoryEntry, Scene, UpdateSceneRequest};
use rusqlite::params;

use super::super::Database;

impl Database {
    /// Save current scene text to history before update.
    pub(crate) fn save_scene_to_history(&self, id: &str, now: &str) -> Result<(), String> {
        if let Ok(scene) = self.get_scene(id) {
            let history_id = uuid::Uuid::new_v4().to_string();
            let _ = self.conn.execute(
                "INSERT INTO scene_history (id, scene_id, text, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![history_id, id, scene.text, now],
            );
        }
        Ok(())
    }

    pub fn get_scene_history(&self, scene_id: &str) -> Result<Vec<SceneHistoryEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, scene_id, text, created_at FROM scene_history WHERE scene_id = ?1 ORDER BY created_at DESC LIMIT 100",
            )
            .map_err(|e| e.to_string())?;

        let entries = stmt
            .query_map(params![scene_id], |row| {
                Ok(SceneHistoryEntry {
                    id: row.get(0)?,
                    scene_id: row.get(1)?,
                    text: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(entries)
    }

    pub fn restore_scene_version(&self, scene_id: &str, history_id: &str) -> Result<Scene, String> {
        let history_text: String = self
            .conn
            .query_row(
                "SELECT text FROM scene_history WHERE id = ?1 AND scene_id = ?2",
                params![history_id, scene_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        self.update_scene(
            scene_id,
            &UpdateSceneRequest {
                text: Some(history_text),
                ..Default::default()
            },
        )
    }

    pub fn compare_scene_versions(
        &self,
        scene_id: &str,
        version_id_a: &str,
        version_id_b: &str,
    ) -> Result<(String, String), String> {
        let text_a: String = self
            .conn
            .query_row(
                "SELECT text FROM scene_history WHERE id = ?1 AND scene_id = ?2",
                params![version_id_a, scene_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let text_b: String = self
            .conn
            .query_row(
                "SELECT text FROM scene_history WHERE id = ?1 AND scene_id = ?2",
                params![version_id_b, scene_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        Ok((text_a, text_b))
    }
}
