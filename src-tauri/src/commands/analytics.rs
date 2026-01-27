//! Analytics commands — thin wrappers for stats, health, detection,
//! world state, character threads, and impact previews.

use crate::models::*;
use crate::AppState;
use tauri::State;

// ---- Word counts / Stats ----

#[tauri::command]
pub fn get_word_counts(state: State<AppState>) -> Result<WordCounts, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_word_counts()
}

// ---- Scene Health ----

#[tauri::command]
pub fn get_scene_health_batch(state: State<'_, AppState>) -> Result<Vec<SceneHealth>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_scene_health_batch()
}

// ---- Auto-detection ----

#[tauri::command]
pub fn run_all_detections(state: State<'_, AppState>) -> Result<Vec<Issue>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;

    let detected = db.run_all_detections()?;

    // Capture existing resolved/ignored states before deletion
    let previous_states = db.get_auto_issue_states()?;

    // Delete old auto-detected issues and create new ones
    db.delete_auto_detected_issues()?;

    let mut issues = Vec::new();
    for d in &detected {
        let issue = db.create_issue_from_detection(d)?;

        // Restore previous status if the user had resolved/ignored this issue
        let key = format!("auto_{}:{}", d.issue_type, d.title);
        if let Some((prev_status, prev_note)) = previous_states.get(&key) {
            if prev_status != "open" {
                let updated = db.update_issue(
                    &issue.id,
                    &crate::models::UpdateIssueRequest {
                        status: Some(prev_status.clone()),
                        resolution_note: prev_note.clone(),
                        title: None,
                        description: None,
                        severity: None,
                    },
                )?;
                issues.push(updated);
                continue;
            }
        }

        issues.push(issue);
    }

    Ok(issues)
}

// ---- World State & Character Threads ----

#[tauri::command]
pub fn get_world_state_at_scene(
    state: State<'_, AppState>,
    scene_id: String,
) -> Result<WorldState, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_world_state_at_scene(&scene_id)
}

#[tauri::command]
pub fn get_character_thread(
    state: State<'_, AppState>,
    bible_entry_id: String,
) -> Result<CharacterThread, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_character_thread(&bible_entry_id)
}

// ---- Impact Previews ----

#[tauri::command]
pub fn preview_delete_scene_impact(
    state: State<'_, AppState>,
    scene_id: String,
) -> Result<ImpactPreview, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.preview_delete_scene_impact(&scene_id)
}

#[tauri::command]
pub fn preview_delete_bible_entry_impact(
    state: State<'_, AppState>,
    bible_entry_id: String,
) -> Result<ImpactPreview, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.preview_delete_bible_entry_impact(&bible_entry_id)
}

#[tauri::command]
pub fn preview_delete_chapter_impact(
    state: State<'_, AppState>,
    chapter_id: String,
) -> Result<ImpactPreview, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.preview_delete_chapter_impact(&chapter_id)
}
