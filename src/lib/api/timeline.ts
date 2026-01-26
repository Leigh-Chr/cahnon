/**
 * Timeline API
 *
 * Operations for arcs, timeline events, and conflict detection.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';

import type { Arc, Scene, TimelineConflict, TimelineEvent } from './types';

/**
 * API for plot arc operations.
 */
export const arcApi = {
	create: (request: {
		name: string;
		description?: string;
		stakes?: string;
		status?: string;
		color?: string;
	}) => invoke<Arc>('create_arc', { request }),

	getAll: () => invoke<Arc[]>('get_arcs'),

	get: (id: string) => invoke<Arc>('get_arc', { id }),

	update: (
		id: string,
		request: Partial<Pick<Arc, 'name' | 'description' | 'stakes' | 'status' | 'color'>>
	) => invoke<Arc>('update_arc', { id, request }),

	delete: (id: string) => invoke<void>('delete_arc', { id }),

	linkScene: (sceneId: string, arcId: string) =>
		invoke<void>('link_scene_to_arc', { sceneId, arcId }),

	unlinkScene: (sceneId: string, arcId: string) =>
		invoke<void>('unlink_scene_from_arc', { sceneId, arcId }),

	getSceneArcs: (sceneId: string) => invoke<Arc[]>('get_scene_arcs', { sceneId }),

	setCharacters: (arcId: string, characterIds: string[]) =>
		invoke<string[]>('set_arc_characters', { arcId, characterIds }),
};

/**
 * API for timeline event operations.
 */
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

	// Event-Bible linking
	linkBibleEntry: (bibleEntryId: string, eventId: string) =>
		invoke<void>('link_bible_entry_to_event', { bibleEntryId, eventId }),

	unlinkBibleEntry: (bibleEntryId: string, eventId: string) =>
		invoke<void>('unlink_bible_entry_from_event', { bibleEntryId, eventId }),

	getEventBibleEntries: (eventId: string) =>
		invoke<string[]>('get_event_bible_entries', { eventId }),

	getBibleEntryEvents: (bibleEntryId: string) =>
		invoke<TimelineEvent[]>('get_bible_entry_events', { bibleEntryId }),
};

/**
 * API for timeline conflict detection.
 */
export const timelineApi = {
	detectConflicts: () => invoke<TimelineConflict[]>('detect_timeline_conflicts'),
};
