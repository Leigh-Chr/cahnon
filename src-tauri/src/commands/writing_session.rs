use crate::models::{CreateWritingSessionRequest, UpdateWritingSessionRequest, WritingSession};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn create_writing_session(
    state: State<'_, AppState>,
    request: CreateWritingSessionRequest,
) -> Result<WritingSession, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_writing_session(&request.date, request.words_start)
}

#[tauri::command]
pub fn get_writing_sessions(state: State<'_, AppState>) -> Result<Vec<WritingSession>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_writing_sessions()
}

#[tauri::command]
pub fn get_writing_session_by_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Option<WritingSession>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_writing_session_by_date(&date)
}

#[tauri::command]
pub fn update_writing_session(
    state: State<'_, AppState>,
    id: String,
    request: UpdateWritingSessionRequest,
) -> Result<WritingSession, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.update_writing_session(&id, &request)
}

#[tauri::command]
pub fn delete_writing_session(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_writing_session(&id)
}
