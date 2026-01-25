//! Arc operations

use crate::models::{Arc, CreateArcRequest, UpdateArcRequest};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn create_arc(&self, req: &CreateArcRequest) -> Result<Arc, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let position = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), 0) + 1 FROM arcs WHERE deleted_at IS NULL",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        self.conn
            .execute(
                "INSERT INTO arcs (id, name, description, stakes, characters, status, color, position, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    id,
                    req.name,
                    req.description,
                    req.stakes,
                    req.characters,
                    req.status.as_deref().unwrap_or("setup"),
                    req.color,
                    position,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_arc(&id)
    }

    pub fn get_arcs(&self) -> Result<Vec<Arc>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, description, stakes, characters, status, color, position, created_at, updated_at
             FROM arcs WHERE deleted_at IS NULL ORDER BY position",
            )
            .map_err(|e| e.to_string())?;

        let arcs = stmt
            .query_map([], Self::map_arc)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(arcs)
    }

    fn map_arc(row: &rusqlite::Row) -> rusqlite::Result<Arc> {
        let (id, name, description, stakes, characters) = Self::map_arc_core(row)?;
        let (status, color, position, created_at, updated_at) = Self::map_arc_meta(row)?;
        Ok(Arc {
            id,
            name,
            description,
            stakes,
            characters,
            status,
            color,
            position,
            created_at,
            updated_at,
        })
    }

    #[allow(clippy::type_complexity)]
    fn map_arc_core(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, Option<String>, Option<String>, Option<String>)> {
        Ok((
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn map_arc_meta(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, Option<String>, i32, String, String)> {
        Ok((
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
            row.get(8)?,
            row.get(9)?,
        ))
    }

    pub fn get_arc(&self, id: &str) -> Result<Arc, String> {
        self.conn
            .query_row(
                "SELECT id, name, description, stakes, characters, status, color, position, created_at, updated_at
             FROM arcs WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                Self::map_arc,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_arc(&self, id: &str, req: &UpdateArcRequest) -> Result<Arc, String> {
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

        add_field!(req.name, "name");
        add_field!(req.description, "description");
        add_field!(req.stakes, "stakes");
        add_field!(req.characters, "characters");
        add_field!(req.status, "status");
        add_field!(req.color, "color");

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE arcs SET {} WHERE id = ?{}",
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

        self.get_arc(id)
    }

    pub fn delete_arc(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE arcs SET deleted_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn link_scene_to_arc(&self, scene_id: &str, arc_id: &str) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT OR IGNORE INTO scene_arcs (id, scene_id, arc_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, scene_id, arc_id, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn unlink_scene_from_arc(&self, scene_id: &str, arc_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM scene_arcs WHERE scene_id = ?1 AND arc_id = ?2",
                params![scene_id, arc_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_scene_arcs(&self, scene_id: &str) -> Result<Vec<Arc>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.name, a.description, a.stakes, a.characters, a.status, a.color, a.position, a.created_at, a.updated_at
             FROM arcs a
             JOIN scene_arcs sa ON a.id = sa.arc_id
             WHERE sa.scene_id = ?1 AND a.deleted_at IS NULL
             ORDER BY a.position",
            )
            .map_err(|e| e.to_string())?;

        let arcs = stmt
            .query_map(params![scene_id], Self::map_arc)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(arcs)
    }
}
