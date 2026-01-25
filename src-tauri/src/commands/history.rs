use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn get_scene_history(
    state: State<AppState>,
    scene_id: String,
) -> Result<Vec<SceneHistoryEntry>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_history(&scene_id)
}

#[tauri::command]
pub fn restore_scene_version(
    state: State<AppState>,
    scene_id: String,
    history_id: String,
) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.restore_scene_version(&scene_id, &history_id)
}

#[tauri::command]
pub fn compare_scene_versions(
    state: State<AppState>,
    scene_id: String,
    history_id_a: String,
    history_id_b: String,
) -> Result<VersionDiff, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    let (text_a, text_b) = db.compare_scene_versions(&scene_id, &history_id_a, &history_id_b)?;
    Ok(VersionDiff { text_a, text_b })
}
