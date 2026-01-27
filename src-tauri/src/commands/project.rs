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
    *state
        .db
        .lock()
        .map_err(|_| "Database lock poisoned".to_string())? = Some(db);
    *state
        .current_project_path
        .lock()
        .map_err(|_| "Path lock poisoned".to_string())? = Some(path.clone());

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

    // Check for existing non-stale lock before opening
    if let Some(lock_info) = read_lock_file(&path) {
        if !is_lock_stale(&lock_info) {
            // Lock exists from a different process — allow same PID (re-open)
            if lock_info.pid != std::process::id() {
                return Err(format!(
                    "Project is locked by {} (PID {}). Close it there first or force acquire the lock.",
                    lock_info.machine_name, lock_info.pid
                ));
            }
        }
    }

    let db = Database::open(&path)?;
    let project = db.get_project()?;

    // Create lock file
    create_lock_file(&path)?;

    // Track file modification time
    if let Ok(metadata) = fs::metadata(&path) {
        if let Ok(modified) = metadata.modified() {
            *state
                .last_file_modified
                .lock()
                .map_err(|_| "Lock poisoned".to_string())? = Some(modified);
        }
    }

    *state
        .db
        .lock()
        .map_err(|_| "Database lock poisoned".to_string())? = Some(db);
    *state
        .current_project_path
        .lock()
        .map_err(|_| "Path lock poisoned".to_string())? = Some(path.clone());

    // Update recent projects
    update_recent_projects(&path, &project.title);

    Ok(project)
}

#[tauri::command]
pub fn close_project(state: State<AppState>) -> Result<(), String> {
    let is_demo = *state
        .is_demo
        .lock()
        .map_err(|_| "Lock poisoned".to_string())?;

    // Release database first, then file lock (consistent ordering)
    *state.db.lock().map_err(|_| "Lock poisoned".to_string())? = None;

    // Now release lock file and clear path
    let path = state
        .current_project_path
        .lock()
        .map_err(|_| "Lock poisoned".to_string())?
        .take();

    if let Some(ref path) = path {
        if is_demo {
            // Demo mode: delete the temp file and SQLite sidecar files
            remove_demo_files(path);
        } else {
            remove_lock_file(path);
        }
    }

    // Reset demo flag
    *state
        .is_demo
        .lock()
        .map_err(|_| "Lock poisoned".to_string())? = false;

    *state
        .last_file_modified
        .lock()
        .map_err(|_| "Lock poisoned".to_string())? = None;
    Ok(())
}

#[tauri::command]
pub fn get_project(state: State<AppState>) -> Result<Project, String> {
    let db = state.get_db()?;
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

    let db = state.get_db()?;
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

        if let Ok(json) = serde_json::to_string_pretty(&projects) {
            let _ = fs::write(&recent_file, json);
        }
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
    let has_lock = lock_info.as_ref().is_some_and(|info| !is_lock_stale(info));

    // Check for external modifications
    let is_modified_externally = {
        let last_modified = state
            .last_file_modified
            .lock()
            .map_err(|_| "Lock poisoned".to_string())?;
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

/// Checks the integrity of the currently open database file.
/// Returns Ok(true) if healthy, or an error if corrupted.
#[tauri::command]
pub fn check_database_integrity(state: State<AppState>) -> Result<bool, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;
    db.check_integrity()
}

/// Embedded demo project data (generated by `generate_seed_file` test).
const DEMO_DATA: &[u8] = include_bytes!("../../test-data.cahnon");

/// Removes the demo temp file and its SQLite sidecar files (WAL, SHM).
fn remove_demo_files(path: &Path) {
    let _ = fs::remove_file(path);
    // SQLite WAL/SHM files are named <dbfile>-wal and <dbfile>-shm
    let mut wal = path.as_os_str().to_owned();
    wal.push("-wal");
    let _ = fs::remove_file(PathBuf::from(&wal));
    let mut shm = path.as_os_str().to_owned();
    shm.push("-shm");
    let _ = fs::remove_file(PathBuf::from(&shm));
}

/// Opens the embedded demo project.
///
/// Writes the demo data to a temporary file and opens it as a normal project.
/// No lock file is created and the project is not added to recent projects.
/// The temporary file is deleted when the project is closed.
#[tauri::command]
pub fn open_demo_project(state: State<AppState>) -> Result<Project, String> {
    // Use a securely generated unique temp file to avoid predictable path attacks
    let temp_path = tempfile::Builder::new()
        .prefix("cahnon-demo-")
        .suffix(".cahnon")
        .tempfile()
        .map_err(|e| format!("Failed to create temp file: {}", e))?
        .into_temp_path()
        .keep()
        .map_err(|e| format!("Failed to persist temp file: {}", e))?;

    // Write embedded data to temp file
    fs::write(&temp_path, DEMO_DATA).map_err(|e| format!("Failed to write demo file: {}", e))?;

    let db = Database::open(&temp_path)?;
    let project = db.get_project()?;

    *state
        .db
        .lock()
        .map_err(|_| "Database lock poisoned".to_string())? = Some(db);
    *state
        .current_project_path
        .lock()
        .map_err(|_| "Path lock poisoned".to_string())? = Some(temp_path);
    *state
        .is_demo
        .lock()
        .map_err(|_| "Lock poisoned".to_string())? = true;

    Ok(project)
}

/// Returns whether the currently open project is the demo.
#[tauri::command]
pub fn get_is_demo(state: State<AppState>) -> Result<bool, String> {
    Ok(*state
        .is_demo
        .lock()
        .map_err(|_| "Lock poisoned".to_string())?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // --- get_lock_path ---

    #[test]
    fn test_get_lock_path() {
        let path = Path::new("/tmp/myproject.cahnon");
        let lock = get_lock_path(path);
        assert_eq!(lock, PathBuf::from("/tmp/myproject.cahnon.lock"));
    }

    #[test]
    fn test_get_lock_path_no_extension() {
        let path = Path::new("/tmp/myproject");
        let lock = get_lock_path(path);
        assert_eq!(lock, PathBuf::from("/tmp/myproject.cahnon.lock"));
    }

    #[test]
    fn test_get_lock_path_nested() {
        let path = Path::new("/home/user/docs/novel.cahnon");
        let lock = get_lock_path(path);
        assert_eq!(lock, PathBuf::from("/home/user/docs/novel.cahnon.lock"));
    }

    // --- is_lock_stale ---

    #[test]
    fn test_is_lock_stale_recent() {
        let lock = LockInfo {
            machine_name: "test".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            pid: 1234,
        };
        assert!(!is_lock_stale(&lock));
    }

    #[test]
    fn test_is_lock_stale_old() {
        let old_time = chrono::Utc::now() - chrono::Duration::minutes(10);
        let lock = LockInfo {
            machine_name: "test".to_string(),
            timestamp: old_time.to_rfc3339(),
            pid: 1234,
        };
        assert!(is_lock_stale(&lock));
    }

    #[test]
    fn test_is_lock_stale_exactly_5_minutes() {
        let boundary = chrono::Utc::now() - chrono::Duration::minutes(5);
        let lock = LockInfo {
            machine_name: "test".to_string(),
            timestamp: boundary.to_rfc3339(),
            pid: 1234,
        };
        // 5 minutes is NOT > 5, so not stale
        assert!(!is_lock_stale(&lock));
    }

    #[test]
    fn test_is_lock_stale_just_over_5_minutes() {
        let boundary = chrono::Utc::now() - chrono::Duration::minutes(6);
        let lock = LockInfo {
            machine_name: "test".to_string(),
            timestamp: boundary.to_rfc3339(),
            pid: 1234,
        };
        assert!(is_lock_stale(&lock));
    }

    #[test]
    fn test_is_lock_stale_invalid_timestamp() {
        let lock = LockInfo {
            machine_name: "test".to_string(),
            timestamp: "not-a-date".to_string(),
            pid: 1234,
        };
        // Invalid timestamp → considered stale
        assert!(is_lock_stale(&lock));
    }

    #[test]
    fn test_is_lock_stale_empty_timestamp() {
        let lock = LockInfo {
            machine_name: "test".to_string(),
            timestamp: "".to_string(),
            pid: 1234,
        };
        assert!(is_lock_stale(&lock));
    }

    // --- is_conflict_file ---

    #[test]
    fn test_is_conflict_file_dropbox_pattern() {
        assert!(is_conflict_file("novel (conflicted copy).cahnon", "novel"));
    }

    #[test]
    fn test_is_conflict_file_google_drive_pattern() {
        assert!(is_conflict_file("novel (1).cahnon", "novel"));
    }

    #[test]
    fn test_is_conflict_file_onedrive_pattern() {
        assert!(is_conflict_file("novel-conflict.cahnon", "novel"));
    }

    #[test]
    fn test_is_conflict_file_no_cahnon_extension() {
        assert!(!is_conflict_file("novel (conflicted copy).txt", "novel"));
    }

    #[test]
    fn test_is_conflict_file_same_name_no_conflict() {
        assert!(!is_conflict_file("novel.cahnon", "novel"));
    }

    #[test]
    fn test_is_conflict_file_different_stem() {
        assert!(!is_conflict_file("other (conflicted copy).cahnon", "novel"));
    }

    #[test]
    fn test_is_conflict_file_parenthesis_conflict() {
        assert!(is_conflict_file("novel(conflict).cahnon", "novel"));
    }

    #[test]
    fn test_is_conflict_file_icloud_numbered() {
        // iCloud pattern: "novel (2).cahnon"
        assert!(is_conflict_file("novel (2).cahnon", "novel"));
    }

    // --- create_lock_file / read_lock_file / remove_lock_file ---

    #[test]
    fn test_create_and_read_lock_file() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test.cahnon");
        fs::write(&project_path, "").unwrap();

        create_lock_file(&project_path).unwrap();

        let lock = read_lock_file(&project_path);
        assert!(lock.is_some());
        let lock = lock.unwrap();
        assert!(!lock.machine_name.is_empty());
        assert!(lock.pid > 0);
        assert!(!is_lock_stale(&lock));
    }

    #[test]
    fn test_read_lock_file_nonexistent() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("nonexistent.cahnon");
        let lock = read_lock_file(&project_path);
        assert!(lock.is_none());
    }

    #[test]
    fn test_remove_lock_file() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test.cahnon");
        fs::write(&project_path, "").unwrap();

        create_lock_file(&project_path).unwrap();
        let lock_path = get_lock_path(&project_path);
        assert!(lock_path.exists());

        remove_lock_file(&project_path);
        assert!(!lock_path.exists());
    }

    #[test]
    fn test_remove_lock_file_nonexistent_is_noop() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test.cahnon");
        // Should not panic
        remove_lock_file(&project_path);
    }

    // --- find_conflict_files ---

    #[test]
    fn test_find_conflict_files_none() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("novel.cahnon");
        fs::write(&project_path, "").unwrap();

        let conflicts = find_conflict_files(&project_path);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_find_conflict_files_with_conflicts() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("novel.cahnon");
        fs::write(&project_path, "").unwrap();

        // Create conflict files
        fs::write(temp.path().join("novel (conflicted copy).cahnon"), "").unwrap();
        fs::write(temp.path().join("novel-conflict.cahnon"), "").unwrap();
        // Non-conflict file
        fs::write(temp.path().join("other.cahnon"), "").unwrap();

        let conflicts = find_conflict_files(&project_path);
        assert_eq!(conflicts.len(), 2);
    }

    #[test]
    fn test_find_conflict_files_no_parent() {
        // Root path has no parent → empty vec
        let path = Path::new("novel.cahnon");
        // This should return empty (parent might be "" which is current dir, not None)
        let _conflicts = find_conflict_files(path);
        // Just checking it doesn't panic
    }

    // --- lock round-trip ---

    #[test]
    fn test_lock_roundtrip_create_read_stale_remove() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("project.cahnon");
        fs::write(&project_path, "").unwrap();

        // No lock initially
        assert!(read_lock_file(&project_path).is_none());

        // Create lock
        create_lock_file(&project_path).unwrap();

        // Read lock - should be fresh
        let lock = read_lock_file(&project_path).unwrap();
        assert!(!is_lock_stale(&lock));
        assert_eq!(lock.pid, std::process::id());

        // Remove lock
        remove_lock_file(&project_path);
        assert!(read_lock_file(&project_path).is_none());
    }

    #[test]
    fn test_read_lock_file_corrupted_json() {
        let temp = TempDir::new().unwrap();
        let project_path = temp.path().join("test.cahnon");
        let lock_path = get_lock_path(&project_path);

        // Write invalid JSON to lock file
        fs::write(&lock_path, "not valid json {{{").unwrap();

        let lock = read_lock_file(&project_path);
        assert!(lock.is_none()); // Should return None for invalid JSON
    }
}
