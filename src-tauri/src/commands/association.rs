use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn auto_link_bible_entries(
    scene_id: String,
    state: State<AppState>,
) -> Result<AutoLinkResult, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.auto_link_bible_entries(&scene_id)
}

#[tauri::command]
pub fn create_association(
    request: CreateAssociationRequest,
    state: State<AppState>,
) -> Result<CanonicalAssociation, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.create_association(&request)
}

#[tauri::command]
pub fn get_scene_associations(
    scene_id: String,
    state: State<AppState>,
) -> Result<Vec<BibleEntry>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_scene_associations(&scene_id)
}

#[tauri::command]
pub fn delete_association(
    scene_id: String,
    bible_entry_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.delete_association(&scene_id, &bible_entry_id)
}

#[tauri::command]
pub fn get_bible_entry_scenes(
    bible_entry_id: String,
    state: State<AppState>,
) -> Result<Vec<Scene>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_bible_entry_scenes(&bible_entry_id)
}
