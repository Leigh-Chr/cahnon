import { invoke } from '@tauri-apps/api/core';

import type { Fact, FactCharacter } from './types';

export const factApi = {
	create: (request: {
		content: string;
		category?: string;
		revealed_in_scene_id?: string;
		status?: string;
	}) => invoke<Fact>('create_fact', { request }),

	getAll: () => invoke<Fact[]>('get_facts'),

	get: (id: string) => invoke<Fact>('get_fact', { id }),

	update: (
		id: string,
		request: {
			content?: string;
			category?: string;
			revealed_in_scene_id?: string;
			status?: string;
		}
	) => invoke<Fact>('update_fact', { id, request }),

	delete: (id: string) => invoke<void>('delete_fact', { id }),

	getForScene: (sceneId: string) => invoke<Fact[]>('get_facts_for_scene', { sceneId }),

	linkCharacter: (factId: string, bibleEntryId: string, learnedInSceneId?: string) =>
		invoke<FactCharacter>('link_character_to_fact', {
			factId,
			bibleEntryId,
			learnedInSceneId: learnedInSceneId ?? null,
		}),

	unlinkCharacter: (factId: string, bibleEntryId: string) =>
		invoke<void>('unlink_character_from_fact', { factId, bibleEntryId }),

	getCharacters: (factId: string) => invoke<FactCharacter[]>('get_fact_characters', { factId }),

	getCharacterKnowledge: (bibleEntryId: string, sceneId: string) =>
		invoke<Fact[]>('get_character_knowledge_at_scene', { bibleEntryId, sceneId }),
};
