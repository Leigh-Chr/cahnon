//! Validation constants and utilities for input sanitization

pub const MAX_TITLE_LENGTH: usize = 200;
pub const MAX_AUTHOR_LENGTH: usize = 100;
pub const MAX_DESCRIPTION_LENGTH: usize = 10_000;
pub const MAX_CHAPTER_TITLE_LENGTH: usize = 200;
pub const MAX_SCENE_TITLE_LENGTH: usize = 200;
pub const MAX_SYNOPSIS_LENGTH: usize = 5_000;
pub const MAX_NOTES_LENGTH: usize = 50_000;
pub const MAX_CONTENT_LENGTH: usize = 500_000; // ~500KB of text
pub const MAX_BIBLE_ENTRY_NAME_LENGTH: usize = 100;
pub const MAX_ALIASES_PER_ENTRY: usize = 20;
pub const MAX_BIBLE_ENTRY_DESCRIPTION_LENGTH: usize = 100_000;

/// Returns true for characters that should be stripped from user input:
/// control characters and invisible Unicode format characters (zero-width spaces, etc.).
fn is_unwanted_char(c: char) -> bool {
    if c.is_control() {
        return true;
    }
    // Filter Unicode format characters (category Cf) that are invisible
    matches!(
        c,
        '\u{200B}'  // Zero-Width Space
        | '\u{200C}' // Zero-Width Non-Joiner
        | '\u{200D}' // Zero-Width Joiner
        | '\u{200E}' // Left-to-Right Mark
        | '\u{200F}' // Right-to-Left Mark
        | '\u{FEFF}' // BOM / Zero-Width No-Break Space
        | '\u{2060}' // Word Joiner
        | '\u{2061}'..='\u{2064}' // Invisible operators
        | '\u{FFF9}'..='\u{FFFB}' // Interlinear annotation
    )
}

/// Sanitize text input: remove control and invisible format characters, trim, and limit length.
/// Note: Space (U+0020) is not a control character so it is preserved by the filter.
pub fn sanitize_text(text: &str, max_length: usize) -> String {
    text.chars()
        .filter(|c| !is_unwanted_char(*c))
        .take(max_length)
        .collect::<String>()
        .trim()
        .to_string()
}

/// Sanitize multiline text: allow newlines but remove other control and invisible format characters
pub fn sanitize_multiline_text(text: &str, max_length: usize) -> String {
    text.chars()
        .filter(|c| (!is_unwanted_char(*c)) || *c == '\n' || *c == '\r')
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

#[cfg(test)]
mod tests {
    use super::*;

    // --- is_unwanted_char ---

    #[test]
    fn test_is_unwanted_char_regular_chars() {
        assert!(!is_unwanted_char('a'));
        assert!(!is_unwanted_char('Z'));
        assert!(!is_unwanted_char('0'));
        assert!(!is_unwanted_char(' '));
        assert!(!is_unwanted_char('é'));
        assert!(!is_unwanted_char('漢'));
    }

    #[test]
    fn test_is_unwanted_char_control_chars() {
        assert!(is_unwanted_char('\x00')); // NUL
        assert!(is_unwanted_char('\x01')); // SOH
        assert!(is_unwanted_char('\x7F')); // DEL
        assert!(is_unwanted_char('\x08')); // Backspace
        assert!(is_unwanted_char('\x1B')); // Escape
    }

    #[test]
    fn test_is_unwanted_char_newline_tab_are_control() {
        // \n and \t are control chars per char::is_control()
        assert!(is_unwanted_char('\n'));
        assert!(is_unwanted_char('\t'));
        assert!(is_unwanted_char('\r'));
    }

    #[test]
    fn test_is_unwanted_char_zero_width_spaces() {
        assert!(is_unwanted_char('\u{200B}')); // Zero-Width Space
        assert!(is_unwanted_char('\u{200C}')); // Zero-Width Non-Joiner
        assert!(is_unwanted_char('\u{200D}')); // Zero-Width Joiner
        assert!(is_unwanted_char('\u{FEFF}')); // BOM
        assert!(is_unwanted_char('\u{2060}')); // Word Joiner
    }

    #[test]
    fn test_is_unwanted_char_directional_marks() {
        assert!(is_unwanted_char('\u{200E}')); // LTR Mark
        assert!(is_unwanted_char('\u{200F}')); // RTL Mark
    }

    #[test]
    fn test_is_unwanted_char_invisible_operators() {
        assert!(is_unwanted_char('\u{2061}')); // Function Application
        assert!(is_unwanted_char('\u{2062}')); // Invisible Times
        assert!(is_unwanted_char('\u{2063}')); // Invisible Separator
        assert!(is_unwanted_char('\u{2064}')); // Invisible Plus
    }

    #[test]
    fn test_is_unwanted_char_interlinear_annotation() {
        assert!(is_unwanted_char('\u{FFF9}')); // Interlinear Annotation Anchor
        assert!(is_unwanted_char('\u{FFFA}')); // Interlinear Annotation Separator
        assert!(is_unwanted_char('\u{FFFB}')); // Interlinear Annotation Terminator
    }

    #[test]
    fn test_is_unwanted_char_emoji_allowed() {
        assert!(!is_unwanted_char('😀'));
        assert!(!is_unwanted_char('🎉'));
    }

    // --- sanitize_text ---

    #[test]
    fn test_sanitize_text_normal_text() {
        assert_eq!(sanitize_text("Hello World", 100), "Hello World");
    }

    #[test]
    fn test_sanitize_text_empty() {
        assert_eq!(sanitize_text("", 100), "");
    }

    #[test]
    fn test_sanitize_text_strips_control_chars() {
        assert_eq!(sanitize_text("He\x00llo\x01", 100), "Hello");
    }

    #[test]
    fn test_sanitize_text_strips_zero_width() {
        assert_eq!(sanitize_text("He\u{200B}llo\u{FEFF}!", 100), "Hello!");
    }

    #[test]
    fn test_sanitize_text_strips_newlines() {
        // \n is control, so sanitize_text removes it
        assert_eq!(sanitize_text("Line1\nLine2", 100), "Line1Line2");
    }

    #[test]
    fn test_sanitize_text_truncates_at_max_length() {
        assert_eq!(sanitize_text("Hello World", 5), "Hello");
    }

    #[test]
    fn test_sanitize_text_trims_whitespace() {
        assert_eq!(sanitize_text("  Hello  ", 100), "Hello");
    }

    #[test]
    fn test_sanitize_text_truncate_then_trim() {
        // Take 7 chars "  Hello", then trim → "Hello"
        assert_eq!(sanitize_text("  Hello  World", 7), "Hello");
    }

    #[test]
    fn test_sanitize_text_unicode_preserved() {
        assert_eq!(sanitize_text("Héllo 漢字 🎉", 100), "Héllo 漢字 🎉");
    }

    #[test]
    fn test_sanitize_text_max_zero() {
        assert_eq!(sanitize_text("Hello", 0), "");
    }

    // --- sanitize_multiline_text ---

    #[test]
    fn test_sanitize_multiline_preserves_newlines() {
        assert_eq!(
            sanitize_multiline_text("Line1\nLine2\nLine3", 100),
            "Line1\nLine2\nLine3"
        );
    }

    #[test]
    fn test_sanitize_multiline_preserves_carriage_return() {
        assert_eq!(
            sanitize_multiline_text("Line1\r\nLine2", 100),
            "Line1\r\nLine2"
        );
    }

    #[test]
    fn test_sanitize_multiline_strips_other_control() {
        assert_eq!(
            sanitize_multiline_text("He\x00llo\n\x01World", 100),
            "Hello\nWorld"
        );
    }

    #[test]
    fn test_sanitize_multiline_strips_zero_width() {
        assert_eq!(
            sanitize_multiline_text("He\u{200B}llo\nWorld\u{FEFF}", 100),
            "Hello\nWorld"
        );
    }

    #[test]
    fn test_sanitize_multiline_truncates() {
        assert_eq!(
            sanitize_multiline_text("Line1\nLine2\nLine3", 11),
            "Line1\nLine2"
        );
    }

    #[test]
    fn test_sanitize_multiline_trims() {
        assert_eq!(
            sanitize_multiline_text("  \n  Hello  \n  ", 100),
            "Hello"
        );
    }

    // --- validate_required ---

    #[test]
    fn test_validate_required_non_empty() {
        assert!(validate_required("Hello", "title").is_ok());
    }

    #[test]
    fn test_validate_required_empty() {
        let err = validate_required("", "title").unwrap_err();
        assert_eq!(err, "title cannot be empty");
    }

    #[test]
    fn test_validate_required_whitespace_not_empty() {
        // validate_required checks is_empty(), not is_blank
        // Whitespace passes because it's not empty string
        assert!(validate_required("  ", "title").is_ok());
    }

    #[test]
    fn test_validate_required_error_includes_field_name() {
        let err = validate_required("", "description").unwrap_err();
        assert!(err.contains("description"));
    }
}
