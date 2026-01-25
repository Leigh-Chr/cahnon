//! Event operations (Timeline)

use crate::models::{CreateEventRequest, Event, UpdateEventRequest};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn create_event(&self, req: &CreateEventRequest) -> Result<Event, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO events (id, title, description, time_point, time_start, time_end, event_type, importance, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    id,
                    req.title,
                    req.description,
                    req.time_point,
                    req.time_start,
                    req.time_end,
                    req.event_type.as_deref().unwrap_or("scene"),
                    req.importance.as_deref().unwrap_or("normal"),
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_event(&id)
    }

    pub fn get_events(&self) -> Result<Vec<Event>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, description, time_point, time_start, time_end, event_type, importance, created_at, updated_at
             FROM events WHERE deleted_at IS NULL ORDER BY time_point, time_start, created_at",
            )
            .map_err(|e| e.to_string())?;

        let events = stmt
            .query_map([], Self::map_event)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(events)
    }

    fn map_event(row: &rusqlite::Row) -> rusqlite::Result<Event> {
        let (time_point, time_start, time_end, event_type, importance) =
            Self::map_event_time_fields(row)?;

        Ok(Event {
            id: row.get(0)?,
            title: row.get(1)?,
            description: row.get(2)?,
            time_point,
            time_start,
            time_end,
            event_type,
            importance,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }

    #[allow(clippy::type_complexity)]
    fn map_event_time_fields(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
    )> {
        Ok((
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
        ))
    }

    pub fn get_event(&self, id: &str) -> Result<Event, String> {
        self.conn
            .query_row(
                "SELECT id, title, description, time_point, time_start, time_end, event_type, importance, created_at, updated_at
             FROM events WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                Self::map_event,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_event(&self, id: &str, req: &UpdateEventRequest) -> Result<Event, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        macro_rules! add_field {
            ($field:expr, $column:literal) => {
                if let Some(val) = &$field {
                    set_clauses.push(format!("{} = ?{}", $column, params_vec.len() + 1));
                    params_vec.push(Box::new(val.clone()));
                }
            };
        }

        add_field!(req.title, "title");
        add_field!(req.description, "description");
        add_field!(req.time_point, "time_point");
        add_field!(req.time_start, "time_start");
        add_field!(req.time_end, "time_end");
        add_field!(req.event_type, "event_type");
        add_field!(req.importance, "importance");

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE events SET {} WHERE id = ?{}",
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

        self.get_event(id)
    }

    pub fn delete_event(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE events SET deleted_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // Event-Scene linking
    pub fn link_scene_to_event(&self, scene_id: &str, event_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO event_scenes (event_id, scene_id) VALUES (?1, ?2)",
                params![event_id, scene_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn unlink_scene_from_event(&self, scene_id: &str, event_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM event_scenes WHERE event_id = ?1 AND scene_id = ?2",
                params![event_id, scene_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_scene_events(&self, scene_id: &str) -> Result<Vec<Event>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT e.id, e.title, e.description, e.time_point, e.time_start, e.time_end,
                    e.event_type, e.importance, e.created_at, e.updated_at
             FROM events e
             JOIN event_scenes es ON e.id = es.event_id
             WHERE es.scene_id = ?1 AND e.deleted_at IS NULL
             ORDER BY COALESCE(e.time_point, e.time_start)",
            )
            .map_err(|e| e.to_string())?;

        let events = stmt
            .query_map(params![scene_id], Self::map_event)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(events)
    }

    pub fn get_event_scenes(&self, event_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT scene_id FROM event_scenes WHERE event_id = ?1")
            .map_err(|e| e.to_string())?;

        let scene_ids = stmt
            .query_map(params![event_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        scene_ids
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    // Event-Bible linking (per spec Section 9.1 - Links to Bible entries)
    pub fn link_bible_entry_to_event(
        &self,
        bible_entry_id: &str,
        event_id: &str,
    ) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO event_bible (id, event_id, bible_entry_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, event_id, bible_entry_id, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn unlink_bible_entry_from_event(
        &self,
        bible_entry_id: &str,
        event_id: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM event_bible WHERE event_id = ?1 AND bible_entry_id = ?2",
                params![event_id, bible_entry_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_event_bible_entries(&self, event_id: &str) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT bible_entry_id FROM event_bible WHERE event_id = ?1")
            .map_err(|e| e.to_string())?;

        let entry_ids = stmt
            .query_map(params![event_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;

        entry_ids
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn get_bible_entry_events(&self, bible_entry_id: &str) -> Result<Vec<Event>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT e.id, e.title, e.description, e.time_point, e.time_start, e.time_end,
                    e.event_type, e.importance, e.created_at, e.updated_at
             FROM events e
             JOIN event_bible eb ON e.id = eb.event_id
             WHERE eb.bible_entry_id = ?1 AND e.deleted_at IS NULL
             ORDER BY COALESCE(e.time_point, e.time_start)",
            )
            .map_err(|e| e.to_string())?;

        let events = stmt
            .query_map(params![bible_entry_id], Self::map_event)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(events)
    }
}
