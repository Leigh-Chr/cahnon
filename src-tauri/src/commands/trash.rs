use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn get_deleted_scenes(state: State<AppState>) -> Result<Vec<Scene>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_deleted_scenes()
}

#[tauri::command]
pub fn restore_scene(id: String, state: State<AppState>) -> Result<Scene, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.restore_scene(&id)
}

#[tauri::command]
pub fn get_deleted_chapters(state: State<AppState>) -> Result<Vec<Chapter>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_deleted_chapters()
}

#[tauri::command]
pub fn restore_chapter(id: String, state: State<AppState>) -> Result<Chapter, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.restore_chapter(&id)
}

#[tauri::command]
pub fn purge_expired_trash(state: State<AppState>) -> Result<(usize, usize), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.purge_expired_trash()
}

#[tauri::command]
pub fn duplicate_scene(
    id: String,
    structure_only: Option<bool>,
    state: State<AppState>,
) -> Result<Scene, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.duplicate_scene(&id, structure_only.unwrap_or(false))
}
