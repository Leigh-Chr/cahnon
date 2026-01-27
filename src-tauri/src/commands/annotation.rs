use crate::validation::{sanitize_multiline_text, MAX_NOTES_LENGTH};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_annotation(
    request: CreateAnnotationRequest,
    state: State<AppState>,
) -> Result<Annotation, String> {
    if request.start_offset < 0 || request.end_offset < 0 {
        return Err("Offsets must be non-negative".to_string());
    }
    if request.end_offset <= request.start_offset {
        return Err("end_offset must be greater than start_offset".to_string());
    }

    let sanitized_request = CreateAnnotationRequest {
        scene_id: request.scene_id,
        start_offset: request.start_offset,
        end_offset: request.end_offset,
        annotation_type: request.annotation_type,
        content: sanitize_multiline_text(&request.content, MAX_NOTES_LENGTH),
    };

    if sanitized_request.content.is_empty() {
        return Err("Annotation content cannot be empty".to_string());
    }

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.create_annotation(&sanitized_request)
}

#[tauri::command]
pub fn get_annotations(
    scene_id: String,
    state: State<AppState>,
) -> Result<Vec<Annotation>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_annotations(&scene_id)
}

#[tauri::command]
pub fn update_annotation(
    id: String,
    request: UpdateAnnotationRequest,
    state: State<AppState>,
) -> Result<Annotation, String> {
    let sanitized_request = UpdateAnnotationRequest {
        content: request
            .content
            .map(|c| sanitize_multiline_text(&c, MAX_NOTES_LENGTH)),
        status: request.status,
    };

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.update_annotation(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_annotation(id: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.delete_annotation(&id)
}
