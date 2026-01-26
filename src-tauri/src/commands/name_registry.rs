use crate::validation::{sanitize_text, MAX_TITLE_LENGTH};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_name_registry_entry(
    request: CreateNameRegistryRequest,
    state: State<AppState>,
) -> Result<NameRegistryEntry, String> {
    let canonical_name = sanitize_text(&request.canonical_name, MAX_TITLE_LENGTH);
    if canonical_name.is_empty() {
        return Err("Canonical name cannot be empty".to_string());
    }

    let sanitized_request = CreateNameRegistryRequest {
        canonical_name,
        name_type: request.name_type,
        bible_entry_id: request.bible_entry_id,
        aliases: request.aliases.map(|a| sanitize_text(&a, MAX_TITLE_LENGTH)),
    };

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_name_registry_entry(&sanitized_request)
}

#[tauri::command]
pub fn get_name_registry_entries(
    name_type: Option<String>,
    state: State<AppState>,
) -> Result<Vec<NameRegistryEntry>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_name_registry_entries(name_type.as_deref())
}

#[tauri::command]
pub fn get_name_registry_entry(
    id: String,
    state: State<AppState>,
) -> Result<NameRegistryEntry, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_name_registry_entry(&id)
}

#[tauri::command]
pub fn update_name_registry_entry(
    id: String,
    request: UpdateNameRegistryRequest,
    state: State<AppState>,
) -> Result<NameRegistryEntry, String> {
    let sanitized_request = UpdateNameRegistryRequest {
        canonical_name: request
            .canonical_name
            .map(|n| sanitize_text(&n, MAX_TITLE_LENGTH)),
        name_type: request.name_type,
        bible_entry_id: request.bible_entry_id,
        aliases: request.aliases.map(|a| sanitize_text(&a, MAX_TITLE_LENGTH)),
        is_confirmed: request.is_confirmed,
    };

    if let Some(ref name) = sanitized_request.canonical_name {
        if name.is_empty() {
            return Err("Canonical name cannot be empty".to_string());
        }
    }

    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.update_name_registry_entry(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_name_registry_entry(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_name_registry_entry(&id)
}

#[tauri::command]
pub fn get_name_mentions_by_scene(
    scene_id: String,
    state: State<AppState>,
) -> Result<Vec<NameMention>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_name_mentions_by_scene(&scene_id)
}

#[tauri::command]
pub fn get_name_mentions_by_registry(
    registry_id: String,
    state: State<AppState>,
) -> Result<Vec<NameMention>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_name_mentions_by_registry(&registry_id)
}

#[tauri::command]
pub fn update_name_mention(
    id: String,
    request: UpdateNameMentionRequest,
    state: State<AppState>,
) -> Result<NameMention, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.update_name_mention(&id, &request)
}

#[tauri::command]
pub fn delete_name_mention(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_name_mention(&id)
}

#[tauri::command]
pub fn scan_names(state: State<AppState>) -> Result<(i32, i32), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.scan_names()
}

#[tauri::command]
pub fn merge_name_entries(
    keep_id: String,
    merge_id: String,
    state: State<AppState>,
) -> Result<NameRegistryEntry, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.merge_name_entries(&keep_id, &merge_id)
}
