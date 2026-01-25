/**
 * Manuscript API
 *
 * Operations for chapters and scenes - the core manuscript structure.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core';
import type { Chapter, Scene, SceneHistoryEntry, VersionDiff } from './types';

/**
 * API for chapter operations.
 *
 * @example
 * ```typescript
 * const chapter = await chapterApi.create('Chapter 1', 'The beginning');
 * const chapters = await chapterApi.getAll();
 * await chapterApi.reorder(chapters.map(c => c.id).reverse());
 * ```
 */
export const chapterApi = {
	create: (title: string, summary?: string, position?: number) =>
		invoke<Chapter>('create_chapter', {
			request: { title, summary, position },
		}),

	getAll: () => invoke<Chapter[]>('get_chapters'),

	get: (id: string) => invoke<Chapter>('get_chapter', { id }),

	update: (
		id: string,
		request: Partial<Pick<Chapter, 'title' | 'summary' | 'status' | 'notes' | 'position'>>
	) => invoke<Chapter>('update_chapter', { id, request }),

	delete: (id: string) => invoke<void>('delete_chapter', { id }),

	reorder: (ids: string[]) => invoke<void>('reorder_chapters', { ids }),
};

/**
 * API for scene operations.
 *
 * @example
 * ```typescript
 * // Create a scene
 * const scene = await sceneApi.create(chapterId, 'Opening Scene');
 *
 * // Update scene text
 * await sceneApi.update(scene.id, { text: '<p>New content</p>' });
 *
 * // Split a scene at character position
 * const { first_scene, second_scene } = await sceneApi.split(scene.id, 500);
 *
 * // Merge multiple scenes
 * const merged = await sceneApi.merge([scene1.id, scene2.id]);
 * ```
 */
export const sceneApi = {
	create: (chapterId: string, title: string, summary?: string, position?: number) =>
		invoke<Scene>('create_scene', {
			request: { chapter_id: chapterId, title, summary, position },
		}),

	getByChapter: (chapterId: string) => invoke<Scene[]>('get_scenes', { chapterId }),

	get: (id: string) => invoke<Scene>('get_scene', { id }),

	update: (
		id: string,
		request: Partial<Omit<Scene, 'id' | 'chapter_id' | 'created_at' | 'updated_at'>>
	) => invoke<Scene>('update_scene', { id, request }),

	delete: (id: string) => invoke<void>('delete_scene', { id }),

	reorder: (chapterId: string, ids: string[]) => invoke<void>('reorder_scenes', { chapterId, ids }),

	moveToChapter: (sceneId: string, targetChapterId: string, position: number) =>
		invoke<Scene>('move_scene_to_chapter', { sceneId, targetChapterId, position }),

	split: (sceneId: string, splitPosition: number, newSceneTitle?: string) =>
		invoke<{ first_scene: Scene; second_scene: Scene }>('split_scene', {
			request: { scene_id: sceneId, split_position: splitPosition, new_scene_title: newSceneTitle },
		}),

	merge: (sceneIds: string[]) =>
		invoke<Scene>('merge_scenes', { request: { scene_ids: sceneIds } }),
};

/**
 * API for scene version history.
 */
export const historyApi = {
	getSceneHistory: (sceneId: string) =>
		invoke<SceneHistoryEntry[]>('get_scene_history', { sceneId }),

	restoreVersion: (sceneId: string, historyId: string) =>
		invoke<Scene>('restore_scene_version', { sceneId, historyId }),

	compareVersions: (sceneId: string, historyIdA: string, historyIdB: string) =>
		invoke<VersionDiff>('compare_scene_versions', { sceneId, historyIdA, historyIdB }),
};
