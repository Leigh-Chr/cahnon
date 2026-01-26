use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_SYNOPSIS_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_snapshot(
    name: String,
    description: Option<String>,
    snapshot_type: Option<String>,
    state: State<AppState>,
) -> Result<Snapshot, String> {
    let name = sanitize_text(&name, MAX_TITLE_LENGTH);
    if name.is_empty() {
        return Err("Snapshot name cannot be empty".to_string());
    }

    let description = description.map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH));

    let snapshot_type = snapshot_type
        .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH))
        .unwrap_or_else(|| "manual".to_string());

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_snapshot(&name, description.as_deref(), &snapshot_type)
}

#[tauri::command]
pub fn get_snapshots(state: State<AppState>) -> Result<Vec<Snapshot>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_snapshots()
}

#[tauri::command]
pub fn get_snapshot(id: String, state: State<AppState>) -> Result<Snapshot, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_snapshot(&id)
}

#[tauri::command]
pub fn delete_snapshot(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_snapshot(&id)
}

#[tauri::command]
pub fn restore_snapshot(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.restore_snapshot(&id)
}

#[tauri::command]
pub fn cleanup_expired_snapshots(state: State<AppState>) -> Result<i32, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.cleanup_expired_snapshots()
}

#[tauri::command]
pub fn get_snapshot_scenes(id: String, state: State<AppState>) -> Result<Vec<Scene>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_snapshot_scenes(&id)
}

#[tauri::command]
pub fn restore_scene_from_snapshot(
    snapshot_id: String,
    scene_id: String,
    state: State<AppState>,
) -> Result<Scene, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.restore_scene_from_snapshot(&snapshot_id, &scene_id)
}
