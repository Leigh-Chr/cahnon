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
//! - And many more (see `schema.rs` for full list)

mod annotation;
mod arc;
mod bible;
mod chapter;
mod cut;
mod detection;
mod event;
mod export;
mod export_csv;
mod fact;
mod health;
mod impact;
mod issue;
mod name_registry;
mod project;
mod saved_filter;
mod scene;
mod scene_context;
mod schema;
mod search;
mod settings;
mod snapshot;
mod template;
mod timeline;
mod trash;
mod world_state;
mod writing_session;

use once_cell::sync::Lazy;
use rusqlite::Connection;
use std::path::Path;

/// Compiled regex for stripping HTML tags from text.
pub(crate) static HTML_TAG_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"<[^>]+>").expect("Invalid HTML tag regex"));

/// Compiled regex for converting HTML links to markdown format.
pub(crate) static LINK_REGEX: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r#"<a\s+href="([^"]+)"[^>]*>([^<]*)</a>"#).expect("Invalid link regex")
});

/// SQLite database wrapper for a Cahnon project.
///
/// Provides methods for all database operations. The connection is
/// held open for the duration of the project session.
pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    /// Enables foreign key enforcement and sets recommended pragmas.
    fn configure_connection(conn: &Connection) -> Result<(), String> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| format!("Failed to configure database connection: {}", e))?;

        // Enable WAL mode and verify it was applied (may fail on network filesystems)
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
            .map_err(|e| format!("Failed to set journal mode: {}", e))?;

        if journal_mode.to_lowercase() != "wal" {
            return Err(format!(
                "Failed to enable WAL mode (got '{}'); the database may be on an unsupported filesystem",
                journal_mode
            ));
        }

        Ok(())
    }

    /// Creates a new project database at the given path.
    ///
    /// Initializes the full schema and creates an empty project.
    /// Fails if a file already exists at the path.
    pub fn create(path: &Path) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Database { conn };
        Self::configure_connection(&db.conn)?;
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
        Self::configure_connection(&db.conn)?;
        db.run_migrations()?;
        Ok(db)
    }
}
