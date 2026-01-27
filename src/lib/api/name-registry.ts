/**
 * Name Registry API
 *
 * Operations for tracking proper nouns and their mentions in scenes.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';

import type { NameMention, NameRegistryEntry, ScanResult } from './types';

/**
 * API for name registry operations.
 */
export const nameRegistryApi = {
	create: (request: {
		canonical_name: string;
		name_type?: string;
		bible_entry_id?: string;
		aliases?: string;
	}) => invoke<NameRegistryEntry>('create_name_registry_entry', { request }),

	getAll: (nameType?: string) =>
		invoke<NameRegistryEntry[]>('get_name_registry_entries', { nameType }),

	get: (id: string) => invoke<NameRegistryEntry>('get_name_registry_entry', { id }),

	update: (
		id: string,
		request: Partial<
			Pick<
				NameRegistryEntry,
				'canonical_name' | 'name_type' | 'bible_entry_id' | 'aliases' | 'is_confirmed'
			>
		>
	) => invoke<NameRegistryEntry>('update_name_registry_entry', { id, request }),

	delete: (id: string) => invoke<void>('delete_name_registry_entry', { id }),

	scan: () => invoke<[number, number]>('scan_names'),

	scanForScene: (sceneId: string) => invoke<ScanResult>('scan_names_for_scene', { sceneId }),

	merge: (keepId: string, mergeId: string) =>
		invoke<NameRegistryEntry>('merge_name_entries', { keepId, mergeId }),
};

/**
 * API for name mention operations.
 */
export const nameMentionApi = {
	getByScene: (sceneId: string) => invoke<NameMention[]>('get_name_mentions_by_scene', { sceneId }),

	getByRegistry: (registryId: string) =>
		invoke<NameMention[]>('get_name_mentions_by_registry', { registryId }),

	update: (id: string, request: { status: string }) =>
		invoke<NameMention>('update_name_mention', { id, request }),

	delete: (id: string) => invoke<void>('delete_name_mention', { id }),
};
