use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_ALIASES_PER_ENTRY,
    MAX_BIBLE_ENTRY_DESCRIPTION_LENGTH, MAX_BIBLE_ENTRY_NAME_LENGTH, MAX_NOTES_LENGTH,
    MAX_SYNOPSIS_LENGTH, MAX_TITLE_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn create_bible_entry(
    request: CreateBibleEntryRequest,
    state: State<AppState>,
) -> Result<BibleEntry, String> {
    let name = sanitize_text(&request.name, MAX_BIBLE_ENTRY_NAME_LENGTH);
    if name.is_empty() {
        return Err("Bible entry name cannot be empty".to_string());
    }

    // Validate alias count
    if let Some(ref aliases) = request.aliases {
        let alias_count = aliases.split(',').filter(|a| !a.trim().is_empty()).count();
        if alias_count > MAX_ALIASES_PER_ENTRY {
            return Err(format!(
                "Too many aliases ({alias_count}). Maximum is {MAX_ALIASES_PER_ENTRY}."
            ));
        }
    }

    let sanitized_request = CreateBibleEntryRequest {
        entry_type: sanitize_text(&request.entry_type, MAX_TITLE_LENGTH),
        name,
        aliases: request
            .aliases
            .map(|a| sanitize_text(&a, MAX_SYNOPSIS_LENGTH)),
        short_description: request
            .short_description
            .map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH)),
        full_description: request
            .full_description
            .map(|d| sanitize_multiline_text(&d, MAX_BIBLE_ENTRY_DESCRIPTION_LENGTH)),
        status: request.status.map(|s| sanitize_text(&s, MAX_TITLE_LENGTH)),
        tags: request.tags.map(|t| sanitize_text(&t, MAX_SYNOPSIS_LENGTH)),
        color: request.color,
    };

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.create_bible_entry(&sanitized_request)
}

#[tauri::command]
pub fn get_bible_entries(
    entry_type: Option<String>,
    state: State<AppState>,
) -> Result<Vec<BibleEntry>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_bible_entries(entry_type.as_deref())
}

#[tauri::command]
pub fn get_bible_entry(id: String, state: State<AppState>) -> Result<BibleEntry, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_bible_entry(&id)
}

#[tauri::command]
pub fn update_bible_entry(
    id: String,
    request: UpdateBibleEntryRequest,
    state: State<AppState>,
) -> Result<BibleEntry, String> {
    // Validate custom_fields is valid JSON and reasonable size if provided
    if let Some(ref cf) = request.custom_fields {
        const MAX_CUSTOM_FIELDS_SIZE: usize = 64 * 1024; // 64 KB
        if cf.len() > MAX_CUSTOM_FIELDS_SIZE {
            return Err(format!(
                "custom_fields too large ({} bytes). Maximum is {} bytes.",
                cf.len(),
                MAX_CUSTOM_FIELDS_SIZE
            ));
        }
        if serde_json::from_str::<serde_json::Value>(cf).is_err() {
            return Err("custom_fields must be valid JSON".to_string());
        }
    }

    // Validate image_path against path traversal
    if let Some(ref path) = request.image_path {
        if path.contains("..") {
            return Err("image_path must not contain path traversal sequences".to_string());
        }
    }

    let sanitized_request = UpdateBibleEntryRequest {
        name: request
            .name
            .map(|n| sanitize_text(&n, MAX_BIBLE_ENTRY_NAME_LENGTH)),
        aliases: request
            .aliases
            .map(|a| sanitize_text(&a, MAX_SYNOPSIS_LENGTH)),
        short_description: request
            .short_description
            .map(|d| sanitize_multiline_text(&d, MAX_SYNOPSIS_LENGTH)),
        full_description: request
            .full_description
            .map(|d| sanitize_multiline_text(&d, MAX_BIBLE_ENTRY_DESCRIPTION_LENGTH)),
        status: request.status.map(|s| sanitize_text(&s, MAX_TITLE_LENGTH)),
        tags: request.tags.map(|t| sanitize_text(&t, MAX_SYNOPSIS_LENGTH)),
        image_path: request.image_path,
        notes: request
            .notes
            .map(|n| sanitize_multiline_text(&n, MAX_NOTES_LENGTH)),
        todos: request
            .todos
            .map(|t| sanitize_multiline_text(&t, MAX_NOTES_LENGTH)),
        color: request.color,
        custom_fields: request.custom_fields,
    };

    if let Some(ref name) = sanitized_request.name {
        if name.is_empty() {
            return Err("Bible entry name cannot be empty".to_string());
        }
    }

    // Validate alias count
    if let Some(ref aliases) = sanitized_request.aliases {
        let alias_count = aliases.split(',').filter(|a| !a.trim().is_empty()).count();
        if alias_count > MAX_ALIASES_PER_ENTRY {
            return Err(format!(
                "Too many aliases ({alias_count}). Maximum is {MAX_ALIASES_PER_ENTRY}."
            ));
        }
    }

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.update_bible_entry(&id, &sanitized_request)
}

#[tauri::command]
pub fn delete_bible_entry(id: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.delete_bible_entry(&id)
}

#[tauri::command]
pub fn search_bible(query: String, state: State<AppState>) -> Result<Vec<BibleEntry>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.search_bible(&query)
}
