use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_SYNOPSIS_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_event(request: CreateEventRequest, state: State<AppState>) -> Result<Event, String> {
    let title = sanitize_text(&request.title, MAX_TITLE_LENGTH);
    if title.is_empty() {
        return Err("Event title cannot be empty".to_string());
    }

    let sanitized_request = CreateEventRequest {
        title,
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH)),
        time_point: request
            .time_point
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        time_start: request
            .time_start
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        time_end: request
            .time_end
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        event_type: request
            .event_type
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        importance: request
            .importance
            .map(|i| sanitize_text(&i, MAX_TITLE_LENGTH)),
    };

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_event(&sanitized_request)
}

#[tauri::command]
pub fn get_events(state: State<AppState>) -> Result<Vec<Event>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_events()
}

#[tauri::command]
pub fn get_event(id: String, state: State<AppState>) -> Result<Event, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_event(&id)
}

#[tauri::command]
pub fn update_event(
    id: String,
    request: UpdateEventRequest,
    state: State<AppState>,
) -> Result<Event, String> {
    let sanitized_request = UpdateEventRequest {
        title: request.title.map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH)),
        time_point: request
            .time_point
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        time_start: request
            .time_start
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        time_end: request
            .time_end
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        event_type: request
            .event_type
            .map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        importance: request
            .importance
            .map(|i| sanitize_text(&i, MAX_TITLE_LENGTH)),
    };

    if let Some(ref title) = sanitized_request.title {
        if title.is_empty() {
            return Err("Event title cannot be empty".to_string());
        }
    }

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.update_event(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_event(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_event(&id)
}

#[tauri::command]
pub fn get_timeline_scenes(state: State<AppState>) -> Result<Vec<Scene>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_all_scenes_for_timeline()
}

#[tauri::command]
pub fn detect_timeline_conflicts(state: State<AppState>) -> Result<Vec<TimelineConflict>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.detect_timeline_conflicts()
}

#[tauri::command]
pub fn link_scene_to_event(
    scene_id: String,
    event_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.link_scene_to_event(&scene_id, &event_id)
}

#[tauri::command]
pub fn unlink_scene_from_event(
    scene_id: String,
    event_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_scene_from_event(&scene_id, &event_id)
}

#[tauri::command]
pub fn get_scene_events(scene_id: String, state: State<AppState>) -> Result<Vec<Event>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_events(&scene_id)
}

#[tauri::command]
pub fn get_event_scenes(event_id: String, state: State<AppState>) -> Result<Vec<Scene>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_event_scenes(&event_id)
}

// Event-Bible linking commands (per spec Section 9.1)
#[tauri::command]
pub fn link_bible_entry_to_event(
    bible_entry_id: String,
    event_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.link_bible_entry_to_event(&bible_entry_id, &event_id)
}

#[tauri::command]
pub fn unlink_bible_entry_from_event(
    bible_entry_id: String,
    event_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_bible_entry_from_event(&bible_entry_id, &event_id)
}

#[tauri::command]
pub fn get_event_bible_entries(
    event_id: String,
    state: State<AppState>,
) -> Result<Vec<BibleEntry>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_event_bible_entries(&event_id)
}

#[tauri::command]
pub fn get_bible_entry_events(
    bible_entry_id: String,
    state: State<AppState>,
) -> Result<Vec<Event>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_bible_entry_events(&bible_entry_id)
}
