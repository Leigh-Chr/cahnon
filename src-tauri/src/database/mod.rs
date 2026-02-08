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

mod macros;

mod annotation;
mod arc;
mod bible;
mod chapter;
mod cut;
mod detection;
mod event;
mod export;
mod export_csv;
mod health;
mod impact;
mod issue;
mod project;
mod scene;
mod schema;
mod search;
mod settings;
mod snapshot;
mod template;
mod timeline;
mod trash;
mod world_state;

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

/// Whitelist of known table names for safe use in dynamic SQL.
const KNOWN_TABLES: &[&str] = &[
    "project",
    "chapters",
    "scenes",
    "bible_entries",
    "canonical_associations",
    "bible_relationships",
    "scene_history",
    "cuts",
    "arcs",
    "arc_characters",
    "scene_arcs",
    "events",
    "event_scenes",
    "event_bible",
    "templates",
    "template_steps",
    "scene_steps",
    "snapshots",
    "issues",
    "issue_scenes",
    "issue_bible",
    "annotations",
    "settings",
    "name_registry",
    "name_mentions",
    "saved_filters",
    "writing_sessions",
    "facts",
    "fact_characters",
    "auto_link_dismissed",
    "personal_dictionary",
    "schema_version",
];

impl Database {
    /// Validates that a table name is in the known whitelist.
    /// Prevents SQL injection when table names must be interpolated.
    pub(crate) fn validate_table_name(name: &str) -> Result<&str, String> {
        if KNOWN_TABLES.contains(&name) {
            Ok(name)
        } else {
            Err(format!("Unknown table name: {}", name))
        }
    }

    /// Validates that a SQL identifier (column name) contains only safe characters.
    /// Allows `[a-zA-Z_][a-zA-Z0-9_]*` — standard SQL identifier format.
    pub(crate) fn validate_identifier(name: &str) -> Result<&str, String> {
        if name.is_empty() {
            return Err("Identifier cannot be empty".to_string());
        }
        let first = name.as_bytes()[0];
        if !(first.is_ascii_alphabetic() || first == b'_') {
            return Err(format!("Invalid identifier: {name}"));
        }
        if name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            Ok(name)
        } else {
            Err(format!("Invalid identifier: {name}"))
        }
    }

    /// Validates that a column type definition contains no SQL injection vectors.
    /// Allows alphanumeric, spaces, parentheses, single quotes, dots, and common keywords.
    pub(crate) fn validate_column_type(col_type: &str) -> Result<&str, String> {
        if col_type.contains(';')
            || col_type.contains("--")
            || col_type.contains("/*")
            || col_type.contains('"')
        {
            return Err(format!("Invalid column type: {col_type}"));
        }
        Ok(col_type)
    }

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

    /// Runs a closure inside a BEGIN/COMMIT transaction, rolling back on error.
    ///
    /// Uses a RAII guard to ensure ROLLBACK on panic or early return.
    pub(crate) fn run_in_transaction<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        self.conn
            .execute("BEGIN TRANSACTION", [])
            .map_err(|e| e.to_string())?;

        let guard = RollbackGuard::new(&self.conn);

        match f() {
            Ok(value) => {
                guard.commit()?;
                Ok(value)
            }
            Err(e) => {
                // Guard will ROLLBACK on drop
                Err(e)
            }
        }
    }
}

/// RAII guard that automatically rolls back a transaction on drop unless committed.
struct RollbackGuard<'a> {
    conn: &'a Connection,
    committed: bool,
}

impl<'a> RollbackGuard<'a> {
    fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            committed: false,
        }
    }

    fn commit(mut self) -> Result<(), String> {
        self.conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
        self.committed = true;
        Ok(())
    }
}

impl<'a> Drop for RollbackGuard<'a> {
    fn drop(&mut self) {
        if !self.committed {
            if let Err(e) = self.conn.execute("ROLLBACK", []) {
                eprintln!("Failed to rollback transaction: {}", e);
            }
        }
    }
}
