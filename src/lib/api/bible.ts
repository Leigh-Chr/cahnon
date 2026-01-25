/**
 * Bible API
 *
 * Operations for the story bible (knowledge base) - characters, locations,
 * objects, factions, concepts, and glossary terms.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';
import type { BibleEntry, BibleRelationship, BibleRelationshipWithEntry } from './types';

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

/**
 * API for scene-to-bible associations.
 */
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

/**
 * API for bible entry relationships.
 */
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
