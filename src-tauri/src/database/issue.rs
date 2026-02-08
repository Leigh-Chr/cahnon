//! Issue tracking database operations.

use super::macros::add_field;
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

    pub(crate) fn map_issue(row: &rusqlite::Row) -> rusqlite::Result<crate::models::Issue> {
        Ok(crate::models::Issue {
            id: row.get(0)?,
            issue_type: row.get(1)?,
            title: row.get(2)?,
            description: row.get(3)?,
            severity: row.get(4)?,
            status: row.get(5)?,
            resolution_note: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
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

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        add_field!(set_clauses, params_vec, req.status, "status");
        add_field!(
            set_clauses,
            params_vec,
            req.resolution_note,
            "resolution_note"
        );
        add_field!(set_clauses, params_vec, req.title, "title");
        add_field!(set_clauses, params_vec, req.description, "description");
        add_field!(set_clauses, params_vec, req.severity, "severity");

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE issues SET {} WHERE id = ?{}",
                set_clauses.join(", "),
                id_param_idx
            );

            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();
            let mut all_params = params_refs;
            all_params.push(&id);

            self.conn
                .execute(&query, all_params.as_slice())
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

    /// Gets all scenes linked to an issue.
    pub fn get_issue_scenes(&self, issue_id: &str) -> Result<Vec<crate::models::Scene>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.chapter_id, s.title, s.summary, s.text, s.status, s.pov, s.tags,
                    s.notes, s.todos, s.word_target, s.time_point, s.time_start, s.time_end,
                    s.on_timeline, s.position, s.pov_goal, s.has_conflict, s.has_change, s.tension,
                    s.setup_for_scene_id, s.payoff_of_scene_id, s.revision_notes, s.revision_checklist,
                    s.word_count, s.created_at, s.updated_at
             FROM scenes s
             JOIN issue_scenes isc ON s.id = isc.scene_id
             WHERE isc.issue_id = ?1 AND s.deleted_at IS NULL
             ORDER BY s.position",
            )
            .map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map(rusqlite::params![issue_id], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
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

    /// Deletes an issue and its links to scenes and bible entries.
    pub fn delete_issue(&self, id: &str) -> Result<(), String> {
        self.run_in_transaction(|| {
            // Clean up junction tables first
            self.conn
                .execute(
                    "DELETE FROM issue_scenes WHERE issue_id = ?1",
                    rusqlite::params![id],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM issue_bible WHERE issue_id = ?1",
                    rusqlite::params![id],
                )
                .map_err(|e| e.to_string())?;

            let rows = self
                .conn
                .execute("DELETE FROM issues WHERE id = ?1", rusqlite::params![id])
                .map_err(|e| e.to_string())?;

            if rows == 0 {
                return Err("Issue not found".to_string());
            }
            Ok(())
        })
    }

    /// Gets all bible entries linked to an issue.
    pub fn get_issue_bible_entries(
        &self,
        issue_id: &str,
    ) -> Result<Vec<crate::models::BibleEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT b.id, b.entry_type, b.name, b.aliases, b.short_description, b.full_description,
                    b.status, b.tags, b.image_path, b.notes, b.todos, b.color, b.custom_fields,
                    b.created_at, b.updated_at, b.deleted_at
             FROM bible_entries b
             JOIN issue_bible ib ON b.id = ib.bible_entry_id
             WHERE ib.issue_id = ?1 AND b.deleted_at IS NULL
             ORDER BY b.entry_type, b.name",
            )
            .map_err(|e| e.to_string())?;

        let entries = stmt
            .query_map(rusqlite::params![issue_id], Self::map_bible_entry)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(entries)
    }

    /// Gets status and resolution_note for all auto-detected issues, keyed by `issue_type:title`.
    pub fn get_auto_issue_states(
        &self,
    ) -> Result<std::collections::HashMap<String, (String, Option<String>)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT issue_type, title, status, resolution_note FROM issues WHERE issue_type LIKE 'auto_%'",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let issue_type: String = row.get(0)?;
                let title: String = row.get(1)?;
                let status: String = row.get(2)?;
                let resolution_note: Option<String> = row.get(3)?;
                Ok((
                    format!("{}:{}", issue_type, title),
                    (status, resolution_note),
                ))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().collect())
    }

    /// Deletes all auto-detected issues (issue_type starts with "auto_").
    pub fn delete_auto_detected_issues(&self) -> Result<(), String> {
        self.run_in_transaction(|| {
            // Clean junction tables
            self.conn
                .execute(
                    "DELETE FROM issue_scenes WHERE issue_id IN (SELECT id FROM issues WHERE issue_type LIKE 'auto_%')",
                    [],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM issue_bible WHERE issue_id IN (SELECT id FROM issues WHERE issue_type LIKE 'auto_%')",
                    [],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute("DELETE FROM issues WHERE issue_type LIKE 'auto_%'", [])
                .map_err(|e| e.to_string())?;
            Ok(())
        })
    }

    /// Creates an issue from a detection result and links scenes/bible entries.
    pub fn create_issue_from_detection(
        &self,
        d: &crate::models::DetectedIssue,
    ) -> Result<crate::models::Issue, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO issues (id, issue_type, title, description, severity, status, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    id,
                    format!("auto_{}", d.issue_type),
                    d.title,
                    d.description,
                    d.severity,
                    "open",
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        for scene_id in &d.scene_ids {
            self.link_scene_to_issue(scene_id, &id)?;
        }
        for bible_id in &d.bible_entry_ids {
            self.link_bible_entry_to_issue(bible_id, &id)?;
        }

        self.get_issue(&id)
    }

    /// Gets all issues linked to a bible entry.
    pub fn get_bible_entry_issues(
        &self,
        bible_entry_id: &str,
    ) -> Result<Vec<crate::models::Issue>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT i.id, i.issue_type, i.title, i.description, i.severity, i.status,
                        i.resolution_note, i.created_at, i.updated_at
                 FROM issues i
                 JOIN issue_bible ib ON i.id = ib.issue_id
                 WHERE ib.bible_entry_id = ?1
                 ORDER BY i.severity, i.created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let issues = stmt
            .query_map(rusqlite::params![bible_entry_id], Self::map_issue)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(issues)
    }
}
