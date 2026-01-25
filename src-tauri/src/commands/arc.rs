use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_arc(state: State<AppState>, request: CreateArcRequest) -> Result<Arc, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_arc(&request)
}

#[tauri::command]
pub fn get_arcs(state: State<AppState>) -> Result<Vec<Arc>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_arcs()
}

#[tauri::command]
pub fn get_arc(state: State<AppState>, id: String) -> Result<Arc, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_arc(&id)
}

#[tauri::command]
pub fn update_arc(
    state: State<AppState>,
    id: String,
    request: UpdateArcRequest,
) -> Result<Arc, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_arc(&id, &request)
}

#[tauri::command]
pub fn delete_arc(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_arc(&id)
}

#[tauri::command]
pub fn link_scene_to_arc(
    state: State<AppState>,
    scene_id: String,
    arc_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.link_scene_to_arc(&scene_id, &arc_id)
}

#[tauri::command]
pub fn unlink_scene_from_arc(
    state: State<AppState>,
    scene_id: String,
    arc_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_scene_from_arc(&scene_id, &arc_id)
}

#[tauri::command]
pub fn get_scene_arcs(state: State<AppState>, scene_id: String) -> Result<Vec<Arc>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_arcs(&scene_id)
}
