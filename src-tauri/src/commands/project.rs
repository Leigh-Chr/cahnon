//! Project management commands.
//!
//! Handles project creation, opening, closing, and file locking for
//! multi-device safety.

use crate::{database::Database, models::*, AppState};
use hostname;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::validation::{
    sanitize_multiline_text, sanitize_text, MAX_AUTHOR_LENGTH, MAX_DESCRIPTION_LENGTH,
    MAX_TITLE_LENGTH,
};

/// Information about a file lock, used to detect if a project is open elsewhere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub machine_name: String,
    pub timestamp: String,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub has_lock: bool,
    pub lock_info: Option<LockInfo>,
    pub is_modified_externally: bool,
    pub has_conflict_files: Vec<String>,
}

/// Creates a new Cahnon project at the specified path.
///
/// Initializes the SQLite database with the full schema and creates
/// the project record. Adds `.cahnon` extension if not present.
#[tauri::command]
pub fn create_project(
    path: String,
    request: CreateProjectRequest,
    state: State<AppState>,
) -> Result<Project, String> {
    // Validate and sanitize title
    let title = sanitize_text(&request.title, MAX_TITLE_LENGTH);
    if title.is_empty() {
        return Err("Project title cannot be empty".to_string());
    }

    // Sanitize optional fields
    let author = request
        .author
        .as_ref()
        .map(|a| sanitize_text(a, MAX_AUTHOR_LENGTH));
    let description = request
        .description
        .as_ref()
        .map(|d| sanitize_multiline_text(d, MAX_DESCRIPTION_LENGTH));

    // Create sanitized request
    let sanitized_request = CreateProjectRequest {
        title,
        author,
        description,
    };

    let path = PathBuf::from(&path);

    // Ensure .cahnon extension
    let path = if path.extension().is_none_or(|ext| ext != "cahnon") {
        path.with_extension("cahnon")
    } else {
        path
    };

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let db = Database::create(&path)?;
    let project = db.create_project(&sanitized_request)?;

    // Store database and path
    *state.db.lock().unwrap() = Some(db);
    *state.current_project_path.lock().unwrap() = Some(path.clone());

    // Update recent projects
    update_recent_projects(&path, &project.title);

    Ok(project)
}

/// Opens an existing Cahnon project file.
///
/// Runs any pending migrations and loads the project data.
/// Updates the recent projects list.
#[tauri::command]
pub fn open_project(path: String, state: State<AppState>) -> Result<Project, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("Project file does not exist".to_string());
    }

    let db = Database::open(&path)?;
    let project = db.get_project()?;

    // Create lock file
    create_lock_file(&path)?;

    // Track file modification time
    if let Ok(metadata) = fs::metadata(&path) {
        if let Ok(modified) = metadata.modified() {
            *state.last_file_modified.lock().unwrap() = Some(modified);
        }
    }

    *state.db.lock().unwrap() = Some(db);
    *state.current_project_path.lock().unwrap() = Some(path.clone());

    // Update recent projects
    update_recent_projects(&path, &project.title);

    Ok(project)
}

#[tauri::command]
pub fn close_project(state: State<AppState>) -> Result<(), String> {
    // Release lock file
    if let Some(ref path) = *state.current_project_path.lock().unwrap() {
        remove_lock_file(path);
    }

    *state.db.lock().unwrap() = None;
    *state.current_project_path.lock().unwrap() = None;
    *state.last_file_modified.lock().unwrap() = None;
    Ok(())
}

#[tauri::command]
pub fn get_project(state: State<AppState>) -> Result<Project, String> {
    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.get_project()
}

#[tauri::command]
pub fn update_project(
    request: UpdateProjectRequest,
    state: State<AppState>,
) -> Result<Project, String> {
    // Sanitize inputs
    let sanitized_request = UpdateProjectRequest {
        title: request.title.map(|t| sanitize_text(&t, MAX_TITLE_LENGTH)),
        author: request.author.map(|a| sanitize_text(&a, MAX_AUTHOR_LENGTH)),
        description: request
            .description
            .map(|d| sanitize_multiline_text(&d, MAX_DESCRIPTION_LENGTH)),
        word_target: request.word_target, // Integer doesn't need sanitization
        daily_word_target: request.daily_word_target, // Integer doesn't need sanitization
    };

    // Validate title if provided
    if let Some(ref title) = sanitized_request.title {
        if title.is_empty() {
            return Err("Project title cannot be empty".to_string());
        }
    }

    let db = state.db.lock().unwrap();
    let db = db.as_ref().ok_or("No project open")?;
    db.update_project(&sanitized_request)
}

#[tauri::command]
pub fn get_recent_projects() -> Result<Vec<RecentProject>, String> {
    let config_dir = directories::ProjectDirs::from("com", "cahnon", "Cahnon")
        .ok_or("Could not find config directory")?
        .config_dir()
        .to_path_buf();

    let recent_file = config_dir.join("recent_projects.json");

    if !recent_file.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&recent_file).map_err(|e| e.to_string())?;
    let projects: Vec<RecentProject> = serde_json::from_str(&content).unwrap_or_default();

    // Filter out projects that no longer exist
    let valid_projects: Vec<RecentProject> = projects
        .into_iter()
        .filter(|p| PathBuf::from(&p.path).exists())
        .collect();

    Ok(valid_projects)
}

fn update_recent_projects(path: &Path, title: &str) {
    if let Some(config_dir) = directories::ProjectDirs::from("com", "cahnon", "Cahnon") {
        let config_path = config_dir.config_dir();
        let _ = fs::create_dir_all(config_path);

        let recent_file = config_path.join("recent_projects.json");
        let mut projects: Vec<RecentProject> = if recent_file.exists() {
            fs::read_to_string(&recent_file)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // Remove existing entry for this path
        let path_str = path.to_string_lossy().to_string();
        projects.retain(|p| p.path != path_str);

        // Add new entry at the front
        projects.insert(
            0,
            RecentProject {
                path: path_str,
                title: title.to_string(),
                last_opened: chrono::Utc::now().to_rfc3339(),
            },
        );

        // Keep only 10 most recent
        projects.truncate(10);

        let _ = fs::write(
            &recent_file,
            serde_json::to_string_pretty(&projects).unwrap_or_default(),
        );
    }
}

// Lock file management
fn get_lock_path(project_path: &Path) -> PathBuf {
    let mut lock_path = project_path.to_path_buf();
    lock_path.set_extension("cahnon.lock");
    lock_path
}

fn create_lock_file(project_path: &Path) -> Result<(), String> {
    let lock_path = get_lock_path(project_path);
    let machine_name = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let lock_info = LockInfo {
        machine_name,
        timestamp: chrono::Utc::now().to_rfc3339(),
        pid: std::process::id(),
    };

    let content = serde_json::to_string_pretty(&lock_info).map_err(|e| e.to_string())?;
    fs::write(&lock_path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn remove_lock_file(project_path: &Path) {
    let lock_path = get_lock_path(project_path);
    let _ = fs::remove_file(lock_path);
}

fn read_lock_file(project_path: &Path) -> Option<LockInfo> {
    let lock_path = get_lock_path(project_path);
    if !lock_path.exists() {
        return None;
    }

    fs::read_to_string(&lock_path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
}

fn is_lock_stale(lock_info: &LockInfo) -> bool {
    // Consider lock stale if older than 5 minutes
    if let Ok(lock_time) = chrono::DateTime::parse_from_rfc3339(&lock_info.timestamp) {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(lock_time);
        duration.num_minutes() > 5
    } else {
        true
    }
}

fn find_conflict_files(project_path: &Path) -> Vec<String> {
    let parent = match project_path.parent() {
        Some(p) => p,
        None => return Vec::new(),
    };
    let stem = match project_path.file_stem() {
        Some(s) => s.to_string_lossy(),
        None => return Vec::new(),
    };
    let entries = match fs::read_dir(parent) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    entries
        .flatten()
        .filter_map(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if is_conflict_file(&file_name, &stem) {
                Some(entry.path().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect()
}

fn is_conflict_file(file_name: &str, stem: &str) -> bool {
    if !file_name.ends_with(".cahnon") {
        return false;
    }
    // Check for common conflict patterns from cloud services
    // Dropbox: "file (conflicted copy).ext"
    // Google Drive: "file (1).ext"
    // OneDrive: "file-conflict.ext"
    // iCloud: "file 2.ext"
    file_name.contains(&format!("{} (", stem))
        || file_name.contains(&format!("{}-conflict", stem))
        || file_name.contains(&format!("{}(conflict", stem))
}

#[tauri::command]
pub fn check_file_status(path: String, state: State<AppState>) -> Result<FileStatus, String> {
    let path = PathBuf::from(&path);

    if !path.exists() {
        return Err("Project file does not exist".to_string());
    }

    let lock_info = read_lock_file(&path);
    let has_lock = lock_info.is_some() && !is_lock_stale(lock_info.as_ref().unwrap());

    // Check for external modifications
    let is_modified_externally = {
        let last_modified = state.last_file_modified.lock().unwrap();
        if let Some(last_mod) = *last_modified {
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    modified > last_mod
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    };

    let conflict_files = find_conflict_files(&path);

    Ok(FileStatus {
        has_lock,
        lock_info,
        is_modified_externally,
        has_conflict_files: conflict_files,
    })
}

#[tauri::command]
pub fn acquire_lock(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    create_lock_file(&path)
}

#[tauri::command]
pub fn release_lock(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    remove_lock_file(&path);
    Ok(())
}

#[tauri::command]
pub fn force_acquire_lock(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    // Remove existing lock and create new one
    remove_lock_file(&path);
    create_lock_file(&path)
}
