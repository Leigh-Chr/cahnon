/**
 * Cahnon API Layer
 *
 * This module provides TypeScript types and wrapper functions for all
 * Tauri IPC calls. It serves as the bridge between the Svelte frontend
 * and the Rust backend.
 *
 * @example
 * ```typescript
 * // Create a new project
 * const project = await projectApi.create('/path/to/novel.cahnon', 'My Novel', 'Author');
 *
 * // Create a chapter and scene
 * const chapter = await chapterApi.create('Chapter 1');
 * const scene = await sceneApi.create(chapter.id, 'Opening Scene');
 *
 * // Update scene text
 * await sceneApi.update(scene.id, { text: '<p>It was a dark and stormy night...</p>' });
 * ```
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';

// =============================================================================
// Types
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
	created_at: string;
	updated_at: string;
}

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

export interface RecentProject {
	path: string;
	title: string;
	last_opened: string;
}

// Arc types
export interface Arc {
	id: string;
	name: string;
	description: string | null;
	stakes: string | null;
	/** Key characters involved (comma-separated Bible entry IDs) */
	characters: string | null;
	status: string;
	color: string | null;
	position: number;
	created_at: string;
	updated_at: string;
}

// Event types
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

// Bible Relationship types
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

// Scene History
export interface SceneHistoryEntry {
	id: string;
	scene_id: string;
	text: string;
	created_at: string;
}

// Template types
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

// Annotation types
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

// Issue types
export interface Issue {
	id: string;
	issue_type: string;
	title: string;
	description: string | null;
	severity: string;
	status: string;
	resolution_note: string | null;
	created_at: string;
	updated_at: string;
}

// Snapshot types
export interface Snapshot {
	id: string;
	name: string;
	description: string | null;
	snapshot_type: string;
	data: string;
	created_at: string;
}

// Cut types
export interface Cut {
	id: string;
	scene_id: string | null;
	text: string;
	created_at: string;
}

// Lock/Conflict types
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

// Timeline Conflict types
export interface TimelineConflict {
	conflict_type: string;
	description: string;
	scene_ids: string[];
	character_id: string | null;
	character_name: string | null;
	time_point: string | null;
}

// Version Diff types
export interface VersionDiff {
	text_a: string;
	text_b: string;
}

// =============================================================================
// API Objects
// =============================================================================

/**
 * API for project-level operations.
 *
 * @example
 * ```typescript
 * // Create and open a project
 * const project = await projectApi.create('/path/to/novel.cahnon', 'My Novel', 'Author');
 *
 * // Open an existing project
 * const project = await projectApi.open('/path/to/novel.cahnon');
 *
 * // Check for lock conflicts before opening
 * const status = await projectApi.checkFileStatus('/path/to/novel.cahnon');
 * if (status.has_lock) {
 *   console.warn('Project may be open elsewhere:', status.lock_info);
 * }
 * ```
 */
export const projectApi = {
	create: (path: string, title: string, author?: string, description?: string) =>
		invoke<Project>('create_project', {
			path,
			request: { title, author, description },
		}),

	open: (path: string) => invoke<Project>('open_project', { path }),

	close: () => invoke<void>('close_project'),

	get: () => invoke<Project>('get_project'),

	update: (request: Partial<Pick<Project, 'title' | 'author' | 'description' | 'word_target'>>) =>
		invoke<Project>('update_project', { request }),

	getRecent: () => invoke<RecentProject[]>('get_recent_projects'),

	checkFileStatus: (path: string) => invoke<FileStatus>('check_file_status', { path }),

	acquireLock: (path: string) => invoke<void>('acquire_lock', { path }),

	releaseLock: (path: string) => invoke<void>('release_lock', { path }),

	forceAcquireLock: (path: string) => invoke<void>('force_acquire_lock', { path }),
};

/**
 * API for chapter operations.
 *
 * @example
 * ```typescript
 * const chapter = await chapterApi.create('Chapter 1', 'The beginning');
 * const chapters = await chapterApi.getAll();
 * await chapterApi.reorder(chapters.map(c => c.id).reverse());
 * ```
 */
export const chapterApi = {
	create: (title: string, summary?: string, position?: number) =>
		invoke<Chapter>('create_chapter', {
			request: { title, summary, position },
		}),

	getAll: () => invoke<Chapter[]>('get_chapters'),

	get: (id: string) => invoke<Chapter>('get_chapter', { id }),

	update: (
		id: string,
		request: Partial<Pick<Chapter, 'title' | 'summary' | 'status' | 'notes' | 'position'>>
	) => invoke<Chapter>('update_chapter', { id, request }),

	delete: (id: string) => invoke<void>('delete_chapter', { id }),

	reorder: (ids: string[]) => invoke<void>('reorder_chapters', { ids }),
};

/**
 * API for scene operations.
 *
 * @example
 * ```typescript
 * // Create a scene
 * const scene = await sceneApi.create(chapterId, 'Opening Scene');
 *
 * // Update scene text
 * await sceneApi.update(scene.id, { text: '<p>New content</p>' });
 *
 * // Split a scene at character position
 * const { first_scene, second_scene } = await sceneApi.split(scene.id, 500);
 *
 * // Merge multiple scenes
 * const merged = await sceneApi.merge([scene1.id, scene2.id]);
 * ```
 */
export const sceneApi = {
	create: (chapterId: string, title: string, summary?: string, position?: number) =>
		invoke<Scene>('create_scene', {
			request: { chapter_id: chapterId, title, summary, position },
		}),

	getByChapter: (chapterId: string) => invoke<Scene[]>('get_scenes', { chapterId }),

	get: (id: string) => invoke<Scene>('get_scene', { id }),

	update: (
		id: string,
		request: Partial<Omit<Scene, 'id' | 'chapter_id' | 'created_at' | 'updated_at'>>
	) => invoke<Scene>('update_scene', { id, request }),

	delete: (id: string) => invoke<void>('delete_scene', { id }),

	reorder: (chapterId: string, ids: string[]) => invoke<void>('reorder_scenes', { chapterId, ids }),

	moveToChapter: (sceneId: string, targetChapterId: string, position: number) =>
		invoke<Scene>('move_scene_to_chapter', { sceneId, targetChapterId, position }),

	split: (sceneId: string, splitPosition: number, newSceneTitle?: string) =>
		invoke<{ first_scene: Scene; second_scene: Scene }>('split_scene', {
			request: { scene_id: sceneId, split_position: splitPosition, new_scene_title: newSceneTitle },
		}),

	merge: (sceneIds: string[]) =>
		invoke<Scene>('merge_scenes', { request: { scene_ids: sceneIds } }),
};

/**
 * API for bible (knowledge base) entries.
 *
 * @example
 * ```typescript
 * // Create a character
 * const character = await bibleApi.create({
 *   entry_type: 'character',
 *   name: 'John Smith',
 *   aliases: 'Johnny, The Protagonist',
 *   short_description: 'A detective with a troubled past'
 * });
 *
 * // Get all locations
 * const locations = await bibleApi.getAll('location');
 *
 * // Search bible entries
 * const results = await bibleApi.search('smith');
 * ```
 */
export const bibleApi = {
	create: (request: {
		entry_type: string;
		name: string;
		aliases?: string;
		short_description?: string;
		full_description?: string;
		status?: string;
		tags?: string;
		color?: string;
	}) => invoke<BibleEntry>('create_bible_entry', { request }),

	getAll: (entryType?: string) => invoke<BibleEntry[]>('get_bible_entries', { entryType }),

	get: (id: string) => invoke<BibleEntry>('get_bible_entry', { id }),

	update: (
		id: string,
		request: Partial<
			Omit<BibleEntry, 'id' | 'entry_type' | 'created_at' | 'updated_at' | 'deleted_at'>
		>
	) => invoke<BibleEntry>('update_bible_entry', { id, request }),

	delete: (id: string) => invoke<void>('delete_bible_entry', { id }),

	search: (query: string) => invoke<BibleEntry[]>('search_bible', { query }),
};

// Association API
export const associationApi = {
	create: (sceneId: string, bibleEntryId: string) =>
		invoke<{ id: string; scene_id: string; bible_entry_id: string; created_at: string }>(
			'create_association',
			{
				request: { scene_id: sceneId, bible_entry_id: bibleEntryId },
			}
		),

	getByScene: (sceneId: string) => invoke<BibleEntry[]>('get_scene_associations', { sceneId }),

	delete: (sceneId: string, bibleEntryId: string) =>
		invoke<void>('delete_association', { sceneId, bibleEntryId }),
};

// Search API
export const searchApi = {
	global: (query: string, scope?: string[]) =>
		invoke<SearchResult[]>('global_search', { query, scope }),
};

// Stats API
export const statsApi = {
	getWordCounts: () => invoke<WordCounts>('get_word_counts'),
};

// Arc API
export const arcApi = {
	create: (request: {
		name: string;
		description?: string;
		stakes?: string;
		characters?: string;
		status?: string;
		color?: string;
	}) => invoke<Arc>('create_arc', { request }),

	getAll: () => invoke<Arc[]>('get_arcs'),

	get: (id: string) => invoke<Arc>('get_arc', { id }),

	update: (
		id: string,
		request: Partial<
			Pick<Arc, 'name' | 'description' | 'stakes' | 'characters' | 'status' | 'color'>
		>
	) => invoke<Arc>('update_arc', { id, request }),

	delete: (id: string) => invoke<void>('delete_arc', { id }),

	linkScene: (sceneId: string, arcId: string) =>
		invoke<void>('link_scene_to_arc', { sceneId, arcId }),

	unlinkScene: (sceneId: string, arcId: string) =>
		invoke<void>('unlink_scene_from_arc', { sceneId, arcId }),

	getSceneArcs: (sceneId: string) => invoke<Arc[]>('get_scene_arcs', { sceneId }),
};

// Event API (Timeline)
export const eventApi = {
	create: (request: {
		title: string;
		description?: string;
		time_point?: string;
		time_start?: string;
		time_end?: string;
		event_type?: string;
		importance?: string;
	}) => invoke<TimelineEvent>('create_event', { request }),

	getAll: () => invoke<TimelineEvent[]>('get_events'),

	get: (id: string) => invoke<TimelineEvent>('get_event', { id }),

	update: (
		id: string,
		request: Partial<
			Pick<
				TimelineEvent,
				| 'title'
				| 'description'
				| 'time_point'
				| 'time_start'
				| 'time_end'
				| 'event_type'
				| 'importance'
			>
		>
	) => invoke<TimelineEvent>('update_event', { id, request }),

	delete: (id: string) => invoke<void>('delete_event', { id }),

	getTimelineScenes: () => invoke<Scene[]>('get_timeline_scenes'),

	linkScene: (sceneId: string, eventId: string) =>
		invoke<void>('link_scene_to_event', { sceneId, eventId }),

	unlinkScene: (sceneId: string, eventId: string) =>
		invoke<void>('unlink_scene_from_event', { sceneId, eventId }),

	getSceneEvents: (sceneId: string) => invoke<TimelineEvent[]>('get_scene_events', { sceneId }),

	getEventScenes: (eventId: string) => invoke<string[]>('get_event_scenes', { eventId }),

	// Event-Bible linking (per spec Section 9.1)
	linkBibleEntry: (bibleEntryId: string, eventId: string) =>
		invoke<void>('link_bible_entry_to_event', { bibleEntryId, eventId }),

	unlinkBibleEntry: (bibleEntryId: string, eventId: string) =>
		invoke<void>('unlink_bible_entry_from_event', { bibleEntryId, eventId }),

	getEventBibleEntries: (eventId: string) =>
		invoke<string[]>('get_event_bible_entries', { eventId }),

	getBibleEntryEvents: (bibleEntryId: string) =>
		invoke<TimelineEvent[]>('get_bible_entry_events', { bibleEntryId }),
};

// Bible Relationship API
export const relationshipApi = {
	create: (request: {
		source_id: string;
		target_id: string;
		relationship_type: string;
		note?: string;
		status?: string;
	}) => invoke<BibleRelationship>('create_bible_relationship', { request }),

	getByEntry: (entryId: string) =>
		invoke<BibleRelationshipWithEntry[]>('get_bible_relationships', { entryId }),

	update: (
		id: string,
		request: Partial<Pick<BibleRelationship, 'relationship_type' | 'note' | 'status'>>
	) => invoke<BibleRelationship>('update_bible_relationship', { id, request }),

	delete: (id: string) => invoke<void>('delete_bible_relationship', { id }),
};

// Scene History API
export const historyApi = {
	getSceneHistory: (sceneId: string) =>
		invoke<SceneHistoryEntry[]>('get_scene_history', { sceneId }),

	restoreVersion: (sceneId: string, historyId: string) =>
		invoke<Scene>('restore_scene_version', { sceneId, historyId }),
};

// Template API
export const templateApi = {
	getAll: () => invoke<Template[]>('get_templates'),

	getSteps: (templateId: string) => invoke<TemplateStep[]>('get_template_steps', { templateId }),

	setActive: (templateId: string) => invoke<void>('set_active_template', { templateId }),

	assignSceneToStep: (sceneId: string, stepId: string) =>
		invoke<void>('assign_scene_to_step', { sceneId, stepId }),

	getSceneStep: (sceneId: string) => invoke<TemplateStep | null>('get_scene_step', { sceneId }),

	initBuiltin: () => invoke<void>('init_builtin_templates'),

	create: (name: string) => invoke<Template>('create_template', { request: { name } }),

	update: (id: string, name: string) =>
		invoke<Template>('update_template', { id, request: { name } }),

	delete: (id: string) => invoke<void>('delete_template', { id }),

	createStep: (request: {
		template_id: string;
		name: string;
		description?: string;
		typical_position?: number;
		color?: string;
	}) => invoke<TemplateStep>('create_template_step', { request }),

	updateStep: (
		id: string,
		request: {
			name?: string;
			description?: string;
			typical_position?: number;
			color?: string;
			position?: number;
		}
	) => invoke<TemplateStep>('update_template_step', { id, request }),

	deleteStep: (id: string) => invoke<void>('delete_template_step', { id }),
};

// Annotation API
export const annotationApi = {
	create: (request: {
		scene_id: string;
		start_offset: number;
		end_offset: number;
		annotation_type?: string;
		content: string;
	}) => invoke<Annotation>('create_annotation', { request }),

	getByScene: (sceneId: string) => invoke<Annotation[]>('get_annotations', { sceneId }),

	update: (id: string, request: Partial<Pick<Annotation, 'content' | 'status'>>) =>
		invoke<Annotation>('update_annotation', { id, request }),

	delete: (id: string) => invoke<void>('delete_annotation', { id }),
};

// Issue API
export const issueApi = {
	create: (request: {
		issue_type: string;
		title: string;
		description?: string;
		severity?: string;
	}) => invoke<Issue>('create_issue', { request }),

	getAll: (status?: string) => invoke<Issue[]>('get_issues', { status }),

	get: (id: string) => invoke<Issue>('get_issue', { id }),

	update: (id: string, request: Partial<Pick<Issue, 'status' | 'resolution_note'>>) =>
		invoke<Issue>('update_issue', { id, request }),

	// Issue-Scene linking (per spec Section 14.2)
	linkScene: (sceneId: string, issueId: string) =>
		invoke<void>('link_scene_to_issue', { sceneId, issueId }),

	unlinkScene: (sceneId: string, issueId: string) =>
		invoke<void>('unlink_scene_from_issue', { sceneId, issueId }),

	getIssueScenes: (issueId: string) => invoke<string[]>('get_issue_scenes', { issueId }),

	getSceneIssues: (sceneId: string) => invoke<Issue[]>('get_scene_issues', { sceneId }),

	// Issue-Bible linking (per spec Section 14.2)
	linkBibleEntry: (bibleEntryId: string, issueId: string) =>
		invoke<void>('link_bible_entry_to_issue', { bibleEntryId, issueId }),

	unlinkBibleEntry: (bibleEntryId: string, issueId: string) =>
		invoke<void>('unlink_bible_entry_from_issue', { bibleEntryId, issueId }),

	getIssueBibleEntries: (issueId: string) =>
		invoke<string[]>('get_issue_bible_entries', { issueId }),
};

// Snapshot API
export const snapshotApi = {
	create: (name: string, description?: string, snapshotType?: string) =>
		invoke<Snapshot>('create_snapshot', { name, description, snapshotType }),

	getAll: () => invoke<Snapshot[]>('get_snapshots'),

	get: (id: string) => invoke<Snapshot>('get_snapshot', { id }),
};

// Export API
export const exportApi = {
	markdown: () => invoke<string>('export_markdown'),

	plainText: () => invoke<string>('export_plain_text'),

	jsonBackup: () => invoke<string>('export_json_backup'),

	outline: () => invoke<string>('export_outline'),

	bible: () => invoke<string>('export_bible'),

	timeline: () => invoke<string>('export_timeline'),
};

// Trash API
export const trashApi = {
	getDeletedScenes: () => invoke<Scene[]>('get_deleted_scenes'),

	restoreScene: (id: string) => invoke<Scene>('restore_scene', { id }),

	getDeletedChapters: () => invoke<Chapter[]>('get_deleted_chapters'),

	restoreChapter: (id: string) => invoke<Chapter>('restore_chapter', { id }),

	duplicateScene: (id: string, structureOnly?: boolean) =>
		invoke<Scene>('duplicate_scene', { id, structureOnly }),
};

// Cut Library API
export const cutApi = {
	create: (text: string, sceneId?: string) => invoke<Cut>('create_cut', { sceneId, text }),

	getAll: () => invoke<Cut[]>('get_cuts'),

	delete: (id: string) => invoke<void>('delete_cut', { id }),
};

// Import API
export interface ImportResult {
	chapters_created: number;
	scenes_created: number;
}

export const importApi = {
	markdownAsScene: (chapterId: string, title: string, content: string) =>
		invoke<Scene>('import_markdown_as_scene', { chapterId, title, content }),

	markdownStructured: (content: string) =>
		invoke<ImportResult>('import_markdown_structured', { content }),

	textAsScene: (chapterId: string, title: string, content: string) =>
		invoke<Scene>('import_text_as_scene', { chapterId, title, content }),
};

// Timeline Conflict Detection API
export const timelineApi = {
	detectConflicts: () => invoke<TimelineConflict[]>('detect_timeline_conflicts'),
};

// Extended History API with version comparison
export const historyApiExtended = {
	...historyApi,
	compareVersions: (sceneId: string, historyIdA: string, historyIdB: string) =>
		invoke<VersionDiff>('compare_scene_versions', { sceneId, historyIdA, historyIdB }),
};
