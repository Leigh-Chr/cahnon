//! Export operations (markdown, outline, bible, timeline)

use super::{Database, HTML_TAG_REGEX};
use crate::models::{BibleEntry, Event, Scene};

impl Database {
    pub fn export_markdown(&self) -> Result<String, String> {
        self.export_markdown_with_options(None, None, true)
    }

    pub fn export_markdown_with_options(
        &self,
        chapter_ids: Option<&[String]>,
        scene_separator: Option<&str>,
        include_titles: bool,
    ) -> Result<String, String> {
        let project = self.get_project()?;
        let chapters = self.filter_chapters(chapter_ids)?;
        let separator = scene_separator.unwrap_or("###");

        let mut output = format!("# {}\n\n", project.title);
        Self::append_optional(&mut output, &project.author, "**By {}**\n\n");
        Self::append_optional(&mut output, &project.description, "{}\n\n");
        output.push_str("---\n\n");

        for chapter in &chapters {
            self.format_chapter_markdown(&mut output, chapter, separator, include_titles)?;
        }

        Ok(output)
    }

    fn filter_chapters(
        &self,
        chapter_ids: Option<&[String]>,
    ) -> Result<Vec<crate::models::Chapter>, String> {
        let all_chapters = self.get_chapters()?;
        Ok(match chapter_ids {
            Some(ids) => all_chapters
                .into_iter()
                .filter(|c| ids.contains(&c.id))
                .collect(),
            None => all_chapters,
        })
    }

    fn format_chapter_markdown(
        &self,
        output: &mut String,
        chapter: &crate::models::Chapter,
        separator: &str,
        include_titles: bool,
    ) -> Result<(), String> {
        output.push_str(&format!("## {}\n\n", chapter.title));
        Self::append_optional(output, &chapter.summary, "*{}*\n\n");

        let scenes = self.get_scenes(&chapter.id)?;
        for (i, scene) in scenes.iter().enumerate() {
            if include_titles {
                output.push_str(&format!("{} {}\n\n", separator, scene.title));
            } else if i > 0 {
                output.push_str(&format!("{}\n\n", separator));
            }
            let plain_text = Self::html_to_markdown(&scene.text);
            output.push_str(&format!("{}\n\n", plain_text.trim()));
        }
        Ok(())
    }

    fn append_optional(output: &mut String, value: &Option<String>, format_str: &str) {
        if let Some(v) = value {
            output.push_str(&format_str.replace("{}", v));
        }
    }

    fn html_to_markdown(html: &str) -> String {
        let mut text = html.to_string();

        text = Self::replace_html_block_elements(&text);
        text = Self::replace_html_inline_elements(&text);

        // Process links: <a href="url">text</a> -> [text](url)
        text = Self::convert_links_to_markdown(&text);

        // Clean up remaining HTML tags
        let text = HTML_TAG_REGEX.replace_all(&text, "").to_string();

        Self::collapse_blank_lines(&text)
    }

    fn replace_html_block_elements(text: &str) -> String {
        text.replace("<h1>", "\n# ")
            .replace("</h1>", "\n")
            .replace("<h2>", "\n## ")
            .replace("</h2>", "\n")
            .replace("<h3>", "\n### ")
            .replace("</h3>", "\n")
            .replace("<blockquote>", "\n> ")
            .replace("</blockquote>", "\n")
            .replace("<p>", "")
            .replace("</p>", "\n\n")
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("<ul>", "\n")
            .replace("</ul>", "\n")
            .replace("<ol>", "\n")
            .replace("</ol>", "\n")
            .replace("<li>", "- ")
            .replace("</li>", "\n")
            .replace("<hr>", "\n---\n")
            .replace("<hr/>", "\n---\n")
    }

    fn replace_html_inline_elements(text: &str) -> String {
        text.replace("<strong>", "**")
            .replace("</strong>", "**")
            .replace("<b>", "**")
            .replace("</b>", "**")
            .replace("<em>", "*")
            .replace("</em>", "*")
            .replace("<i>", "*")
            .replace("</i>", "*")
            .replace("<code>", "`")
            .replace("</code>", "`")
            .replace("<mark>", "==")
            .replace("</mark>", "==")
    }

    fn collapse_blank_lines(text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut result = String::new();
        let mut prev_empty = false;
        for line in lines {
            let is_empty = line.trim().is_empty();
            if is_empty && prev_empty {
                continue;
            }
            result.push_str(line);
            result.push('\n');
            prev_empty = is_empty;
        }
        result.trim().to_string()
    }

    /// Convert HTML links to markdown format
    fn convert_links_to_markdown(html: &str) -> String {
        use super::LINK_REGEX;
        LINK_REGEX
            .replace_all(html, |caps: &regex::Captures| {
                format!("[{}]({})", &caps[2], &caps[1])
            })
            .to_string()
    }

    fn html_to_plain(html: &str) -> String {
        let text = html
            .replace("<p>", "")
            .replace("</p>", "\n\n")
            .replace("<br>", "\n")
            .replace("<br/>", "\n");
        HTML_TAG_REGEX.replace_all(&text, "").to_string()
    }

    pub fn export_plain_text(&self) -> Result<String, String> {
        self.export_plain_text_with_options(None, None)
    }

    pub fn export_plain_text_with_options(
        &self,
        chapter_ids: Option<&[String]>,
        scene_separator: Option<&str>,
    ) -> Result<String, String> {
        let project = self.get_project()?;
        let all_chapters = self.get_chapters()?;
        let chapters: Vec<_> = match chapter_ids {
            Some(ids) => all_chapters
                .into_iter()
                .filter(|c| ids.contains(&c.id))
                .collect(),
            None => all_chapters,
        };
        let separator = scene_separator.unwrap_or("* * *");

        let mut output = format!("{}\n", project.title);
        output.push_str(&"=".repeat(project.title.len()));
        output.push_str("\n\n");

        Self::append_optional(&mut output, &project.author, "By {}\n\n");

        for chapter in &chapters {
            output.push_str(&format!("\n{}\n", chapter.title));
            output.push_str(&"-".repeat(chapter.title.len()));
            output.push_str("\n\n");

            let scenes = self.get_scenes(&chapter.id)?;
            for scene in &scenes {
                let plain_text = Self::html_to_plain(&scene.text);
                output.push_str(&format!("{}\n\n", plain_text.trim()));
                output.push_str(&format!("{}\n\n", separator));
            }
        }

        Ok(output)
    }

    /// Loads all non-deleted scenes in a single query, ordered by chapter and position.
    fn get_all_scenes_for_export(&self) -> Result<Vec<Scene>, String> {
        let query = format!(
            "{} FROM scenes WHERE deleted_at IS NULL ORDER BY chapter_id, position",
            crate::database::scene::crud::SCENE_SELECT
        );
        let mut stmt = self.conn.prepare(&query).map_err(|e| e.to_string())?;
        let scenes = stmt
            .query_map([], Self::map_scene)
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(scenes)
    }

    pub fn export_json_backup(&self) -> Result<String, String> {
        let project = self.get_project()?;
        let chapters = self.get_chapters()?;
        let all_scenes = self.get_all_scenes_for_export()?;
        let bible_entries = self.get_bible_entries(None)?;
        let arcs = self.get_arcs()?;
        let events = self.get_events()?;

        let data = serde_json::json!({
            "version": "1.0",
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "project": project,
            "chapters": chapters,
            "scenes": all_scenes,
            "bible_entries": bible_entries,
            "arcs": arcs,
            "events": events,
        });

        serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
    }

    /// Export outline only (chapters and scene titles/summaries)
    pub fn export_outline(&self) -> Result<String, String> {
        let project = self.get_project()?;
        let chapters = self.get_chapters()?;

        let mut output = format!("# {} - Outline\n\n", project.title);

        for chapter in &chapters {
            output.push_str(&format!("## {} [{}]\n\n", chapter.title, chapter.status));
            Self::append_optional(&mut output, &chapter.summary, "*{}*\n\n");

            let scenes = self.get_scenes(&chapter.id)?;
            for (i, scene) in scenes.iter().enumerate() {
                Self::format_outline_scene(&mut output, i + 1, scene);
            }
            output.push('\n');
        }

        Ok(output)
    }

    fn format_outline_scene(output: &mut String, index: usize, scene: &Scene) {
        output.push_str(&format!(
            "{}. **{}** [{}]\n",
            index, scene.title, scene.status
        ));
        Self::append_optional(output, &scene.summary, "   *{}*\n");
        Self::append_optional(output, &scene.pov, "   POV: {}\n");
        output.push('\n');
    }

    /// Export bible entries only
    pub fn export_bible(&self) -> Result<String, String> {
        let project = self.get_project()?;
        let entries = self.get_bible_entries(None)?;

        let mut output = format!("# {} - Story Bible\n\n", project.title);

        let type_config = Self::bible_type_config();

        for (entry_type, label) in type_config {
            let typed_entries: Vec<_> = entries
                .iter()
                .filter(|e| e.entry_type == entry_type)
                .collect();
            if typed_entries.is_empty() {
                continue;
            }

            output.push_str(&format!("## {}\n\n", label));

            for entry in typed_entries {
                self.format_bible_entry(&mut output, entry)?;
            }
        }

        Ok(output)
    }

    fn bible_type_config() -> [(&'static str, &'static str); 6] {
        [
            ("character", "Characters"),
            ("location", "Locations"),
            ("object", "Objects"),
            ("faction", "Factions"),
            ("concept", "Concepts & Rules"),
            ("glossary", "Glossary"),
        ]
    }

    fn format_bible_entry(&self, output: &mut String, entry: &BibleEntry) -> Result<(), String> {
        output.push_str(&format!("### {} [{}]\n\n", entry.name, entry.status));

        Self::append_non_empty(output, &entry.aliases, "*Aliases: {}*\n\n");
        Self::append_optional(output, &entry.short_description, "{}\n\n");
        Self::append_optional(output, &entry.full_description, "{}\n\n");
        Self::append_non_empty(output, &entry.notes, "**Notes:** {}\n\n");

        self.format_entry_relationships(output, &entry.id)?;

        output.push_str("---\n\n");
        Ok(())
    }

    fn append_non_empty(output: &mut String, value: &Option<String>, format_str: &str) {
        if let Some(v) = value {
            if !v.is_empty() {
                output.push_str(&format_str.replace("{}", v));
            }
        }
    }

    fn format_entry_relationships(
        &self,
        output: &mut String,
        entry_id: &str,
    ) -> Result<(), String> {
        if let Ok(relationships) = self.get_bible_relationships(entry_id) {
            if !relationships.is_empty() {
                output.push_str("**Relationships:**\n");
                for rel in relationships {
                    output.push_str(&format!(
                        "- {} {}\n",
                        rel.relationship_type.replace('_', " "),
                        rel.related_entry_name
                    ));
                }
                output.push('\n');
            }
        }
        Ok(())
    }

    /// Export timeline only (events and scenes with time)
    pub fn export_timeline(&self) -> Result<String, String> {
        let project = self.get_project()?;
        let events = self.get_events()?;
        let timeline_scenes = self.get_all_scenes_for_timeline()?;

        let mut output = format!("# {} - Timeline\n\n", project.title);

        output.push_str("## Events\n\n");
        Self::format_events_by_type(&mut output, &events);

        output.push_str("## Scenes on Timeline\n\n");
        Self::format_timeline_scenes(&mut output, &timeline_scenes);

        Ok(output)
    }

    fn format_events_by_type(output: &mut String, events: &[Event]) {
        let event_types = [
            ("backstory", "Backstory"),
            ("historical", "Historical"),
            ("scene", "Story Events"),
        ];

        for (event_type, label) in event_types {
            let typed_events: Vec<_> = events
                .iter()
                .filter(|e| e.event_type == event_type)
                .collect();
            if typed_events.is_empty() {
                continue;
            }

            output.push_str(&format!("### {}\n\n", label));

            for event in typed_events {
                Self::format_event(output, event);
            }
            output.push('\n');
        }
    }

    fn format_event(output: &mut String, event: &Event) {
        let time_str =
            Self::format_time_range(&event.time_point, &event.time_start, &event.time_end);
        output.push_str(&format!("- **{}** ({})\n", event.title, time_str));
        Self::append_optional(output, &event.description, "  {}\n");
    }

    fn format_time_range(
        time_point: &Option<String>,
        time_start: &Option<String>,
        time_end: &Option<String>,
    ) -> String {
        if let Some(tp) = time_point {
            tp.clone()
        } else if let (Some(ts), Some(te)) = (time_start, time_end) {
            format!("{} - {}", ts, te)
        } else if let Some(ts) = time_start {
            format!("{} - ...", ts)
        } else {
            "No time set".to_string()
        }
    }

    fn format_timeline_scenes(output: &mut String, scenes: &[Scene]) {
        for scene in scenes {
            let time_str = Self::get_scene_time_str(scene);
            if let Some(time) = time_str {
                output.push_str(&format!("- **{}** ({})\n", scene.title, time));
                Self::append_optional(output, &scene.summary, "  *{}*\n");
            }
        }
    }

    fn get_scene_time_str(scene: &Scene) -> Option<String> {
        if let Some(tp) = &scene.time_point {
            Some(tp.clone())
        } else if let (Some(ts), Some(te)) = (&scene.time_start, &scene.time_end) {
            Some(format!("{} - {}", ts, te))
        } else {
            scene.time_start.as_ref().map(|ts| format!("{} - ...", ts))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- html_to_markdown ---

    #[test]
    fn test_html_to_markdown_plain_text() {
        assert_eq!(Database::html_to_markdown("Hello world"), "Hello world");
    }

    #[test]
    fn test_html_to_markdown_paragraphs() {
        let html = "<p>First paragraph</p><p>Second paragraph</p>";
        let md = Database::html_to_markdown(html);
        assert!(md.contains("First paragraph"));
        assert!(md.contains("Second paragraph"));
    }

    #[test]
    fn test_html_to_markdown_bold() {
        assert_eq!(
            Database::html_to_markdown("<strong>bold</strong>"),
            "**bold**"
        );
        assert_eq!(Database::html_to_markdown("<b>bold</b>"), "**bold**");
    }

    #[test]
    fn test_html_to_markdown_italic() {
        assert_eq!(Database::html_to_markdown("<em>italic</em>"), "*italic*");
        assert_eq!(Database::html_to_markdown("<i>italic</i>"), "*italic*");
    }

    #[test]
    fn test_html_to_markdown_headings() {
        let md = Database::html_to_markdown("<h1>Title</h1>");
        assert!(md.contains("# Title"));

        let md = Database::html_to_markdown("<h2>Subtitle</h2>");
        assert!(md.contains("## Subtitle"));

        let md = Database::html_to_markdown("<h3>Section</h3>");
        assert!(md.contains("### Section"));
    }

    #[test]
    fn test_html_to_markdown_blockquote() {
        let md = Database::html_to_markdown("<blockquote>A quote</blockquote>");
        assert!(md.contains("> A quote"));
    }

    #[test]
    fn test_html_to_markdown_code() {
        assert_eq!(Database::html_to_markdown("<code>foo()</code>"), "`foo()`");
    }

    #[test]
    fn test_html_to_markdown_mark() {
        assert_eq!(
            Database::html_to_markdown("<mark>highlight</mark>"),
            "==highlight=="
        );
    }

    #[test]
    fn test_html_to_markdown_line_breaks() {
        let md = Database::html_to_markdown("Line1<br>Line2");
        assert!(md.contains("Line1\nLine2"));

        let md = Database::html_to_markdown("Line1<br/>Line2");
        assert!(md.contains("Line1\nLine2"));
    }

    #[test]
    fn test_html_to_markdown_list() {
        let html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
        let md = Database::html_to_markdown(html);
        assert!(md.contains("- Item 1"));
        assert!(md.contains("- Item 2"));
    }

    #[test]
    fn test_html_to_markdown_horizontal_rule() {
        let md = Database::html_to_markdown("Before<hr>After");
        assert!(md.contains("---"));
    }

    #[test]
    fn test_html_to_markdown_strips_unknown_tags() {
        assert_eq!(Database::html_to_markdown("<div>Content</div>"), "Content");
        assert_eq!(
            Database::html_to_markdown("<span class=\"foo\">Text</span>"),
            "Text"
        );
    }

    #[test]
    fn test_html_to_markdown_empty() {
        assert_eq!(Database::html_to_markdown(""), "");
    }

    #[test]
    fn test_html_to_markdown_collapses_blank_lines() {
        let html = "<p>A</p><p></p><p></p><p>B</p>";
        let md = Database::html_to_markdown(html);
        // Should not have more than 2 consecutive newlines
        assert!(!md.contains("\n\n\n\n"));
    }

    // --- convert_links_to_markdown ---

    #[test]
    fn test_convert_links_to_markdown() {
        let html = r#"<a href="https://example.com">Click here</a>"#;
        let result = Database::convert_links_to_markdown(html);
        assert_eq!(result, "[Click here](https://example.com)");
    }

    #[test]
    fn test_convert_links_to_markdown_no_links() {
        let html = "Just plain text";
        assert_eq!(Database::convert_links_to_markdown(html), "Just plain text");
    }

    #[test]
    fn test_convert_links_to_markdown_multiple() {
        let html = r#"<a href="https://a.com">A</a> and <a href="https://b.com">B</a>"#;
        let result = Database::convert_links_to_markdown(html);
        assert!(result.contains("[A](https://a.com)"));
        assert!(result.contains("[B](https://b.com)"));
    }

    // --- html_to_plain ---

    #[test]
    fn test_html_to_plain_paragraphs() {
        let result = Database::html_to_plain("<p>Hello</p><p>World</p>");
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }

    #[test]
    fn test_html_to_plain_strips_all_tags() {
        let result = Database::html_to_plain("<strong>Bold</strong> and <em>italic</em>");
        assert_eq!(result, "Bold and italic");
    }

    #[test]
    fn test_html_to_plain_br_to_newline() {
        let result = Database::html_to_plain("Line1<br>Line2");
        assert!(result.contains("Line1\nLine2"));
    }

    // --- append_optional ---

    #[test]
    fn test_append_optional_some() {
        let mut output = String::new();
        Database::append_optional(&mut output, &Some("Alice".to_string()), "By {}\n");
        assert_eq!(output, "By Alice\n");
    }

    #[test]
    fn test_append_optional_none() {
        let mut output = String::new();
        Database::append_optional(&mut output, &None, "By {}\n");
        assert_eq!(output, "");
    }

    // --- append_non_empty ---

    #[test]
    fn test_append_non_empty_with_content() {
        let mut output = String::new();
        Database::append_non_empty(&mut output, &Some("aliases".to_string()), "*{}*\n");
        assert_eq!(output, "*aliases*\n");
    }

    #[test]
    fn test_append_non_empty_with_empty_string() {
        let mut output = String::new();
        Database::append_non_empty(&mut output, &Some("".to_string()), "*{}*\n");
        assert_eq!(output, ""); // Empty string → nothing appended
    }

    #[test]
    fn test_append_non_empty_none() {
        let mut output = String::new();
        Database::append_non_empty(&mut output, &None, "*{}*\n");
        assert_eq!(output, "");
    }

    // --- format_time_range ---

    #[test]
    fn test_format_time_range_point() {
        let result = Database::format_time_range(&Some("1200".to_string()), &None, &None);
        assert_eq!(result, "1200");
    }

    #[test]
    fn test_format_time_range_start_and_end() {
        let result = Database::format_time_range(
            &None,
            &Some("1000".to_string()),
            &Some("1200".to_string()),
        );
        assert_eq!(result, "1000 - 1200");
    }

    #[test]
    fn test_format_time_range_start_only() {
        let result = Database::format_time_range(&None, &Some("1000".to_string()), &None);
        assert_eq!(result, "1000 - ...");
    }

    #[test]
    fn test_format_time_range_nothing() {
        let result = Database::format_time_range(&None, &None, &None);
        assert_eq!(result, "No time set");
    }

    #[test]
    fn test_format_time_range_point_takes_priority() {
        // If time_point is set, it wins over start/end
        let result = Database::format_time_range(
            &Some("1200".to_string()),
            &Some("1000".to_string()),
            &Some("1100".to_string()),
        );
        assert_eq!(result, "1200");
    }

    // --- bible_type_config ---

    #[test]
    fn test_bible_type_config_has_six_types() {
        let config = Database::bible_type_config();
        assert_eq!(config.len(), 6);
        assert_eq!(config[0], ("character", "Characters"));
        assert_eq!(config[5], ("glossary", "Glossary"));
    }
}
