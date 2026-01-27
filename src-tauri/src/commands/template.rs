use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_SYNOPSIS_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn get_templates(state: State<AppState>) -> Result<Vec<Template>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_templates()
}

#[tauri::command]
pub fn get_template_steps(
    template_id: String,
    state: State<AppState>,
) -> Result<Vec<TemplateStep>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_template_steps(&template_id)
}

#[tauri::command]
pub fn set_active_template(template_id: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.set_active_template(&template_id)
}

#[tauri::command]
pub fn assign_scene_to_step(
    scene_id: String,
    step_id: String,
    state: State<AppState>,
) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.assign_scene_to_step(&scene_id, &step_id)
}

#[tauri::command]
pub fn get_scene_step(
    scene_id: String,
    state: State<AppState>,
) -> Result<Option<TemplateStep>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_scene_step(&scene_id)
}

#[tauri::command]
pub fn init_builtin_templates(state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.init_builtin_templates()
}

#[tauri::command]
pub fn create_template(
    request: CreateTemplateRequest,
    state: State<AppState>,
) -> Result<Template, String> {
    let name = sanitize_text(&request.name, MAX_TITLE_LENGTH);
    if name.is_empty() {
        return Err("Template name cannot be empty".to_string());
    }

    let sanitized_request = CreateTemplateRequest { name };

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.create_template(&sanitized_request)
}

#[tauri::command]
pub fn update_template(
    id: String,
    request: UpdateTemplateRequest,
    state: State<AppState>,
) -> Result<Template, String> {
    let sanitized_request = UpdateTemplateRequest {
        name: request.name.map(|n| sanitize_text(&n, MAX_TITLE_LENGTH)),
    };

    if let Some(ref name) = sanitized_request.name {
        if name.is_empty() {
            return Err("Template name cannot be empty".to_string());
        }
    }

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.update_template(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_template(id: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.delete_template(&id)
}

#[tauri::command]
pub fn create_template_step(
    request: CreateTemplateStepRequest,
    state: State<AppState>,
) -> Result<TemplateStep, String> {
    let name = sanitize_text(&request.name, MAX_TITLE_LENGTH);
    if name.is_empty() {
        return Err("Template step name cannot be empty".to_string());
    }

    if let Some(pos) = request.typical_position {
        if pos.is_nan() || pos.is_infinite() || !(0.0..=1.0).contains(&pos) {
            return Err("typical_position must be between 0.0 and 1.0".to_string());
        }
    }

    let sanitized_request = CreateTemplateStepRequest {
        template_id: request.template_id,
        name,
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH)),
        typical_position: request.typical_position,
        color: request.color,
    };

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.create_template_step(&sanitized_request)
}

#[tauri::command]
pub fn update_template_step(
    id: String,
    request: UpdateTemplateStepRequest,
    state: State<AppState>,
) -> Result<TemplateStep, String> {
    if let Some(pos) = request.typical_position {
        if pos.is_nan() || pos.is_infinite() || !(0.0..=1.0).contains(&pos) {
            return Err("typical_position must be between 0.0 and 1.0".to_string());
        }
    }

    let sanitized_request = UpdateTemplateStepRequest {
        name: request.name.map(|n| sanitize_text(&n, MAX_TITLE_LENGTH)),
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH)),
        typical_position: request.typical_position,
        color: request.color,
        position: request.position,
    };

    if let Some(ref name) = sanitized_request.name {
        if name.is_empty() {
            return Err("Template step name cannot be empty".to_string());
        }
    }

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.update_template_step(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_template_step(id: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.delete_template_step(&id)
}
