/**
 * Svelte 5 stores for Cahnon application state.
 *
 * All application state is managed through the AppState class using runes.
 * Components access state directly via appState properties (reactive).
 *
 * @module
 */

// =============================================================================
// Re-exports from split modules
// =============================================================================

// Types
export type { ViewMode, WorkMode, EditorSettings, FocusSettings } from './types';
export { defaultEditorSettings, defaultFocusSettings } from './types';

// App State (main state class)
export { appState } from './app-state.svelte';

// Recovery utilities
export type { RecoveryDraft } from './recovery';
export { saveRecoveryDraft, getRecoveryDraft, clearRecoveryDraft } from './recovery';

// =============================================================================
// Autosave (wrapper around recovery module)
// =============================================================================

import { appState } from './app-state.svelte';

let autosaveTimeout: ReturnType<typeof setTimeout> | null = null;

export function scheduleAutosave() {
	appState.hasUnsavedChanges = true;
	if (autosaveTimeout) {
		clearTimeout(autosaveTimeout);
	}
	autosaveTimeout = setTimeout(async () => {
		const scene = appState.selectedScene;
		if (scene) {
			// Autosave is handled by the editor component
		}
	}, 30000);
}
