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
 * - `bible`: Knowledge base browser (Codex)
 * - `issues`: Consistency problems and issues view (Continuity)
 */
export type ViewMode = 'editor' | 'corkboard' | 'timeline' | 'bible' | 'issues' | 'dashboard';

/**
 * Current work mode affecting UI presentation.
 * - `writing`: Minimal interface for distraction-free writing
 * - `revision`: Shows analytical tools (annotations, review grid, etc.)
 */
export type WorkMode = 'writing' | 'revision';

/**
 * CA4: Editor color theme options.
 */
export type EditorTheme = 'default' | 'sepia' | 'dark' | 'low-contrast';

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
	/** CA4: Editor color theme */
	theme: EditorTheme;
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
	fontFamily: '"Source Serif 4", Charter, Georgia, serif',
	fontSize: 18,
	lineHeight: 1.8,
	textWidth: 700,
	theme: 'default',
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
	viewDashboard: ShortcutBinding;
	toggleWorkMode: ShortcutBinding;
	nextScene: ShortcutBinding;
	prevScene: ShortcutBinding;
	nextChapter: ShortcutBinding; // AD4
	prevChapter: ShortcutBinding; // AD4
	newChapter: ShortcutBinding;
	newScene: ShortcutBinding;
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
	addAnnotation: ShortcutBinding;
	showShortcuts: ShortcutBinding;
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
	viewDashboard: { key: '6', mod: true, shift: false },
	toggleWorkMode: { key: 'd', mod: true, shift: true },
	nextScene: { key: 'ArrowDown', mod: true, shift: false },
	prevScene: { key: 'ArrowUp', mod: true, shift: false },
	nextChapter: { key: 'ArrowDown', mod: true, shift: true }, // AD4
	prevChapter: { key: 'ArrowUp', mod: true, shift: true }, // AD4
	newChapter: { key: 'n', mod: true, shift: true },
	newScene: { key: 'n', mod: true, shift: false },
	save: { key: 's', mod: true, shift: false },
	find: { key: 'f', mod: true, shift: false },
	findReplace: { key: 'h', mod: true, shift: false },
	export: { key: 'e', mod: true, shift: false },
	reviewGrid: { key: 'g', mod: true, shift: false },
	importDialog: { key: 'i', mod: true, shift: true },
	arcsManager: { key: 'a', mod: true, shift: true },
	eventsManager: { key: 'v', mod: true, shift: false },
	templatesManager: { key: 't', mod: true, shift: true },
	fullscreen: { key: 'F11', mod: false, shift: false },
	focusMode: { key: 'f', mod: true, shift: true },
	addAnnotation: { key: 'm', mod: true, shift: true },
	showShortcuts: { key: '/', mod: true, shift: false },
};

/** Friendly labels for shortcut actions. */
export const shortcutLabels: Record<keyof KeyboardShortcuts, string> = {
	quickOpen: 'Quick Open',
	toggleOutline: 'Toggle Outline',
	toggleContextPanel: 'Toggle Context Panel',
	viewEditor: 'Editor View',
	viewCorkboard: 'Corkboard View',
	viewTimeline: 'Timeline View',
	viewBible: 'Codex View',
	viewIssues: 'Continuity View',
	viewDashboard: 'Dashboard View',
	toggleWorkMode: 'Toggle Work Mode',
	nextScene: 'Next Scene',
	prevScene: 'Previous Scene',
	nextChapter: 'Next Chapter', // AD4
	prevChapter: 'Previous Chapter', // AD4
	newChapter: 'New Chapter',
	newScene: 'New Scene',
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
	addAnnotation: 'Add Annotation',
	showShortcuts: 'Keyboard Shortcuts',
};
