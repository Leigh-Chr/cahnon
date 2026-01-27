use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_SYNOPSIS_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_arc(request: CreateArcRequest, state: State<AppState>) -> Result<Arc, String> {
    let name = sanitize_text(&request.name, MAX_TITLE_LENGTH);
    if name.is_empty() {
        return Err("Arc name cannot be empty".to_string());
    }

    let sanitized_request = CreateArcRequest {
        name,
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH)),
        stakes: request
            .stakes
            .map(|s| sanitize_multiline_text(&s, MAX_SYNOPSIS_LENGTH)),
        status: request.status.map(|s| sanitize_text(&s, MAX_TITLE_LENGTH)),
        color: request.color,
    };

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_arc(&sanitized_request)
}

#[tauri::command]
pub fn get_arcs(state: State<AppState>) -> Result<Vec<Arc>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_arcs()
}

#[tauri::command]
pub fn get_arc(id: String, state: State<AppState>) -> Result<Arc, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_arc(&id)
}

#[tauri::command]
pub fn update_arc(
    id: String,
    request: UpdateArcRequest,
    state: State<AppState>,
) -> Result<Arc, String> {
    let sanitized_request = UpdateArcRequest {
        name: request.name.map(|n| sanitize_text(&n, MAX_TITLE_LENGTH)),
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH)),
        stakes: request
            .stakes
            .map(|s| sanitize_multiline_text(&s, MAX_SYNOPSIS_LENGTH)),
        status: request.status.map(|s| sanitize_text(&s, MAX_TITLE_LENGTH)),
        color: request.color,
    };

    if let Some(ref name) = sanitized_request.name {
        if name.is_empty() {
            return Err("Arc name cannot be empty".to_string());
        }
    }

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.update_arc(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_arc(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_arc(&id)
}

#[tauri::command]
pub fn link_scene_to_arc(
    scene_id: String,
    arc_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.link_scene_to_arc(&scene_id, &arc_id)
}

#[tauri::command]
pub fn unlink_scene_from_arc(
    scene_id: String,
    arc_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_scene_from_arc(&scene_id, &arc_id)
}

#[tauri::command]
pub fn get_scene_arcs(scene_id: String, state: State<AppState>) -> Result<Vec<Arc>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_arcs(&scene_id)
}

#[tauri::command]
pub fn set_arc_characters(
    arc_id: String,
    character_ids: Vec<String>,
    state: State<AppState>,
) -> Result<Vec<String>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.set_arc_characters(&arc_id, &character_ids)
}

#[tauri::command]
pub fn get_arc_scenes(arc_id: String, state: State<AppState>) -> Result<Vec<Scene>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_arc_scenes(&arc_id)
}

#[tauri::command]
pub fn get_character_arcs(
    bible_entry_id: String,
    state: State<AppState>,
) -> Result<Vec<Arc>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_character_arcs(&bible_entry_id)
}
