//! Global search and word counts

use crate::models::{ChapterWordCount, SearchResult, StatusWordCount, WordCounts};
use rusqlite::params;

use super::Database;

impl Database {
    pub fn global_search(
        &self,
        query: &str,
        scope: Option<Vec<String>>,
    ) -> Result<Vec<SearchResult>, String> {
        let mut results = Vec::new();
        let scope = scope.unwrap_or_else(|| vec!["scenes".to_string(), "bible".to_string()]);

        if scope.contains(&"scenes".to_string()) {
            results.extend(self.search_scenes(query)?);
        }

        if scope.contains(&"bible".to_string()) {
            results.extend(self.search_bible_entries(query)?);
        }

        Ok(results)
    }

    fn search_scenes(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.title, snippet(scenes_fts, 2, '<mark>', '</mark>', '...', 32) as snippet, s.chapter_id, c.title
             FROM scenes s
             JOIN scenes_fts ON s.rowid = scenes_fts.rowid
             LEFT JOIN chapters c ON s.chapter_id = c.id
             WHERE scenes_fts MATCH ?1 AND s.deleted_at IS NULL
             LIMIT 50",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![query], Self::map_scene_search_result)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(results)
    }

    fn map_scene_search_result(row: &rusqlite::Row) -> rusqlite::Result<SearchResult> {
        Ok(SearchResult {
            result_type: "scene".to_string(),
            id: row.get(0)?,
            title: row.get(1)?,
            snippet: row.get(2)?,
            parent_id: row.get(3)?,
            parent_title: row.get(4)?,
        })
    }

    fn search_bible_entries(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT b.id, b.name, snippet(bible_fts, 2, '<mark>', '</mark>', '...', 32) as snippet, b.entry_type
             FROM bible_entries b
             JOIN bible_fts ON b.rowid = bible_fts.rowid
             WHERE bible_fts MATCH ?1 AND b.deleted_at IS NULL
             LIMIT 50",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![query], |row| {
                Ok(SearchResult {
                    result_type: "bible_entry".to_string(),
                    id: row.get(0)?,
                    title: row.get(1)?,
                    snippet: row.get(2)?,
                    parent_id: None,
                    parent_title: row.get::<_, Option<String>>(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(results)
    }

    pub fn get_word_counts(&self) -> Result<WordCounts, String> {
        Ok(WordCounts {
            total: self.get_total_word_count()?,
            by_chapter: self.get_word_counts_by_chapter()?,
            by_status: self.get_word_counts_by_status()?,
        })
    }

    fn get_total_word_count(&self) -> Result<i32, String> {
        Ok(self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(LENGTH(text) - LENGTH(REPLACE(text, ' ', '')) + 1), 0)
             FROM scenes WHERE deleted_at IS NULL AND text != ''",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0))
    }

    fn get_word_counts_by_chapter(&self) -> Result<Vec<ChapterWordCount>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.id, c.title,
                    COALESCE(SUM(LENGTH(s.text) - LENGTH(REPLACE(s.text, ' ', '')) + 1), 0) as word_count,
                    COUNT(s.id) as scene_count
             FROM chapters c
             LEFT JOIN scenes s ON c.id = s.chapter_id AND s.deleted_at IS NULL AND s.text != ''
             WHERE c.deleted_at IS NULL
             GROUP BY c.id
             ORDER BY c.position",
            )
            .map_err(|e| e.to_string())?;

        let by_chapter = stmt
            .query_map([], Self::map_chapter_word_count)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(by_chapter)
    }

    fn map_chapter_word_count(row: &rusqlite::Row) -> rusqlite::Result<ChapterWordCount> {
        Ok(ChapterWordCount {
            chapter_id: row.get(0)?,
            chapter_title: row.get(1)?,
            word_count: row.get(2)?,
            scene_count: row.get(3)?,
        })
    }

    fn get_word_counts_by_status(&self) -> Result<Vec<StatusWordCount>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT status,
                    COALESCE(SUM(LENGTH(text) - LENGTH(REPLACE(text, ' ', '')) + 1), 0) as word_count,
                    COUNT(id) as scene_count
             FROM scenes WHERE deleted_at IS NULL
             GROUP BY status",
            )
            .map_err(|e| e.to_string())?;

        let by_status = stmt
            .query_map([], Self::map_status_word_count)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(by_status)
    }

    fn map_status_word_count(row: &rusqlite::Row) -> rusqlite::Result<StatusWordCount> {
        Ok(StatusWordCount {
            status: row.get(0)?,
            word_count: row.get(1)?,
            scene_count: row.get(2)?,
        })
    }
}
