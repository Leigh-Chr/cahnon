//! Data models for Cahnon.
//!
//! This module defines all data structures used throughout the application,
//! including entities (Project, Chapter, Scene, etc.) and request/response
//! types for Tauri commands.

use serde::{Deserialize, Serialize};

// ============================================================================
// Project
// ============================================================================

/// A writing project containing chapters, scenes, and bible entries.
///
/// Projects are stored as `.cahnon` files (SQLite databases). Each project
/// contains a single manuscript with chapters and scenes, plus a bible
/// for tracking characters, locations, and other story elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub word_target: Option<i32>,
    pub daily_word_target: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub word_target: Option<i32>,
    pub daily_word_target: Option<i32>,
}

// ============================================================================
// Chapter
// ============================================================================

/// A chapter in the manuscript, containing an ordered list of scenes.
///
/// Chapters provide the top-level organization of the manuscript. They can
/// have a status (draft, revision, done) and optional notes for planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChapterRequest {
    pub title: String,
    pub summary: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChapterRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
    pub position: Option<i32>,
}

// ============================================================================
// Scene
// ============================================================================

/// A scene within a chapter, the basic unit of writing.
///
/// Scenes contain the actual narrative text (stored as TipTap HTML) and can
/// be linked to bible entries (canonical associations), timeline events,
/// and plot arcs. They support rich metadata including POV character, tags,
/// status, and revision-specific fields like tension level and setup/payoff
/// relationships.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub chapter_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub text: String,
    pub status: String,
    pub pov: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub todos: Option<String>,
    pub word_target: Option<i32>,
    pub time_point: Option<String>,
    pub time_start: Option<String>,
    pub time_end: Option<String>,
    pub on_timeline: bool,
    pub position: i32,
    // Revision fields
    pub pov_goal: Option<String>,
    pub has_conflict: Option<bool>,
    pub has_change: Option<bool>,
    pub tension: Option<String>,
    pub setup_for_scene_id: Option<String>,
    pub payoff_of_scene_id: Option<String>,
    pub revision_notes: Option<String>,
    pub revision_checklist: Option<String>,
    pub word_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSceneRequest {
    pub chapter_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSceneRequest {
    pub title: Option<String>,
    pub summary: Option<String>,
    pub text: Option<String>,
    pub status: Option<String>,
    pub pov: Option<String>,
    pub tags: Option<String>,
    pub notes: Option<String>,
    pub todos: Option<String>,
    pub word_target: Option<i32>,
    pub time_point: Option<String>,
    pub time_start: Option<String>,
    pub time_end: Option<String>,
    pub on_timeline: Option<bool>,
    pub position: Option<i32>,
    // Revision fields
    pub pov_goal: Option<String>,
    pub has_conflict: Option<bool>,
    pub has_change: Option<bool>,
    pub tension: Option<String>,
    pub setup_for_scene_id: Option<String>,
    pub payoff_of_scene_id: Option<String>,
    pub revision_notes: Option<String>,
    pub revision_checklist: Option<String>,
}

// ============================================================================
// Bible Entry
// ============================================================================

/// An entry in the story bible (knowledge base).
///
/// Bible entries store canonical information about story elements:
/// - `character`: People in the story
/// - `location`: Places and settings
/// - `object`: Important items
/// - `faction`: Groups and organizations
/// - `concept`: Rules, magic systems, etc.
/// - `glossary`: Terms and definitions
///
/// Entries can have aliases (alternate names), relationships to other entries,
/// and custom fields defined as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibleEntry {
    pub id: String,
    pub entry_type: String, // character, location, object, faction, concept, glossary
    pub name: String,
    pub aliases: Option<String>,
    pub short_description: Option<String>,
    pub full_description: Option<String>,
    pub status: String,
    pub tags: Option<String>,
    pub image_path: Option<String>,
    pub notes: Option<String>,
    pub todos: Option<String>,
    pub color: Option<String>,
    pub custom_fields: Option<String>, // JSON string
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBibleEntryRequest {
    pub entry_type: String,
    pub name: String,
    pub aliases: Option<String>,
    pub short_description: Option<String>,
    pub full_description: Option<String>,
    pub status: Option<String>,
    pub tags: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBibleEntryRequest {
    pub name: Option<String>,
    pub aliases: Option<String>,
    pub short_description: Option<String>,
    pub full_description: Option<String>,
    pub status: Option<String>,
    pub tags: Option<String>,
    pub image_path: Option<String>,
    pub notes: Option<String>,
    pub todos: Option<String>,
    pub color: Option<String>,
    pub custom_fields: Option<String>,
}

// ============================================================================
// Canonical Association
// ============================================================================

/// A link between a scene and a bible entry.
///
/// Canonical associations are the "source of truth" for which story elements
/// appear in which scenes. Unlike automatic detection, these are explicitly
/// created by the writer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalAssociation {
    pub id: String,
    pub scene_id: String,
    pub bible_entry_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssociationRequest {
    pub scene_id: String,
    pub bible_entry_id: String,
}

// ============================================================================
// Search
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub result_type: String, // scene, chapter, bible_entry
    pub id: String,
    pub title: String,
    pub snippet: Option<String>,
    pub parent_id: Option<String>,
    pub parent_title: Option<String>,
}

// ============================================================================
// Word Count Stats
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordCounts {
    pub total: i32,
    pub by_chapter: Vec<ChapterWordCount>,
    pub by_status: Vec<StatusWordCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterWordCount {
    pub chapter_id: String,
    pub chapter_title: String,
    pub word_count: i32,
    pub scene_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusWordCount {
    pub status: String,
    pub word_count: i32,
    pub scene_count: i32,
}

// ============================================================================
// Recent Project
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub title: String,
    pub last_opened: String,
}

// ============================================================================
// Arc
// ============================================================================

/// A plot arc or narrative thread running through the story.
///
/// Arcs help track thematic or plot threads across multiple scenes.
/// Each arc has a name, description, stakes, and status. Scenes can be
/// linked to multiple arcs to show which threads are advanced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arc {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub stakes: Option<String>,
    /// Key characters involved in this arc (Bible entry IDs)
    #[serde(deserialize_with = "deserialize_characters", default)]
    pub characters: Vec<String>,
    pub status: String,
    pub color: Option<String>,
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Custom deserializer that accepts both old format (comma-separated string or null)
/// and new format (array of strings) for backward compatibility with snapshots.
fn deserialize_characters<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct CharactersVisitor;

    impl<'de> de::Visitor<'de> for CharactersVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string, array of strings, or null")
        }

        fn visit_unit<E: de::Error>(self) -> Result<Vec<String>, E> {
            Ok(Vec::new())
        }

        fn visit_none<E: de::Error>(self) -> Result<Vec<String>, E> {
            Ok(Vec::new())
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Vec<String>, E> {
            Ok(value
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect())
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<String>, A::Error> {
            let mut vec = Vec::new();
            while let Some(val) = seq.next_element::<String>()? {
                vec.push(val);
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_any(CharactersVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArcRequest {
    pub name: String,
    pub description: Option<String>,
    pub stakes: Option<String>,
    pub status: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateArcRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub stakes: Option<String>,
    pub status: Option<String>,
    pub color: Option<String>,
}

// ============================================================================
// Event (Timeline)
// ============================================================================

/// A timeline event in the story's chronology.
///
/// Events mark points or periods in the story's internal timeline. They can
/// be linked to scenes to establish when scenes occur. The timeline view
/// can detect conflicts (e.g., a character in two places at once).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub time_point: Option<String>,
    pub time_start: Option<String>,
    pub time_end: Option<String>,
    pub event_type: String,
    pub importance: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub description: Option<String>,
    pub time_point: Option<String>,
    pub time_start: Option<String>,
    pub time_end: Option<String>,
    pub event_type: Option<String>,
    pub importance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEventRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub time_point: Option<String>,
    pub time_start: Option<String>,
    pub time_end: Option<String>,
    pub event_type: Option<String>,
    pub importance: Option<String>,
}

// ============================================================================
// Bible Relationship
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibleRelationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub note: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibleRelationshipWithEntry {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub note: Option<String>,
    pub status: String,
    pub created_at: String,
    pub related_entry_id: String,
    pub related_entry_type: String,
    pub related_entry_name: String,
    pub related_entry_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBibleRelationshipRequest {
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub note: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBibleRelationshipRequest {
    pub relationship_type: Option<String>,
    pub note: Option<String>,
    pub status: Option<String>,
}

// ============================================================================
// Scene History
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneHistoryEntry {
    pub id: String,
    pub scene_id: String,
    pub text: String,
    pub created_at: String,
}

// ============================================================================
// Template
// ============================================================================

/// A narrative template (story structure).
///
/// Templates provide predefined story structures like the 3-Act Structure,
/// Save the Cat, Hero's Journey, or 7-Point Story Structure. Each template
/// has steps that scenes can be assigned to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub is_active: bool,
    pub is_builtin: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// A step within a narrative template.
///
/// Steps represent story beats like "Inciting Incident" or "All Is Lost".
/// Each step has a typical position (0.0-1.0 as percentage through the story)
/// and can have scenes assigned to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateStep {
    pub id: String,
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub typical_position: f64,
    pub color: Option<String>,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateStepRequest {
    pub template_id: String,
    pub name: String,
    pub description: Option<String>,
    pub typical_position: Option<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTemplateStepRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub typical_position: Option<f64>,
    pub color: Option<String>,
    pub position: Option<i32>,
}

// ============================================================================
// Annotation
// ============================================================================

/// A comment or note attached to a text range in a scene.
///
/// Annotations are used during revision to mark issues, questions, or
/// notes about specific passages. They track character offsets in the
/// scene text and have a status (open/resolved).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub scene_id: String,
    pub start_offset: i32,
    pub end_offset: i32,
    pub annotation_type: String,
    pub content: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnnotationRequest {
    pub scene_id: String,
    pub start_offset: i32,
    pub end_offset: i32,
    pub annotation_type: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAnnotationRequest {
    pub content: Option<String>,
    pub status: Option<String>,
}

// ============================================================================
// Issue
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub issue_type: String,
    pub title: String,
    pub description: Option<String>,
    pub severity: String,
    pub status: String,
    pub resolution_note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub issue_type: String,
    pub title: String,
    pub description: Option<String>,
    pub severity: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateIssueRequest {
    pub status: Option<String>,
    pub resolution_note: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
}

// ============================================================================
// Snapshot
// ============================================================================

/// A named point-in-time backup of project data.
///
/// Snapshots capture the project state at a specific moment, allowing
/// writers to preserve milestones and restore previous versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub snapshot_type: String,
    pub data: String,
    pub created_at: String,
}

// ============================================================================
// Cut
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cut {
    pub id: String,
    pub scene_id: Option<String>,
    pub text: String,
    pub created_at: String,
}

// ============================================================================
// Scene Split/Merge
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitSceneRequest {
    pub scene_id: String,
    pub split_position: i32,
    pub new_scene_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitSceneResult {
    pub first_scene: Scene,
    pub second_scene: Scene,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeScenesRequest {
    pub scene_ids: Vec<String>,
}

// ============================================================================
// Timeline Conflicts
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineConflict {
    pub conflict_type: String,
    pub description: String,
    pub scene_ids: Vec<String>,
    pub character_id: Option<String>,
    pub character_name: Option<String>,
    pub time_point: Option<String>,
}

// ============================================================================
// Version Diff
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiff {
    pub text_a: String,
    pub text_b: String,
}

// ============================================================================
// Junction Row (generic row for snapshot serialization)
// ============================================================================

/// Generic representation of a junction/link table row for snapshot serialization.
/// Each junction table has an id, two foreign key fields, and a created_at timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunctionRow {
    pub id: String,
    pub field_a: String,
    pub field_b: String,
    pub created_at: Option<String>,
}

/// Issue junction rows (no created_at column).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueJunctionRow {
    pub id: String,
    pub field_a: String,
    pub field_b: String,
}

// ============================================================================
// Detection
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedIssue {
    pub issue_type: String,
    pub title: String,
    pub description: String,
    pub severity: String,
    pub scene_ids: Vec<String>,
    pub bible_entry_ids: Vec<String>,
}

// ============================================================================
// Character Thread
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterThread {
    pub bible_entry_id: String,
    pub character_name: String,
    pub scenes: Vec<CharacterThreadScene>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterThreadScene {
    pub scene_id: String,
    pub scene_title: String,
    pub chapter_title: String,
    pub chapter_id: String,
    pub position_index: i32,
    pub pov: Option<String>,
    pub tension: Option<String>,
    pub summary: Option<String>,
    pub other_characters: Vec<String>,
    pub gap_from_previous: i32,
}

// ============================================================================
// Scene Health (Narrative Health Indicator)
// ============================================================================

/// Health score for a single scene, computed from multiple checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneHealth {
    pub scene_id: String,
    pub score: f64,
    pub checks: Vec<HealthCheck>,
}

/// A single health check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub passed: bool,
    pub weight: f64,
    pub label: String,
}

// ============================================================================
// World State at Scene N (NarrativeContext)
// ============================================================================

/// Complete narrative world state at a given scene point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub scene_id: String,
    pub character_presences: Vec<CharacterPresence>,
    pub open_setups: Vec<OpenSetup>,
    pub active_arcs: Vec<ActiveArcState>,
}

/// Tracking a character's presence up to a point in the manuscript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterPresence {
    pub bible_entry_id: String,
    pub name: String,
    pub appearance_count: i32,
    pub last_scene_id: String,
    pub last_scene_title: String,
    pub gap_scenes: i32,
    pub present_here: bool,
}

/// A setup scene that has no payoff yet at this point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenSetup {
    pub scene_id: String,
    pub scene_title: String,
    pub setup_for_scene_id: String,
}

/// An arc's state at a given point in the manuscript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveArcState {
    pub arc_id: String,
    pub arc_name: String,
    pub color: Option<String>,
    pub scenes_before: i32,
    pub scenes_total: i32,
    pub last_scene_title: String,
}

// ============================================================================
// Impact Awareness (Delete Preview)
// ============================================================================

/// Preview of what would be affected by a deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactPreview {
    pub items: Vec<ImpactItem>,
    pub total_count: i32,
}

/// A single impact item describing what would be affected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactItem {
    pub impact_type: String,
    pub description: String,
    pub entity_id: Option<String>,
    pub entity_name: Option<String>,
}

// ============================================================================
// File Locking & Status
// ============================================================================

/// Information about a file lock, used to detect if a project is open elsewhere.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub machine_name: String,
    pub timestamp: String,
    pub pid: u32,
}

/// Status of a project file on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    pub has_lock: bool,
    pub lock_info: Option<LockInfo>,
    pub is_modified_externally: bool,
    pub has_conflict_files: Vec<String>,
}

// ============================================================================
// Import
// ============================================================================

/// Result of a structured import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub chapters_created: i32,
    pub scenes_created: i32,
}
