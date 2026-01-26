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

	findReplaceInScenes: (options: {
		find: string;
		replace: string;
		caseSensitive?: boolean;
		wholeWord?: boolean;
		chapterId?: string;
	}) =>
		invoke<number>('find_replace_in_scenes', {
			find: options.find,
			replace: options.replace,
			caseSensitive: options.caseSensitive,
			wholeWord: options.wholeWord,
			chapterId: options.chapterId,
		}),
};

/**
 * API for word count statistics.
 */
export const statsApi = {
	getWordCounts: () => invoke<WordCounts>('get_word_counts'),
};
