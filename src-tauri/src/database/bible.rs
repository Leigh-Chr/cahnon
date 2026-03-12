//! Bible entry and relationship operations

use crate::models::{
    AutoLinkResult, BibleEntry, BibleRelationship, BibleRelationshipWithEntry,
    CanonicalAssociation, CreateAssociationRequest, CreateBibleEntryRequest,
    CreateBibleRelationshipRequest, Scene, UpdateBibleEntryRequest, UpdateBibleRelationshipRequest,
};
use rusqlite::params;

use super::macros::add_field;
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
                "INSERT INTO bible_entries (id, entry_type, name, aliases, summary, full_description, status, tags, color, custom_fields, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    id,
                    req.entry_type,
                    req.name,
                    req.aliases,
                    req.summary,
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
            "SELECT id, entry_type, name, aliases, summary, full_description, status, tags, image_path, notes, todos, color, custom_fields, created_at, updated_at, deleted_at
             FROM bible_entries WHERE entry_type = ?1 AND deleted_at IS NULL ORDER BY name"
        } else {
            "SELECT id, entry_type, name, aliases, summary, full_description, status, tags, image_path, notes, todos, color, custom_fields, created_at, updated_at, deleted_at
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
        Ok(BibleEntry {
            id: row.get(0)?,
            entry_type: row.get(1)?,
            name: row.get(2)?,
            aliases: row.get(3)?,
            summary: row.get(4)?,
            full_description: row.get(5)?,
            status: row.get(6)?,
            tags: row.get(7)?,
            image_path: row.get(8)?,
            notes: row.get(9)?,
            todos: row.get(10)?,
            color: row.get(11)?,
            custom_fields: row.get(12)?,
            created_at: row.get(13)?,
            updated_at: row.get(14)?,
            deleted_at: row.get(15)?,
        })
    }

    pub fn get_bible_entry(&self, id: &str) -> Result<BibleEntry, String> {
        self.conn
            .query_row(
                "SELECT id, entry_type, name, aliases, summary, full_description, status, tags, image_path, notes, todos, color, custom_fields, created_at, updated_at, deleted_at
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
        add_field!(set_clauses, params_vec, req.name, "name");
        add_field!(set_clauses, params_vec, req.aliases, "aliases");
        add_field!(set_clauses, params_vec, req.summary, "summary");
        add_field!(
            set_clauses,
            params_vec,
            req.full_description,
            "full_description"
        );
        add_field!(set_clauses, params_vec, req.status, "status");
        add_field!(set_clauses, params_vec, req.tags, "tags");
        add_field!(set_clauses, params_vec, req.image_path, "image_path");
        add_field!(set_clauses, params_vec, req.notes, "notes");
        add_field!(set_clauses, params_vec, req.todos, "todos");
        add_field!(set_clauses, params_vec, req.color, "color");
        add_field!(set_clauses, params_vec, req.custom_fields, "custom_fields");
    }

    pub fn delete_bible_entry(&self, id: &str) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        self.run_in_transaction(|| {
            // Clean up junction tables to avoid orphaned records
            self.conn
                .execute(
                    "DELETE FROM canonical_associations WHERE bible_entry_id = ?1",
                    params![id],
                )
                .map_err(|e| e.to_string())?;
            self.conn
                .execute(
                    "DELETE FROM auto_link_dismissed WHERE bible_entry_id = ?1",
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
        })
    }

    pub fn search_bible(&self, query: &str) -> Result<Vec<BibleEntry>, String> {
        let sanitized = Self::sanitize_fts5_query(query);
        if sanitized.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = self
            .conn
            .prepare(
                "SELECT b.id, b.entry_type, b.name, b.aliases, b.summary, b.full_description,
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

        // Clear any dismissal so auto-link respects manual re-linking
        self.conn
            .execute(
                "DELETE FROM auto_link_dismissed WHERE scene_id = ?1 AND bible_entry_id = ?2",
                params![req.scene_id, req.bible_entry_id],
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
                "SELECT b.id, b.entry_type, b.name, b.aliases, b.summary, b.full_description,
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
        let rows = self
            .conn
            .execute(
                "DELETE FROM canonical_associations WHERE scene_id = ?1 AND bible_entry_id = ?2",
                params![scene_id, bible_entry_id],
            )
            .map_err(|e| e.to_string())?;

        // Record dismissal so auto-link does not recreate it (only if association existed)
        if rows > 0 {
            let now = chrono::Utc::now().to_rfc3339();
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO auto_link_dismissed (scene_id, bible_entry_id, created_at)
                     VALUES (?1, ?2, ?3)",
                    params![scene_id, bible_entry_id, now],
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    pub fn auto_link_bible_entries(&self, scene_id: &str) -> Result<AutoLinkResult, String> {
        // Load scene text and strip HTML
        let text: String = self
            .conn
            .query_row(
                "SELECT text FROM scenes WHERE id = ?1 AND deleted_at IS NULL",
                params![scene_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Scene not found: {e}"))?;

        let plain = super::HTML_TAG_REGEX.replace_all(&text, " ");

        // Load dismissed pairs for this scene
        let mut dismissed_stmt = self
            .conn
            .prepare("SELECT bible_entry_id FROM auto_link_dismissed WHERE scene_id = ?1")
            .map_err(|e| e.to_string())?;
        let dismissed: std::collections::HashSet<String> = dismissed_stmt
            .query_map(params![scene_id], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<std::collections::HashSet<_>, _>>()
            .map_err(|e| e.to_string())?;

        // Load all bible entries (id, name, aliases)
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, aliases FROM bible_entries WHERE deleted_at IS NULL")
            .map_err(|e| e.to_string())?;

        let entries: Vec<(String, String, Option<String>)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut created_count = 0i32;
        let mut new_entry_ids = Vec::new();

        for (entry_id, name, aliases) in &entries {
            // Skip entries the user explicitly dismissed for this scene
            if dismissed.contains(entry_id) {
                continue;
            }
            // Collect all terms (name + aliases), filter < 3 chars
            let mut terms: Vec<String> = Vec::new();
            let trimmed = name.trim();
            if trimmed.chars().count() >= 3 {
                terms.push(regex::escape(trimmed));
            }
            if let Some(alias_str) = aliases {
                for alias in alias_str.split(',') {
                    let a = alias.trim();
                    if a.chars().count() >= 3 {
                        terms.push(regex::escape(a));
                    }
                }
            }

            if terms.is_empty() {
                continue;
            }

            // Build case-insensitive word-boundary regex
            let pattern = format!(r"(?i)\b({})\b", terms.join("|"));
            let re = match regex::Regex::new(&pattern) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if re.is_match(&plain) {
                let id = uuid::Uuid::new_v4().to_string();
                let now = chrono::Utc::now().to_rfc3339();
                let rows = self
                    .conn
                    .execute(
                        "INSERT OR IGNORE INTO canonical_associations (id, scene_id, bible_entry_id, created_at)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![id, scene_id, entry_id, now],
                    )
                    .map_err(|e| e.to_string())?;

                if rows > 0 {
                    created_count += 1;
                    new_entry_ids.push(entry_id.clone());
                }
            }
        }

        Ok(AutoLinkResult {
            created_count,
            new_entry_ids,
        })
    }

    pub fn get_bible_entry_scenes(&self, bible_entry_id: &str) -> Result<Vec<Scene>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.chapter_id, s.title, s.summary, s.text, s.status, s.pov, s.tags,
                    s.notes, s.todos, s.word_target, s.time_point, s.time_start, s.time_end,
                    s.on_timeline, s.position, s.pov_goal, s.has_dramatic_conflict, s.has_change, s.tension,
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
                    b.id, b.entry_type, b.name, b.summary
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
        Ok(BibleRelationshipWithEntry {
            id: row.get(0)?,
            source_id: row.get(1)?,
            target_id: row.get(2)?,
            relationship_type: row.get(3)?,
            note: row.get(4)?,
            status: row.get(5)?,
            created_at: row.get(6)?,
            related_entry_id: row.get(7)?,
            related_entry_type: row.get(8)?,
            related_entry_name: row.get(9)?,
            related_entry_description: row.get(10)?,
        })
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

        add_field!(
            set_clauses,
            params_vec,
            req.relationship_type,
            "relationship_type"
        );
        add_field!(set_clauses, params_vec, req.note, "note");
        add_field!(set_clauses, params_vec, req.status, "status");

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
