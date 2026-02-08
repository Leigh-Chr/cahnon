use crate::database::Database;
use crate::models::{
    CreateChapterRequest, CreateSceneRequest, ImportResult, Scene, UpdateSceneRequest,
};
use crate::validation::{sanitize_text, MAX_SCENE_TITLE_LENGTH};
use crate::AppState;
use tauri::State;

/// Escape HTML special characters to prevent XSS
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Import markdown content as a new scene in the specified chapter
#[tauri::command]
pub fn import_markdown_as_scene(
    state: State<AppState>,
    chapter_id: String,
    title: String,
    content: String,
) -> Result<Scene, String> {
    let title = sanitize_text(&title, MAX_SCENE_TITLE_LENGTH);
    if title.is_empty() {
        return Err("Scene title cannot be empty".to_string());
    }

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;

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
                let title = sanitize_text(sc_title, MAX_SCENE_TITLE_LENGTH);
                let title = if title.is_empty() {
                    "Imported Scene".to_string()
                } else {
                    title
                };
                let create_req = CreateSceneRequest {
                    chapter_id: ch_id.clone(),
                    title,
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
        let title = sanitize_text(title, MAX_SCENE_TITLE_LENGTH);
        let title = if title.is_empty() {
            "Imported Chapter".to_string()
        } else {
            title
        };
        let create_req = CreateChapterRequest {
            title,
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
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;

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
    let title = sanitize_text(&title, MAX_SCENE_TITLE_LENGTH);
    if title.is_empty() {
        return Err("Scene title cannot be empty".to_string());
    }

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;

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
            let escaped = escape_html(content);
            self.html.push_str(&format!("<h3>{}</h3>", escaped));
        } else if let Some(content) = trimmed.strip_prefix("> ") {
            self.close_paragraph();
            let escaped = escape_html(content);
            self.html
                .push_str(&format!("<blockquote>{}</blockquote>", escaped));
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
/// Escapes HTML in text content before wrapping in tags.
fn process_inline_markdown(text: &str) -> String {
    // First escape HTML entities in the raw text
    let escaped = escape_html(text);
    let mut result = escaped;

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

    // Italic: *text* -> <em>text</em> (bold ** already converted to <strong>)
    while let Some(start) = result.find('*') {
        // Skip unclosed ** remnants (bold already processed)
        if result[start..].starts_with("**") {
            break;
        }
        if let Some(end) = result[start + 1..].find('*') {
            let before = &result[..start];
            let content = &result[start + 1..start + 1 + end];
            let after = &result[start + 1 + end + 1..];
            result = format!("{}<em>{}</em>{}", before, content, after);
        } else {
            break;
        }
    }

    result
}

/// Convert plain text to HTML paragraphs
fn text_to_html(text: &str) -> String {
    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    paragraphs
        .iter()
        .filter(|p| !p.trim().is_empty())
        .map(|p| {
            let escaped = escape_html(p.trim()).replace('\n', " ");
            format!("<p>{}</p>", escaped)
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Import a JSON backup to restore project data.
/// Creates an automatic backup before importing.
#[tauri::command]
pub fn import_json_backup(state: State<AppState>, content: String) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.import_json_backup(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== escape_html =====

    #[test]
    fn test_escape_html_ampersand() {
        assert_eq!(escape_html("A & B"), "A &amp; B");
    }

    #[test]
    fn test_escape_html_less_than() {
        assert_eq!(escape_html("a < b"), "a &lt; b");
    }

    #[test]
    fn test_escape_html_greater_than() {
        assert_eq!(escape_html("a > b"), "a &gt; b");
    }

    #[test]
    fn test_escape_html_double_quote() {
        assert_eq!(escape_html(r#"say "hello""#), "say &quot;hello&quot;");
    }

    #[test]
    fn test_escape_html_single_quote() {
        assert_eq!(escape_html("it's"), "it&#x27;s");
    }

    #[test]
    fn test_escape_html_all_entities() {
        assert_eq!(
            escape_html(r#"<a href="x">&'test'"#),
            "&lt;a href=&quot;x&quot;&gt;&amp;&#x27;test&#x27;"
        );
    }

    #[test]
    fn test_escape_html_no_special_chars() {
        assert_eq!(escape_html("Hello world"), "Hello world");
    }

    #[test]
    fn test_escape_html_empty() {
        assert_eq!(escape_html(""), "");
    }

    // ===== classify_line =====

    #[test]
    fn test_classify_line_chapter() {
        match classify_line("# My Chapter") {
            LineType::Chapter(title) => assert_eq!(title, "My Chapter"),
            _ => panic!("Expected Chapter"),
        }
    }

    #[test]
    fn test_classify_line_scene() {
        match classify_line("## Scene One") {
            LineType::Scene(title) => assert_eq!(title, "Scene One"),
            _ => panic!("Expected Scene"),
        }
    }

    #[test]
    fn test_classify_line_content() {
        match classify_line("Just some text") {
            LineType::Content => {}
            _ => panic!("Expected Content"),
        }
    }

    #[test]
    fn test_classify_line_empty() {
        match classify_line("") {
            LineType::Empty => {}
            _ => panic!("Expected Empty"),
        }
    }

    #[test]
    fn test_classify_line_hash_without_space_is_content() {
        // "#noSpace" should NOT be treated as a chapter heading
        match classify_line("#noSpace") {
            LineType::Content => {}
            _ => panic!("Expected Content for '#noSpace'"),
        }
    }

    #[test]
    fn test_classify_line_triple_hash_is_content() {
        // "### Subheading" is not chapter or scene
        match classify_line("### Subheading") {
            LineType::Content => {}
            _ => panic!("Expected Content for '### Subheading'"),
        }
    }

    #[test]
    fn test_classify_line_scene_trims_title() {
        match classify_line("## Title with spaces  ") {
            LineType::Scene(title) => assert_eq!(title, "Title with spaces"),
            _ => panic!("Expected Scene"),
        }
    }

    // ===== process_inline_markdown =====

    #[test]
    fn test_inline_markdown_bold() {
        let result = process_inline_markdown("Hello **world**");
        assert_eq!(result, "Hello <strong>world</strong>");
    }

    #[test]
    fn test_inline_markdown_italic() {
        let result = process_inline_markdown("Hello *world*");
        assert_eq!(result, "Hello <em>world</em>");
    }

    #[test]
    fn test_inline_markdown_bold_and_italic() {
        let result = process_inline_markdown("**bold** and *italic*");
        assert_eq!(result, "<strong>bold</strong> and <em>italic</em>");
    }

    #[test]
    fn test_inline_markdown_no_formatting() {
        let result = process_inline_markdown("Plain text here");
        assert_eq!(result, "Plain text here");
    }

    #[test]
    fn test_inline_markdown_unclosed_bold() {
        // Unclosed ** should be left as-is (after HTML escaping)
        let result = process_inline_markdown("Hello **world");
        assert!(result.contains("**"));
    }

    #[test]
    fn test_inline_markdown_escapes_html_first() {
        let result = process_inline_markdown("x < y & z > w");
        assert!(result.contains("&lt;"));
        assert!(result.contains("&amp;"));
        assert!(result.contains("&gt;"));
    }

    // ===== markdown_to_html =====

    #[test]
    fn test_markdown_to_html_paragraphs() {
        let result = markdown_to_html("Hello world");
        assert_eq!(result, "<p>Hello world</p>");
    }

    #[test]
    fn test_markdown_to_html_two_paragraphs() {
        let result = markdown_to_html("First paragraph\n\nSecond paragraph");
        assert_eq!(result, "<p>First paragraph</p><p>Second paragraph</p>");
    }

    #[test]
    fn test_markdown_to_html_h3() {
        let result = markdown_to_html("### My Heading");
        assert_eq!(result, "<h3>My Heading</h3>");
    }

    #[test]
    fn test_markdown_to_html_blockquote() {
        let result = markdown_to_html("> A wise quote");
        assert_eq!(result, "<blockquote>A wise quote</blockquote>");
    }

    #[test]
    fn test_markdown_to_html_mixed_content() {
        let result = markdown_to_html("### Title\n\nSome text.\n\n> A quote");
        assert_eq!(
            result,
            "<h3>Title</h3><p>Some text.</p><blockquote>A quote</blockquote>"
        );
    }

    #[test]
    fn test_markdown_to_html_bold_in_paragraph() {
        let result = markdown_to_html("She was **brave**.");
        assert_eq!(result, "<p>She was <strong>brave</strong>.</p>");
    }

    #[test]
    fn test_markdown_to_html_multiline_paragraph() {
        // Lines without blank line between them join into one paragraph
        let result = markdown_to_html("Line one\nLine two");
        assert_eq!(result, "<p>Line one Line two</p>");
    }

    #[test]
    fn test_markdown_to_html_empty() {
        assert_eq!(markdown_to_html(""), "");
    }

    #[test]
    fn test_markdown_to_html_only_blank_lines() {
        assert_eq!(markdown_to_html("\n\n\n"), "");
    }

    #[test]
    fn test_markdown_to_html_h3_escapes_html() {
        let result = markdown_to_html("### <script>alert('xss')</script>");
        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>"));
    }

    // ===== text_to_html =====

    #[test]
    fn test_text_to_html_single_paragraph() {
        assert_eq!(text_to_html("Hello world"), "<p>Hello world</p>");
    }

    #[test]
    fn test_text_to_html_two_paragraphs() {
        assert_eq!(
            text_to_html("First para\n\nSecond para"),
            "<p>First para</p><p>Second para</p>"
        );
    }

    #[test]
    fn test_text_to_html_empty_paragraphs_filtered() {
        assert_eq!(
            text_to_html("Text\n\n\n\nMore text"),
            "<p>Text</p><p>More text</p>"
        );
    }

    #[test]
    fn test_text_to_html_escapes_html() {
        let result = text_to_html("<b>bold</b>");
        assert_eq!(result, "<p>&lt;b&gt;bold&lt;/b&gt;</p>");
    }

    #[test]
    fn test_text_to_html_newlines_within_paragraph_become_spaces() {
        let result = text_to_html("line one\nline two");
        assert_eq!(result, "<p>line one line two</p>");
    }

    #[test]
    fn test_text_to_html_empty() {
        assert_eq!(text_to_html(""), "");
    }

    #[test]
    fn test_text_to_html_whitespace_only() {
        assert_eq!(text_to_html("   \n\n   "), "");
    }
}
