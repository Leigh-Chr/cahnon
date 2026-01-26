use crate::validation::{sanitize_text, MAX_TITLE_LENGTH};
use crate::{models::*, AppState};
use tauri::State;

const MAX_FILTER_DATA_SIZE: usize = 64 * 1024; // 64 KB

#[tauri::command]
pub fn create_saved_filter(
    request: CreateSavedFilterRequest,
    state: State<AppState>,
) -> Result<SavedFilter, String> {
    let name = sanitize_text(&request.name, MAX_TITLE_LENGTH);
    if name.is_empty() {
        return Err("Filter name cannot be empty".to_string());
    }

    if request.filter_data.len() > MAX_FILTER_DATA_SIZE {
        return Err("Filter data is too large".to_string());
    }
    if serde_json::from_str::<serde_json::Value>(&request.filter_data).is_err() {
        return Err("filter_data must be valid JSON".to_string());
    }

    let sanitized_request = CreateSavedFilterRequest {
        name,
        filter_type: sanitize_text(&request.filter_type, MAX_TITLE_LENGTH),
        filter_data: request.filter_data,
    };

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_saved_filter(&sanitized_request)
}

#[tauri::command]
pub fn get_saved_filters(
    filter_type: Option<String>,
    state: State<AppState>,
) -> Result<Vec<SavedFilter>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_saved_filters(filter_type.as_deref())
}

#[tauri::command]
pub fn update_saved_filter(
    id: String,
    request: UpdateSavedFilterRequest,
    state: State<AppState>,
) -> Result<SavedFilter, String> {
    let sanitized_request = UpdateSavedFilterRequest {
        name: request.name.map(|n| sanitize_text(&n, MAX_TITLE_LENGTH)),
        filter_data: request.filter_data,
    };

    if let Some(ref name) = sanitized_request.name {
        if name.is_empty() {
            return Err("Filter name cannot be empty".to_string());
        }
    }

    if let Some(ref data) = sanitized_request.filter_data {
        if data.len() > MAX_FILTER_DATA_SIZE {
            return Err("Filter data is too large".to_string());
        }
        if serde_json::from_str::<serde_json::Value>(data).is_err() {
            return Err("filter_data must be valid JSON".to_string());
        }
    }

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.update_saved_filter(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_saved_filter(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_saved_filter(&id)
}
