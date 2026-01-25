use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn get_templates(state: State<AppState>) -> Result<Vec<Template>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_templates()
}

#[tauri::command]
pub fn get_template_steps(
    state: State<AppState>,
    template_id: String,
) -> Result<Vec<TemplateStep>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_template_steps(&template_id)
}

#[tauri::command]
pub fn set_active_template(state: State<AppState>, template_id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.set_active_template(&template_id)
}

#[tauri::command]
pub fn assign_scene_to_step(
    state: State<AppState>,
    scene_id: String,
    step_id: String,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.assign_scene_to_step(&scene_id, &step_id)
}

#[tauri::command]
pub fn get_scene_step(
    state: State<AppState>,
    scene_id: String,
) -> Result<Option<TemplateStep>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene_step(&scene_id)
}

#[tauri::command]
pub fn init_builtin_templates(state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.init_builtin_templates()
}

#[tauri::command]
pub fn create_template(
    state: State<AppState>,
    request: CreateTemplateRequest,
) -> Result<Template, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_template(&request)
}

#[tauri::command]
pub fn update_template(
    state: State<AppState>,
    id: String,
    request: UpdateTemplateRequest,
) -> Result<Template, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_template(&id, &request)
}

#[tauri::command]
pub fn delete_template(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_template(&id)
}

#[tauri::command]
pub fn create_template_step(
    state: State<AppState>,
    request: CreateTemplateStepRequest,
) -> Result<TemplateStep, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_template_step(&request)
}

#[tauri::command]
pub fn update_template_step(
    state: State<AppState>,
    id: String,
    request: UpdateTemplateStepRequest,
) -> Result<TemplateStep, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_template_step(&id, &request)
}

#[tauri::command]
pub fn delete_template_step(state: State<AppState>, id: String) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_template_step(&id)
}
