use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_annotation(
    state: State<AppState>,
    request: CreateAnnotationRequest,
) -> Result<Annotation, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_annotation(&request)
}

#[tauri::command]
pub fn get_annotations(
    state: State<AppState>,
    scene_id: String,
) -> Result<Vec<Annotation>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_annotations(&scene_id)
}

#[tauri::command]
pub fn update_annotation(
    state: State<AppState>,
    id: String,
    request: UpdateAnnotationRequest,
) -> Result<Annotation, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_annotation(&id, &request)
}

#[tauri::command]
pub fn delete_annotation(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_annotation(&id)
}
