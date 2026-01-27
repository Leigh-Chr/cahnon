use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_CHAPTER_TITLE_LENGTH, MAX_NOTES_LENGTH,
    MAX_SYNOPSIS_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_chapter(
    request: CreateChapterRequest,
    state: State<AppState>,
) -> Result<Chapter, String> {
    // Sanitize inputs
    let title = sanitize_text(&request.title, MAX_CHAPTER_TITLE_LENGTH);
    if title.is_empty() {
        return Err("Chapter title cannot be empty".to_string());
    }

    let sanitized_request = CreateChapterRequest {
        title,
        summary: request
            .summary
            .map(|s| sanitize_multiline_text(&s, MAX_SYNOPSIS_LENGTH)),
        position: request.position,
    };

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.create_chapter(&sanitized_request)
}

#[tauri::command]
pub fn get_chapters(state: State<AppState>) -> Result<Vec<Chapter>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_chapters()
}

#[tauri::command]
pub fn get_chapter(id: String, state: State<AppState>) -> Result<Chapter, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_chapter(&id)
}

#[tauri::command]
pub fn update_chapter(
    id: String,
    request: UpdateChapterRequest,
    state: State<AppState>,
) -> Result<Chapter, String> {
    // Sanitize inputs
    let sanitized_request = UpdateChapterRequest {
        title: request
            .title
            .map(|t| sanitize_text(&t, MAX_CHAPTER_TITLE_LENGTH)),
        summary: request
            .summary
            .map(|s| sanitize_multiline_text(&s, MAX_SYNOPSIS_LENGTH)),
        status: request.status, // Status is validated by enum/type
        notes: request
            .notes
            .map(|n| sanitize_multiline_text(&n, MAX_NOTES_LENGTH)),
        position: request.position,
    };

    // Validate title if provided
    if let Some(ref title) = sanitized_request.title {
        if title.is_empty() {
            return Err("Chapter title cannot be empty".to_string());
        }
    }

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.update_chapter(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_chapter(id: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.delete_chapter(&id)
}

#[tauri::command]
pub fn reorder_chapters(ids: Vec<String>, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.reorder_chapters(&ids)
}
