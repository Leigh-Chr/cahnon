import { invoke } from '@tauri-apps/api/core';

import type { SceneHealth } from './types';

export const healthApi = {
	getBatch: () => invoke<SceneHealth[]>('get_scene_health_batch'),
};
