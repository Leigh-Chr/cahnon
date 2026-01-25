//! Cut library database operations.

use super::Database;

impl Database {
    /// Creates a new cut in the cut library.
    pub fn create_cut(
        &self,
        scene_id: Option<&str>,
        text: &str,
    ) -> Result<crate::models::Cut, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO cuts (id, scene_id, text, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![id, scene_id, text, now],
            )
            .map_err(|e| e.to_string())?;

        self.conn
            .query_row(
                "SELECT id, scene_id, text, created_at FROM cuts WHERE id = ?1",
                rusqlite::params![id],
                |row| {
                    Ok(crate::models::Cut {
                        id: row.get(0)?,
                        scene_id: row.get(1)?,
                        text: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )
            .map_err(|e| e.to_string())
    }

    /// Gets all non-deleted cuts from the library.
    pub fn get_cuts(&self) -> Result<Vec<crate::models::Cut>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, scene_id, text, created_at FROM cuts WHERE deleted_at IS NULL ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let cuts = stmt
            .query_map([], |row| {
                Ok(crate::models::Cut {
                    id: row.get(0)?,
                    scene_id: row.get(1)?,
                    text: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(cuts)
    }

    /// Soft-deletes a cut from the library.
    pub fn delete_cut(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE cuts SET deleted_at = ?1 WHERE id = ?2",
                rusqlite::params![now, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
