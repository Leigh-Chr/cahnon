/**
 * Cahnon API Types
 *
 * This module contains all TypeScript type definitions for the Tauri API layer.
 * Types are organized by domain for better maintainability.
 *
 * @module
 */

// =============================================================================
// Project & Manuscript Types
// =============================================================================

/**
 * A writing project containing chapters, scenes, and bible entries.
 * Projects are stored as `.cahnon` SQLite database files.
 */
export interface Project {
	id: string;
	title: string;
	author: string | null;
	description: string | null;
	word_target: number | null;
	daily_word_target: number | null;
	created_at: string;
	updated_at: string;
}

/**
 * A chapter in the manuscript, containing an ordered list of scenes.
 */
export interface Chapter {
	id: string;
	title: string;
	summary: string | null;
	status: string;
	notes: string | null;
	position: number;
	created_at: string;
	updated_at: string;
}

/**
 * A scene within a chapter, the basic unit of writing in Cahnon.
 *
 * Scenes contain narrative text (as TipTap HTML) and can be linked to
 * bible entries, timeline events, and plot arcs. They include rich
 * metadata for both writing (POV, tags, notes) and revision (tension,
 * setup/payoff relationships).
 */
export interface Scene {
	id: string;
	chapter_id: string;
	title: string;
	summary: string | null;
	/** Scene content as TipTap/ProseMirror HTML */
	text: string;
	/** Status: draft, revision, done */
	status: string;
	/** Point-of-view character (by name or bible entry reference) */
	pov: string | null;
	/** Comma-separated tags */
	tags: string | null;
	notes: string | null;
	/** Inline TODOs for the scene */
	todos: string | null;
	word_target: number | null;
	/** Timeline position (single point) */
	time_point: string | null;
	/** Timeline range start */
	time_start: string | null;
	/** Timeline range end */
	time_end: string | null;
	/** Whether to show on timeline view */
	on_timeline: boolean;
	position: number;
	// Revision fields
	pov_goal: string | null;
	has_conflict: boolean | null;
	has_change: boolean | null;
	/** Tension level: low, medium, high */
	tension: string | null;
	setup_for_scene_id: string | null;
	payoff_of_scene_id: string | null;
	revision_notes: string | null;
	revision_checklist: string | null;
	/** Cached word count, updated on each text save */
	word_count: number;
	created_at: string;
	updated_at: string;
}

/** Scene version history entry */
export interface SceneHistoryEntry {
	id: string;
	scene_id: string;
	text: string;
	created_at: string;
}

export interface RecentProject {
	path: string;
	title: string;
	last_opened: string;
}

// =============================================================================
// Bible Types
// =============================================================================

/**
 * An entry in the story bible (knowledge base).
 *
 * Bible entries store canonical information about story elements:
 * characters, locations, objects, factions, concepts, and glossary terms.
 */
export interface BibleEntry {
	id: string;
	/** Type: character, location, object, faction, concept, glossary */
	entry_type: string;
	name: string;
	/** Comma-separated alternate names */
	aliases: string | null;
	short_description: string | null;
	full_description: string | null;
	status: string;
	tags: string | null;
	image_path: string | null;
	notes: string | null;
	todos: string | null;
	/** Display color for UI elements */
	color: string | null;
	/** JSON string of custom fields */
	custom_fields: string | null;
	created_at: string;
	updated_at: string;
	deleted_at: string | null;
}

export interface BibleRelationship {
	id: string;
	source_id: string;
	target_id: string;
	relationship_type: string;
	note: string | null;
	status: string;
	created_at: string;
}

export interface BibleRelationshipWithEntry extends BibleRelationship {
	related_entry_id: string;
	related_entry_type: string;
	related_entry_name: string;
	related_entry_description: string | null;
}

// =============================================================================
// Arc & Timeline Types
// =============================================================================

export interface Arc {
	id: string;
	name: string;
	description: string | null;
	stakes: string | null;
	/** Key characters involved (Bible entry IDs) */
	characters: string[];
	status: string;
	color: string | null;
	position: number;
	created_at: string;
	updated_at: string;
}

export interface TimelineEvent {
	id: string;
	title: string;
	description: string | null;
	time_point: string | null;
	time_start: string | null;
	time_end: string | null;
	event_type: string;
	importance: string;
	created_at: string;
	updated_at: string;
}

export interface TimelineConflict {
	conflict_type: string;
	description: string;
	scene_ids: string[];
	character_id: string | null;
	character_name: string | null;
	time_point: string | null;
}

// =============================================================================
// Content Types (Templates, Annotations, Issues)
// =============================================================================

export interface Template {
	id: string;
	name: string;
	is_active: boolean;
	is_builtin: boolean;
	created_at: string;
	updated_at: string;
}

export interface TemplateStep {
	id: string;
	template_id: string;
	name: string;
	description: string | null;
	typical_position: number;
	color: string | null;
	position: number;
}

export interface Annotation {
	id: string;
	scene_id: string;
	start_offset: number;
	end_offset: number;
	annotation_type: string;
	content: string;
	status: string;
	created_at: string;
	updated_at: string;
}

export interface Snapshot {
	id: string;
	name: string;
	description: string | null;
	snapshot_type: string;
	data: string;
	created_at: string;
}

export interface Cut {
	id: string;
	scene_id: string | null;
	text: string;
	created_at: string;
}

/**
 * A consistency issue or problem in the manuscript.
 *
 * Issues can be auto-detected (timeline conflicts, TBD in done scenes)
 * or manually created (bible contradictions, continuity errors).
 */
export interface Issue {
	id: string;
	/** Type: timeline_conflict, tbd_in_done, orphan_mention, bible_contradiction, continuity_error */
	issue_type: string;
	title: string;
	description: string | null;
	/** Severity: info, warning, error */
	severity: string;
	/** Status: open, resolved, ignored */
	status: string;
	resolution_note: string | null;
	created_at: string;
	updated_at: string;
}

// =============================================================================
// Search & Stats Types
// =============================================================================

export interface SearchResult {
	result_type: string;
	id: string;
	title: string;
	snippet: string | null;
	parent_id: string | null;
	parent_title: string | null;
}

export interface WordCounts {
	total: number;
	by_chapter: Array<{
		chapter_id: string;
		chapter_title: string;
		word_count: number;
		scene_count: number;
	}>;
	by_status: Array<{
		status: string;
		word_count: number;
		scene_count: number;
	}>;
}

// =============================================================================
// File & Lock Types
// =============================================================================

export interface LockInfo {
	machine_name: string;
	timestamp: string;
	pid: number;
}

export interface FileStatus {
	has_lock: boolean;
	lock_info: LockInfo | null;
	is_modified_externally: boolean;
	has_conflict_files: string[];
}

// =============================================================================
// Import/Export Types
// =============================================================================

export interface ImportResult {
	chapters_created: number;
	scenes_created: number;
}

export interface VersionDiff {
	text_a: string;
	text_b: string;
}

// =============================================================================
// Name Registry Types
// =============================================================================

export interface NameRegistryEntry {
	id: string;
	canonical_name: string;
	/** Type: character, location */
	name_type: string;
	bible_entry_id: string | null;
	/** Comma-separated aliases */
	aliases: string | null;
	is_confirmed: boolean;
	created_at: string;
	updated_at: string;
}

export interface NameMention {
	id: string;
	name_registry_id: string;
	scene_id: string;
	mention_text: string;
	start_offset: number;
	end_offset: number;
	/** Status: pending, accepted, ignored */
	status: string;
	created_at: string;
}

// =============================================================================
// Saved Filter Types
// =============================================================================

export interface SavedFilter {
	id: string;
	name: string;
	/** Type: outline, corkboard, timeline */
	filter_type: string;
	/** JSON-encoded filter configuration */
	filter_data: string;
	created_at: string;
	updated_at: string;
}

// =============================================================================
// Writing Session Types
// =============================================================================

export interface WritingSession {
	id: string;
	date: string;
	words_start: number;
	words_end: number;
	duration_minutes: number;
	scenes_edited: string;
	created_at: string;
}

// =============================================================================
// Fact / Revelation Types
// =============================================================================

export interface Fact {
	id: string;
	content: string;
	category: string;
	revealed_in_scene_id: string | null;
	status: string;
	created_at: string;
	updated_at: string;
}

export interface FactCharacter {
	id: string;
	fact_id: string;
	bible_entry_id: string;
	learned_in_scene_id: string | null;
	created_at: string;
}

// =============================================================================
// Name Registry Scan Types
// =============================================================================

export interface AssociationSuggestion {
	scene_id: string;
	bible_entry_id: string;
	bible_entry_name: string;
	scene_title: string;
}

export interface ScanResult {
	new_entries: number;
	new_mentions: number;
	suggestions: AssociationSuggestion[];
}

// =============================================================================
// Scene Health Types (Narrative GPS)
// =============================================================================

export interface SceneHealth {
	scene_id: string;
	score: number;
	checks: HealthCheck[];
}

export interface HealthCheck {
	name: string;
	passed: boolean;
	weight: number;
	label: string;
}

// =============================================================================
// World State Types (Narrative Context)
// =============================================================================

export interface WorldState {
	scene_id: string;
	character_presences: CharacterPresence[];
	character_knowledge: CharacterKnowledgeState[];
	open_setups: OpenSetup[];
	active_arcs: ActiveArcState[];
	dramatic_irony: DramaticIronyItem[];
	location_history: LocationHistoryItem[];
}

export interface CharacterPresence {
	bible_entry_id: string;
	name: string;
	appearance_count: number;
	last_scene_id: string;
	last_scene_title: string;
	gap_scenes: number;
	present_here: boolean;
}

export interface CharacterKnowledgeState {
	bible_entry_id: string;
	name: string;
	known_facts: string[];
}

export interface OpenSetup {
	scene_id: string;
	scene_title: string;
	setup_for_scene_id: string;
}

export interface ActiveArcState {
	arc_id: string;
	arc_name: string;
	color: string | null;
	scenes_before: number;
	scenes_total: number;
	last_scene_title: string;
}

export interface DramaticIronyItem {
	fact_content: string;
	character_name: string;
	revealed_in_scene_title: string;
}

export interface LocationHistoryItem {
	scene_id: string;
	scene_title: string;
	chapter_title: string;
}

// =============================================================================
// Scene Context Types ("Previously On...")
// =============================================================================

export interface SceneContext {
	scene_id: string;
	previous_scenes: PreviousSceneSummary[];
	present_characters: PresentCharacter[];
	nearby_issues: NearbyIssue[];
	todos: string[];
	last_session: LastSessionInfo | null;
}

export interface PreviousSceneSummary {
	scene_id: string;
	title: string;
	summary: string | null;
	pov: string | null;
	word_count: number;
}

export interface PresentCharacter {
	bible_entry_id: string;
	name: string;
	short_description: string | null;
	entry_type: string;
}

export interface NearbyIssue {
	issue_id: string;
	title: string;
	severity: string;
	status: string;
	linked_scene_id: string;
}

export interface LastSessionInfo {
	date: string;
	words_written: number;
	duration_minutes: number;
}

// =============================================================================
// Impact Preview Types
// =============================================================================

export interface ImpactPreview {
	items: ImpactItem[];
	total_count: number;
}

export interface ImpactItem {
	impact_type: string;
	description: string;
	entity_id: string | null;
	entity_name: string | null;
}
