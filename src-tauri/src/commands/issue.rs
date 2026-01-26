use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_NOTES_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_issue(request: CreateIssueRequest, state: State<AppState>) -> Result<Issue, String> {
    let title = sanitize_text(&request.title, MAX_TITLE_LENGTH);
    if title.is_empty() {
        return Err("Issue title cannot be empty".to_string());
    }

    let sanitized_request = CreateIssueRequest {
        issue_type: sanitize_text(&request.issue_type, MAX_TITLE_LENGTH),
        title,
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_NOTES_LENGTH)),
        severity: request
            .severity
            .map(|s| sanitize_text(&s, MAX_TITLE_LENGTH)),
    };

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_issue(&sanitized_request)
}

#[tauri::command]
pub fn get_issues(status: Option<String>, state: State<AppState>) -> Result<Vec<Issue>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_issues(status.as_deref())
}

#[tauri::command]
pub fn get_issue(id: String, state: State<AppState>) -> Result<Issue, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_issue(&id)
}

#[tauri::command]
pub fn update_issue(
    id: String,
    request: UpdateIssueRequest,
    state: State<AppState>,
) -> Result<Issue, String> {
    let sanitized_request = UpdateIssueRequest {
        status: request.status,
        resolution_note: request
            .resolution_note
            .map(|n| sanitize_multiline_text(&n, MAX_NOTES_LENGTH)),
        title: request.title.map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_NOTES_LENGTH)),
        severity: request.severity,
    };

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.update_issue(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_issue(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_issue(&id)
}

// Issue-Scene linking commands (per spec Section 14.2)
#[tauri::command]
pub fn link_scene_to_issue(
    scene_id: String,
    issue_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.link_scene_to_issue(&scene_id, &issue_id)
}

#[tauri::command]
pub fn unlink_scene_from_issue(
    scene_id: String,
    issue_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_scene_from_issue(&scene_id, &issue_id)
}

#[tauri::command]
pub fn get_issue_scenes(issue_id: String, state: State<AppState>) -> Result<Vec<String>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_issue_scenes(&issue_id)
}

#[tauri::command]
pub fn get_scene_issues(scene_id: String, state: State<AppState>) -> Result<Vec<Issue>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_issues(&scene_id)
}

// Issue-Bible linking commands (per spec Section 14.2)
#[tauri::command]
pub fn link_bible_entry_to_issue(
    bible_entry_id: String,
    issue_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.link_bible_entry_to_issue(&bible_entry_id, &issue_id)
}

#[tauri::command]
pub fn unlink_bible_entry_from_issue(
    bible_entry_id: String,
    issue_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_bible_entry_from_issue(&bible_entry_id, &issue_id)
}

#[tauri::command]
pub fn get_issue_bible_entries(
    issue_id: String,
    state: State<AppState>,
) -> Result<Vec<String>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_issue_bible_entries(&issue_id)
}
