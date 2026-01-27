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
	// Name Registry Scan
	AssociationSuggestion,
	// Bible
	BibleEntry,
	BibleRelationshipWithEntry,
	Chapter,
	CharacterKnowledgeState,
	CharacterPresence,
	Cut,
	DramaticIronyItem,
	// Facts
	Fact,
	FactCharacter,
	// Health
	HealthCheck,
	// Impact
	ImpactItem,
	ImpactPreview,
	Issue,
	// Scene Context
	LastSessionInfo,
	LocationHistoryItem,
	NameMention,
	// Name Registry
	NameRegistryEntry,
	NearbyIssue,
	OpenSetup,
	PresentCharacter,
	PreviousSceneSummary,
	// Project & Manuscript
	Project,
	RecentProject,
	// Saved Filters
	SavedFilter,
	ScanResult,
	Scene,
	SceneContext,
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
	// Writing Session
	WritingSession,
} from './types';

// =============================================================================
// API Objects
// =============================================================================

export { associationApi, bibleApi, relationshipApi } from './bible';
export { annotationApi, cutApi, snapshotApi, templateApi, trashApi } from './content';
export { exportApi, importApi } from './export';
export { factApi } from './fact';
export { healthApi } from './health';
export { impactApi } from './impact';
export { issueApi } from './issue';
export { chapterApi, historyApi, sceneApi } from './manuscript';
export { nameMentionApi, nameRegistryApi } from './name-registry';
export { projectApi } from './project';
export { savedFilterApi } from './saved-filter';
export { sceneContextApi } from './scene-context';
export { searchApi, statsApi } from './search';
export { arcApi, eventApi, timelineApi } from './timeline';
export { worldStateApi } from './world-state';
export { writingSessionApi } from './writing-session';
