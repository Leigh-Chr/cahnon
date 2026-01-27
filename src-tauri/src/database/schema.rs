//! Database schema initialization and migrations.

use super::Database;

impl Database {
    /// Runs migrations to update an existing database to the latest schema.
    pub(super) fn run_migrations(&self) -> Result<(), String> {
        self.migrate_scene_revision_fields()?;
        self.migrate_name_registry_tables()?;
        self.migrate_saved_filters_table()?;
        self.migrate_daily_word_target()?;
        self.migrate_arcs_tables()?;
        self.migrate_events_tables()?;
        self.migrate_templates_tables()?;
        self.migrate_issue_tables()?;
        self.migrate_annotations_table()?;
        self.migrate_arc_characters_table()?;
        self.migrate_missing_indexes()?;
        self.migrate_bible_relationships_unique()?;
        self.migrate_word_count_cache()?;
        self.migrate_writing_sessions_table()?;
        self.migrate_facts_tables()?;
        Ok(())
    }

    fn migrate_scene_revision_fields(&self) -> Result<(), String> {
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
        Ok(())
    }

    fn migrate_name_registry_tables(&self) -> Result<(), String> {
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

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_name_registry_type ON name_registry(name_type)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_name_registry_bible ON name_registry(bible_entry_id)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_name_mentions_registry ON name_mentions(name_registry_id)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_name_mentions_scene ON name_mentions(scene_id)",
                [],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn migrate_saved_filters_table(&self) -> Result<(), String> {
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

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_saved_filters_type ON saved_filters(filter_type)",
                [],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn migrate_daily_word_target(&self) -> Result<(), String> {
        let has_daily_target: bool = self
            .conn
            .prepare("SELECT daily_word_target FROM project LIMIT 1")
            .is_ok();
        if !has_daily_target {
            self.conn
                .execute(
                    "ALTER TABLE project ADD COLUMN daily_word_target INTEGER",
                    [],
                )
                .map_err(|e| format!("Failed to add daily_word_target column: {}", e))?;
        }
        Ok(())
    }

    fn migrate_arcs_tables(&self) -> Result<(), String> {
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
        Ok(())
    }

    fn migrate_events_tables(&self) -> Result<(), String> {
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
        Ok(())
    }

    fn migrate_templates_tables(&self) -> Result<(), String> {
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
        Ok(())
    }

    fn migrate_issue_tables(&self) -> Result<(), String> {
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

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS issue_scenes (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                scene_id TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id),
                FOREIGN KEY (scene_id) REFERENCES scenes(id),
                UNIQUE(issue_id, scene_id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS issue_bible (
                id TEXT PRIMARY KEY,
                issue_id TEXT NOT NULL,
                bible_entry_id TEXT NOT NULL,
                FOREIGN KEY (issue_id) REFERENCES issues(id),
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
                UNIQUE(issue_id, bible_entry_id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Add UNIQUE indexes for existing databases that were created before the constraint
        self.conn
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_issue_scenes_unique ON issue_scenes(issue_id, scene_id)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_issue_bible_unique ON issue_bible(issue_id, bible_entry_id)",
                [],
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn migrate_annotations_table(&self) -> Result<(), String> {
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
        Ok(())
    }

    fn migrate_arc_characters_table(&self) -> Result<(), String> {
        // Create the junction table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS arc_characters (
                id TEXT PRIMARY KEY,
                arc_id TEXT NOT NULL,
                bible_entry_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (arc_id) REFERENCES arcs(id),
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
                UNIQUE(arc_id, bible_entry_id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_arc_characters_arc ON arc_characters(arc_id)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_arc_characters_bible ON arc_characters(bible_entry_id)",
                [],
            )
            .map_err(|e| e.to_string())?;

        // Migrate existing data from arcs.characters column (if it exists)
        let has_characters_column: bool = self
            .conn
            .prepare("SELECT characters FROM arcs LIMIT 1")
            .is_ok();

        if has_characters_column {
            let mut stmt = self
                .conn
                .prepare("SELECT id, characters FROM arcs WHERE characters IS NOT NULL AND characters != ''")
                .map_err(|e| e.to_string())?;

            let rows: Vec<(String, String)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            let now = chrono::Utc::now().to_rfc3339();
            for (arc_id, characters) in rows {
                for char_id in characters
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                {
                    // Verify the bible entry exists before inserting
                    let exists: bool = self
                        .conn
                        .query_row(
                            "SELECT COUNT(*) > 0 FROM bible_entries WHERE id = ?1",
                            rusqlite::params![char_id],
                            |row| row.get(0),
                        )
                        .unwrap_or(false);

                    if exists {
                        let id = uuid::Uuid::new_v4().to_string();
                        let _ = self.conn.execute(
                            "INSERT OR IGNORE INTO arc_characters (id, arc_id, bible_entry_id, created_at) VALUES (?1, ?2, ?3, ?4)",
                            rusqlite::params![id, arc_id, char_id, now],
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn migrate_missing_indexes(&self) -> Result<(), String> {
        let indexes = [
            "CREATE INDEX IF NOT EXISTS idx_event_scenes_scene ON event_scenes(scene_id)",
            "CREATE INDEX IF NOT EXISTS idx_event_bible_bible ON event_bible(bible_entry_id)",
            "CREATE INDEX IF NOT EXISTS idx_issue_scenes_scene ON issue_scenes(scene_id)",
            "CREATE INDEX IF NOT EXISTS idx_issue_bible_bible ON issue_bible(bible_entry_id)",
            "CREATE INDEX IF NOT EXISTS idx_template_steps_template ON template_steps(template_id)",
            "CREATE INDEX IF NOT EXISTS idx_scene_steps_step ON scene_steps(step_id)",
            "CREATE INDEX IF NOT EXISTS idx_cuts_scene ON cuts(scene_id)",
            "CREATE INDEX IF NOT EXISTS idx_scenes_pov ON scenes(pov) WHERE pov IS NOT NULL AND deleted_at IS NULL",
            "CREATE INDEX IF NOT EXISTS idx_scenes_time ON scenes(time_point, time_start) WHERE deleted_at IS NULL",
        ];
        for sql in &indexes {
            self.conn.execute(sql, []).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn migrate_word_count_cache(&self) -> Result<(), String> {
        let has_word_count: bool = self
            .conn
            .prepare("SELECT word_count FROM scenes LIMIT 0")
            .is_ok();
        if !has_word_count {
            self.conn
                .execute(
                    "ALTER TABLE scenes ADD COLUMN word_count INTEGER NOT NULL DEFAULT 0",
                    [],
                )
                .map_err(|e| format!("Failed to add word_count column: {}", e))?;

            // Backfill existing scenes
            let mut stmt = self
                .conn
                .prepare("SELECT id, text FROM scenes WHERE text != ''")
                .map_err(|e| e.to_string())?;
            let rows: Vec<(String, String)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            for (id, text) in rows {
                let plain = crate::database::HTML_TAG_REGEX.replace_all(&text, " ");
                let wc = plain.split_whitespace().count() as i32;
                self.conn
                    .execute(
                        "UPDATE scenes SET word_count = ?1 WHERE id = ?2",
                        rusqlite::params![wc, id],
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    fn migrate_writing_sessions_table(&self) -> Result<(), String> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS writing_sessions (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                words_start INTEGER NOT NULL DEFAULT 0,
                words_end INTEGER NOT NULL DEFAULT 0,
                duration_minutes INTEGER NOT NULL DEFAULT 0,
                scenes_edited TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_writing_sessions_date ON writing_sessions(date)",
                [],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn migrate_facts_tables(&self) -> Result<(), String> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS facts (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                category TEXT NOT NULL DEFAULT 'plot',
                revealed_in_scene_id TEXT,
                status TEXT NOT NULL DEFAULT 'secret',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (revealed_in_scene_id) REFERENCES scenes(id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS fact_characters (
                id TEXT PRIMARY KEY,
                fact_id TEXT NOT NULL,
                bible_entry_id TEXT NOT NULL,
                learned_in_scene_id TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (fact_id) REFERENCES facts(id),
                FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
                FOREIGN KEY (learned_in_scene_id) REFERENCES scenes(id),
                UNIQUE(fact_id, bible_entry_id)
            )",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_facts_status ON facts(status)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_facts_revealed ON facts(revealed_in_scene_id)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_fact_characters_fact ON fact_characters(fact_id)",
                [],
            )
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_fact_characters_bible ON fact_characters(bible_entry_id)",
                [],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn migrate_bible_relationships_unique(&self) -> Result<(), String> {
        // Remove duplicates before creating the unique index
        self.conn
            .execute(
                "DELETE FROM bible_relationships WHERE id NOT IN (
                    SELECT MIN(id) FROM bible_relationships
                    GROUP BY source_id, target_id, relationship_type
                )",
                [],
            )
            .map_err(|e| e.to_string())?;

        self.conn
            .execute(
                "CREATE UNIQUE INDEX IF NOT EXISTS idx_bible_relationships_unique
                    ON bible_relationships(source_id, target_id, relationship_type)",
                [],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Initializes the full database schema for a new project.
    pub(super) fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(SCHEMA_SQL)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Complete database schema SQL for new projects.
const SCHEMA_SQL: &str = r"
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
    -- Cached word count
    word_count INTEGER NOT NULL DEFAULT 0,
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
    status TEXT NOT NULL DEFAULT 'setup',
    color TEXT,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);

-- Arc-Character associations
CREATE TABLE IF NOT EXISTS arc_characters (
    id TEXT PRIMARY KEY,
    arc_id TEXT NOT NULL,
    bible_entry_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (arc_id) REFERENCES arcs(id),
    FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
    UNIQUE(arc_id, bible_entry_id)
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
    FOREIGN KEY (scene_id) REFERENCES scenes(id),
    UNIQUE(issue_id, scene_id)
);

-- Issue-Bible links
CREATE TABLE IF NOT EXISTS issue_bible (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    bible_entry_id TEXT NOT NULL,
    FOREIGN KEY (issue_id) REFERENCES issues(id),
    FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
    UNIQUE(issue_id, bible_entry_id)
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

-- Writing sessions
CREATE TABLE IF NOT EXISTS writing_sessions (
    id TEXT PRIMARY KEY,
    date TEXT NOT NULL,
    words_start INTEGER NOT NULL DEFAULT 0,
    words_end INTEGER NOT NULL DEFAULT 0,
    duration_minutes INTEGER NOT NULL DEFAULT 0,
    scenes_edited TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

-- Narrative facts / revelations
CREATE TABLE IF NOT EXISTS facts (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    category TEXT NOT NULL DEFAULT 'plot',
    revealed_in_scene_id TEXT,
    status TEXT NOT NULL DEFAULT 'secret',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (revealed_in_scene_id) REFERENCES scenes(id)
);

-- Fact-Character knowledge links
CREATE TABLE IF NOT EXISTS fact_characters (
    id TEXT PRIMARY KEY,
    fact_id TEXT NOT NULL,
    bible_entry_id TEXT NOT NULL,
    learned_in_scene_id TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (fact_id) REFERENCES facts(id),
    FOREIGN KEY (bible_entry_id) REFERENCES bible_entries(id),
    FOREIGN KEY (learned_in_scene_id) REFERENCES scenes(id),
    UNIQUE(fact_id, bible_entry_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_scenes_chapter ON scenes(chapter_id);
CREATE INDEX IF NOT EXISTS idx_scenes_status ON scenes(status);
CREATE INDEX IF NOT EXISTS idx_bible_type ON bible_entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_associations_scene ON canonical_associations(scene_id);
CREATE INDEX IF NOT EXISTS idx_associations_bible ON canonical_associations(bible_entry_id);
CREATE INDEX IF NOT EXISTS idx_arcs_position ON arcs(position);
CREATE INDEX IF NOT EXISTS idx_arc_characters_arc ON arc_characters(arc_id);
CREATE INDEX IF NOT EXISTS idx_arc_characters_bible ON arc_characters(bible_entry_id);
CREATE INDEX IF NOT EXISTS idx_scene_arcs_scene ON scene_arcs(scene_id);
CREATE INDEX IF NOT EXISTS idx_scene_arcs_arc ON scene_arcs(arc_id);
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_event_scenes_scene ON event_scenes(scene_id);
CREATE INDEX IF NOT EXISTS idx_event_bible_bible ON event_bible(bible_entry_id);
CREATE INDEX IF NOT EXISTS idx_issue_scenes_scene ON issue_scenes(scene_id);
CREATE INDEX IF NOT EXISTS idx_issue_bible_bible ON issue_bible(bible_entry_id);
CREATE INDEX IF NOT EXISTS idx_template_steps_template ON template_steps(template_id);
CREATE INDEX IF NOT EXISTS idx_scene_steps_step ON scene_steps(step_id);
CREATE INDEX IF NOT EXISTS idx_cuts_scene ON cuts(scene_id);
CREATE INDEX IF NOT EXISTS idx_annotations_scene ON annotations(scene_id);
CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status);
CREATE INDEX IF NOT EXISTS idx_bible_relationships_source ON bible_relationships(source_id);
CREATE INDEX IF NOT EXISTS idx_bible_relationships_target ON bible_relationships(target_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_bible_relationships_unique ON bible_relationships(source_id, target_id, relationship_type);
CREATE INDEX IF NOT EXISTS idx_scene_history_scene ON scene_history(scene_id);
CREATE INDEX IF NOT EXISTS idx_name_registry_type ON name_registry(name_type);
CREATE INDEX IF NOT EXISTS idx_name_registry_bible ON name_registry(bible_entry_id);
CREATE INDEX IF NOT EXISTS idx_name_mentions_registry ON name_mentions(name_registry_id);
CREATE INDEX IF NOT EXISTS idx_name_mentions_scene ON name_mentions(scene_id);
CREATE INDEX IF NOT EXISTS idx_saved_filters_type ON saved_filters(filter_type);
CREATE INDEX IF NOT EXISTS idx_writing_sessions_date ON writing_sessions(date);
CREATE INDEX IF NOT EXISTS idx_facts_status ON facts(status);
CREATE INDEX IF NOT EXISTS idx_facts_revealed ON facts(revealed_in_scene_id);
CREATE INDEX IF NOT EXISTS idx_fact_characters_fact ON fact_characters(fact_id);
CREATE INDEX IF NOT EXISTS idx_fact_characters_bible ON fact_characters(bible_entry_id);
CREATE INDEX IF NOT EXISTS idx_scenes_pov ON scenes(pov) WHERE pov IS NOT NULL AND deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_scenes_time ON scenes(time_point, time_start) WHERE deleted_at IS NULL;

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
";
