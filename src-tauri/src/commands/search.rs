use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn global_search(
    query: String,
    scope: Option<Vec<String>>,
    state: State<AppState>,
) -> Result<Vec<SearchResult>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.global_search(&query, scope)
}
