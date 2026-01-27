import { invoke } from '@tauri-apps/api/core';

import type { ImpactPreview } from './types';

export const impactApi = {
	previewDeleteScene: (sceneId: string) =>
		invoke<ImpactPreview>('preview_delete_scene_impact', { sceneId }),

	previewDeleteBibleEntry: (bibleEntryId: string) =>
		invoke<ImpactPreview>('preview_delete_bible_entry_impact', { bibleEntryId }),

	previewDeleteChapter: (chapterId: string) =>
		invoke<ImpactPreview>('preview_delete_chapter_impact', { chapterId }),
};
