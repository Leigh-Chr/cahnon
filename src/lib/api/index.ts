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
	// World State / Narrative Context
	ActiveArcState,
	Annotation,
	// Arc & Timeline
	Arc,
	// Bible
	BibleEntry,
	BibleRelationshipWithEntry,
	Chapter,
	CharacterPresence,
	CharacterThread,
	CharacterThreadScene,
	Cut,
	// Health
	HealthCheck,
	// Impact
	ImpactItem,
	ImpactPreview,
	Issue,
	OpenSetup,
	// Project & Manuscript
	Project,
	RecentProject,
	Scene,
	SceneHealth,
	SceneHistoryEntry,
	// Search & Stats
	SearchResult,
	Snapshot,
	// Content
	Template,
	TemplateStep,
	TimelineConflict,
	TimelineEvent,
	WordCounts,
	WorldState,
} from './types';

// =============================================================================
// API Objects
// =============================================================================

export { healthApi, impactApi, worldStateApi } from './analytics';
export { associationApi, bibleApi, relationshipApi } from './bible';
export { annotationApi, cutApi, snapshotApi, templateApi, trashApi } from './content';
export { exportApi, importApi } from './export';
export { issueApi } from './issue';
export { chapterApi, historyApi, sceneApi } from './manuscript';
export { projectApi } from './project';
export { searchApi, statsApi } from './search';
export { arcApi, eventApi, timelineApi } from './timeline';
