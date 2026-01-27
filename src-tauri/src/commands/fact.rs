use crate::models::{CreateFactRequest, Fact, FactCharacter, UpdateFactRequest};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn create_fact(state: State<'_, AppState>, request: CreateFactRequest) -> Result<Fact, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.create_fact(&request)
}

#[tauri::command]
pub fn get_facts(state: State<'_, AppState>) -> Result<Vec<Fact>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_facts()
}

#[tauri::command]
pub fn get_fact(state: State<'_, AppState>, id: String) -> Result<Fact, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_fact(&id)
}

#[tauri::command]
pub fn update_fact(
    state: State<'_, AppState>,
    id: String,
    request: UpdateFactRequest,
) -> Result<Fact, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.update_fact(&id, &request)
}

#[tauri::command]
pub fn delete_fact(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_fact(&id)
}

#[tauri::command]
pub fn get_facts_for_scene(
    state: State<'_, AppState>,
    scene_id: String,
) -> Result<Vec<Fact>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_facts_for_scene(&scene_id)
}

#[tauri::command]
pub fn link_character_to_fact(
    state: State<'_, AppState>,
    fact_id: String,
    bible_entry_id: String,
    learned_in_scene_id: Option<String>,
) -> Result<FactCharacter, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.link_character_to_fact(&fact_id, &bible_entry_id, learned_in_scene_id.as_deref())
}

#[tauri::command]
pub fn unlink_character_from_fact(
    state: State<'_, AppState>,
    fact_id: String,
    bible_entry_id: String,
) -> Result<(), String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.unlink_character_from_fact(&fact_id, &bible_entry_id)
}

#[tauri::command]
pub fn get_fact_characters(
    state: State<'_, AppState>,
    fact_id: String,
) -> Result<Vec<FactCharacter>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_fact_characters(&fact_id)
}

#[tauri::command]
pub fn get_character_knowledge_at_scene(
    state: State<'_, AppState>,
    bible_entry_id: String,
    scene_id: String,
) -> Result<Vec<Fact>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_character_knowledge_at_scene(&bible_entry_id, &scene_id)
}
