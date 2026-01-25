/**
 * Export & Import API
 *
 * Operations for exporting manuscripts and importing content.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';
import type { Scene, ImportResult } from './types';

/**
 * API for export operations.
 */
export const exportApi = {
	markdown: () => invoke<string>('export_markdown'),

	plainText: () => invoke<string>('export_plain_text'),

	jsonBackup: () => invoke<string>('export_json_backup'),

	outline: () => invoke<string>('export_outline'),

	bible: () => invoke<string>('export_bible'),

	timeline: () => invoke<string>('export_timeline'),
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
