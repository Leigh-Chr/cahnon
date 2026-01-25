use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_event(state: State<AppState>, request: CreateEventRequest) -> Result<Event, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_event(&request)
}

#[tauri::command]
pub fn get_events(state: State<AppState>) -> Result<Vec<Event>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_events()
}

#[tauri::command]
pub fn get_event(state: State<AppState>, id: String) -> Result<Event, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_event(&id)
}

#[tauri::command]
pub fn update_event(
    state: State<AppState>,
    id: String,
    request: UpdateEventRequest,
) -> Result<Event, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_event(&id, &request)
}

#[tauri::command]
pub fn delete_event(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_event(&id)
}

#[tauri::command]
pub fn get_timeline_scenes(state: State<AppState>) -> Result<Vec<Scene>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_all_scenes_for_timeline()
}

#[tauri::command]
pub fn detect_timeline_conflicts(state: State<AppState>) -> Result<Vec<TimelineConflict>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.detect_timeline_conflicts()
}

#[tauri::command]
pub fn link_scene_to_event(
    state: State<AppState>,
    scene_id: String,
    event_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.link_scene_to_event(&scene_id, &event_id)
}

#[tauri::command]
pub fn unlink_scene_from_event(
    state: State<AppState>,
    scene_id: String,
    event_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_scene_from_event(&scene_id, &event_id)
}

#[tauri::command]
pub fn get_scene_events(state: State<AppState>, scene_id: String) -> Result<Vec<Event>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_events(&scene_id)
}

#[tauri::command]
pub fn get_event_scenes(state: State<AppState>, event_id: String) -> Result<Vec<String>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_event_scenes(&event_id)
}

// Event-Bible linking commands (per spec Section 9.1)
#[tauri::command]
pub fn link_bible_entry_to_event(
    state: State<AppState>,
    bible_entry_id: String,
    event_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.link_bible_entry_to_event(&bible_entry_id, &event_id)
}

#[tauri::command]
pub fn unlink_bible_entry_from_event(
    state: State<AppState>,
    bible_entry_id: String,
    event_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_bible_entry_from_event(&bible_entry_id, &event_id)
}

#[tauri::command]
pub fn get_event_bible_entries(
    state: State<AppState>,
    event_id: String,
) -> Result<Vec<String>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_event_bible_entries(&event_id)
}

#[tauri::command]
pub fn get_bible_entry_events(
    state: State<AppState>,
    bible_entry_id: String,
) -> Result<Vec<Event>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_bible_entry_events(&bible_entry_id)
}
