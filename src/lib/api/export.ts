/**
 * Export & Import API
 *
 * Operations for exporting manuscripts and importing content.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';

import type { ImportResult, Scene } from './types';

/**
 * API for export operations.
 */
export const exportApi = {
	markdown: (options?: {
		chapterIds?: string[];
		sceneSeparator?: string;
		includeTitles?: boolean;
	}) =>
		invoke<string>('export_markdown', {
			chapterIds: options?.chapterIds,
			sceneSeparator: options?.sceneSeparator,
			includeTitles: options?.includeTitles,
		}),

	plainText: (options?: { chapterIds?: string[]; sceneSeparator?: string }) =>
		invoke<string>('export_plain_text', {
			chapterIds: options?.chapterIds,
			sceneSeparator: options?.sceneSeparator,
		}),

	jsonBackup: () => invoke<string>('export_json_backup'),

	outline: () => invoke<string>('export_outline'),

	bible: () => invoke<string>('export_bible'),

	timeline: () => invoke<string>('export_timeline'),

	/** Export bible entries as CSV. */
	exportBibleCsv: () => invoke<string>('export_bible_csv'),

	/** Export timeline (events + scenes) as CSV. */
	exportTimelineCsv: () => invoke<string>('export_timeline_csv'),

	/** Export review grid data as CSV. */
	exportReviewGridCsv: () => invoke<string>('export_review_grid_csv'),

	/** Export word count stats as CSV. */
	exportStatsCsv: () => invoke<string>('export_stats_csv'),
};

/**
 * API for import operations.
 */
export const importApi = {
	markdownAsScene: (chapterId: string, title: string, content: string) =>
		invoke<Scene>('import_markdown_as_scene', { chapterId, title, content }),

	markdownStructured: (content: string) =>
		invoke<ImportResult>('import_markdown_structured', { content }),

	textAsScene: (chapterId: string, title: string, content: string) =>
		invoke<Scene>('import_text_as_scene', { chapterId, title, content }),

	/** Import a JSON backup file to restore project data. Creates auto-backup first. */
	jsonBackup: (content: string) => invoke<void>('import_json_backup', { content }),
};
