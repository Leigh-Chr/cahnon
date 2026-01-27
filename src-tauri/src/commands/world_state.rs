use crate::models::WorldState;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_world_state_at_scene(
    state: State<'_, AppState>,
    scene_id: String,
) -> Result<WorldState, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_world_state_at_scene(&scene_id)
}
