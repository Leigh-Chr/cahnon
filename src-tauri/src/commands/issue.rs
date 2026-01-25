use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_issue(state: State<AppState>, request: CreateIssueRequest) -> Result<Issue, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_issue(&request)
}

#[tauri::command]
pub fn get_issues(state: State<AppState>, status: Option<String>) -> Result<Vec<Issue>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_issues(status.as_deref())
}

#[tauri::command]
pub fn get_issue(state: State<AppState>, id: String) -> Result<Issue, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_issue(&id)
}

#[tauri::command]
pub fn update_issue(
    state: State<AppState>,
    id: String,
    request: UpdateIssueRequest,
) -> Result<Issue, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_issue(&id, &request)
}

// Issue-Scene linking commands (per spec Section 14.2)
#[tauri::command]
pub fn link_scene_to_issue(
    state: State<AppState>,
    scene_id: String,
    issue_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.link_scene_to_issue(&scene_id, &issue_id)
}

#[tauri::command]
pub fn unlink_scene_from_issue(
    state: State<AppState>,
    scene_id: String,
    issue_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_scene_from_issue(&scene_id, &issue_id)
}

#[tauri::command]
pub fn get_issue_scenes(state: State<AppState>, issue_id: String) -> Result<Vec<String>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_issue_scenes(&issue_id)
}

#[tauri::command]
pub fn get_scene_issues(state: State<AppState>, scene_id: String) -> Result<Vec<Issue>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_issues(&scene_id)
}

// Issue-Bible linking commands (per spec Section 14.2)
#[tauri::command]
pub fn link_bible_entry_to_issue(
    state: State<AppState>,
    bible_entry_id: String,
    issue_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.link_bible_entry_to_issue(&bible_entry_id, &issue_id)
}

#[tauri::command]
pub fn unlink_bible_entry_from_issue(
    state: State<AppState>,
    bible_entry_id: String,
    issue_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_bible_entry_from_issue(&bible_entry_id, &issue_id)
}

#[tauri::command]
pub fn get_issue_bible_entries(
    state: State<AppState>,
    issue_id: String,
) -> Result<Vec<String>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_issue_bible_entries(&issue_id)
}
