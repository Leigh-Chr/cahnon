/**
 * Project API
 *
 * Operations for project management, file locking, and recent projects.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';

import type { FileStatus, Project, RecentProject } from './types';

/**
 * API for project-level operations.
 *
 * @example
 * ```typescript
 * // Create and open a project
 * const project = await projectApi.create('/path/to/novel.cahnon', 'My Novel', 'Author');
 *
 * // Open an existing project
 * const project = await projectApi.open('/path/to/novel.cahnon');
 *
 * // Check for lock conflicts before opening
 * const status = await projectApi.checkFileStatus('/path/to/novel.cahnon');
 * if (status.has_lock) {
 *   console.warn('Project may be open elsewhere:', status.lock_info);
 * }
 * ```
 */
export const projectApi = {
	create: (path: string, title: string, author?: string, description?: string) =>
		invoke<Project>('create_project', {
			path,
			request: { title, author, description },
		}),

	open: (path: string) => invoke<Project>('open_project', { path }),

	close: () => invoke<void>('close_project'),

	get: () => invoke<Project>('get_project'),

	update: (
		request: Partial<
			Pick<Project, 'title' | 'author' | 'description' | 'word_target' | 'daily_word_target'>
		>
	) => invoke<Project>('update_project', { request }),

	getRecent: () => invoke<RecentProject[]>('get_recent_projects'),

	checkFileStatus: (path: string) => invoke<FileStatus>('check_file_status', { path }),

	acquireLock: (path: string) => invoke<void>('acquire_lock', { path }),

	releaseLock: (path: string) => invoke<void>('release_lock', { path }),

	forceAcquireLock: (path: string) => invoke<void>('force_acquire_lock', { path }),

	checkDatabaseIntegrity: () => invoke<boolean>('check_database_integrity'),

	openDemo: () => invoke<Project>('open_demo_project'),

	getIsDemo: () => invoke<boolean>('get_is_demo'),
};
