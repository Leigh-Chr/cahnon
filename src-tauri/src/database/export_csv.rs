//! CSV export operations (bible, timeline, review grid, stats)

use super::Database;

/// A CSV field: either a required string or an optional string.
/// Used to build CSV rows generically.
enum CsvField {
    Str(String),
    Opt(Option<String>),
    Bool(Option<bool>),
    Int(i32),
}

/// Escape a value for CSV: wrap in quotes if it contains commas, quotes, or newlines.
/// Double any existing quote characters.
fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Format an optional string for CSV output, returning empty string for None.
fn csv_opt(value: &Option<String>) -> String {
    match value {
        Some(v) => csv_escape(v),
        None => String::new(),
    }
}

/// Format an optional bool for CSV output.
fn format_bool_csv(value: Option<bool>) -> String {
    match value {
        Some(true) => "true".to_string(),
        Some(false) => "false".to_string(),
        None => String::new(),
    }
}

/// Build a CSV row from a list of fields, joining with commas and appending a newline.
fn csv_row(fields: &[CsvField]) -> String {
    let parts: Vec<String> = fields
        .iter()
        .map(|f| match f {
            CsvField::Str(s) => csv_escape(s),
            CsvField::Opt(o) => csv_opt(o),
            CsvField::Bool(b) => format_bool_csv(*b),
            CsvField::Int(i) => i.to_string(),
        })
        .collect();
    let mut row = parts.join(",");
    row.push('\n');
    row
}

impl Database {
    /// Export all bible entries as CSV.
    ///
    /// Columns: id, name, type, aliases, summary, status, tags
    pub fn export_bible_csv(&self) -> Result<String, String> {
        let mut output = String::from("id,name,type,aliases,summary,status,tags\n");

        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, entry_type, aliases, summary, status, tags
                 FROM bible_entries
                 WHERE deleted_at IS NULL
                 ORDER BY entry_type, name",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(csv_row(&[
                    CsvField::Str(row.get(0)?),
                    CsvField::Str(row.get(1)?),
                    CsvField::Str(row.get(2)?),
                    CsvField::Opt(row.get(3)?),
                    CsvField::Opt(row.get(4)?),
                    CsvField::Str(row.get(5)?),
                    CsvField::Opt(row.get(6)?),
                ]))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            output.push_str(&row.map_err(|e| e.to_string())?);
        }

        Ok(output)
    }

    /// Export timeline data as CSV.
    ///
    /// Combines events and scenes that are on the timeline.
    /// Columns: type, id, title, description, time_point, time_start, time_end, event_type, importance
    pub fn export_timeline_csv(&self) -> Result<String, String> {
        let mut output = String::from(
            "type,id,title,description,time_point,time_start,time_end,event_type,importance\n",
        );

        self.append_timeline_events(&mut output)?;
        self.append_timeline_scenes(&mut output)?;

        Ok(output)
    }

    /// Append event rows to the timeline CSV output.
    fn append_timeline_events(&self, output: &mut String) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, description, time_point, time_start, time_end, event_type, importance
                 FROM events
                 WHERE deleted_at IS NULL
                 ORDER BY COALESCE(time_point, time_start), created_at",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(csv_row(&[
                    CsvField::Str("event".to_string()),
                    CsvField::Str(row.get(0)?),
                    CsvField::Str(row.get(1)?),
                    CsvField::Opt(row.get(2)?),
                    CsvField::Opt(row.get(3)?),
                    CsvField::Opt(row.get(4)?),
                    CsvField::Opt(row.get(5)?),
                    CsvField::Str(row.get(6)?),
                    CsvField::Str(row.get(7)?),
                ]))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            output.push_str(&row.map_err(|e| e.to_string())?);
        }

        Ok(())
    }

    /// Append scene rows to the timeline CSV output.
    fn append_timeline_scenes(&self, output: &mut String) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, summary, time_point, time_start, time_end
                 FROM scenes
                 WHERE deleted_at IS NULL AND on_timeline = 1
                   AND (time_point IS NOT NULL OR time_start IS NOT NULL)
                 ORDER BY COALESCE(time_point, time_start)",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(csv_row(&[
                    CsvField::Str("scene".to_string()),
                    CsvField::Str(row.get(0)?),
                    CsvField::Str(row.get(1)?),
                    CsvField::Opt(row.get(2)?),
                    CsvField::Opt(row.get(3)?),
                    CsvField::Opt(row.get(4)?),
                    CsvField::Opt(row.get(5)?),
                    CsvField::Str(String::new()),
                    CsvField::Str(String::new()),
                ]))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            output.push_str(&row.map_err(|e| e.to_string())?);
        }

        Ok(())
    }

    /// Export review grid data as CSV.
    ///
    /// Columns: title, chapter, status, word_count, pov, tension, has_dramatic_conflict, has_change, setup_for, payoff_of
    pub fn export_review_grid_csv(&self) -> Result<String, String> {
        let mut output = String::from(
            "title,chapter,status,word_count,pov,tension,has_dramatic_conflict,has_change,setup_for,payoff_of\n",
        );

        // Pre-load all scene titles in one query to avoid N+1
        let scene_titles = self.get_all_scene_titles()?;

        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.title, c.title, s.status, s.word_count, s.pov, s.tension,
                    s.has_dramatic_conflict, s.has_change, s.setup_for_scene_id, s.payoff_of_scene_id
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                 ORDER BY c.position, s.position",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(ReviewGridRow {
                    title: row.get(0)?,
                    chapter_title: row.get(1)?,
                    status: row.get(2)?,
                    word_count: row.get(3)?,
                    pov: row.get(4)?,
                    tension: row.get(5)?,
                    has_dramatic_conflict: row.get(6)?,
                    has_change: row.get(7)?,
                    setup_for: row.get(8)?,
                    payoff_of: row.get(9)?,
                })
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let r = row.map_err(|e| e.to_string())?;
            output.push_str(&Self::format_review_grid_row(&r, &scene_titles));
        }

        Ok(output)
    }

    /// Pre-load all scene titles into a HashMap for batch lookups.
    fn get_all_scene_titles(&self) -> Result<std::collections::HashMap<String, String>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM scenes WHERE deleted_at IS NULL")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(rows.into_iter().collect())
    }

    /// Format a single review grid row as CSV, resolving scene ID references via pre-loaded titles.
    fn format_review_grid_row(
        r: &ReviewGridRow,
        scene_titles: &std::collections::HashMap<String, String>,
    ) -> String {
        let setup_for_title = r
            .setup_for
            .as_ref()
            .and_then(|id| scene_titles.get(id).cloned());
        let payoff_of_title = r
            .payoff_of
            .as_ref()
            .and_then(|id| scene_titles.get(id).cloned());

        csv_row(&[
            CsvField::Str(r.title.clone()),
            CsvField::Str(r.chapter_title.clone()),
            CsvField::Str(r.status.clone()),
            CsvField::Int(r.word_count),
            CsvField::Opt(r.pov.clone()),
            CsvField::Opt(r.tension.clone()),
            CsvField::Bool(r.has_dramatic_conflict),
            CsvField::Bool(r.has_change),
            CsvField::Opt(setup_for_title),
            CsvField::Opt(payoff_of_title),
        ])
    }

    /// Export word count stats as CSV.
    ///
    /// Two sections: by_chapter and by_status, separated by a blank line.
    pub fn export_stats_csv(&self) -> Result<String, String> {
        let mut output = String::from("section,label,word_count,scene_count\n");

        self.append_stats_section(
            &mut output,
            "by_chapter",
            "SELECT c.title,
                    COALESCE(SUM(s.word_count), 0) as wc,
                    COUNT(s.id) as sc
             FROM chapters c
             LEFT JOIN scenes s ON s.chapter_id = c.id AND s.deleted_at IS NULL
             WHERE c.deleted_at IS NULL
             GROUP BY c.id, c.title
             ORDER BY c.position",
        )?;

        self.append_stats_section(
            &mut output,
            "by_status",
            "SELECT status,
                    COALESCE(SUM(word_count), 0) as wc,
                    COUNT(*) as sc
             FROM scenes
             WHERE deleted_at IS NULL
             GROUP BY status",
        )?;

        Ok(output)
    }

    /// Append a stats section (label, word_count, scene_count) to the output.
    fn append_stats_section(
        &self,
        output: &mut String,
        section: &str,
        sql: &str,
    ) -> Result<(), String> {
        let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let label: String = row.get(0)?;
                let word_count: i32 = row.get(1)?;
                let scene_count: i32 = row.get(2)?;
                Ok((label, word_count, scene_count))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (label, word_count, scene_count) = row.map_err(|e| e.to_string())?;
            output.push_str(&csv_row(&[
                CsvField::Str(section.to_string()),
                CsvField::Str(label),
                CsvField::Int(word_count),
                CsvField::Int(scene_count),
            ]));
        }

        Ok(())
    }
}

/// Intermediate struct for review grid rows, used to separate query logic from formatting.
struct ReviewGridRow {
    title: String,
    chapter_title: String,
    status: String,
    word_count: i32,
    pov: Option<String>,
    tension: Option<String>,
    has_dramatic_conflict: Option<bool>,
    has_change: Option<bool>,
    setup_for: Option<String>,
    payoff_of: Option<String>,
}
