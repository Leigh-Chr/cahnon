import { invoke } from '@tauri-apps/api/core';

import type { WorldState } from './types';

export const worldStateApi = {
	getAtScene: (sceneId: string) => invoke<WorldState>('get_world_state_at_scene', { sceneId }),
};
