use crate::{models::*, AppState};
use tauri::State;

#[tauri::command]
pub fn get_word_counts(state: State<AppState>) -> Result<WordCounts, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.get_word_counts()
}
