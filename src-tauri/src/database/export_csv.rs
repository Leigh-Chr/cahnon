//! CSV export operations (bible, timeline, review grid, stats)

use rusqlite::params;

use super::Database;

impl Database {
    /// Escape a value for CSV: wrap in quotes if it contains commas, quotes, or newlines.
    /// Double any existing quote characters.
    fn csv_escape(value: &str) -> String {
        if value.contains(',')
            || value.contains('"')
            || value.contains('\n')
            || value.contains('\r')
        {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }

    /// Format an optional string for CSV output, returning empty string for None.
    fn csv_opt(value: &Option<String>) -> String {
        match value {
            Some(v) => Self::csv_escape(v),
            None => String::new(),
        }
    }

    /// Export all bible entries as CSV.
    ///
    /// Columns: id, name, type, aliases, short_description, status, tags
    pub fn export_bible_csv(&self) -> Result<String, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, entry_type, aliases, short_description, status, tags
                 FROM bible_entries
                 WHERE deleted_at IS NULL
                 ORDER BY entry_type, name",
            )
            .map_err(|e| e.to_string())?;

        let mut output = String::from("id,name,type,aliases,short_description,status,tags\n");

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let entry_type: String = row.get(2)?;
                let aliases: Option<String> = row.get(3)?;
                let short_description: Option<String> = row.get(4)?;
                let status: String = row.get(5)?;
                let tags: Option<String> = row.get(6)?;
                Ok((
                    id,
                    name,
                    entry_type,
                    aliases,
                    short_description,
                    status,
                    tags,
                ))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (id, name, entry_type, aliases, short_description, status, tags) =
                row.map_err(|e| e.to_string())?;
            output.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                Self::csv_escape(&id),
                Self::csv_escape(&name),
                Self::csv_escape(&entry_type),
                Self::csv_opt(&aliases),
                Self::csv_opt(&short_description),
                Self::csv_escape(&status),
                Self::csv_opt(&tags),
            ));
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

        // Export events
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, description, time_point, time_start, time_end, event_type, importance
                 FROM events
                 WHERE deleted_at IS NULL
                 ORDER BY COALESCE(time_point, time_start), created_at",
            )
            .map_err(|e| e.to_string())?;

        let events = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let description: Option<String> = row.get(2)?;
                let time_point: Option<String> = row.get(3)?;
                let time_start: Option<String> = row.get(4)?;
                let time_end: Option<String> = row.get(5)?;
                let event_type: String = row.get(6)?;
                let importance: String = row.get(7)?;
                Ok((
                    id,
                    title,
                    description,
                    time_point,
                    time_start,
                    time_end,
                    event_type,
                    importance,
                ))
            })
            .map_err(|e| e.to_string())?;

        for row in events {
            let (id, title, description, time_point, time_start, time_end, event_type, importance) =
                row.map_err(|e| e.to_string())?;
            output.push_str(&format!(
                "event,{},{},{},{},{},{},{},{}\n",
                Self::csv_escape(&id),
                Self::csv_escape(&title),
                Self::csv_opt(&description),
                Self::csv_opt(&time_point),
                Self::csv_opt(&time_start),
                Self::csv_opt(&time_end),
                Self::csv_escape(&event_type),
                Self::csv_escape(&importance),
            ));
        }

        // Export scenes on timeline
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

        let scenes = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let title: String = row.get(1)?;
                let summary: Option<String> = row.get(2)?;
                let time_point: Option<String> = row.get(3)?;
                let time_start: Option<String> = row.get(4)?;
                let time_end: Option<String> = row.get(5)?;
                Ok((id, title, summary, time_point, time_start, time_end))
            })
            .map_err(|e| e.to_string())?;

        for row in scenes {
            let (id, title, summary, time_point, time_start, time_end) =
                row.map_err(|e| e.to_string())?;
            output.push_str(&format!(
                "scene,{},{},{},{},{},{},,\n",
                Self::csv_escape(&id),
                Self::csv_escape(&title),
                Self::csv_opt(&summary),
                Self::csv_opt(&time_point),
                Self::csv_opt(&time_start),
                Self::csv_opt(&time_end),
            ));
        }

        Ok(output)
    }

    /// Export review grid data as CSV.
    ///
    /// Columns: title, chapter, status, word_count, pov, tension, has_conflict, has_change, setup_for, payoff_of
    pub fn export_review_grid_csv(&self) -> Result<String, String> {
        let mut output = String::from(
            "title,chapter,status,word_count,pov,tension,has_conflict,has_change,setup_for,payoff_of\n",
        );

        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.title, c.title, s.status, s.word_count, s.pov, s.tension,
                    s.has_conflict, s.has_change, s.setup_for_scene_id, s.payoff_of_scene_id
                 FROM scenes s
                 JOIN chapters c ON s.chapter_id = c.id
                 WHERE s.deleted_at IS NULL AND c.deleted_at IS NULL
                 ORDER BY c.position, s.position",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let title: String = row.get(0)?;
                let chapter_title: String = row.get(1)?;
                let status: String = row.get(2)?;
                let word_count: i32 = row.get(3)?;
                let pov: Option<String> = row.get(4)?;
                let tension: Option<String> = row.get(5)?;
                let has_conflict: Option<bool> = row.get(6)?;
                let has_change: Option<bool> = row.get(7)?;
                let setup_for: Option<String> = row.get(8)?;
                let payoff_of: Option<String> = row.get(9)?;
                Ok((
                    title,
                    chapter_title,
                    status,
                    word_count,
                    pov,
                    tension,
                    has_conflict,
                    has_change,
                    setup_for,
                    payoff_of,
                ))
            })
            .map_err(|e| e.to_string())?;

        for row in rows {
            let (
                title,
                chapter_title,
                status,
                word_count,
                pov,
                tension,
                has_conflict,
                has_change,
                setup_for,
                payoff_of,
            ) = row.map_err(|e| e.to_string())?;

            // Resolve setup_for and payoff_of scene IDs to titles
            let setup_for_title = self.resolve_scene_title(&setup_for);
            let payoff_of_title = self.resolve_scene_title(&payoff_of);

            output.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                Self::csv_escape(&title),
                Self::csv_escape(&chapter_title),
                Self::csv_escape(&status),
                word_count,
                Self::csv_opt(&pov),
                Self::csv_opt(&tension),
                Self::format_bool_csv(has_conflict),
                Self::format_bool_csv(has_change),
                Self::csv_opt(&setup_for_title),
                Self::csv_opt(&payoff_of_title),
            ));
        }

        Ok(output)
    }

    /// Resolve an optional scene ID to a scene title.
    fn resolve_scene_title(&self, scene_id: &Option<String>) -> Option<String> {
        scene_id.as_ref().and_then(|id| {
            self.conn
                .query_row(
                    "SELECT title FROM scenes WHERE id = ?1 AND deleted_at IS NULL",
                    params![id],
                    |row| row.get(0),
                )
                .ok()
        })
    }

    /// Format an optional bool for CSV output.
    fn format_bool_csv(value: Option<bool>) -> String {
        match value {
            Some(true) => "true".to_string(),
            Some(false) => "false".to_string(),
            None => String::new(),
        }
    }

    /// Export word count stats as CSV.
    ///
    /// Two sections: by_chapter and by_status, separated by a blank line.
    pub fn export_stats_csv(&self) -> Result<String, String> {
        let mut output = String::from("section,label,word_count,scene_count\n");

        // By chapter
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.title,
                    COALESCE(SUM(s.word_count), 0) as wc,
                    COUNT(s.id) as sc
                 FROM chapters c
                 LEFT JOIN scenes s ON s.chapter_id = c.id AND s.deleted_at IS NULL
                 WHERE c.deleted_at IS NULL
                 GROUP BY c.id, c.title
                 ORDER BY c.position",
            )
            .map_err(|e| e.to_string())?;

        let chapters = stmt
            .query_map([], |row| {
                let title: String = row.get(0)?;
                let word_count: i32 = row.get(1)?;
                let scene_count: i32 = row.get(2)?;
                Ok((title, word_count, scene_count))
            })
            .map_err(|e| e.to_string())?;

        for row in chapters {
            let (title, word_count, scene_count) = row.map_err(|e| e.to_string())?;
            output.push_str(&format!(
                "by_chapter,{},{},{}\n",
                Self::csv_escape(&title),
                word_count,
                scene_count,
            ));
        }

        // By status
        let mut stmt = self
            .conn
            .prepare(
                "SELECT status,
                    COALESCE(SUM(word_count), 0) as wc,
                    COUNT(*) as sc
                 FROM scenes
                 WHERE deleted_at IS NULL
                 GROUP BY status",
            )
            .map_err(|e| e.to_string())?;

        let statuses = stmt
            .query_map([], |row| {
                let status: String = row.get(0)?;
                let word_count: i32 = row.get(1)?;
                let scene_count: i32 = row.get(2)?;
                Ok((status, word_count, scene_count))
            })
            .map_err(|e| e.to_string())?;

        for row in statuses {
            let (status, word_count, scene_count) = row.map_err(|e| e.to_string())?;
            output.push_str(&format!(
                "by_status,{},{},{}\n",
                Self::csv_escape(&status),
                word_count,
                scene_count,
            ));
        }

        Ok(output)
    }
}
