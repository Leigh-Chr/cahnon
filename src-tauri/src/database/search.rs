//! Global search and word counts

use crate::models::{ChapterWordCount, SearchResult, StatusWordCount, WordCounts};
use rusqlite::params;

use super::Database;

/// Safely truncate a string at a char boundary, appending "..." if truncated.
fn safe_truncate(s: &str, max_chars: usize) -> String {
    let mut char_indices = s.char_indices();
    if let Some((byte_pos, _)) = char_indices.nth(max_chars) {
        format!("{}...", &s[..byte_pos])
    } else {
        s.to_string()
    }
}

impl Database {
    /// Escapes FTS5 special characters by wrapping each token in double quotes.
    /// This prevents FTS5 query syntax injection (e.g., `OR`, `NOT`, `NEAR`, `*`).
    pub(crate) fn sanitize_fts5_query(query: &str) -> String {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        // Wrap each whitespace-separated token in double quotes,
        // escaping any embedded double quotes by doubling them.
        trimmed
            .split_whitespace()
            .map(|token| {
                let escaped = token.replace('"', "\"\"");
                format!("\"{}\"", escaped)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn global_search(
        &self,
        query: &str,
        scope: Option<Vec<String>>,
    ) -> Result<Vec<SearchResult>, String> {
        let sanitized = Self::sanitize_fts5_query(query);
        if sanitized.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let scope = scope.unwrap_or_else(|| {
            vec![
                "scenes".to_string(),
                "bible".to_string(),
                "events".to_string(),
                "annotations".to_string(),
                "cuts".to_string(),
            ]
        });

        if scope.contains(&"scenes".to_string()) {
            results.extend(self.search_scenes(&sanitized)?);
        }

        if scope.contains(&"bible".to_string()) {
            results.extend(self.search_bible_entries(&sanitized)?);
        }

        if scope.contains(&"events".to_string()) {
            results.extend(self.search_events(query)?);
        }

        if scope.contains(&"annotations".to_string()) {
            results.extend(self.search_annotations(query)?);
        }

        if scope.contains(&"cuts".to_string()) {
            results.extend(self.search_cuts(query)?);
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

    fn search_events(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, title, description
             FROM events
             WHERE deleted_at IS NULL AND (title LIKE ?1 ESCAPE '\\' OR description LIKE ?1 ESCAPE '\\')
             LIMIT 50",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![pattern], |row| {
                let desc: Option<String> = row.get(2)?;
                Ok(SearchResult {
                    result_type: "event".to_string(),
                    id: row.get(0)?,
                    title: row.get(1)?,
                    snippet: desc,
                    parent_id: None,
                    parent_title: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(results)
    }

    fn search_annotations(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = self
            .conn
            .prepare(
                "SELECT a.id, a.content, a.scene_id, s.title
             FROM annotations a
             LEFT JOIN scenes s ON a.scene_id = s.id
             WHERE a.content LIKE ?1 ESCAPE '\\'
             LIMIT 50",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![pattern], |row| {
                let content: String = row.get(1)?;
                let snippet = safe_truncate(&content, 100);
                Ok(SearchResult {
                    result_type: "annotation".to_string(),
                    id: row.get(0)?,
                    title: content,
                    snippet: Some(snippet),
                    parent_id: row.get(2)?,
                    parent_title: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(results)
    }

    fn search_cuts(&self, query: &str) -> Result<Vec<SearchResult>, String> {
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, text, scene_id
             FROM cuts
             WHERE deleted_at IS NULL AND text LIKE ?1 ESCAPE '\\'
             LIMIT 50",
            )
            .map_err(|e| e.to_string())?;

        let results = stmt
            .query_map(params![pattern], |row| {
                let text: String = row.get(1)?;
                let snippet = safe_truncate(&text, 100);
                let title = safe_truncate(&text, 50);
                Ok(SearchResult {
                    result_type: "cut".to_string(),
                    id: row.get(0)?,
                    title,
                    snippet: Some(snippet),
                    parent_id: row.get(2)?,
                    parent_title: None,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(results)
    }

    /// Find and replace text across multiple scenes.
    /// Returns the number of scenes modified.
    pub fn find_replace_in_scenes(
        &self,
        find: &str,
        replace: &str,
        case_sensitive: bool,
        whole_word: bool,
        chapter_id: Option<&str>,
    ) -> Result<i32, String> {
        use regex::RegexBuilder;

        if find.is_empty() {
            return Ok(0);
        }

        let escaped = regex::escape(find);
        let pattern = if whole_word {
            format!(r"\b{}\b", escaped)
        } else {
            escaped
        };

        let re = RegexBuilder::new(&pattern)
            .case_insensitive(!case_sensitive)
            .build()
            .map_err(|e| format!("Invalid search pattern: {}", e))?;

        // Fetch scenes to process
        let scenes: Vec<(String, String)> = if let Some(cid) = chapter_id {
            let mut stmt = self
                .conn
                .prepare("SELECT id, text FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL")
                .map_err(|e| e.to_string())?;
            let result = stmt
                .query_map(params![cid], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            result
        } else {
            let mut stmt = self
                .conn
                .prepare("SELECT id, text FROM scenes WHERE deleted_at IS NULL")
                .map_err(|e| e.to_string())?;
            let result = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;
            result
        };

        // We need to replace within text nodes only, not HTML tags.
        // Extract text content, find positions, and replace.
        self.conn.execute("BEGIN", []).map_err(|e| e.to_string())?;

        let result = (|| -> Result<i32, String> {
            let mut count = 0;
            for (scene_id, html) in &scenes {
                // Only replace within text content (between tags), not in tag attributes
                let new_html = Self::replace_in_html_text(&re, html, replace);
                if &new_html != html {
                    self.conn
                        .execute(
                            "UPDATE scenes SET text = ?1, updated_at = datetime('now') WHERE id = ?2",
                            params![new_html, scene_id],
                        )
                        .map_err(|e| e.to_string())?;
                    count += 1;
                }
            }
            Ok(count)
        })();

        match result {
            Ok(count) => {
                self.conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
                Ok(count)
            }
            Err(e) => {
                let _ = self.conn.execute("ROLLBACK", []);
                Err(e)
            }
        }
    }

    /// Replace regex matches only within HTML text nodes, preserving tags.
    fn replace_in_html_text(re: &regex::Regex, html: &str, replacement: &str) -> String {
        use super::HTML_TAG_REGEX;

        let mut result = String::with_capacity(html.len());
        let mut last_end = 0;

        for tag_match in HTML_TAG_REGEX.find_iter(html) {
            // Process text before this tag
            let text_segment = &html[last_end..tag_match.start()];
            if !text_segment.is_empty() {
                result.push_str(&re.replace_all(text_segment, replacement));
            }
            // Append the tag unchanged
            result.push_str(tag_match.as_str());
            last_end = tag_match.end();
        }

        // Process remaining text after last tag
        let remaining = &html[last_end..];
        if !remaining.is_empty() {
            result.push_str(&re.replace_all(remaining, replacement));
        }

        result
    }

    pub fn get_word_counts(&self) -> Result<WordCounts, String> {
        Ok(WordCounts {
            total: self.get_total_word_count()?,
            by_chapter: self.get_word_counts_by_chapter()?,
            by_status: self.get_word_counts_by_status()?,
        })
    }

    fn get_total_word_count(&self) -> Result<i32, String> {
        // Count words by counting spaces in stripped text + 1
        // First strip HTML tags, then count spaces
        let mut stmt = self
            .conn
            .prepare("SELECT id, text FROM scenes WHERE deleted_at IS NULL AND text != ''")
            .map_err(|e| e.to_string())?;

        let total: i32 = stmt
            .query_map([], |row| {
                let text: String = row.get(1)?;
                Ok(text)
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .map(|text| {
                let plain = crate::database::HTML_TAG_REGEX.replace_all(&text, " ");
                plain.split_whitespace().count() as i32
            })
            .sum();

        Ok(total)
    }

    fn get_word_counts_by_chapter(&self) -> Result<Vec<ChapterWordCount>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT c.id, c.title, c.position
             FROM chapters c
             WHERE c.deleted_at IS NULL
             ORDER BY c.position",
            )
            .map_err(|e| e.to_string())?;

        let chapters: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut by_chapter = Vec::new();
        for (chapter_id, chapter_title) in chapters {
            let mut scene_stmt = self
                .conn
                .prepare(
                    "SELECT text FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL AND text != ''",
                )
                .map_err(|e| e.to_string())?;

            let texts: Vec<String> = scene_stmt
                .query_map(params![chapter_id], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            let scene_count = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM scenes WHERE chapter_id = ?1 AND deleted_at IS NULL",
                    params![chapter_id],
                    |row| row.get::<_, i32>(0),
                )
                .unwrap_or(0);

            let word_count: i32 = texts
                .iter()
                .map(|text| {
                    let plain = crate::database::HTML_TAG_REGEX.replace_all(text, " ");
                    plain.split_whitespace().count() as i32
                })
                .sum();

            by_chapter.push(ChapterWordCount {
                chapter_id,
                chapter_title,
                word_count,
                scene_count,
            });
        }

        Ok(by_chapter)
    }

    fn get_word_counts_by_status(&self) -> Result<Vec<StatusWordCount>, String> {
        // Get distinct statuses
        let mut status_stmt = self
            .conn
            .prepare("SELECT DISTINCT status FROM scenes WHERE deleted_at IS NULL")
            .map_err(|e| e.to_string())?;

        let statuses: Vec<String> = status_stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        let mut by_status = Vec::new();
        for status in statuses {
            let mut scene_stmt = self
                .conn
                .prepare(
                    "SELECT text FROM scenes WHERE status = ?1 AND deleted_at IS NULL AND text != ''",
                )
                .map_err(|e| e.to_string())?;

            let texts: Vec<String> = scene_stmt
                .query_map(params![status], |row| row.get(0))
                .map_err(|e| e.to_string())?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?;

            let scene_count = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM scenes WHERE status = ?1 AND deleted_at IS NULL",
                    params![status],
                    |row| row.get::<_, i32>(0),
                )
                .unwrap_or(0);

            let word_count: i32 = texts
                .iter()
                .map(|text| {
                    let plain = crate::database::HTML_TAG_REGEX.replace_all(text, " ");
                    plain.split_whitespace().count() as i32
                })
                .sum();

            by_status.push(StatusWordCount {
                status,
                word_count,
                scene_count,
            });
        }

        Ok(by_status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_truncate_short_string() {
        assert_eq!(safe_truncate("hello", 10), "hello");
    }

    #[test]
    fn test_safe_truncate_exact_length() {
        assert_eq!(safe_truncate("hello", 5), "hello");
    }

    #[test]
    fn test_safe_truncate_over_limit() {
        assert_eq!(safe_truncate("hello world", 5), "hello...");
    }

    #[test]
    fn test_safe_truncate_empty() {
        assert_eq!(safe_truncate("", 10), "");
    }

    #[test]
    fn test_safe_truncate_unicode() {
        // "café" is 4 chars — truncating at 3 should give "caf..."
        assert_eq!(safe_truncate("café", 3), "caf...");
    }

    #[test]
    fn test_safe_truncate_unicode_exact() {
        assert_eq!(safe_truncate("café", 4), "café");
    }

    #[test]
    fn test_safe_truncate_emoji() {
        // Emoji is 1 char — truncating at 1 should preserve it
        assert_eq!(safe_truncate("😊hello", 1), "😊...");
    }

    #[test]
    fn test_safe_truncate_zero_limit() {
        assert_eq!(safe_truncate("hello", 0), "...");
    }
}
