use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn get_deleted_scenes(state: State<AppState>) -> Result<Vec<Scene>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_deleted_scenes()
}

#[tauri::command]
pub fn restore_scene(state: State<AppState>, id: String) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.restore_scene(&id)
}

#[tauri::command]
pub fn get_deleted_chapters(state: State<AppState>) -> Result<Vec<Chapter>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_deleted_chapters()
}

#[tauri::command]
pub fn restore_chapter(state: State<AppState>, id: String) -> Result<Chapter, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.restore_chapter(&id)
}

#[tauri::command]
pub fn duplicate_scene(
    state: State<AppState>,
    id: String,
    structure_only: Option<bool>,
) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.duplicate_scene(&id, structure_only.unwrap_or(false))
}
