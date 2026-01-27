use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_NOTES_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_bible_relationship(
    request: CreateBibleRelationshipRequest,
    state: State<AppState>,
) -> Result<BibleRelationship, String> {
    let relationship_type = sanitize_text(&request.relationship_type, MAX_TITLE_LENGTH);
    if relationship_type.is_empty() {
        return Err("Relationship type cannot be empty".to_string());
    }

    let sanitized_request = CreateBibleRelationshipRequest {
        source_id: request.source_id,
        target_id: request.target_id,
        relationship_type,
        note: request
            .note
            .map(|n| sanitize_multiline_text(&n, MAX_NOTES_LENGTH)),
        status: request.status,
    };

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.create_bible_relationship(&sanitized_request)
}

#[tauri::command]
pub fn get_bible_relationships(
    entry_id: String,
    state: State<AppState>,
) -> Result<Vec<BibleRelationshipWithEntry>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_bible_relationships(&entry_id)
}

#[tauri::command]
pub fn update_bible_relationship(
    id: String,
    request: UpdateBibleRelationshipRequest,
    state: State<AppState>,
) -> Result<BibleRelationship, String> {
    let sanitized_request = UpdateBibleRelationshipRequest {
        relationship_type: request
            .relationship_type
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        note: request
            .note
            .map(|n| sanitize_multiline_text(&n, MAX_NOTES_LENGTH)),
        status: request.status,
    };

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.update_bible_relationship(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_bible_relationship(id: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.delete_bible_relationship(&id)
}
