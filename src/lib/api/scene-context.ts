import { invoke } from '@tauri-apps/api/core';

import type { SceneContext } from './types';

export const sceneContextApi = {
	get: (sceneId: string) => invoke<SceneContext>('get_scene_context', { sceneId }),
};
