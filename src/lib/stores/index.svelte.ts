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
export type { EditorSettings, FocusSettings, ViewMode, WorkMode } from './types';

// App State (main state class)
export { appState } from './app-state.svelte';

// Recovery utilities
export type { RecoveryDraft } from './recovery';
export {
	clearRecoveryDraft,
	clearRecoveryDraftForScene,
	getAllRecoveryDrafts,
	getRecoveryDraft,
	hasRecoveryDrafts,
	saveRecoveryDraft,
} from './recovery';

// Onboarding utilities (AL1)
export type { OnboardingState } from './onboarding';

// Global Undo Stack (UD1, UD2)
export { undoStack } from './undo';
