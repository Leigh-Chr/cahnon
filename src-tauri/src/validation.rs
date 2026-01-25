//! Validation constants and utilities for input sanitization

pub const MAX_TITLE_LENGTH: usize = 200;
pub const MAX_AUTHOR_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 10_000;
pub const MAX_CHAPTER_TITLE_LENGTH: usize = 200;
pub const MAX_SCENE_TITLE_LENGTH: usize = 200;
pub const MAX_SYNOPSIS_LENGTH: usize = 5_000;
pub const MAX_NOTES_LENGTH: usize = 50_000;
pub const MAX_CONTENT_LENGTH: usize = 500_000; // ~500KB of text
pub const MAX_BIBLE_ENTRY_NAME_LENGTH: usize = 200;
pub const MAX_BIBLE_ENTRY_DESCRIPTION_LENGTH: usize = 100_000;
pub const MAX_CHARACTER_NAME_LENGTH: usize = 100;
pub const MAX_ALIASES_PER_ENTRY: usize = 20;

/// Sanitize text input: remove control characters (except spaces), trim, and limit length
pub fn sanitize_text(text: &str, max_length: usize) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == ' ')
        .take(max_length)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Sanitize multiline text: allow newlines but remove other control characters
pub fn sanitize_multiline_text(text: &str, max_length: usize) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == ' ' || *c == '\n' || *c == '\r')
        .take(max_length)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validate that a required string field is not empty after sanitization
pub fn validate_required(value: &str, field_name: &str) -> Result<(), String> {
    if value.is_empty() {
        Err(format!("{} cannot be empty", field_name))
    } else {
        Ok(())
    }
}
