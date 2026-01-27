//! Project CRUD operations

use crate::models::{CreateProjectRequest, Project, UpdateProjectRequest};
use rusqlite::params;

use super::macros::add_field;
use super::Database;

impl Database {
    pub fn create_project(&self, req: &CreateProjectRequest) -> Result<Project, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO project (id, title, author, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![id, req.title, req.author, req.description, now, now],
            )
            .map_err(|e| e.to_string())?;

        self.get_project()
    }

    pub fn get_project(&self) -> Result<Project, String> {
        self.conn
            .query_row(
                "SELECT id, title, author, description, word_target, daily_word_target, created_at, updated_at FROM project LIMIT 1",
                [],
                Self::map_project,
            )
            .map_err(|e| e.to_string())
    }

    fn map_project(row: &rusqlite::Row) -> rusqlite::Result<Project> {
        let core = Self::map_project_core(row)?;
        let meta = Self::map_project_meta(row)?;
        Ok(Project {
            id: core.0,
            title: core.1,
            author: core.2,
            description: core.3,
            word_target: meta.0,
            daily_word_target: meta.1,
            created_at: meta.2,
            updated_at: meta.3,
        })
    }

    fn map_project_core(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, Option<String>, Option<String>)> {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }

    fn map_project_meta(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(Option<i32>, Option<i32>, String, String)> {
        Ok((row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?))
    }

    /// Runs SQLite PRAGMA integrity_check to detect database corruption.
    /// Returns Ok(true) if the database is healthy, or an error message if corrupted.
    pub fn check_integrity(&self) -> Result<bool, String> {
        let result: String = self
            .conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| format!("Failed to run integrity check: {}", e))?;
        if result == "ok" {
            Ok(true)
        } else {
            Err(format!("Database integrity check failed: {}", result))
        }
    }

    pub fn update_project(&self, req: &UpdateProjectRequest) -> Result<Project, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        add_field!(set_clauses, params_vec, req.title, "title");
        add_field!(set_clauses, params_vec, req.author, "author");
        add_field!(set_clauses, params_vec, req.description, "description");
        add_field!(set_clauses, params_vec, req.word_target, "word_target", int);
        add_field!(
            set_clauses,
            params_vec,
            req.daily_word_target,
            "daily_word_target",
            int
        );

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let query = format!("UPDATE project SET {}", set_clauses.join(", "));
            let params_refs: Vec<&dyn rusqlite::ToSql> =
                params_vec.iter().map(|p| p.as_ref()).collect();

            self.conn
                .execute(&query, params_refs.as_slice())
                .map_err(|e| e.to_string())?;
        }

        self.get_project()
    }
}
