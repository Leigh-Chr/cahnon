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
export type ViewMode =
	| 'editor'
	| 'corkboard'
	| 'timeline'
	| 'bible'
	| 'issues'
	| 'names'
	| 'dashboard';

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

/**
 * A keyboard shortcut binding.
 * key: the keyboard key (e.g. 'k', '1', 'ArrowDown')
 * mod: requires Cmd/Ctrl
 * shift: requires Shift
 */
export interface ShortcutBinding {
	key: string;
	mod: boolean;
	shift: boolean;
}

/**
 * All customizable keyboard shortcuts, keyed by action name.
 */
export interface KeyboardShortcuts {
	quickOpen: ShortcutBinding;
	toggleOutline: ShortcutBinding;
	toggleContextPanel: ShortcutBinding;
	viewEditor: ShortcutBinding;
	viewCorkboard: ShortcutBinding;
	viewTimeline: ShortcutBinding;
	viewBible: ShortcutBinding;
	viewIssues: ShortcutBinding;
	viewNames: ShortcutBinding;
	viewDashboard: ShortcutBinding;
	toggleWorkMode: ShortcutBinding;
	nextScene: ShortcutBinding;
	prevScene: ShortcutBinding;
	save: ShortcutBinding;
	find: ShortcutBinding;
	findReplace: ShortcutBinding;
	export: ShortcutBinding;
	reviewGrid: ShortcutBinding;
	importDialog: ShortcutBinding;
	arcsManager: ShortcutBinding;
	eventsManager: ShortcutBinding;
	templatesManager: ShortcutBinding;
	fullscreen: ShortcutBinding;
	focusMode: ShortcutBinding;
}

export const defaultKeyboardShortcuts: KeyboardShortcuts = {
	quickOpen: { key: 'k', mod: true, shift: false },
	toggleOutline: { key: '\\', mod: true, shift: false },
	toggleContextPanel: { key: '\\', mod: true, shift: true },
	viewEditor: { key: '1', mod: true, shift: false },
	viewCorkboard: { key: '2', mod: true, shift: false },
	viewTimeline: { key: '3', mod: true, shift: false },
	viewBible: { key: '4', mod: true, shift: false },
	viewIssues: { key: '5', mod: true, shift: false },
	viewNames: { key: '6', mod: true, shift: false },
	viewDashboard: { key: '7', mod: true, shift: false },
	toggleWorkMode: { key: 'd', mod: true, shift: false },
	nextScene: { key: 'ArrowDown', mod: true, shift: false },
	prevScene: { key: 'ArrowUp', mod: true, shift: false },
	save: { key: 's', mod: true, shift: false },
	find: { key: 'f', mod: true, shift: false },
	findReplace: { key: 'h', mod: true, shift: false },
	export: { key: 'e', mod: true, shift: false },
	reviewGrid: { key: 'g', mod: true, shift: false },
	importDialog: { key: 'i', mod: true, shift: false },
	arcsManager: { key: 'a', mod: true, shift: false },
	eventsManager: { key: 'v', mod: true, shift: false },
	templatesManager: { key: 't', mod: true, shift: false },
	fullscreen: { key: 'F11', mod: false, shift: false },
	focusMode: { key: 'f', mod: true, shift: true },
};

/** Friendly labels for shortcut actions. */
export const shortcutLabels: Record<keyof KeyboardShortcuts, string> = {
	quickOpen: 'Quick Open',
	toggleOutline: 'Toggle Outline',
	toggleContextPanel: 'Toggle Context Panel',
	viewEditor: 'Editor View',
	viewCorkboard: 'Corkboard View',
	viewTimeline: 'Timeline View',
	viewBible: 'Bible View',
	viewIssues: 'Issues View',
	viewNames: 'Names View',
	viewDashboard: 'Dashboard View',
	toggleWorkMode: 'Toggle Work Mode',
	nextScene: 'Next Scene',
	prevScene: 'Previous Scene',
	save: 'Save',
	find: 'Find',
	findReplace: 'Find & Replace',
	export: 'Export',
	reviewGrid: 'Review Grid',
	importDialog: 'Import',
	arcsManager: 'Arcs Manager',
	eventsManager: 'Events Manager',
	templatesManager: 'Templates Manager',
	fullscreen: 'Fullscreen',
	focusMode: 'Focus Mode',
};
