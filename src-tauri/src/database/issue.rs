//! Issue tracking database operations.

use super::Database;

impl Database {
    /// Creates a new issue.
    pub fn create_issue(
        &self,
        req: &crate::models::CreateIssueRequest,
    ) -> Result<crate::models::Issue, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO issues (id, issue_type, title, description, severity, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    id,
                    req.issue_type,
                    req.title,
                    req.description,
                    req.severity.as_deref().unwrap_or("warning"),
                    "open",
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_issue(&id)
    }

    /// Gets issues, optionally filtered by status.
    pub fn get_issues(&self, status: Option<&str>) -> Result<Vec<crate::models::Issue>, String> {
        let query = if status.is_some() {
            "SELECT id, issue_type, title, description, severity, status, resolution_note, created_at, updated_at
             FROM issues WHERE status = ?1 ORDER BY severity, created_at DESC"
        } else {
            "SELECT id, issue_type, title, description, severity, status, resolution_note, created_at, updated_at
             FROM issues ORDER BY status, severity, created_at DESC"
        };

        let mut stmt = self.conn.prepare(query).map_err(|e| e.to_string())?;

        let issues = if let Some(s) = status {
            stmt.query_map(rusqlite::params![s], Self::map_issue)
        } else {
            stmt.query_map([], Self::map_issue)
        }
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(issues)
    }

    fn map_issue(row: &rusqlite::Row) -> rusqlite::Result<crate::models::Issue> {
        let (id, issue_type, title, description, severity) = Self::map_issue_core(row)?;
        let (status, resolution_note, created_at, updated_at) = Self::map_issue_meta(row)?;
        Ok(crate::models::Issue {
            id,
            issue_type,
            title,
            description,
            severity,
            status,
            resolution_note,
            created_at,
            updated_at,
        })
    }

    #[allow(clippy::type_complexity)]
    fn map_issue_core(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, String, Option<String>, String)> {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    }

    fn map_issue_meta(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, Option<String>, String, String)> {
        Ok((row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?))
    }

    /// Gets a single issue by ID.
    pub fn get_issue(&self, id: &str) -> Result<crate::models::Issue, String> {
        self.conn
            .query_row(
                "SELECT id, issue_type, title, description, severity, status, resolution_note, created_at, updated_at
             FROM issues WHERE id = ?1",
                rusqlite::params![id],
                Self::map_issue,
            )
            .map_err(|e| e.to_string())
    }

    /// Updates an issue.
    pub fn update_issue(
        &self,
        id: &str,
        req: &crate::models::UpdateIssueRequest,
    ) -> Result<crate::models::Issue, String> {
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(status) = &req.status {
            self.conn
                .execute(
                    "UPDATE issues SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![status, now, id],
                )
                .map_err(|e| e.to_string())?;
        }
        if let Some(resolution_note) = &req.resolution_note {
            self.conn
                .execute(
                    "UPDATE issues SET resolution_note = ?1, updated_at = ?2 WHERE id = ?3",
                    rusqlite::params![resolution_note, now, id],
                )
                .map_err(|e| e.to_string())?;
        }

        self.get_issue(id)
    }

    /// Links a scene to an issue.
    pub fn link_scene_to_issue(&self, scene_id: &str, issue_id: &str) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO issue_scenes (id, issue_id, scene_id) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, issue_id, scene_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Unlinks a scene from an issue.
    pub fn unlink_scene_from_issue(&self, scene_id: &str, issue_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM issue_scenes WHERE issue_id = ?1 AND scene_id = ?2",
                rusqlite::params![issue_id, scene_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Gets all scene IDs linked to an issue.
    pub fn get_issue_scenes(&self, issue_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT scene_id FROM issue_scenes WHERE issue_id = ?1")
            .map_err(|e| e.to_string())?;

        let scene_ids = stmt
            .query_map(rusqlite::params![issue_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        scene_ids
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    /// Gets all issues linked to a scene.
    pub fn get_scene_issues(&self, scene_id: &str) -> Result<Vec<crate::models::Issue>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT i.id, i.issue_type, i.title, i.description, i.severity, i.status,
                        i.resolution_note, i.created_at, i.updated_at
                 FROM issues i
                 JOIN issue_scenes isc ON i.id = isc.issue_id
                 WHERE isc.scene_id = ?1
                 ORDER BY i.severity, i.created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let issues = stmt
            .query_map(rusqlite::params![scene_id], Self::map_issue)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(issues)
    }

    /// Links a bible entry to an issue.
    pub fn link_bible_entry_to_issue(
        &self,
        bible_entry_id: &str,
        issue_id: &str,
    ) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO issue_bible (id, issue_id, bible_entry_id) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, issue_id, bible_entry_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Unlinks a bible entry from an issue.
    pub fn unlink_bible_entry_from_issue(
        &self,
        bible_entry_id: &str,
        issue_id: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM issue_bible WHERE issue_id = ?1 AND bible_entry_id = ?2",
                rusqlite::params![issue_id, bible_entry_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Gets all bible entry IDs linked to an issue.
    pub fn get_issue_bible_entries(&self, issue_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT bible_entry_id FROM issue_bible WHERE issue_id = ?1")
            .map_err(|e| e.to_string())?;

        let entry_ids = stmt
            .query_map(rusqlite::params![issue_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        entry_ids
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }
}
