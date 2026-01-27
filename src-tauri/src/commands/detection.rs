use crate::models::Issue;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn run_all_detections(state: State<'_, AppState>) -> Result<Vec<Issue>, String> {
    let db = state.get_db()?;
    let db = db.as_ref().ok_or("No project open")?;

    let detected = db.run_all_detections()?;

    // Delete old auto-detected issues
    db.delete_auto_detected_issues()?;

    // Create new issues from detections
    let mut issues = Vec::new();
    for d in &detected {
        let issue = db.create_issue_from_detection(d)?;
        issues.push(issue);
    }

    Ok(issues)
}
