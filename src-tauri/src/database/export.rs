//! Export operations (markdown, outline, bible, timeline)

use super::{Database, HTML_TAG_REGEX};
use crate::models::{BibleEntry, Event, Scene};

impl Database {
    pub fn export_markdown(&self) -> Result<String, String> {
        let project = self.get_project()?;
        let chapters = self.get_chapters()?;

        let mut output = format!("# {}\n\n", project.title);
        Self::append_optional(&mut output, &project.author, "**By {}**\n\n");
        Self::append_optional(&mut output, &project.description, "{}\n\n");
        output.push_str("---\n\n");

        for chapter in &chapters {
            output.push_str(&format!("## {}\n\n", chapter.title));
            Self::append_optional(&mut output, &chapter.summary, "*{}*\n\n");

            let scenes = self.get_scenes(&chapter.id)?;
            for scene in &scenes {
                output.push_str(&format!("### {}\n\n", scene.title));
                let plain_text = Self::html_to_markdown(&scene.text);
                output.push_str(&format!("{}\n\n", plain_text.trim()));
            }
        }

        Ok(output)
    }

    fn append_optional(output: &mut String, value: &Option<String>, format_str: &str) {
        if let Some(v) = value {
            output.push_str(&format_str.replace("{}", v));
        }
    }

    fn html_to_markdown(html: &str) -> String {
        let text = html
            .replace("<p>", "")
            .replace("</p>", "\n\n")
            .replace("<br>", "\n")
            .replace("<br/>", "\n")
            .replace("<strong>", "**")
            .replace("</strong>", "**")
            .replace("<em>", "*")
            .replace("</em>", "*");
        HTML_TAG_REGEX.replace_all(&text, "").to_string()
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
        let project = self.get_project()?;
        let chapters = self.get_chapters()?;

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
                output.push_str("* * *\n\n");
            }
        }

        Ok(output)
    }

    pub fn export_json_backup(&self) -> Result<String, String> {
        let project = self.get_project()?;
        let chapters = self.get_chapters()?;
        let mut all_scenes = Vec::new();
        for chapter in &chapters {
            let scenes = self.get_scenes(&chapter.id)?;
            all_scenes.extend(scenes);
        }
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
        output.push_str(&format!("{}. **{}** [{}]\n", index, scene.title, scene.status));
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

    fn format_entry_relationships(&self, output: &mut String, entry_id: &str) -> Result<(), String> {
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
        let time_str = Self::format_time_range(&event.time_point, &event.time_start, &event.time_end);
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
