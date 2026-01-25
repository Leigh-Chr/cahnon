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

// App State (main state class)
export { appState } from './app-state.svelte';

// Recovery utilities
export type { RecoveryDraft } from './recovery';
export { saveRecoveryDraft, getRecoveryDraft, clearRecoveryDraft } from './recovery';

