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

// =============================================================================
// Types
// =============================================================================

export type {
	// Project & Manuscript
	Project,
	Chapter,
	Scene,
	SceneHistoryEntry,
	RecentProject,
	// Bible
	BibleEntry,
	BibleRelationshipWithEntry,
	// Arc & Timeline
	Arc,
	TimelineEvent,
	TimelineConflict,
	// Content
	TemplateStep,
	Annotation,
	Snapshot,
	Cut,
	// Search & Stats
	SearchResult,
	WordCounts,
} from './types';

// =============================================================================
// API Objects
// =============================================================================

export { projectApi } from './project';
export { chapterApi, sceneApi, historyApi } from './manuscript';
export { bibleApi, associationApi, relationshipApi } from './bible';
export { arcApi, eventApi, timelineApi } from './timeline';
export { templateApi, annotationApi, snapshotApi, cutApi, trashApi } from './content';
export { searchApi, statsApi } from './search';
export { exportApi, importApi } from './export';

