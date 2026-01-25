use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_bible_entry(
    request: CreateBibleEntryRequest,
    state: State<AppState>,
) -> Result<BibleEntry, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_bible_entry(&request)
}

#[tauri::command]
pub fn get_bible_entries(
    entry_type: Option<String>,
    state: State<AppState>,
) -> Result<Vec<BibleEntry>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_bible_entries(entry_type.as_deref())
}

#[tauri::command]
pub fn get_bible_entry(id: String, state: State<AppState>) -> Result<BibleEntry, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_bible_entry(&id)
}

#[tauri::command]
pub fn update_bible_entry(
    id: String,
    request: UpdateBibleEntryRequest,
    state: State<AppState>,
) -> Result<BibleEntry, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_bible_entry(&id, &request)
}

#[tauri::command]
pub fn delete_bible_entry(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_bible_entry(&id)
}

#[tauri::command]
pub fn search_bible(query: String, state: State<AppState>) -> Result<Vec<BibleEntry>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.search_bible(&query)
}
