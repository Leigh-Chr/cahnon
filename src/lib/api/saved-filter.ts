/**
 * Saved Filter API
 *
 * Operations for saving and managing filter collections.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';

import type { SavedFilter } from './types';

/**
 * API for saved filter operations.
 */
export const savedFilterApi = {
	create: (request: { name: string; filter_type: string; filter_data: string }) =>
		invoke<SavedFilter>('create_saved_filter', { request }),

	getAll: (filterType?: string) => invoke<SavedFilter[]>('get_saved_filters', { filterType }),

	update: (id: string, request: { name?: string; filter_data?: string }) =>
		invoke<SavedFilter>('update_saved_filter', { id, request }),

	delete: (id: string) => invoke<void>('delete_saved_filter', { id }),
};
