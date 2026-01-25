use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_snapshot(
    state: State<AppState>,
    name: String,
    description: Option<String>,
    snapshot_type: Option<String>,
) -> Result<Snapshot, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_snapshot(
        &name,
        description.as_deref(),
        snapshot_type.as_deref().unwrap_or("manual"),
    )
}

#[tauri::command]
pub fn get_snapshots(state: State<AppState>) -> Result<Vec<Snapshot>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_snapshots()
}

#[tauri::command]
pub fn get_snapshot(state: State<AppState>, id: String) -> Result<Snapshot, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_snapshot(&id)
}

#[tauri::command]
pub fn delete_snapshot(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_snapshot(&id)
}

#[tauri::command]
pub fn restore_snapshot(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.restore_snapshot(&id)
}
