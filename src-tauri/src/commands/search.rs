use crate::{models::*, AppState};
use tauri::State;

const MAX_SEARCH_QUERY_LENGTH: usize = 1_000;
const MAX_FIND_LENGTH: usize = 10_000;
const MAX_REPLACE_LENGTH: usize = 100_000;

#[tauri::command]
pub fn global_search(
    query: String,
    scope: Option<Vec<String>>,
    state: State<AppState>,
) -> Result<Vec<SearchResult>, String> {
    if query.len() > MAX_SEARCH_QUERY_LENGTH {
        return Err(format!(
            "Search query too long (max {} characters)",
            MAX_SEARCH_QUERY_LENGTH
        ));
    }
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
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
    if find.len() > MAX_FIND_LENGTH {
        return Err(format!(
            "Find pattern too long (max {} characters)",
            MAX_FIND_LENGTH
        ));
    }
    if replace.len() > MAX_REPLACE_LENGTH {
        return Err(format!(
            "Replace text too long (max {} characters)",
            MAX_REPLACE_LENGTH
        ));
    }
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.find_replace_in_scenes(
        &find,
        &replace,
        case_sensitive.unwrap_or(false),
        whole_word.unwrap_or(false),
        chapter_id.as_deref(),
    )
}
