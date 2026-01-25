/**
 * Search & Stats API
 *
 * Operations for global search and word count statistics.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';
import type { SearchResult, WordCounts } from './types';

/**
 * API for global search operations.
 */
export const searchApi = {
	global: (query: string, scope?: string[]) =>
		invoke<SearchResult[]>('global_search', { query, scope }),
};

/**
 * API for word count statistics.
 */
export const statsApi = {
	getWordCounts: () => invoke<WordCounts>('get_word_counts'),
};
