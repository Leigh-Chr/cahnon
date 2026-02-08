//! Cahnon - A desktop application for fiction writers.
//!
//! This crate provides the Tauri backend for Cahnon, handling:
//! - `SQLite` database operations for project storage
//! - Tauri commands for IPC with the Svelte frontend
//! - File locking for multi-device safety
//!
//! # Architecture
//!
//! The backend is organized into:
//! - `database`: `SQLite` operations and schema management
//! - `models`: Data structures shared with the frontend
//! - `commands`: Tauri command handlers grouped by domain
//! - `validation`: Input sanitization and validation

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

mod commands;
mod database;
mod models;
pub mod validation;

#[cfg(test)]
mod tests;

use database::Database;
use tauri::menu::{MenuBuilder, PredefinedMenuItem, SubmenuBuilder};
use tauri::Emitter;

/// Internal project state protected by a single mutex.
#[derive(Default)]
pub struct ProjectState {
    /// The currently open project database, if any.
    pub db: Option<Database>,
    /// Path to the currently open `.cahnon` file.
    pub current_project_path: Option<PathBuf>,
    /// Last known modification time of the project file for detecting external changes.
    pub last_file_modified: Option<SystemTime>,
    /// Whether the currently open project is the embedded demo.
    pub is_demo: bool,
}

/// Global application state managed by Tauri.
///
/// This state is shared across all Tauri commands via `State<AppState>`.
/// A single mutex protects all project-related fields to prevent
/// inconsistent state across multiple locks.
pub struct AppState {
    pub inner: Mutex<ProjectState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(ProjectState::default()),
        }
    }
}

impl AppState {
    /// Acquires the project state lock, handling mutex poisoning gracefully.
    pub fn get_state(&self) -> Result<std::sync::MutexGuard<'_, ProjectState>, String> {
        self.inner
            .lock()
            .map_err(|_| "State lock is poisoned".to_string())
    }

    /// Convenience: acquires the lock and returns a guard that provides
    /// access to the database. Most commands use this pattern.
    pub fn get_db(&self) -> Result<std::sync::MutexGuard<'_, ProjectState>, String> {
        self.get_state()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // Build native application menu
            // Note: Accelerators are intentionally omitted from custom items.
            // Keyboard shortcuts are handled in the frontend (Layout.svelte) to
            // support user-customizable bindings via keyboardShortcuts settings.
            // PredefinedMenuItems keep their OS-level accelerators (Undo, Copy, etc.)
            // since they operate directly on the focused DOM element.
            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "new_project",
                    "New Project",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "open_project",
                    "Open Project…",
                    true,
                    None::<&str>,
                )?)
                .separator()
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "save",
                    "Save",
                    true,
                    None::<&str>,
                )?)
                .separator()
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "export",
                    "Export…",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "import",
                    "Import…",
                    true,
                    None::<&str>,
                )?)
                .separator()
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "close_project",
                    "Close Project",
                    true,
                    None::<&str>,
                )?)
                .build()?;

            let edit_menu = SubmenuBuilder::new(app, "Edit")
                .item(&PredefinedMenuItem::undo(app, None)?)
                .item(&PredefinedMenuItem::redo(app, None)?)
                .separator()
                .item(&PredefinedMenuItem::cut(app, None)?)
                .item(&PredefinedMenuItem::copy(app, None)?)
                .item(&PredefinedMenuItem::paste(app, None)?)
                .item(&PredefinedMenuItem::select_all(app, None)?)
                .build()?;

            let view_menu = SubmenuBuilder::new(app, "View")
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "view_editor",
                    "Editor",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "view_corkboard",
                    "Corkboard",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "view_timeline",
                    "Timeline",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "view_bible",
                    "Codex",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "view_issues",
                    "Continuity",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "view_dashboard",
                    "Dashboard",
                    true,
                    None::<&str>,
                )?)
                .separator()
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "toggle_outline",
                    "Toggle Outline",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "toggle_context_panel",
                    "Toggle Context Panel",
                    true,
                    None::<&str>,
                )?)
                .separator()
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "focus_mode",
                    "Focus Mode",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "review_grid",
                    "Review Grid",
                    true,
                    None::<&str>,
                )?)
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "quick_open",
                    "Quick Open",
                    true,
                    None::<&str>,
                )?)
                .build()?;

            let help_menu = SubmenuBuilder::new(app, "Help")
                .item(&tauri::menu::MenuItem::with_id(
                    app,
                    "about",
                    "About Cahnon",
                    true,
                    None::<&str>,
                )?)
                .build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&view_menu)
                .item(&help_menu)
                .build()?;

            app.set_menu(menu)?;

            // Handle file passed as CLI argument (Phase 8.1)
            let args: Vec<String> = std::env::args().collect();
            if let Some(file_arg) = args.get(1) {
                if file_arg.ends_with(".cahnon") {
                    let path = file_arg.to_string();
                    let handle = app.handle().clone();
                    // Emit after setup completes
                    std::thread::spawn(move || {
                        // Small delay to let frontend initialize
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        let _ = handle.emit("open-file", path);
                    });
                }
            }

            Ok(())
        })
        .on_menu_event(|app, event| {
            let _ = app.emit("menu-event", event.id().0.as_str());
        })
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::project::create_project,
            commands::project::open_project,
            commands::project::close_project,
            commands::project::get_project,
            commands::project::update_project,
            commands::project::get_recent_projects,
            commands::project::check_file_status,
            commands::project::acquire_lock,
            commands::project::release_lock,
            commands::project::force_acquire_lock,
            commands::project::check_database_integrity,
            commands::project::open_demo_project,
            commands::project::get_is_demo,
            // Chapter commands
            commands::chapter::create_chapter,
            commands::chapter::get_chapters,
            commands::chapter::get_chapter,
            commands::chapter::update_chapter,
            commands::chapter::delete_chapter,
            commands::chapter::reorder_chapters,
            // Scene commands
            commands::scene::create_scene,
            commands::scene::get_scenes,
            commands::scene::get_scene,
            commands::scene::update_scene,
            commands::scene::delete_scene,
            commands::scene::reorder_scenes,
            commands::scene::move_scene_to_chapter,
            commands::scene::split_scene,
            commands::scene::merge_scenes,
            // Bible commands
            commands::bible::create_bible_entry,
            commands::bible::get_bible_entries,
            commands::bible::get_bible_entry,
            commands::bible::update_bible_entry,
            commands::bible::delete_bible_entry,
            commands::bible::search_bible,
            // Association commands
            commands::association::create_association,
            commands::association::get_scene_associations,
            commands::association::delete_association,
            commands::association::get_bible_entry_scenes,
            commands::association::auto_link_bible_entries,
            // Search commands
            commands::search::global_search,
            commands::search::find_replace_in_scenes,
            // Stats commands
            commands::analytics::get_word_counts,
            // Arc commands
            commands::arc::create_arc,
            commands::arc::get_arcs,
            commands::arc::get_arc,
            commands::arc::update_arc,
            commands::arc::delete_arc,
            commands::arc::link_scene_to_arc,
            commands::arc::unlink_scene_from_arc,
            commands::arc::get_scene_arcs,
            commands::arc::set_arc_characters,
            commands::arc::get_arc_scenes,
            commands::arc::get_character_arcs,
            // Event commands
            commands::event::create_event,
            commands::event::get_events,
            commands::event::get_event,
            commands::event::update_event,
            commands::event::delete_event,
            commands::event::get_timeline_scenes,
            commands::event::detect_timeline_conflicts,
            commands::event::link_scene_to_event,
            commands::event::unlink_scene_from_event,
            commands::event::get_scene_events,
            commands::event::get_event_scenes,
            commands::event::link_bible_entry_to_event,
            commands::event::unlink_bible_entry_from_event,
            commands::event::get_event_bible_entries,
            commands::event::get_bible_entry_events,
            // Relationship commands
            commands::relationship::create_bible_relationship,
            commands::relationship::get_bible_relationships,
            commands::relationship::update_bible_relationship,
            commands::relationship::delete_bible_relationship,
            // History commands
            commands::history::get_scene_history,
            commands::history::restore_scene_version,
            commands::history::compare_scene_versions,
            // Template commands
            commands::template::get_templates,
            commands::template::get_template_steps,
            commands::template::set_active_template,
            commands::template::assign_scene_to_step,
            commands::template::get_scene_step,
            commands::template::init_builtin_templates,
            commands::template::create_template,
            commands::template::update_template,
            commands::template::delete_template,
            commands::template::create_template_step,
            commands::template::update_template_step,
            commands::template::delete_template_step,
            // Annotation commands
            commands::annotation::create_annotation,
            commands::annotation::get_annotations,
            commands::annotation::update_annotation,
            commands::annotation::delete_annotation,
            commands::annotation::batch_update_annotation_offsets,
            // Issue commands
            commands::issue::create_issue,
            commands::issue::get_issues,
            commands::issue::get_issue,
            commands::issue::update_issue,
            commands::issue::delete_issue,
            commands::issue::link_scene_to_issue,
            commands::issue::unlink_scene_from_issue,
            commands::issue::get_issue_scenes,
            commands::issue::get_scene_issues,
            commands::issue::link_bible_entry_to_issue,
            commands::issue::unlink_bible_entry_from_issue,
            commands::issue::get_issue_bible_entries,
            commands::issue::get_bible_entry_issues,
            // Snapshot commands
            commands::snapshot::create_snapshot,
            commands::snapshot::get_snapshots,
            commands::snapshot::get_snapshot,
            commands::snapshot::delete_snapshot,
            commands::snapshot::restore_snapshot,
            commands::snapshot::cleanup_expired_snapshots,
            commands::snapshot::get_snapshot_scenes,
            commands::snapshot::restore_scene_from_snapshot,
            // Export commands
            commands::export::export_markdown,
            commands::export::export_plain_text,
            commands::export::export_json_backup,
            commands::export::export_outline,
            commands::export::export_bible,
            commands::export::export_timeline,
            // CSV Export commands
            commands::export_csv::export_bible_csv,
            commands::export_csv::export_timeline_csv,
            commands::export_csv::export_review_grid_csv,
            commands::export_csv::export_stats_csv,
            // Import commands
            commands::import::import_markdown_as_scene,
            commands::import::import_markdown_structured,
            commands::import::import_text_as_scene,
            commands::import::import_json_backup,
            // Trash commands
            commands::trash::get_deleted_scenes,
            commands::trash::restore_scene,
            commands::trash::get_deleted_chapters,
            commands::trash::restore_chapter,
            commands::trash::duplicate_scene,
            commands::trash::purge_expired_trash,
            // Cut commands
            commands::cut::create_cut,
            commands::cut::get_cuts,
            commands::cut::delete_cut,
            // Analytics commands (stats, health, detection, world state, impact)
            commands::analytics::run_all_detections,
            commands::analytics::get_scene_health_batch,
            commands::analytics::get_world_state_at_scene,
            commands::analytics::get_character_thread,
            commands::analytics::preview_delete_scene_impact,
            commands::analytics::preview_delete_bible_entry_impact,
            commands::analytics::preview_delete_chapter_impact,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
