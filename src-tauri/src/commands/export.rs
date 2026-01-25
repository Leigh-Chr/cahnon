use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn export_markdown(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.export_markdown()
}

#[tauri::command]
pub fn export_plain_text(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.export_plain_text()
}

#[tauri::command]
pub fn export_json_backup(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.export_json_backup()
}

#[tauri::command]
pub fn export_outline(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.export_outline()
}

#[tauri::command]
pub fn export_bible(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.export_bible()
}

#[tauri::command]
pub fn export_timeline(state: State<AppState>) -> Result<String, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.export_timeline()
}
