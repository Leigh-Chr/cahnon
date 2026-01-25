/**
 * Type definitions for Cahnon application state.
 *
 * @module
 */

/**
 * Current view mode of the application.
 * - `editor`: Main writing view with TipTap editor
 * - `corkboard`: Card-based scene organization
 * - `timeline`: Chronological event view
 * - `bible`: Knowledge base browser
 * - `issues`: Consistency problems and issues view
 */
export type ViewMode = 'editor' | 'corkboard' | 'timeline' | 'bible' | 'issues';

/**
 * Current work mode affecting UI presentation.
 * - `writing`: Minimal interface for distraction-free writing
 * - `revision`: Shows analytical tools (annotations, review grid, etc.)
 */
export type WorkMode = 'writing' | 'revision';

/**
 * Typography settings for the text editor.
 * Persisted in localStorage.
 */
export interface EditorSettings {
	/** Font family (e.g., 'Georgia, serif') */
	fontFamily: string;
	/** Font size in pixels */
	fontSize: number;
	/** Line height multiplier */
	lineHeight: number;
	/** Max width of text column in pixels */
	textWidth: number;
}

/**
 * Focus mode settings for distraction-free writing.
 * Persisted in localStorage.
 */
export interface FocusSettings {
	/** Keep cursor vertically centered while typing */
	typewriterMode: boolean;
	/** Dim paragraphs other than the current one */
	dimSurroundings: boolean;
	/** Hide all UI except the editor */
	fullscreenMode: boolean;
}

export const defaultEditorSettings: EditorSettings = {
	fontFamily: 'Georgia, serif',
	fontSize: 18,
	lineHeight: 1.8,
	textWidth: 700,
};

export const defaultFocusSettings: FocusSettings = {
	typewriterMode: false,
	dimSurroundings: false,
	fullscreenMode: false,
};
