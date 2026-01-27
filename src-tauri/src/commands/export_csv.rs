use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn export_bible_csv(state: State<AppState>) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_bible_csv()
}

#[tauri::command]
pub fn export_timeline_csv(state: State<AppState>) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_timeline_csv()
}

#[tauri::command]
pub fn export_review_grid_csv(state: State<AppState>) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_review_grid_csv()
}

#[tauri::command]
pub fn export_stats_csv(state: State<AppState>) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_stats_csv()
}
