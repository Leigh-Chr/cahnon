use crate::database::Database;
use crate::models::{CreateChapterRequest, CreateSceneRequest, Scene, UpdateSceneRequest};
use crate::AppState;
use tauri::State;

/// Import markdown content as a new scene in the specified chapter
#[tauri::command]
pub fn import_markdown_as_scene(
    state: State<AppState>,
    chapter_id: String,
    title: String,
    content: String,
) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;

    // Convert markdown to HTML for the editor
    let html = markdown_to_html(&content);

    let create_req = CreateSceneRequest {
        chapter_id,
        title,
        summary: None,
        position: None,
    };

    db.create_scene(&create_req).and_then(|scene| {
        let update_req = UpdateSceneRequest {
            text: Some(html),
            ..Default::default()
        };
        db.update_scene(&scene.id, &update_req)
    })
}

/// State for tracking import progress
struct ImportState {
    chapters_created: i32,
    scenes_created: i32,
    current_chapter_id: Option<String>,
    current_scene_content: String,
    current_scene_title: Option<String>,
}

impl ImportState {
    fn new() -> Self {
        Self {
            chapters_created: 0,
            scenes_created: 0,
            current_chapter_id: None,
            current_scene_content: String::new(),
            current_scene_title: None,
        }
    }

    /// Save the current scene if there is content
    fn save_current_scene(&mut self, db: &Database) -> Result<(), String> {
        if let (Some(ch_id), Some(sc_title)) = (&self.current_chapter_id, &self.current_scene_title)
        {
            if !self.current_scene_content.trim().is_empty() {
                let html = markdown_to_html(&self.current_scene_content);
                let create_req = CreateSceneRequest {
                    chapter_id: ch_id.clone(),
                    title: sc_title.clone(),
                    summary: None,
                    position: None,
                };
                let scene = db.create_scene(&create_req)?;
                let update_req = UpdateSceneRequest {
                    text: Some(html),
                    ..Default::default()
                };
                db.update_scene(&scene.id, &update_req)?;
                self.scenes_created += 1;
            }
        }
        Ok(())
    }

    /// Create a new chapter and update state
    fn create_chapter(&mut self, db: &Database, title: &str) -> Result<(), String> {
        let create_req = CreateChapterRequest {
            title: title.to_string(),
            summary: None,
            position: None,
        };
        let chapter = db.create_chapter(&create_req)?;
        self.current_chapter_id = Some(chapter.id);
        self.chapters_created += 1;
        Ok(())
    }

    /// Ensure a default chapter exists
    fn ensure_chapter(&mut self, db: &Database) -> Result<(), String> {
        if self.current_chapter_id.is_none() {
            self.create_chapter(db, "Imported Chapter")?;
        }
        Ok(())
    }

    /// Start a new scene
    fn start_scene(&mut self, title: String) {
        self.current_scene_title = Some(title);
        self.current_scene_content.clear();
    }

    /// Add content to current scene
    fn add_content(&mut self, line: &str) {
        self.current_scene_content.push_str(line);
        self.current_scene_content.push('\n');
    }
}

/// Import markdown content with chapter/scene detection
/// Uses # for chapters and ## for scenes
#[tauri::command]
pub fn import_markdown_structured(
    state: State<AppState>,
    content: String,
) -> Result<ImportResult, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;

    let mut import_state = ImportState::new();

    for line in content.lines() {
        let line = line.trim();
        process_line(db, &mut import_state, line)?;
    }

    // Save the last scene
    import_state.save_current_scene(db)?;

    Ok(ImportResult {
        chapters_created: import_state.chapters_created,
        scenes_created: import_state.scenes_created,
    })
}

/// Process a single line during import
fn process_line(db: &Database, state: &mut ImportState, line: &str) -> Result<(), String> {
    match classify_line(line) {
        LineType::Chapter(title) => {
            state.save_current_scene(db)?;
            state.create_chapter(db, title)?;
            state.current_scene_title = None;
            state.current_scene_content.clear();
        }
        LineType::Scene(title) => {
            state.save_current_scene(db)?;
            state.ensure_chapter(db)?;
            state.start_scene(title.to_string());
        }
        LineType::Content => {
            state.ensure_chapter(db)?;
            if state.current_scene_title.is_none() {
                state.start_scene("Imported Scene".to_string());
            }
            state.add_content(line);
        }
        LineType::Empty => {}
    }
    Ok(())
}

enum LineType<'a> {
    Chapter(&'a str),
    Scene(&'a str),
    Content,
    Empty,
}

fn classify_line(line: &str) -> LineType<'_> {
    if line.is_empty() {
        LineType::Empty
    } else if let Some(title) = line.strip_prefix("## ") {
        LineType::Scene(title.trim())
    } else if let Some(title) = line.strip_prefix("# ") {
        LineType::Chapter(title.trim())
    } else {
        LineType::Content
    }
}

/// Import plain text as a single scene
#[tauri::command]
pub fn import_text_as_scene(
    state: State<AppState>,
    chapter_id: String,
    title: String,
    content: String,
) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;

    // Wrap paragraphs in <p> tags
    let html = text_to_html(&content);

    let create_req = CreateSceneRequest {
        chapter_id,
        title,
        summary: None,
        position: None,
    };

    db.create_scene(&create_req).and_then(|scene| {
        let update_req = UpdateSceneRequest {
            text: Some(html),
            ..Default::default()
        };
        db.update_scene(&scene.id, &update_req)
    })
}

#[derive(serde::Serialize)]
pub struct ImportResult {
    pub chapters_created: i32,
    pub scenes_created: i32,
}

/// Simple markdown to HTML converter
fn markdown_to_html(markdown: &str) -> String {
    let mut state = MarkdownState::new();

    for line in markdown.lines() {
        state.process_line(line.trim());
    }

    state.finalize()
}

struct MarkdownState {
    html: String,
    in_paragraph: bool,
}

impl MarkdownState {
    fn new() -> Self {
        Self {
            html: String::new(),
            in_paragraph: false,
        }
    }

    fn process_line(&mut self, trimmed: &str) {
        if trimmed.is_empty() {
            self.close_paragraph();
            return;
        }

        if let Some(content) = trimmed.strip_prefix("### ") {
            self.close_paragraph();
            self.html.push_str(&format!("<h3>{}</h3>", content));
        } else if let Some(content) = trimmed.strip_prefix("> ") {
            self.close_paragraph();
            self.html
                .push_str(&format!("<blockquote>{}</blockquote>", content));
        } else {
            self.append_paragraph_content(trimmed);
        }
    }

    fn close_paragraph(&mut self) {
        if self.in_paragraph {
            self.html.push_str("</p>");
            self.in_paragraph = false;
        }
    }

    fn append_paragraph_content(&mut self, text: &str) {
        let processed = process_inline_markdown(text);
        if !self.in_paragraph {
            self.html.push_str("<p>");
            self.in_paragraph = true;
        } else {
            self.html.push(' ');
        }
        self.html.push_str(&processed);
    }

    fn finalize(mut self) -> String {
        self.close_paragraph();
        self.html
    }
}

/// Process inline markdown (bold, italic)
fn process_inline_markdown(text: &str) -> String {
    let mut result = text.to_string();

    // Bold: **text** -> <strong>text</strong>
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let before = &result[..start];
            let content = &result[start + 2..start + 2 + end];
            let after = &result[start + 2 + end + 2..];
            result = format!("{}<strong>{}</strong>{}", before, content, after);
        } else {
            break;
        }
    }

    // Italic: *text* -> <em>text</em> (but not **)
    let mut i = 0;
    let chars: Vec<char> = result.chars().collect();
    let mut new_result = String::new();

    while i < chars.len() {
        if chars[i] == '*' && (i + 1 >= chars.len() || chars[i + 1] != '*') {
            // Find closing *
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '*') {
                let content: String = chars[i + 1..i + 1 + end].iter().collect();
                new_result.push_str(&format!("<em>{}</em>", content));
                i = i + 1 + end + 1;
                continue;
            }
        }
        new_result.push(chars[i]);
        i += 1;
    }

    new_result
}

/// Convert plain text to HTML paragraphs
fn text_to_html(text: &str) -> String {
    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    paragraphs
        .iter()
        .filter(|p| !p.trim().is_empty())
        .map(|p| format!("<p>{}</p>", p.trim().replace('\n', " ")))
        .collect::<Vec<_>>()
        .join("")
}
