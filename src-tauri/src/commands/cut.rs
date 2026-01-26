use crate::validation::{sanitize_multiline_text, MAX_CONTENT_LENGTH};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_cut(
    scene_id: Option<String>,
    text: String,
    state: State<AppState>,
) -> Result<Cut, String> {
    let text = sanitize_multiline_text(&text, MAX_CONTENT_LENGTH);
    if text.is_empty() {
        return Err("Cut text cannot be empty".to_string());
    }

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_cut(scene_id.as_deref(), &text)
}

#[tauri::command]
pub fn get_cuts(state: State<AppState>) -> Result<Vec<Cut>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_cuts()
}

#[tauri::command]
pub fn delete_cut(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_cut(&id)
}
