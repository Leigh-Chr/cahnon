//! Scene management commands.
//!
//! Handles CRUD operations for scenes, plus split, merge, move, and reorder.

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
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
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
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.merge_scenes(&request.scene_ids)
}

/// Creates a new scene in a chapter.
#[tauri::command]
pub fn create_scene(request: CreateSceneRequest, state: State<AppState>) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.create_scene(&request)
}

/// Gets all scenes in a chapter, ordered by position.
#[tauri::command]
pub fn get_scenes(chapter_id: String, state: State<AppState>) -> Result<Vec<Scene>, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scenes(&chapter_id)
}

/// Gets a single scene by ID.
#[tauri::command]
pub fn get_scene(id: String, state: State<AppState>) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_scene(&id)
}

/// Updates a scene. Creates a version history entry if text changes.
#[tauri::command]
pub fn update_scene(
    id: String,
    request: UpdateSceneRequest,
    state: State<AppState>,
) -> Result<Scene, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_scene(&id, &request)
}

/// Soft-deletes a scene (moves to trash for 30 days).
#[tauri::command]
pub fn delete_scene(id: String, state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.delete_scene(&id)
}

/// Reorders scenes within a chapter.
#[tauri::command]
pub fn reorder_scenes(
    chapter_id: String,
    ids: Vec<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
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
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.move_scene_to_chapter(&scene_id, &target_chapter_id, position)
}
