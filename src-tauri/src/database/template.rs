//! Template operations

use crate::models::{
    CreateTemplateRequest, CreateTemplateStepRequest, Template, TemplateStep,
    UpdateTemplateRequest, UpdateTemplateStepRequest,
};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn get_templates(&self) -> Result<Vec<Template>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, is_active, is_builtin, created_at, updated_at FROM templates ORDER BY is_builtin DESC, name",
            )
            .map_err(|e| e.to_string())?;

        let templates = stmt
            .query_map([], Self::map_template)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(templates)
    }

    fn map_template(row: &rusqlite::Row) -> rusqlite::Result<Template> {
        Ok(Template {
            id: row.get(0)?,
            name: row.get(1)?,
            is_active: row.get::<_, i32>(2)? != 0,
            is_builtin: row.get::<_, i32>(3)? != 0,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }

    pub fn get_template_steps(&self, template_id: &str) -> Result<Vec<TemplateStep>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, template_id, name, description, typical_position, color, position
             FROM template_steps WHERE template_id = ?1 ORDER BY position",
            )
            .map_err(|e| e.to_string())?;

        let steps = stmt
            .query_map(params![template_id], Self::map_template_step)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(steps)
    }

    fn map_template_step(row: &rusqlite::Row) -> rusqlite::Result<TemplateStep> {
        Ok(TemplateStep {
            id: row.get(0)?,
            template_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            typical_position: row.get(4)?,
            color: row.get(5)?,
            position: row.get(6)?,
        })
    }

    pub fn set_active_template(&self, template_id: &str) -> Result<(), String> {
        // Verify the template exists before changing anything
        self.get_template(template_id)?;

        self.conn
            .execute("BEGIN TRANSACTION", [])
            .map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            self.conn
                .execute("UPDATE templates SET is_active = 0", [])
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "UPDATE templates SET is_active = 1 WHERE id = ?1",
                    params![template_id],
                )
                .map_err(|e| e.to_string())?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    pub fn assign_scene_to_step(&self, scene_id: &str, step_id: &str) -> Result<(), String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT OR REPLACE INTO scene_steps (id, scene_id, step_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![id, scene_id, step_id, now],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_scene_step(&self, scene_id: &str) -> Result<Option<TemplateStep>, String> {
        let result = self.conn.query_row(
            "SELECT ts.id, ts.template_id, ts.name, ts.description, ts.typical_position, ts.color, ts.position
             FROM template_steps ts
             JOIN scene_steps ss ON ts.id = ss.step_id
             WHERE ss.scene_id = ?1",
            params![scene_id],
            Self::map_template_step,
        );

        match result {
            Ok(step) => Ok(Some(step)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn init_builtin_templates(&self) -> Result<(), String> {
        let count: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM templates WHERE is_builtin = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check builtin templates: {}", e))?;
        if count > 0 {
            return Ok(());
        }

        let now = chrono::Utc::now().to_rfc3339();

        self.init_three_act_structure(&now)?;
        self.init_save_the_cat(&now)?;
        self.init_heros_journey(&now)?;
        self.init_seven_point_structure(&now)?;

        Ok(())
    }

    fn create_builtin_template(
        &self,
        name: &str,
        now: &str,
        steps: &[(&str, &str, f64, &str)],
    ) -> Result<(), String> {
        let template_id = uuid::Uuid::new_v4().to_string();
        self.conn
            .execute(
                "INSERT INTO templates (id, name, is_active, is_builtin, created_at, updated_at) VALUES (?1, ?2, 0, 1, ?3, ?4)",
                params![template_id, name, now, now],
            )
            .map_err(|e| e.to_string())?;

        for (i, (step_name, desc, pos, color)) in steps.iter().enumerate() {
            let step_id = uuid::Uuid::new_v4().to_string();
            self.conn
                .execute(
                    "INSERT INTO template_steps (id, template_id, name, description, typical_position, color, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![step_id, template_id, step_name, desc, pos, color, i as i32],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn init_three_act_structure(&self, now: &str) -> Result<(), String> {
        let steps = [
            (
                "Setup",
                "Introduce characters, world, and status quo",
                0.1,
                "#3b82f6",
            ),
            (
                "Inciting Incident",
                "Event that disrupts the status quo",
                0.12,
                "#8b5cf6",
            ),
            (
                "First Plot Point",
                "Hero commits to the journey",
                0.25,
                "#f59e0b",
            ),
            (
                "Rising Action",
                "Obstacles and complications",
                0.5,
                "#10b981",
            ),
            ("Midpoint", "Major revelation or shift", 0.5, "#ef4444"),
            (
                "Second Plot Point",
                "All seems lost moment",
                0.75,
                "#f59e0b",
            ),
            ("Climax", "Final confrontation", 0.9, "#ef4444"),
            ("Resolution", "New status quo established", 0.95, "#10b981"),
        ];
        self.create_builtin_template("Three-Act Structure", now, &steps)
    }

    fn init_save_the_cat(&self, now: &str) -> Result<(), String> {
        let steps = [
            (
                "Opening Image",
                "Snapshot of the hero before the journey",
                0.01,
                "#3b82f6",
            ),
            (
                "Theme Stated",
                "Hint at the lesson to be learned",
                0.05,
                "#8b5cf6",
            ),
            ("Setup", "Introduce the hero's world", 0.1, "#3b82f6"),
            ("Catalyst", "Life-changing event", 0.12, "#f59e0b"),
            ("Debate", "Hero hesitates", 0.15, "#6b7280"),
            ("Break into Two", "Hero enters new world", 0.25, "#10b981"),
            (
                "B Story",
                "Love story or friendship introduced",
                0.3,
                "#ec4899",
            ),
            ("Fun and Games", "Promise of the premise", 0.4, "#10b981"),
            ("Midpoint", "False victory or defeat", 0.5, "#ef4444"),
            (
                "Bad Guys Close In",
                "Internal and external pressure",
                0.6,
                "#f59e0b",
            ),
            ("All Is Lost", "Darkest moment", 0.75, "#6b7280"),
            ("Dark Night of the Soul", "Hero regroups", 0.8, "#6b7280"),
            ("Break into Three", "Solution found", 0.85, "#10b981"),
            ("Finale", "Hero transforms and wins", 0.9, "#ef4444"),
            ("Final Image", "Proof of change", 0.99, "#3b82f6"),
        ];
        self.create_builtin_template("Save the Cat", now, &steps)
    }

    fn init_heros_journey(&self, now: &str) -> Result<(), String> {
        let steps = [
            ("Ordinary World", "Hero's normal life", 0.05, "#3b82f6"),
            (
                "Call to Adventure",
                "Problem or challenge appears",
                0.1,
                "#f59e0b",
            ),
            ("Refusal of the Call", "Hero hesitates", 0.15, "#6b7280"),
            ("Meeting the Mentor", "Hero gets help", 0.2, "#8b5cf6"),
            (
                "Crossing the Threshold",
                "Hero commits to the journey",
                0.25,
                "#10b981",
            ),
            (
                "Tests, Allies, Enemies",
                "Hero faces challenges",
                0.4,
                "#f59e0b",
            ),
            (
                "Approach to the Inmost Cave",
                "Hero prepares",
                0.55,
                "#6b7280",
            ),
            ("Ordeal", "Hero faces greatest fear", 0.65, "#ef4444"),
            ("Reward", "Hero seizes the prize", 0.75, "#10b981"),
            ("The Road Back", "Hero must return", 0.8, "#f59e0b"),
            (
                "Resurrection",
                "Final test, hero transformed",
                0.9,
                "#ef4444",
            ),
            (
                "Return with the Elixir",
                "Hero returns changed",
                0.95,
                "#3b82f6",
            ),
        ];
        self.create_builtin_template("Hero's Journey", now, &steps)
    }

    fn init_seven_point_structure(&self, now: &str) -> Result<(), String> {
        let steps = [
            ("Hook", "Opposite of the resolution", 0.05, "#3b82f6"),
            ("Plot Turn 1", "Call to action", 0.2, "#f59e0b"),
            ("Pinch Point 1", "Apply pressure", 0.35, "#ef4444"),
            ("Midpoint", "Move from reaction to action", 0.5, "#8b5cf6"),
            ("Pinch Point 2", "Apply more pressure", 0.65, "#ef4444"),
            ("Plot Turn 2", "Hero obtains final piece", 0.8, "#f59e0b"),
            ("Resolution", "Hero achieves goal", 0.95, "#10b981"),
        ];
        self.create_builtin_template("Seven-Point Structure", now, &steps)
    }

    pub fn create_template(&self, req: &CreateTemplateRequest) -> Result<Template, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO templates (id, name, is_active, is_builtin, created_at, updated_at) VALUES (?1, ?2, 0, 0, ?3, ?4)",
                params![id, req.name, now, now],
            )
            .map_err(|e| e.to_string())?;

        self.get_template(&id)
    }

    pub fn get_template(&self, id: &str) -> Result<Template, String> {
        self.conn
            .query_row(
                "SELECT id, name, is_active, is_builtin, created_at, updated_at FROM templates WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Template {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        is_active: row.get::<_, i32>(2)? != 0,
                        is_builtin: row.get::<_, i32>(3)? != 0,
                        created_at: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                },
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_template(
        &self,
        id: &str,
        req: &UpdateTemplateRequest,
    ) -> Result<Template, String> {
        let template = self.get_template(id)?;
        if template.is_builtin {
            return Err("Cannot modify builtin templates".to_string());
        }

        let now = chrono::Utc::now().to_rfc3339();

        if let Some(name) = &req.name {
            self.conn
                .execute(
                    "UPDATE templates SET name = ?1, updated_at = ?2 WHERE id = ?3",
                    params![name, now, id],
                )
                .map_err(|e| e.to_string())?;
        }

        self.get_template(id)
    }

    pub fn delete_template(&self, id: &str) -> Result<(), String> {
        let template = self.get_template(id)?;
        if template.is_builtin {
            return Err("Cannot delete builtin templates".to_string());
        }

        self.conn.execute("BEGIN", []).map_err(|e| e.to_string())?;

        let result = (|| -> Result<(), String> {
            // Clean up scene assignments for all steps in this template
            self.conn
                .execute(
                    "DELETE FROM scene_steps WHERE step_id IN (SELECT id FROM template_steps WHERE template_id = ?1)",
                    params![id],
                )
                .map_err(|e| e.to_string())?;

            self.conn
                .execute(
                    "DELETE FROM template_steps WHERE template_id = ?1",
                    params![id],
                )
                .map_err(|e| e.to_string())?;

            self.conn
                .execute("DELETE FROM templates WHERE id = ?1", params![id])
                .map_err(|e| e.to_string())?;

            Ok(())
        })();

        match result {
            Ok(()) => {
                self.conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
                Ok(())
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    pub fn create_template_step(
        &self,
        req: &CreateTemplateStepRequest,
    ) -> Result<TemplateStep, String> {
        let id = uuid::Uuid::new_v4().to_string();

        // Get next position
        let position: i32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM template_steps WHERE template_id = ?1",
                params![req.template_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to get next step position: {}", e))?;

        self.conn
            .execute(
                "INSERT INTO template_steps (id, template_id, name, description, typical_position, color, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    req.template_id,
                    req.name,
                    req.description,
                    req.typical_position.unwrap_or(50.0),
                    req.color,
                    position
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_template_step(&id)
    }

    pub fn get_template_step(&self, id: &str) -> Result<TemplateStep, String> {
        self.conn
            .query_row(
                "SELECT id, template_id, name, description, typical_position, color, position FROM template_steps WHERE id = ?1",
                params![id],
                Self::map_template_step,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_template_step(
        &self,
        id: &str,
        req: &UpdateTemplateStepRequest,
    ) -> Result<TemplateStep, String> {
        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        macro_rules! add_field {
            ($field:expr, $column:literal) => {
                if let Some(val) = &$field {
                    set_clauses.push(format!("{} = ?{}", $column, params_vec.len() + 1));
                    params_vec.push(Box::new(val.clone()));
                }
            };
            ($field:expr, $column:literal, float) => {
                if let Some(val) = $field {
                    set_clauses.push(format!("{} = ?{}", $column, params_vec.len() + 1));
                    params_vec.push(Box::new(val));
                }
            };
            ($field:expr, $column:literal, int) => {
                if let Some(val) = $field {
                    set_clauses.push(format!("{} = ?{}", $column, params_vec.len() + 1));
                    params_vec.push(Box::new(val));
                }
            };
        }

        add_field!(req.name, "name");
        add_field!(req.description, "description");
        add_field!(req.typical_position, "typical_position", float);
        add_field!(req.color, "color");
        add_field!(req.position, "position", int);

        if !set_clauses.is_empty() {
            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE template_steps SET {} WHERE id = ?{}",
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

        self.get_template_step(id)
    }

    pub fn delete_template_step(&self, id: &str) -> Result<(), String> {
        let step = self.get_template_step(id)?;
        let template = self.get_template(&step.template_id)?;
        if template.is_builtin {
            return Err("Cannot delete steps from builtin templates".to_string());
        }

        // Clean up scene assignments for this step
        self.conn
            .execute("DELETE FROM scene_steps WHERE step_id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        self.conn
            .execute("DELETE FROM template_steps WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
