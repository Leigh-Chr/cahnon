use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn export_markdown(
    chapter_ids: Option<Vec<String>>,
    scene_separator: Option<String>,
    include_titles: Option<bool>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_markdown_with_options(
        chapter_ids.as_deref(),
        scene_separator.as_deref(),
        include_titles.unwrap_or(true),
    )
}

#[tauri::command]
pub fn export_plain_text(
    chapter_ids: Option<Vec<String>>,
    scene_separator: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_plain_text_with_options(chapter_ids.as_deref(), scene_separator.as_deref())
}

#[tauri::command]
pub fn export_json_backup(state: State<AppState>) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_json_backup()
}

#[tauri::command]
pub fn export_outline(state: State<AppState>) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_outline()
}

#[tauri::command]
pub fn export_bible(state: State<AppState>) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_bible()
}

#[tauri::command]
pub fn export_timeline(state: State<AppState>) -> Result<String, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.export_timeline()
}
