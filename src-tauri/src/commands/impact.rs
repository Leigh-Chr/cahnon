use crate::models::ImpactPreview;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn preview_delete_scene_impact(
    state: State<'_, AppState>,
    scene_id: String,
) -> Result<ImpactPreview, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.preview_delete_scene_impact(&scene_id)
}

#[tauri::command]
pub fn preview_delete_bible_entry_impact(
    state: State<'_, AppState>,
    bible_entry_id: String,
) -> Result<ImpactPreview, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.preview_delete_bible_entry_impact(&bible_entry_id)
}

#[tauri::command]
pub fn preview_delete_chapter_impact(
    state: State<'_, AppState>,
    chapter_id: String,
) -> Result<ImpactPreview, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.preview_delete_chapter_impact(&chapter_id)
}
