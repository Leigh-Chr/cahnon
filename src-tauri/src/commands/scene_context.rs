use crate::models::SceneContext;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_scene_context(
    state: State<'_, AppState>,
    scene_id: String,
) -> Result<SceneContext, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_context(&scene_id)
}
