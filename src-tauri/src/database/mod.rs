//! Database layer for Cahnon projects.
//!
//! Each project is stored as a single SQLite database file (`.cahnon`).
//! This module handles schema initialization, migrations, and all CRUD
//! operations for project data.
//!
//! # Schema
//!
//! The database contains tables for:
//! - `project`: Project metadata
//! - `chapters`, `scenes`: Manuscript structure
//! - `bible_entries`: Story bible (characters, locations, etc.)
//! - `scene_bible_associations`: Canonical links
//! - `arcs`, `scene_arcs`: Plot threads
//! - `events`, `scene_events`: Timeline
//! - And many more (see `init_schema` for full list)

mod annotation;
mod arc;
mod bible;
mod chapter;
mod cut;
mod event;
mod export;
mod issue;
mod project;
mod scene;
mod search;
mod settings;
mod snapshot;
mod template;
mod timeline;
mod trash;

use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::path::Path;

/// Compiled regex for stripping HTML tags from text.
pub(crate) static HTML_TAG_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"<[^>]+>").expect("Invalid HTML tag regex"));

/// SQLite database wrapper for a Cahnon project.
///
/// Provides methods for all database operations. The connection is
/// held open for the duration of the project session.
pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    /// Creates a new project database at the given path.
    ///
    /// Initializes the full schema and creates an empty project.
    /// Fails if a file already exists at the path.
    pub fn create(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Database { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Opens an existing project database.
    ///
    /// Runs any pending migrations to update the schema.
    /// Fails if the file does not exist.
    pub fn open(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err("Project file does not exist".to_string());
        }
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), String> {
        // Add revision fields to scenes table if they don't exist
        let columns_to_add = vec![
            ("pov_goal", "TEXT"),
            ("has_conflict", "INTEGER"),
            ("has_change", "INTEGER"),
            ("tension", "TEXT"),
            ("setup_for_scene_id", "TEXT"),
            ("payoff_of_scene_id", "TEXT"),
            ("revision_notes", "TEXT"),
            ("revision_checklist", "TEXT"),
        ];

        for (column, col_type) in columns_to_add {
            // Check if column exists
            let column_exists: bool = self
                .conn
                .prepare(&format!("SELECT {} FROM scenes LIMIT 1", column))
                .is_ok();

            if !column_exists {
                self.conn
                    .execute(
                        &format!("ALTER TABLE scenes ADD COLUMN {} {}", column, col_type),
                        [],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }

        // Create name_registry table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS name_registry (
                id TEXT PRIMARY KEY,
                canonical_name TEXT NOT NULL,
                name_type TEXT NOT NULL DEFAULT 'character',
                bible_entry_id TEXT,
                aliases TEXT,
                is_confirmed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create name_mentions table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS name_mentions (
                id TEXT PRIMARY KEY,
                name_registry_id TEXT NOT NULL,
                scene_id TEXT NOT NULL,
                mention_text TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                FOREIGN KEY (name_registry_id) REFERENCES name_registry(id),
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create saved_filters table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS saved_filters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                filter_type TEXT NOT NULL,
                filter_data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create indexes for new tables
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_name_registry_type ON name_registry(name_type)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_name_registry_bible ON name_registry(bible_entry_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_name_mentions_registry ON name_mentions(name_registry_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_name_mentions_scene ON name_mentions(scene_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_saved_filters_type ON saved_filters(filter_type)",
            [],
        );

        // Add daily_word_target to project table if it doesn't exist
        let has_daily_target: bool = self
            .conn
            .prepare("SELECT daily_word_target FROM project LIMIT 1")
            .is_ok();
        if !has_daily_target {
            let _ = self.conn.execute(
                "ALTER TABLE project ADD COLUMN daily_word_target INTEGER",
                [],
            );
        }

        // Create arcs table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS arcs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                stakes TEXT,
                characters TEXT,
                status TEXT NOT NULL DEFAULT 'setup',
                color TEXT,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create scene_arcs table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS scene_arcs (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                arc_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id),
                FOREIGN KEY (arc_id) REFERENCES arcs(id),
                UNIQUE(scene_id, arc_id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create events table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                time_point TEXT,
                time_start TEXT,
                time_end TEXT,
                event_type TEXT NOT NULL DEFAULT 'scene',
                importance TEXT NOT NULL DEFAULT 'normal',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create event_scenes table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS event_scenes (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                scene_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES events(id),
                FOREIGN KEY (scene_id) REFERENCES scenes(id),
                UNIQUE(event_id, scene_id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create event_bible table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS event_bible (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                bible_entry_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES events(id),
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
                UNIQUE(event_id, bible_entry_id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create templates table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create template_steps table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS template_steps (
                id TEXT PRIMARY KEY,
                template_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                typical_position REAL NOT NULL DEFAULT 0,
                color TEXT,
                position INTEGER NOT NULL,
                FOREIGN KEY (template_id) REFERENCES templates(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create scene_steps table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS scene_steps (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                step_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id),
                FOREIGN KEY (step_id) REFERENCES template_steps(id),
                UNIQUE(scene_id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create snapshots table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                snapshot_type TEXT NOT NULL DEFAULT 'manual',
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create issues table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                issue_type TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                severity TEXT NOT NULL DEFAULT 'warning',
                status TEXT NOT NULL DEFAULT 'open',
                resolution_note TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create issue_scenes table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS issue_scenes (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                scene_id TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id),
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create issue_bible table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS issue_bible (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                bible_entry_id TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id),
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create annotations table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS annotations (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                annotation_type TEXT NOT NULL DEFAULT 'comment',
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create settings table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create cuts table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS cuts (
                id TEXT PRIMARY KEY,
                scene_id TEXT,
                text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                deleted_at TEXT,
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create scene_history table if not exists
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS scene_history (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                text TEXT NOT NULL,
                word_count INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Create indexes for new tables
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_arcs_status ON arcs(status)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_scene_arcs_scene ON scene_arcs(scene_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_scene_arcs_arc ON scene_arcs(arc_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_event_scenes_event ON event_scenes(event_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_event_scenes_scene ON event_scenes(scene_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_template_steps_template ON template_steps(template_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_scene_steps_scene ON scene_steps(scene_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_annotations_scene ON annotations(scene_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_scene_history_scene ON scene_history(scene_id)",
            [],
        );
        let _ = self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cuts_scene ON cuts(scene_id)",
            [],
        );

        Ok(())
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r"
            -- Project metadata
            CREATE TABLE IF NOT EXISTS project (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT,
                description TEXT,
                word_target INTEGER,
                daily_word_target INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Chapters
            CREATE TABLE IF NOT EXISTS chapters (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                summary TEXT,
                status TEXT NOT NULL DEFAULT 'planned',
                notes TEXT,
                position INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            );

            -- Scenes
            CREATE TABLE IF NOT EXISTS scenes (
                id TEXT PRIMARY KEY,
                chapter_id TEXT NOT NULL,
                title TEXT NOT NULL,
                summary TEXT,
                text TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'to write',
                pov TEXT,
                tags TEXT,
                notes TEXT,
                todos TEXT,
                word_target INTEGER,
                time_point TEXT,
                time_start TEXT,
                time_end TEXT,
                on_timeline INTEGER NOT NULL DEFAULT 1,
                position INTEGER NOT NULL,
                -- Revision fields
                pov_goal TEXT,
                has_conflict INTEGER,
                has_change INTEGER,
                tension TEXT,
                setup_for_scene_id TEXT,
                payoff_of_scene_id TEXT,
                revision_notes TEXT,
                revision_checklist TEXT,
                -- Timestamps
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT,
                FOREIGN KEY (chapter_id) REFERENCES chapters(id),
                FOREIGN KEY (setup_for_scene_id) REFERENCES scenes(id),
                FOREIGN KEY (payoff_of_scene_id) REFERENCES scenes(id)
            );

            -- Bible entries (knowledge base)
            CREATE TABLE IF NOT EXISTS bible_entries (
                id TEXT PRIMARY KEY,
                entry_type TEXT NOT NULL,
                name TEXT NOT NULL,
                aliases TEXT,
                short_description TEXT,
                full_description TEXT,
                status TEXT NOT NULL DEFAULT 'draft',
                tags TEXT,
                image_path TEXT,
                notes TEXT,
                todos TEXT,
                color TEXT,
                custom_fields TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            );

            -- Canonical associations (scene <-> bible entry)
            CREATE TABLE IF NOT EXISTS canonical_associations (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                bible_entry_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id),
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
                UNIQUE(scene_id, bible_entry_id)
            );

            -- Bible relationships (entry <-> entry)
            CREATE TABLE IF NOT EXISTS bible_relationships (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relationship_type TEXT NOT NULL,
                note TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                FOREIGN KEY (source_id) REFERENCES bible_entries(id),
                FOREIGN KEY (target_id) REFERENCES bible_entries(id)
            );

            -- Scene history (for version tracking)
            CREATE TABLE IF NOT EXISTS scene_history (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            );

            -- Cut library
            CREATE TABLE IF NOT EXISTS cuts (
                id TEXT PRIMARY KEY,
                scene_id TEXT,
                text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                deleted_at TEXT,
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            );

            -- Arcs (plot threads)
            CREATE TABLE IF NOT EXISTS arcs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                stakes TEXT,
                characters TEXT,
                status TEXT NOT NULL DEFAULT 'setup',
                color TEXT,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            );

            -- Scene-Arc associations
            CREATE TABLE IF NOT EXISTS scene_arcs (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                arc_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id),
                FOREIGN KEY (arc_id) REFERENCES arcs(id),
                UNIQUE(scene_id, arc_id)
            );

            -- Timeline events
            CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                time_point TEXT,
                time_start TEXT,
                time_end TEXT,
                event_type TEXT NOT NULL DEFAULT 'scene',
                importance TEXT NOT NULL DEFAULT 'normal',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                deleted_at TEXT
            );

            -- Event-Scene links
            CREATE TABLE IF NOT EXISTS event_scenes (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                scene_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES events(id),
                FOREIGN KEY (scene_id) REFERENCES scenes(id),
                UNIQUE(event_id, scene_id)
            );

            -- Event-Bible links
            CREATE TABLE IF NOT EXISTS event_bible (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                bible_entry_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES events(id),
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
                UNIQUE(event_id, bible_entry_id)
            );

            -- Narrative templates
            CREATE TABLE IF NOT EXISTS templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0,
                is_builtin INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Template steps
            CREATE TABLE IF NOT EXISTS template_steps (
                id TEXT PRIMARY KEY,
                template_id TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                typical_position REAL NOT NULL DEFAULT 0,
                color TEXT,
                position INTEGER NOT NULL,
                FOREIGN KEY (template_id) REFERENCES templates(id)
            );

            -- Scene-Step assignments
            CREATE TABLE IF NOT EXISTS scene_steps (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                step_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id),
                FOREIGN KEY (step_id) REFERENCES template_steps(id),
                UNIQUE(scene_id)
            );

            -- Snapshots
            CREATE TABLE IF NOT EXISTS snapshots (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                snapshot_type TEXT NOT NULL DEFAULT 'manual',
                data TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            -- Issues
            CREATE TABLE IF NOT EXISTS issues (
                id TEXT PRIMARY KEY,
                issue_type TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                severity TEXT NOT NULL DEFAULT 'warning',
                status TEXT NOT NULL DEFAULT 'open',
                resolution_note TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Issue-Scene links
            CREATE TABLE IF NOT EXISTS issue_scenes (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                scene_id TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id),
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            );

            -- Issue-Bible links
            CREATE TABLE IF NOT EXISTS issue_bible (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                bible_entry_id TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id),
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id)
            );

            -- Text annotations
            CREATE TABLE IF NOT EXISTS annotations (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                annotation_type TEXT NOT NULL DEFAULT 'comment',
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'open',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            );

            -- Settings (for app preferences)
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- Name Registry (proper nouns tracking)
            CREATE TABLE IF NOT EXISTS name_registry (
                id TEXT PRIMARY KEY,
                canonical_name TEXT NOT NULL,
                name_type TEXT NOT NULL DEFAULT 'character',
                bible_entry_id TEXT,
                aliases TEXT,
                is_confirmed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id)
            );

            -- Name mentions (detected occurrences in scenes)
            CREATE TABLE IF NOT EXISTS name_mentions (
                id TEXT PRIMARY KEY,
                name_registry_id TEXT NOT NULL,
                scene_id TEXT NOT NULL,
                mention_text TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL,
                FOREIGN KEY (name_registry_id) REFERENCES name_registry(id),
                FOREIGN KEY (scene_id) REFERENCES scenes(id)
            );

            -- Saved filters
            CREATE TABLE IF NOT EXISTS saved_filters (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                filter_type TEXT NOT NULL,
                filter_data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Indexes for performance
            CREATE INDEX IF NOT EXISTS idx_scenes_chapter ON scenes(chapter_id);
            CREATE INDEX IF NOT EXISTS idx_scenes_status ON scenes(status);
            CREATE INDEX IF NOT EXISTS idx_bible_type ON bible_entries(entry_type);
            CREATE INDEX IF NOT EXISTS idx_associations_scene ON canonical_associations(scene_id);
            CREATE INDEX IF NOT EXISTS idx_associations_bible ON canonical_associations(bible_entry_id);
            CREATE INDEX IF NOT EXISTS idx_arcs_position ON arcs(position);
            CREATE INDEX IF NOT EXISTS idx_scene_arcs_scene ON scene_arcs(scene_id);
            CREATE INDEX IF NOT EXISTS idx_scene_arcs_arc ON scene_arcs(arc_id);
            CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
            CREATE INDEX IF NOT EXISTS idx_annotations_scene ON annotations(scene_id);
            CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status);
            CREATE INDEX IF NOT EXISTS idx_bible_relationships_source ON bible_relationships(source_id);
            CREATE INDEX IF NOT EXISTS idx_bible_relationships_target ON bible_relationships(target_id);
            CREATE INDEX IF NOT EXISTS idx_scene_history_scene ON scene_history(scene_id);
            CREATE INDEX IF NOT EXISTS idx_name_registry_type ON name_registry(name_type);
            CREATE INDEX IF NOT EXISTS idx_name_registry_bible ON name_registry(bible_entry_id);
            CREATE INDEX IF NOT EXISTS idx_name_mentions_registry ON name_mentions(name_registry_id);
            CREATE INDEX IF NOT EXISTS idx_name_mentions_scene ON name_mentions(scene_id);
            CREATE INDEX IF NOT EXISTS idx_saved_filters_type ON saved_filters(filter_type);

            -- Full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS scenes_fts USING fts5(
                title, summary, text, notes,
                content='scenes',
                content_rowid='rowid'
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS bible_fts USING fts5(
                name, aliases, short_description, full_description, notes,
                content='bible_entries',
                content_rowid='rowid'
            );

            -- Triggers for FTS sync
            CREATE TRIGGER IF NOT EXISTS scenes_ai AFTER INSERT ON scenes BEGIN
                INSERT INTO scenes_fts(rowid, title, summary, text, notes)
                VALUES (NEW.rowid, NEW.title, NEW.summary, NEW.text, NEW.notes);
            END;

            CREATE TRIGGER IF NOT EXISTS scenes_ad AFTER DELETE ON scenes BEGIN
                INSERT INTO scenes_fts(scenes_fts, rowid, title, summary, text, notes)
                VALUES ('delete', OLD.rowid, OLD.title, OLD.summary, OLD.text, OLD.notes);
            END;

            CREATE TRIGGER IF NOT EXISTS scenes_au AFTER UPDATE ON scenes BEGIN
                INSERT INTO scenes_fts(scenes_fts, rowid, title, summary, text, notes)
                VALUES ('delete', OLD.rowid, OLD.title, OLD.summary, OLD.text, OLD.notes);
                INSERT INTO scenes_fts(rowid, title, summary, text, notes)
                VALUES (NEW.rowid, NEW.title, NEW.summary, NEW.text, NEW.notes);
            END;

            CREATE TRIGGER IF NOT EXISTS bible_ai AFTER INSERT ON bible_entries BEGIN
                INSERT INTO bible_fts(rowid, name, aliases, short_description, full_description, notes)
                VALUES (NEW.rowid, NEW.name, NEW.aliases, NEW.short_description, NEW.full_description, NEW.notes);
            END;

            CREATE TRIGGER IF NOT EXISTS bible_ad AFTER DELETE ON bible_entries BEGIN
                INSERT INTO bible_fts(bible_fts, rowid, name, aliases, short_description, full_description, notes)
                VALUES ('delete', OLD.rowid, OLD.name, OLD.aliases, OLD.short_description, OLD.full_description, OLD.notes);
            END;

            CREATE TRIGGER IF NOT EXISTS bible_au AFTER UPDATE ON bible_entries BEGIN
                INSERT INTO bible_fts(bible_fts, rowid, name, aliases, short_description, full_description, notes)
                VALUES ('delete', OLD.rowid, OLD.name, OLD.aliases, OLD.short_description, OLD.full_description, OLD.notes);
                INSERT INTO bible_fts(rowid, name, aliases, short_description, full_description, notes)
                VALUES (NEW.rowid, NEW.name, NEW.aliases, NEW.short_description, NEW.full_description, NEW.notes);
            END;
        ",
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
