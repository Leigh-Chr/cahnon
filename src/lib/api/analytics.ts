import { invoke } from '@tauri-apps/api/core';

import type { CharacterThread, ImpactPreview, SceneHealth, WorldState } from './types';

export const healthApi = {
	getBatch: () => invoke<SceneHealth[]>('get_scene_health_batch'),
};

export const worldStateApi = {
	getAtScene: (sceneId: string) => invoke<WorldState>('get_world_state_at_scene', { sceneId }),
	getCharacterThread: (bibleEntryId: string) =>
		invoke<CharacterThread>('get_character_thread', { bibleEntryId }),
};

export const impactApi = {
	previewDeleteScene: (sceneId: string) =>
		invoke<ImpactPreview>('preview_delete_scene_impact', { sceneId }),

	previewDeleteBibleEntry: (bibleEntryId: string) =>
		invoke<ImpactPreview>('preview_delete_bible_entry_impact', { bibleEntryId }),

	previewDeleteChapter: (chapterId: string) =>
		invoke<ImpactPreview>('preview_delete_chapter_impact', { chapterId }),
};
