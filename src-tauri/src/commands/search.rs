use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn global_search(
    query: String,
    scope: Option<Vec<String>>,
    state: State<AppState>,
) -> Result<Vec<SearchResult>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.global_search(&query, scope)
}

#[tauri::command]
pub fn find_replace_in_scenes(
    find: String,
    replace: String,
    case_sensitive: Option<bool>,
    whole_word: Option<bool>,
    chapter_id: Option<String>,
    state: State<AppState>,
) -> Result<i32, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.find_replace_in_scenes(
        &find,
        &replace,
        case_sensitive.unwrap_or(false),
        whole_word.unwrap_or(false),
        chapter_id.as_deref(),
    )
}
