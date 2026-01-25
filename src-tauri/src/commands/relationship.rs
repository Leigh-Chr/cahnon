use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_bible_relationship(
    state: State<AppState>,
    request: CreateBibleRelationshipRequest,
) -> Result<BibleRelationship, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_bible_relationship(&request)
}

#[tauri::command]
pub fn get_bible_relationships(
    state: State<AppState>,
    entry_id: String,
) -> Result<Vec<BibleRelationshipWithEntry>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_bible_relationships(&entry_id)
}

#[tauri::command]
pub fn update_bible_relationship(
    state: State<AppState>,
    id: String,
    request: UpdateBibleRelationshipRequest,
) -> Result<BibleRelationship, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_bible_relationship(&id, &request)
}

#[tauri::command]
pub fn delete_bible_relationship(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_bible_relationship(&id)
}
