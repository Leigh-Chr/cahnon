use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_association(
    request: CreateAssociationRequest,
    state: State<AppState>,
) -> Result<CanonicalAssociation, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_association(&request)
}

#[tauri::command]
pub fn get_scene_associations(
    scene_id: String,
    state: State<AppState>,
) -> Result<Vec<BibleEntry>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_associations(&scene_id)
}

#[tauri::command]
pub fn delete_association(
    scene_id: String,
    bible_entry_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_association(&scene_id, &bible_entry_id)
}
