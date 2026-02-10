//! Scene management commands.
//!
//! Handles CRUD operations for scenes, plus split, merge, move, and reorder.

use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_CONTENT_LENGTH, MAX_NOTES_LENGTH,
    MAX_SCENE_TITLE_LENGTH, MAX_SYNOPSIS_LENGTH,
};
use crate::{models::*, AppState};
use tauri::State;

/// Splits a scene into two at the specified character position.
///
/// The first scene retains content before the split point, the second
/// gets content after. Preserves metadata on the first scene.
#[tauri::command]
pub fn split_scene(
    request: SplitSceneRequest,
    state: State<AppState>,
) -> Result<SplitSceneResult, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    let (first, second) = db.split_scene(
        &request.scene_id,
        request.split_position,
        request.new_scene_title.as_deref(),
    )?;
    Ok(SplitSceneResult {
        first_scene: first,
        second_scene: second,
    })
}

/// Merges multiple scenes into one.
///
/// Concatenates text from all scenes in order. The first scene is
/// kept and updated; others are deleted.
#[tauri::command]
pub fn merge_scenes(request: MergeScenesRequest, state: State<AppState>) -> Result<Scene, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.merge_scenes(&request.scene_ids)
}

/// Creates a new scene in a chapter.
#[tauri::command]
pub fn create_scene(request: CreateSceneRequest, state: State<AppState>) -> Result<Scene, String> {
    let title = sanitize_text(&request.title, MAX_SCENE_TITLE_LENGTH);
    if title.is_empty() {
        return Err("Scene title cannot be empty".to_string());
    }

    let sanitized_request = CreateSceneRequest {
        chapter_id: request.chapter_id,
        title,
        summary: request
            .summary
            .map(|s| sanitize_multiline_text(&s, MAX_SYNOPSIS_LENGTH)),
        position: request.position,
    };

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.create_scene(&sanitized_request)
}

/// Gets all scenes in a chapter, ordered by position.
#[tauri::command]
pub fn get_scenes(chapter_id: String, state: State<AppState>) -> Result<Vec<Scene>, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_scenes(&chapter_id)
}

/// Gets a single scene by ID.
#[tauri::command]
pub fn get_scene(id: String, state: State<AppState>) -> Result<Scene, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.get_scene(&id)
}

/// Updates a scene. Creates a version history entry if text changes.
#[tauri::command]
pub fn update_scene(
    id: String,
    request: UpdateSceneRequest,
    state: State<AppState>,
) -> Result<Scene, String> {
    let sanitized_request = UpdateSceneRequest {
        title: request
            .title
            .map(|t| sanitize_text(&t, MAX_SCENE_TITLE_LENGTH)),
        summary: request
            .summary
            .map(|s| sanitize_multiline_text(&s, MAX_SYNOPSIS_LENGTH)),
        text: request
            .text
            .map(|t| sanitize_multiline_text(&t, MAX_CONTENT_LENGTH)),
        status: request
            .status
            .map(|s| sanitize_text(&s, MAX_SCENE_TITLE_LENGTH)),
        pov: request
            .pov
            .map(|p| sanitize_text(&p, MAX_SCENE_TITLE_LENGTH)),
        tags: request.tags.map(|t| sanitize_text(&t, MAX_SYNOPSIS_LENGTH)),
        notes: request
            .notes
            .map(|n| sanitize_multiline_text(&n, MAX_NOTES_LENGTH)),
        todos: request
            .todos
            .map(|t| sanitize_multiline_text(&t, MAX_NOTES_LENGTH)),
        word_target: request.word_target,
        time_point: request
            .time_point
            .map(|t| sanitize_text(&t, MAX_SCENE_TITLE_LENGTH)),
        time_start: request
            .time_start
            .map(|t| sanitize_text(&t, MAX_SCENE_TITLE_LENGTH)),
        time_end: request
            .time_end
            .map(|t| sanitize_text(&t, MAX_SCENE_TITLE_LENGTH)),
        on_timeline: request.on_timeline,
        position: request.position,
        pov_goal: request
            .pov_goal
            .map(|p| sanitize_text(&p, MAX_SYNOPSIS_LENGTH)),
        has_dramatic_conflict: request.has_dramatic_conflict,
        has_change: request.has_change,
        tension: request
            .tension
            .map(|t| sanitize_text(&t, MAX_SCENE_TITLE_LENGTH)),
        setup_for_scene_id: request.setup_for_scene_id,
        payoff_of_scene_id: request.payoff_of_scene_id,
        revision_notes: request
            .revision_notes
            .map(|n| sanitize_multiline_text(&n, MAX_NOTES_LENGTH)),
        revision_checklist: request
            .revision_checklist
            .map(|c| sanitize_multiline_text(&c, MAX_NOTES_LENGTH)),
    };

    if let Some(ref title) = sanitized_request.title {
        if title.is_empty() {
            return Err("Scene title cannot be empty".to_string());
        }
    }

    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.update_scene(&id, &sanitized_request)
}

/// Soft-deletes a scene (moves to trash for 30 days).
#[tauri::command]
pub fn delete_scene(id: String, state: State<AppState>) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.delete_scene(&id)
}

/// Reorders scenes within a chapter.
#[tauri::command]
pub fn reorder_scenes(
    chapter_id: String,
    ids: Vec<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.reorder_scenes(&chapter_id, &ids)
}

/// Moves a scene to a different chapter.
#[tauri::command]
pub fn move_scene_to_chapter(
    scene_id: String,
    target_chapter_id: String,
    position: i32,
    state: State<AppState>,
) -> Result<Scene, String> {
    let guard = state.get_db()?;
    let db = guard.db.as_ref().ok_or("No project open")?;
    db.move_scene_to_chapter(&scene_id, &target_chapter_id, position)
}
