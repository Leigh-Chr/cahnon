/**
 * Content API
 *
 * Operations for templates, annotations, issues, snapshots, and cut library.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';

import type { Annotation, Chapter, Cut, Scene, Snapshot, Template, TemplateStep } from './types';

/**
 * API for writing template operations.
 */
export const templateApi = {
	getAll: () => invoke<Template[]>('get_templates'),

	getSteps: (templateId: string) => invoke<TemplateStep[]>('get_template_steps', { templateId }),

	setActive: (templateId: string) => invoke<void>('set_active_template', { templateId }),

	assignSceneToStep: (sceneId: string, stepId: string) =>
		invoke<void>('assign_scene_to_step', { sceneId, stepId }),

	getSceneStep: (sceneId: string) => invoke<TemplateStep | null>('get_scene_step', { sceneId }),

	initBuiltin: () => invoke<void>('init_builtin_templates'),

	create: (name: string) => invoke<Template>('create_template', { request: { name } }),

	update: (id: string, name: string) =>
		invoke<Template>('update_template', { id, request: { name } }),

	delete: (id: string) => invoke<void>('delete_template', { id }),

	createStep: (request: {
		template_id: string;
		name: string;
		description?: string;
		typical_position?: number;
		color?: string;
	}) => invoke<TemplateStep>('create_template_step', { request }),

	updateStep: (
		id: string,
		request: {
			name?: string;
			description?: string;
			typical_position?: number;
			color?: string;
			position?: number;
		}
	) => invoke<TemplateStep>('update_template_step', { id, request }),

	deleteStep: (id: string) => invoke<void>('delete_template_step', { id }),
};

/**
 * API for text annotation operations.
 */
export const annotationApi = {
	create: (request: {
		scene_id: string;
		start_offset: number;
		end_offset: number;
		annotation_type?: string;
		content: string;
	}) => invoke<Annotation>('create_annotation', { request }),

	getByScene: (sceneId: string) => invoke<Annotation[]>('get_annotations', { sceneId }),

	update: (id: string, request: Partial<Pick<Annotation, 'content' | 'status'>>) =>
		invoke<Annotation>('update_annotation', { id, request }),

	delete: (id: string) => invoke<void>('delete_annotation', { id }),
};

/**
 * API for project snapshot operations.
 */
export const snapshotApi = {
	create: (name: string, description?: string, snapshotType?: string) =>
		invoke<Snapshot>('create_snapshot', { name, description, snapshotType }),

	getAll: () => invoke<Snapshot[]>('get_snapshots'),

	get: (id: string) => invoke<Snapshot>('get_snapshot', { id }),

	delete: (id: string) => invoke<void>('delete_snapshot', { id }),

	restore: (id: string) => invoke<void>('restore_snapshot', { id }),

	cleanupExpired: () => invoke<number>('cleanup_expired_snapshots'),

	getScenes: (id: string) => invoke<Scene[]>('get_snapshot_scenes', { id }),

	restoreScene: (snapshotId: string, sceneId: string) =>
		invoke<Scene>('restore_scene_from_snapshot', { snapshotId, sceneId }),
};

/**
 * API for cut library operations.
 */
export const cutApi = {
	create: (text: string, sceneId?: string) => invoke<Cut>('create_cut', { sceneId, text }),

	getAll: () => invoke<Cut[]>('get_cuts'),

	delete: (id: string) => invoke<void>('delete_cut', { id }),
};

/**
 * API for trash/restore operations.
 */
export const trashApi = {
	getDeletedScenes: () => invoke<Scene[]>('get_deleted_scenes'),

	restoreScene: (id: string) => invoke<Scene>('restore_scene', { id }),

	getDeletedChapters: () => invoke<Chapter[]>('get_deleted_chapters'),

	restoreChapter: (id: string) => invoke<Chapter>('restore_chapter', { id }),

	duplicateScene: (id: string, structureOnly?: boolean) =>
		invoke<Scene>('duplicate_scene', { id, structureOnly }),
};
