use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_cut(
    state: State<AppState>,
    scene_id: Option<String>,
    text: String,
) -> Result<Cut, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_cut(scene_id.as_deref(), &text)
}

#[tauri::command]
pub fn get_cuts(state: State<AppState>) -> Result<Vec<Cut>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_cuts()
}

#[tauri::command]
pub fn delete_cut(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_cut(&id)
}
