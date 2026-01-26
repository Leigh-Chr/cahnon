/**
 * Issue API
 *
 * Operations for managing consistency issues and problems in the manuscript.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';

import type { Issue } from './types';

/**
 * API for issue management operations.
 */
export const issueApi = {
	/**
	 * Create a new issue.
	 */
	create: (request: {
		issue_type: string;
		title: string;
		description?: string;
		severity?: string;
	}) => invoke<Issue>('create_issue', { request }),

	/**
	 * Get all issues, optionally filtered by status.
	 */
	getAll: (status?: string) => invoke<Issue[]>('get_issues', { status }),

	/**
	 * Get a single issue by ID.
	 */
	get: (id: string) => invoke<Issue>('get_issue', { id }),

	/**
	 * Update an issue's status, resolution note, title, description, or severity.
	 */
	update: (
		id: string,
		request: {
			status?: string;
			resolution_note?: string;
			title?: string;
			description?: string;
			severity?: string;
		}
	) => invoke<Issue>('update_issue', { id, request }),

	/**
	 * Delete an issue and its links.
	 */
	delete: (id: string) => invoke<void>('delete_issue', { id }),

	/**
	 * Link a scene to an issue.
	 */
	linkScene: (sceneId: string, issueId: string) =>
		invoke<void>('link_scene_to_issue', { sceneId, issueId }),

	/**
	 * Unlink a scene from an issue.
	 */
	unlinkScene: (sceneId: string, issueId: string) =>
		invoke<void>('unlink_scene_from_issue', { sceneId, issueId }),

	/**
	 * Get all scene IDs linked to an issue.
	 */
	getIssueScenes: (issueId: string) => invoke<string[]>('get_issue_scenes', { issueId }),

	/**
	 * Get all issues linked to a scene.
	 */
	getSceneIssues: (sceneId: string) => invoke<Issue[]>('get_scene_issues', { sceneId }),

	/**
	 * Link a bible entry to an issue.
	 */
	linkBibleEntry: (bibleEntryId: string, issueId: string) =>
		invoke<void>('link_bible_entry_to_issue', { bibleEntryId, issueId }),

	/**
	 * Unlink a bible entry from an issue.
	 */
	unlinkBibleEntry: (bibleEntryId: string, issueId: string) =>
		invoke<void>('unlink_bible_entry_from_issue', { bibleEntryId, issueId }),

	/**
	 * Get all bible entry IDs linked to an issue.
	 */
	getIssueBibleEntries: (issueId: string) =>
		invoke<string[]>('get_issue_bible_entries', { issueId }),
};
