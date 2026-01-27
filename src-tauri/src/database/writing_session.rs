//! Writing session operations

use crate::models::{UpdateWritingSessionRequest, WritingSession};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn create_writing_session(
        &self,
        date: &str,
        words_start: i32,
    ) -> Result<WritingSession, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO writing_sessions (id, date, words_start, words_end, duration_minutes, scenes_edited, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![id, date, words_start, 0, 0, "", now],
            )
            .map_err(|e| e.to_string())?;

        self.get_writing_session(&id)
    }

    pub fn update_writing_session(
        &self,
        id: &str,
        req: &UpdateWritingSessionRequest,
    ) -> Result<WritingSession, String> {
        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(val) = &req.words_end {
            set_clauses.push(format!("words_end = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(*val));
        }
        if let Some(val) = &req.duration_minutes {
            set_clauses.push(format!("duration_minutes = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(*val));
        }
        if let Some(val) = &req.scenes_edited {
            set_clauses.push(format!("scenes_edited = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(val.clone()));
        }

        if !set_clauses.is_empty() {
            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE writing_sessions SET {} WHERE id = ?{}",
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

        self.get_writing_session(id)
    }

    pub fn get_writing_session(&self, id: &str) -> Result<WritingSession, String> {
        self.conn
            .query_row(
                "SELECT id, date, words_start, words_end, duration_minutes, scenes_edited, created_at
             FROM writing_sessions WHERE id = ?1",
                params![id],
                Self::map_writing_session,
            )
            .map_err(|e| e.to_string())
    }

    pub fn get_writing_session_by_date(
        &self,
        date: &str,
    ) -> Result<Option<WritingSession>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, date, words_start, words_end, duration_minutes, scenes_edited, created_at
             FROM writing_sessions WHERE date = ?1",
            )
            .map_err(|e| e.to_string())?;

        let mut rows = stmt
            .query_map(params![date], Self::map_writing_session)
            .map_err(|e| e.to_string())?;

        match rows.next() {
            Some(row) => Ok(Some(row.map_err(|e| e.to_string())?)),
            None => Ok(None),
        }
    }

    pub fn get_writing_sessions(&self) -> Result<Vec<WritingSession>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, date, words_start, words_end, duration_minutes, scenes_edited, created_at
             FROM writing_sessions ORDER BY date DESC",
            )
            .map_err(|e| e.to_string())?;

        let sessions = stmt
            .query_map([], Self::map_writing_session)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(sessions)
    }

    pub fn delete_writing_session(&self, id: &str) -> Result<(), String> {
        let rows = self
            .conn
            .execute("DELETE FROM writing_sessions WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        if rows == 0 {
            return Err("Writing session not found".to_string());
        }
        Ok(())
    }

    fn map_writing_session(row: &rusqlite::Row) -> rusqlite::Result<WritingSession> {
        Ok(WritingSession {
            id: row.get(0)?,
            date: row.get(1)?,
            words_start: row.get(2)?,
            words_end: row.get(3)?,
            duration_minutes: row.get(4)?,
            scenes_edited: row.get(5)?,
            created_at: row.get(6)?,
        })
    }
}
