use crate::models::SceneHealth;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_scene_health_batch(state: State<'_, AppState>) -> Result<Vec<SceneHealth>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_health_batch()
}
