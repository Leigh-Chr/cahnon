//! Bible entry and relationship operations

use crate::models::{
    BibleEntry, BibleRelationship, BibleRelationshipWithEntry, CanonicalAssociation,
    CreateAssociationRequest, CreateBibleEntryRequest, CreateBibleRelationshipRequest, Scene,
    UpdateBibleEntryRequest, UpdateBibleRelationshipRequest,
};
use rusqlite::params;

use super::Database;

impl Database {
    // ========================================================================
    // Bible Entry operations
    // ========================================================================

    pub fn create_bible_entry(&self, req: &CreateBibleEntryRequest) -> Result<BibleEntry, String> {
        const VALID_ENTRY_TYPES: &[&str] = &[
            "character",
            "location",
            "object",
            "faction",
            "concept",
            "glossary",
        ];
        if !VALID_ENTRY_TYPES.contains(&req.entry_type.as_str()) {
            return Err(format!(
                "Invalid entry type '{}'. Must be one of: {}",
                req.entry_type,
                VALID_ENTRY_TYPES.join(", ")
            ));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let status = req.status.clone().unwrap_or_else(|| "draft".to_string());
        let custom_fields = Self::default_custom_fields(&req.entry_type);

        self.conn
            .execute(
                "INSERT INTO bible_entries (id, entry_type, name, aliases, short_description, full_description, status, tags, color, custom_fields, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    id,
                    req.entry_type,
                    req.name,
                    req.aliases,
                    req.short_description,
                    req.full_description,
                    status,
                    req.tags,
                    req.color,
                    custom_fields,
                    now,
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_bible_entry(&id)
    }

    /// Returns default custom fields JSON for a given entry type per spec 6.3-6.4.
    fn default_custom_fields(entry_type: &str) -> Option<String> {
        let fields = match entry_type {
            "character" => serde_json::json!({
                "role": "",
                "voice_notes": ""
            }),
            "location" => serde_json::json!({
                "parent_location": ""
            }),
            "faction" => serde_json::json!({
                "faction_type": "",
                "members": "",
                "headquarters": ""
            }),
            "glossary" => serde_json::json!({
                "pronunciation": "",
                "etymology": "",
                "language": ""
            }),
            _ => return None,
        };
        Some(fields.to_string())
    }

    pub fn get_bible_entries(&self, entry_type: Option<&str>) -> Result<Vec<BibleEntry>, String> {
        let query = if entry_type.is_some() {
            "SELECT id, entry_type, name, aliases, short_description, full_description, status, tags, image_path, notes, todos, color, custom_fields, created_at, updated_at, deleted_at
             FROM bible_entries WHERE entry_type = ?1 AND deleted_at IS NULL ORDER BY name"
        } else {
            "SELECT id, entry_type, name, aliases, short_description, full_description, status, tags, image_path, notes, todos, color, custom_fields, created_at, updated_at, deleted_at
             FROM bible_entries WHERE deleted_at IS NULL ORDER BY entry_type, name"
        };

        let mut stmt = self.conn.prepare(query).map_err(|e| e.to_string())?;

        let entries = if let Some(et) = entry_type {
            stmt.query_map(params![et], Self::map_bible_entry)
        } else {
            stmt.query_map([], Self::map_bible_entry)
        }
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(entries)
    }

    pub(crate) fn map_bible_entry(row: &rusqlite::Row) -> rusqlite::Result<BibleEntry> {
        let core = Self::map_bible_entry_core(row)?;
        let extra = Self::map_bible_entry_extra(row)?;

        Ok(BibleEntry {
            id: core.0,
            entry_type: core.1,
            name: core.2,
            aliases: core.3,
            short_description: core.4,
            full_description: core.5,
            status: core.6,
            tags: core.7,
            image_path: extra.0,
            notes: extra.1,
            todos: extra.2,
            color: extra.3,
            custom_fields: extra.4,
            created_at: extra.5,
            updated_at: extra.6,
            deleted_at: extra.7,
        })
    }

    #[allow(clippy::type_complexity)]
    fn map_bible_entry_core(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
    )> {
        let part1 = Self::map_bible_entry_identity(row)?;
        let part2 = Self::map_bible_entry_description(row)?;
        Ok((
            part1.0, part1.1, part1.2, part1.3, part2.0, part2.1, part2.2, part2.3,
        ))
    }

    fn map_bible_entry_identity(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, String, Option<String>)> {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }

    #[allow(clippy::type_complexity)]
    fn map_bible_entry_description(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(Option<String>, Option<String>, String, Option<String>)> {
        Ok((row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?))
    }

    #[allow(clippy::type_complexity)]
    fn map_bible_entry_extra(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        String,
        String,
        Option<String>,
    )> {
        let part1 = Self::map_bible_entry_media(row)?;
        let part2 = Self::map_bible_entry_timestamps(row)?;
        Ok((
            part1.0, part1.1, part1.2, part1.3, part2.0, part2.1, part2.2, part2.3,
        ))
    }

    #[allow(clippy::type_complexity)]
    fn map_bible_entry_media(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> {
        Ok((row.get(8)?, row.get(9)?, row.get(10)?, row.get(11)?))
    }

    fn map_bible_entry_timestamps(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(Option<String>, String, String, Option<String>)> {
        Ok((row.get(12)?, row.get(13)?, row.get(14)?, row.get(15)?))
    }

    pub fn get_bible_entry(&self, id: &str) -> Result<BibleEntry, String> {
        self.conn
            .query_row(
                "SELECT id, entry_type, name, aliases, short_description, full_description, status, tags, image_path, notes, todos, color, custom_fields, created_at, updated_at, deleted_at
             FROM bible_entries WHERE id = ?1 AND deleted_at IS NULL",
                params![id],
                Self::map_bible_entry,
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_bible_entry(
        &self,
        id: &str,
        req: &UpdateBibleEntryRequest,
    ) -> Result<BibleEntry, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let mut set_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        Self::collect_bible_entry_fields(req, &mut set_clauses, &mut params_vec);

        if !set_clauses.is_empty() {
            set_clauses.push(format!("updated_at = ?{}", params_vec.len() + 1));
            params_vec.push(Box::new(now));

            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE bible_entries SET {} WHERE id = ?{}",
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

        self.get_bible_entry(id)
    }

    fn collect_bible_entry_fields(
        req: &UpdateBibleEntryRequest,
        set_clauses: &mut Vec<String>,
        params_vec: &mut Vec<Box<dyn rusqlite::ToSql>>,
    ) {
        macro_rules! add_field {
            ($field:expr, $column:literal) => {
                if let Some(val) = &$field {
                    set_clauses.push(format!("{} = ?{}", $column, params_vec.len() + 1));
                    params_vec.push(Box::new(val.clone()));
                }
            };
        }

        add_field!(req.name, "name");
        add_field!(req.aliases, "aliases");
        add_field!(req.short_description, "short_description");
        add_field!(req.full_description, "full_description");
        add_field!(req.status, "status");
        add_field!(req.tags, "tags");
        add_field!(req.image_path, "image_path");
        add_field!(req.notes, "notes");
        add_field!(req.todos, "todos");
        add_field!(req.color, "color");
        add_field!(req.custom_fields, "custom_fields");
    }

    pub fn delete_bible_entry(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        // Clean up junction tables to avoid orphaned records
        self.conn
            .execute(
                "DELETE FROM canonical_associations WHERE bible_entry_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "DELETE FROM bible_relationships WHERE source_id = ?1 OR target_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "DELETE FROM event_bible WHERE bible_entry_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "DELETE FROM issue_bible WHERE bible_entry_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "DELETE FROM arc_characters WHERE bible_entry_id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;

        // Soft-delete the entry itself
        self.conn
            .execute(
                "UPDATE bible_entries SET deleted_at = ?1 WHERE id = ?2",
                params![now, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn search_bible(&self, query: &str) -> Result<Vec<BibleEntry>, String> {
        let sanitized = Self::sanitize_fts5_query(query);
        if sanitized.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = self
            .conn
            .prepare(
                "SELECT b.id, b.entry_type, b.name, b.aliases, b.short_description, b.full_description,
                    b.status, b.tags, b.image_path, b.notes, b.todos, b.color, b.custom_fields,
                    b.created_at, b.updated_at, b.deleted_at
             FROM bible_entries b
             JOIN bible_fts ON b.rowid = bible_fts.rowid
             WHERE bible_fts MATCH ?1 AND b.deleted_at IS NULL",
            )
            .map_err(|e| e.to_string())?;

        let entries = stmt
            .query_map(params![sanitized], Self::map_bible_entry)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(entries)
    }

    // ========================================================================
    // Association operations
    // ========================================================================

    pub fn create_association(
        &self,
        req: &CreateAssociationRequest,
    ) -> Result<CanonicalAssociation, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT OR IGNORE INTO canonical_associations (id, scene_id, bible_entry_id, created_at)
             VALUES (?1, ?2, ?3, ?4)",
                params![id, req.scene_id, req.bible_entry_id, now],
            )
            .map_err(|e| e.to_string())?;

        self.conn
            .query_row(
                "SELECT id, scene_id, bible_entry_id, created_at FROM canonical_associations
             WHERE scene_id = ?1 AND bible_entry_id = ?2",
                params![req.scene_id, req.bible_entry_id],
                |row| {
                    Ok(CanonicalAssociation {
                        id: row.get(0)?,
                        scene_id: row.get(1)?,
                        bible_entry_id: row.get(2)?,
                        created_at: row.get(3)?,
                    })
                },
            )
            .map_err(|e| e.to_string())
    }

    pub fn get_scene_associations(&self, scene_id: &str) -> Result<Vec<BibleEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT b.id, b.entry_type, b.name, b.aliases, b.short_description, b.full_description,
                    b.status, b.tags, b.image_path, b.notes, b.todos, b.color, b.custom_fields,
                    b.created_at, b.updated_at, b.deleted_at
             FROM bible_entries b
             JOIN canonical_associations ca ON b.id = ca.bible_entry_id
             WHERE ca.scene_id = ?1 AND b.deleted_at IS NULL
             ORDER BY b.entry_type, b.name",
            )
            .map_err(|e| e.to_string())?;

        let entries = stmt
            .query_map(params![scene_id], Self::map_bible_entry)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(entries)
    }

    pub fn delete_association(&self, scene_id: &str, bible_entry_id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM canonical_associations WHERE scene_id = ?1 AND bible_entry_id = ?2",
                params![scene_id, bible_entry_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_bible_entry_scenes(&self, bible_entry_id: &str) -> Result<Vec<Scene>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.chapter_id, s.title, s.summary, s.text, s.status, s.pov, s.tags,
                    s.notes, s.todos, s.word_target, s.time_point, s.time_start, s.time_end,
                    s.on_timeline, s.position, s.pov_goal, s.has_conflict, s.has_change, s.tension,
                    s.setup_for_scene_id, s.payoff_of_scene_id, s.revision_notes, s.revision_checklist,
                    s.word_count, s.created_at, s.updated_at
             FROM scenes s
             JOIN canonical_associations ca ON s.id = ca.scene_id
             WHERE ca.bible_entry_id = ?1 AND s.deleted_at IS NULL
             ORDER BY s.position",
            )
            .map_err(|e| e.to_string())?;

        let scenes = stmt
            .query_map(params![bible_entry_id], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(scenes)
    }

    // ========================================================================
    // Bible Relationships operations
    // ========================================================================

    pub fn create_bible_relationship(
        &self,
        req: &CreateBibleRelationshipRequest,
    ) -> Result<BibleRelationship, String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO bible_relationships (id, source_id, target_id, relationship_type, note, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id,
                    req.source_id,
                    req.target_id,
                    req.relationship_type,
                    req.note,
                    req.status.as_deref().unwrap_or("active"),
                    now
                ],
            )
            .map_err(|e| e.to_string())?;

        self.get_bible_relationship(&id)
    }

    pub fn get_bible_relationships(
        &self,
        entry_id: &str,
    ) -> Result<Vec<BibleRelationshipWithEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT r.id, r.source_id, r.target_id, r.relationship_type, r.note, r.status, r.created_at,
                    b.id, b.entry_type, b.name, b.short_description
             FROM bible_relationships r
             JOIN bible_entries b ON (r.target_id = b.id OR r.source_id = b.id) AND b.id != ?1
             WHERE (r.source_id = ?1 OR r.target_id = ?1) AND b.deleted_at IS NULL
             ORDER BY r.relationship_type, b.name",
            )
            .map_err(|e| e.to_string())?;

        let relationships = stmt
            .query_map(params![entry_id], Self::map_bible_relationship_with_entry)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(relationships)
    }

    fn map_bible_relationship_with_entry(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<BibleRelationshipWithEntry> {
        let core = Self::map_relationship_core(row)?;
        let related = Self::map_related_entry_fields(row)?;

        Ok(BibleRelationshipWithEntry {
            id: core.0,
            source_id: core.1,
            target_id: core.2,
            relationship_type: core.3,
            note: core.4,
            status: core.5,
            created_at: core.6,
            related_entry_id: related.0,
            related_entry_type: related.1,
            related_entry_name: related.2,
            related_entry_description: related.3,
        })
    }

    #[allow(clippy::type_complexity)]
    fn map_relationship_core(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(
        String,
        String,
        String,
        String,
        Option<String>,
        String,
        String,
    )> {
        let part1 = Self::map_relationship_ids(row)?;
        let part2 = Self::map_relationship_meta(row)?;
        Ok((
            part1.0, part1.1, part1.2, part2.0, part2.1, part2.2, part2.3,
        ))
    }

    fn map_relationship_ids(row: &rusqlite::Row) -> rusqlite::Result<(String, String, String)> {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    }

    fn map_relationship_meta(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, Option<String>, String, String)> {
        Ok((row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?))
    }

    fn map_related_entry_fields(
        row: &rusqlite::Row,
    ) -> rusqlite::Result<(String, String, String, Option<String>)> {
        Ok((row.get(7)?, row.get(8)?, row.get(9)?, row.get(10)?))
    }

    pub fn get_bible_relationship(&self, id: &str) -> Result<BibleRelationship, String> {
        self.conn
            .query_row(
                "SELECT id, source_id, target_id, relationship_type, note, status, created_at
             FROM bible_relationships WHERE id = ?1",
                params![id],
                |row| {
                    Ok(BibleRelationship {
                        id: row.get(0)?,
                        source_id: row.get(1)?,
                        target_id: row.get(2)?,
                        relationship_type: row.get(3)?,
                        note: row.get(4)?,
                        status: row.get(5)?,
                        created_at: row.get(6)?,
                    })
                },
            )
            .map_err(|e| e.to_string())
    }

    pub fn update_bible_relationship(
        &self,
        id: &str,
        req: &UpdateBibleRelationshipRequest,
    ) -> Result<BibleRelationship, String> {
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

        add_field!(req.relationship_type, "relationship_type");
        add_field!(req.note, "note");
        add_field!(req.status, "status");

        if !set_clauses.is_empty() {
            let id_param_idx = params_vec.len() + 1;
            let query = format!(
                "UPDATE bible_relationships SET {} WHERE id = ?{}",
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

        self.get_bible_relationship(id)
    }

    pub fn delete_bible_relationship(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM bible_relationships WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
