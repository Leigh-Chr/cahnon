/**
 * Central application state using Svelte 5 runes.
 *
 * All state is reactive via $state(). Derived values use getters.
 * Actions are methods that mutate state.
 *
 * @module
 */

import { untrack } from 'svelte';
import { SvelteMap } from 'svelte/reactivity';

import type { BibleEntry, Chapter, Project, Scene, SceneHealth, WordCounts } from '$lib/api';
import {
	bibleApi,
	chapterApi,
	healthApi,
	issueApi,
	projectApi,
	sceneApi,
	snapshotApi,
	statsApi,
	trashApi,
} from '$lib/api';
import { showError } from '$lib/toast.svelte';
import { nativeConfirm } from '$lib/utils/native-dialog';
import type { RevisionPassId } from '$lib/utils/revision-passes';

import type {
	EditorSettings,
	FocusSettings,
	KeyboardShortcuts,
	ShortcutBinding,
	ViewMode,
	WorkMode,
} from './types';
import { defaultEditorSettings, defaultFocusSettings, defaultKeyboardShortcuts } from './types';

const AUTOSAVE_INTERVAL_MS = 30_000; // 30 seconds
const SAVE_RETRY_MAX_ATTEMPTS = 3;
const SAVE_RETRY_DELAY_MS = 2_000;

/**
 * Central application state using Svelte 5 runes.
 *
 * @example
 * ```typescript
 * import { appState } from '$lib/stores';
 *
 * // Read state (reactive in components)
 * const mode = appState.viewMode;
 *
 * // Call actions
 * await appState.loadProject('/path/to/novel.cahnon');
 * appState.setViewMode('corkboard');
 * ```
 */
class AppState {
	// -------------------------------------------------------------------------
	// UI State
	// -------------------------------------------------------------------------
	viewMode = $state<ViewMode>('editor');
	workMode = $state<WorkMode>('writing');
	showOutline = $state(true);
	showContextPanel = $state(true);
	isLoading = $state(false);
	error = $state<string | null>(null);

	// -------------------------------------------------------------------------
	// Project State
	// -------------------------------------------------------------------------
	project = $state<Project | null>(null);
	projectPath = $state<string | null>(null);
	hasUnsavedChanges = $state(false);
	isDemo = $state(false);

	// -------------------------------------------------------------------------
	// Manuscript State
	// -------------------------------------------------------------------------
	chapters = $state<Chapter[]>([]);
	scenes = $state(new SvelteMap<string, Scene[]>());
	selectedChapterId = $state<string | null>(null);
	selectedSceneId = $state<string | null>(null);

	// -------------------------------------------------------------------------
	// Bible State
	// -------------------------------------------------------------------------
	bibleEntries = $state<BibleEntry[]>([]);
	selectedBibleEntryId = $state<string | null>(null);
	bibleFilter = $state<string | null>(null);

	// -------------------------------------------------------------------------
	// Entity Selection (for cross-view navigation)
	// -------------------------------------------------------------------------
	selectedArcId = $state<string | null>(null);
	selectedEventId = $state<string | null>(null);
	selectedIssueId = $state<string | null>(null);

	// -------------------------------------------------------------------------
	// Navigation History
	// -------------------------------------------------------------------------
	private _navHistory: Array<{ type: string; id: string; meta?: Record<string, string> }> = [];
	private _navIndex = -1;

	// -------------------------------------------------------------------------
	// Stats
	// -------------------------------------------------------------------------
	wordCounts = $state<WordCounts | null>(null);

	// -------------------------------------------------------------------------
	// Scene Health
	// -------------------------------------------------------------------------
	sceneHealthMap = $state(new SvelteMap<string, SceneHealth>());

	// -------------------------------------------------------------------------
	// Revision Pass
	// -------------------------------------------------------------------------
	revisionPass = $state<RevisionPassId | null>(null);

	// -------------------------------------------------------------------------
	// Status Bar Message
	// -------------------------------------------------------------------------
	statusMessage = $state<{ text: string; type: 'info' | 'success' | 'warning' } | null>(null);
	private _statusTimer: ReturnType<typeof setTimeout> | null = null;

	// -------------------------------------------------------------------------
	// Quick Open
	// -------------------------------------------------------------------------
	isQuickOpenVisible = $state(false);

	// -------------------------------------------------------------------------
	// Search
	// -------------------------------------------------------------------------
	searchQuery = $state('');

	// -------------------------------------------------------------------------
	// Character Thread
	// -------------------------------------------------------------------------
	characterThreadId = $state<string | null>(null);

	// -------------------------------------------------------------------------
	// UI State - Dialogs
	// -------------------------------------------------------------------------
	isExportDialogOpen = $state(false);
	isTrashViewOpen = $state(false);
	isCutLibraryOpen = $state(false);
	isSnapshotsViewOpen = $state(false);

	// -------------------------------------------------------------------------
	// Settings
	// -------------------------------------------------------------------------
	editorSettings = $state<EditorSettings>({ ...defaultEditorSettings });
	focusSettings = $state<FocusSettings>({ ...defaultFocusSettings });
	isFocusMode = $state(false);

	// Theme settings
	colorMode = $state<'light' | 'dark' | 'system'>('system');
	themePalette = $state<'cool' | 'warm'>('cool');

	// Keyboard shortcuts
	keyboardShortcuts = $state<KeyboardShortcuts>({ ...defaultKeyboardShortcuts });

	// -------------------------------------------------------------------------
	// Derived State (Getters)
	// -------------------------------------------------------------------------

	/** Get currently selected scene */
	get selectedScene(): Scene | null {
		if (!this.selectedSceneId) return null;
		for (const sceneList of this.scenes.values()) {
			const found = sceneList.find((s) => s.id === this.selectedSceneId);
			if (found) return found;
		}
		return null;
	}

	/** Get currently selected chapter */
	get selectedChapter(): Chapter | null {
		if (!this.selectedChapterId) return null;
		return this.chapters.find((c) => c.id === this.selectedChapterId) || null;
	}

	/** Get bible entries filtered by type */
	get filteredBibleEntries(): BibleEntry[] {
		if (!this.bibleFilter) return this.bibleEntries;
		return this.bibleEntries.filter((e) => e.entry_type === this.bibleFilter);
	}

	// -------------------------------------------------------------------------
	// Autosave & Retry
	// -------------------------------------------------------------------------
	private _pendingSave: (() => Promise<void>) | null = null;
	private _isSaving = false;
	private _pendingSceneId: string | null = null;
	private _pendingChapterId: string | null = null;
	private _autosaveIntervalId: ReturnType<typeof setInterval> | null = null;
	private _windowFocusHandler: (() => void) | null = null;
	private _colorSchemeHandler: ((e: MediaQueryListEvent) => void) | null = null;
	private _colorSchemeQuery: MediaQueryList | null = null;

	// -------------------------------------------------------------------------
	// Initialization
	// -------------------------------------------------------------------------

	constructor() {
		if (typeof window !== 'undefined') {
			this.initializeFromLocalStorage();
			this.setupStorageSync();
			this.setupSystemColorSchemeListener();
			this.setupAutosaveTimer();
			this.setupWindowFocusDetection();
		}
	}

	private initializeFromLocalStorage() {
		// Color mode (light/dark/system)
		const savedColorMode = localStorage.getItem('colorMode') as 'light' | 'dark' | 'system' | null;
		if (savedColorMode && ['light', 'dark', 'system'].includes(savedColorMode)) {
			this.colorMode = savedColorMode;
		}
		this.applyColorMode();

		// Theme palette (cool/warm)
		const savedThemePalette = localStorage.getItem('themePalette') as 'cool' | 'warm' | null;
		if (savedThemePalette && ['cool', 'warm'].includes(savedThemePalette)) {
			this.themePalette = savedThemePalette;
		}
		this.applyThemePalette();

		// Editor settings
		const savedEditorSettings = localStorage.getItem('editorSettings');
		if (savedEditorSettings) {
			try {
				this.editorSettings = JSON.parse(savedEditorSettings);
			} catch (e) {
				console.error('Failed to parse editor settings:', e);
			}
		}
		this.applyEditorSettings();

		// Focus settings
		const savedFocusSettings = localStorage.getItem('focusSettings');
		if (savedFocusSettings) {
			try {
				this.focusSettings = JSON.parse(savedFocusSettings);
			} catch (e) {
				console.error('Failed to parse focus settings:', e);
			}
		}

		// Keyboard shortcuts
		const savedShortcuts = localStorage.getItem('keyboardShortcuts');
		if (savedShortcuts) {
			try {
				const parsed = JSON.parse(savedShortcuts);
				// Merge with defaults to handle newly added shortcuts
				this.keyboardShortcuts = { ...defaultKeyboardShortcuts, ...parsed };
			} catch (e) {
				console.error('Failed to parse keyboard shortcuts:', e);
			}
		}

		// Panel visibility
		const savedShowOutline = localStorage.getItem('showOutline');
		if (savedShowOutline !== null) {
			this.showOutline = savedShowOutline === 'true';
		}
		const savedShowContextPanel = localStorage.getItem('showContextPanel');
		if (savedShowContextPanel !== null) {
			this.showContextPanel = savedShowContextPanel === 'true';
		}

		// View mode
		const savedViewMode = localStorage.getItem('viewMode') as ViewMode | null;
		const validViewModes: ViewMode[] = [
			'editor',
			'corkboard',
			'timeline',
			'bible',
			'issues',
			'dashboard',
		];
		if (savedViewMode && validViewModes.includes(savedViewMode)) {
			this.viewMode = savedViewMode;
		}

		// Work mode
		const savedWorkMode = localStorage.getItem('workMode') as WorkMode | null;
		if (savedWorkMode && ['writing', 'revision'].includes(savedWorkMode)) {
			this.workMode = savedWorkMode;
		}

		// Pending selection (applied after manuscript loads)
		this._pendingSceneId = localStorage.getItem('selectedSceneId');
		this._pendingChapterId = localStorage.getItem('selectedChapterId');
	}

	private setupStorageSync() {
		// Use $effect.root for effects outside components
		$effect.root(() => {
			// Sync color mode
			$effect(() => {
				const mode = this.colorMode;
				localStorage.setItem('colorMode', mode);
				untrack(() => this.applyColorMode());
			});

			// Sync theme palette
			$effect(() => {
				const palette = this.themePalette;
				localStorage.setItem('themePalette', palette);
				untrack(() => this.applyThemePalette());
			});

			// Sync editor settings
			$effect(() => {
				const settings = this.editorSettings;
				localStorage.setItem('editorSettings', JSON.stringify(settings));
				// Use untrack to avoid creating additional dependencies
				untrack(() => this.applyEditorSettings());
			});

			// Sync focus settings
			$effect(() => {
				const focusSettings = this.focusSettings;
				localStorage.setItem('focusSettings', JSON.stringify(focusSettings));
			});

			// Sync keyboard shortcuts
			$effect(() => {
				const shortcuts = this.keyboardShortcuts;
				localStorage.setItem('keyboardShortcuts', JSON.stringify(shortcuts));
			});

			// Sync panel visibility
			$effect(() => {
				localStorage.setItem('showOutline', String(this.showOutline));
			});
			$effect(() => {
				localStorage.setItem('showContextPanel', String(this.showContextPanel));
			});

			// Sync view mode
			$effect(() => {
				localStorage.setItem('viewMode', this.viewMode);
			});

			// Sync work mode
			$effect(() => {
				localStorage.setItem('workMode', this.workMode);
			});

			// Sync selection
			$effect(() => {
				if (this.selectedSceneId) {
					localStorage.setItem('selectedSceneId', this.selectedSceneId);
				} else {
					localStorage.removeItem('selectedSceneId');
				}
			});
			$effect(() => {
				if (this.selectedChapterId) {
					localStorage.setItem('selectedChapterId', this.selectedChapterId);
				} else {
					localStorage.removeItem('selectedChapterId');
				}
			});
		});
	}

	private setupSystemColorSchemeListener() {
		this._colorSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)');
		this._colorSchemeHandler = () => {
			// Re-apply if using system mode
			if (this.colorMode === 'system') {
				this.applyColorMode();
			}
		};
		this._colorSchemeQuery.addEventListener('change', this._colorSchemeHandler);
	}

	/** Periodic autosave every 30 seconds when there are unsaved changes. */
	private setupAutosaveTimer() {
		this._autosaveIntervalId = setInterval(() => {
			if (this.hasUnsavedChanges && this._pendingSave && !this._isSaving) {
				this._guardedSave();
			}
		}, AUTOSAVE_INTERVAL_MS);
	}

	/** Register a callback to trigger autosave (called by editor). */
	registerAutosaveCallback(callback: () => Promise<void>) {
		this._pendingSave = callback;
	}

	/** Trigger an immediate save (e.g., Cmd+S). */
	triggerImmediateSave() {
		if (this._pendingSave && !this._isSaving) {
			this._guardedSave();
		}
	}

	/** Execute a save with a concurrency guard to prevent overlapping saves. */
	private async _guardedSave() {
		if (this._isSaving || !this._pendingSave) return;
		this._isSaving = true;
		try {
			await this._pendingSave();
		} finally {
			this._isSaving = false;
		}
	}

	/** On window focus, check for external modification of the project file. */
	private setupWindowFocusDetection() {
		this._windowFocusHandler = async () => {
			if (!this.projectPath) return;
			try {
				const status = await projectApi.checkFileStatus(this.projectPath);
				if (status.is_modified_externally) {
					const reload = await nativeConfirm(
						'The project file has been modified externally.\n\n' +
							'Would you like to reload the project to get the latest changes?\n' +
							'Click Cancel to keep your current version.',
						'External Modification Detected'
					);
					if (reload) {
						await this.loadManuscript();
						await this.loadBible();
						await this.loadStats();
						this.hasUnsavedChanges = false;
					}
				}
			} catch {
				// Silently ignore - file status check is best-effort
			}
		};
		window.addEventListener('focus', this._windowFocusHandler);
	}

	/** Clean up all event listeners and timers. */
	destroy() {
		if (this._autosaveIntervalId !== null) {
			clearInterval(this._autosaveIntervalId);
			this._autosaveIntervalId = null;
		}
		if (this._windowFocusHandler) {
			window.removeEventListener('focus', this._windowFocusHandler);
			this._windowFocusHandler = null;
		}
		if (this._colorSchemeQuery && this._colorSchemeHandler) {
			this._colorSchemeQuery.removeEventListener('change', this._colorSchemeHandler);
			this._colorSchemeHandler = null;
			this._colorSchemeQuery = null;
		}
	}

	/** Save with automatic retry on failure. */
	async saveWithRetry(saveFn: () => Promise<void>, context = 'save'): Promise<boolean> {
		for (let attempt = 1; attempt <= SAVE_RETRY_MAX_ATTEMPTS; attempt++) {
			try {
				await saveFn();
				return true;
			} catch (e) {
				console.error(`${context} failed (attempt ${attempt}/${SAVE_RETRY_MAX_ATTEMPTS}):`, e);
				if (attempt < SAVE_RETRY_MAX_ATTEMPTS) {
					await new Promise((r) => setTimeout(r, SAVE_RETRY_DELAY_MS));
				}
			}
		}
		// All retries failed
		this.error = `Failed to ${context} after ${SAVE_RETRY_MAX_ATTEMPTS} attempts. Your changes are preserved in memory.`;
		showError(
			`Failed to ${context} after ${SAVE_RETRY_MAX_ATTEMPTS} attempts. Your changes are preserved in memory.`
		);
		return false;
	}

	private applyColorMode() {
		if (typeof document !== 'undefined') {
			const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
			let effectiveMode: 'light' | 'dark';

			if (this.colorMode === 'system') {
				effectiveMode = prefersDark ? 'dark' : 'light';
			} else {
				effectiveMode = this.colorMode;
			}

			// Set color-scheme property (used by light-dark() CSS function)
			document.documentElement.style.colorScheme = effectiveMode;
			// Set data attribute for potential CSS selectors
			document.documentElement.dataset.colorMode = this.colorMode;
		}
	}

	private applyThemePalette() {
		if (typeof document !== 'undefined') {
			document.documentElement.dataset.theme = this.themePalette;
		}
	}

	private applyEditorSettings() {
		if (typeof document !== 'undefined') {
			document.documentElement.style.setProperty(
				'--editor-font-family',
				this.editorSettings.fontFamily
			);
			document.documentElement.style.setProperty(
				'--editor-font-size',
				`${this.editorSettings.fontSize}px`
			);
			document.documentElement.style.setProperty(
				'--editor-line-height',
				String(this.editorSettings.lineHeight)
			);
		}
	}

	// -------------------------------------------------------------------------
	// Actions - Project
	// -------------------------------------------------------------------------

	/** Opens a project file and loads all data. */
	async loadProject(path: string) {
		// Check for unsaved changes in current project
		if (this.project && this.hasUnsavedChanges) {
			const confirmed = await nativeConfirm(
				'You have unsaved changes in the current project. Are you sure you want to open a different project?\n\n' +
					'Any unsaved changes will be lost.',
				'Unsaved Changes'
			);
			if (!confirmed) {
				return;
			}
		}

		this.isLoading = true;
		this.error = null;
		try {
			// Check file status and lock before opening
			const fileStatus = await projectApi.checkFileStatus(path);
			if (fileStatus.has_lock && fileStatus.lock_info) {
				const lockInfo = fileStatus.lock_info;
				const proceed = await nativeConfirm(
					`This project may be open on another device:\n\n` +
						`Machine: ${lockInfo.machine_name}\n` +
						`Since: ${new Date(lockInfo.timestamp).toLocaleString()}\n\n` +
						`Opening it here may cause conflicts. Continue anyway?`,
					'Project Locked'
				);
				if (!proceed) {
					this.isLoading = false;
					return;
				}
				// Force acquire lock if user proceeds
				await projectApi.forceAcquireLock(path);
			} else {
				// Acquire lock for this session
				await projectApi.acquireLock(path);
			}

			const p = await projectApi.open(path);
			this.project = p;
			this.projectPath = path;

			// Check database integrity on open
			try {
				await projectApi.checkDatabaseIntegrity();
			} catch (integrityError) {
				const msg =
					integrityError instanceof Error ? integrityError.message : String(integrityError);
				const proceed = await nativeConfirm(
					`Warning: Database integrity check detected an issue:\n\n${msg}\n\n` +
						`The file may be corrupted. It is recommended to create a backup before continuing.\n\n` +
						`Continue opening the project anyway?`,
					'Database Integrity Warning'
				);
				if (!proceed) {
					await projectApi.close();
					this.project = null;
					this.projectPath = null;
					this.isLoading = false;
					return;
				}
			}

			// Cleanup expired pre_bulk snapshots (30-day retention)
			try {
				await snapshotApi.cleanupExpired();
			} catch {
				// Non-critical, ignore
			}

			// Purge trash items older than 30 days
			try {
				await trashApi.purgeExpired();
			} catch {
				// Non-critical, ignore
			}

			await this.loadManuscript();
			await this.loadBible();
			await this.loadStats();
			this.loadSceneHealth(); // Non-blocking
			this.runDetections(); // Non-blocking
			this.hasUnsavedChanges = false;
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
			throw e;
		} finally {
			this.isLoading = false;
		}
	}

	async createProject(path: string, title: string, author?: string, description?: string) {
		this.isLoading = true;
		this.error = null;
		try {
			const p = await projectApi.create(path, title, author, description);
			this.project = p;
			this.projectPath = path;
			this.chapters = [];
			this.scenes = new SvelteMap();
			this.bibleEntries = [];
			this.wordCounts = { total: 0, by_chapter: [], by_status: [] };
			this.hasUnsavedChanges = false;
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
			throw e;
		} finally {
			this.isLoading = false;
		}
	}

	async closeProject(force = false): Promise<boolean> {
		// Check for unsaved changes unless force closing (skip for demo)
		if (!force && !this.isDemo && this.hasUnsavedChanges) {
			const confirmed = await nativeConfirm(
				'You have unsaved changes. Are you sure you want to close this project?\n\n' +
					'Any unsaved changes will be lost.',
				'Unsaved Changes'
			);
			if (!confirmed) {
				return false;
			}
		}

		// Release lock before closing (not needed for demo)
		if (this.projectPath && !this.isDemo) {
			try {
				await projectApi.releaseLock(this.projectPath);
			} catch (e) {
				console.warn('Failed to release lock:', e);
			}
		}
		await projectApi.close();
		this.project = null;
		this.projectPath = null;
		this.isDemo = false;
		this.chapters = [];
		this.scenes = new SvelteMap();
		this.bibleEntries = [];
		this.selectedChapterId = null;
		this.selectedSceneId = null;
		this.wordCounts = null;
		this.sceneHealthMap = new SvelteMap();
		this.revisionPass = null;
		this.hasUnsavedChanges = false;
		return true;
	}

	async loadDemoProject() {
		this.isLoading = true;
		this.error = null;
		try {
			const p = await projectApi.openDemo();
			this.project = p;
			this.projectPath = null;
			this.isDemo = true;

			await this.loadManuscript();
			await this.loadBible();
			await this.loadStats();
			this.loadSceneHealth(); // Non-blocking
			this.runDetections(); // Non-blocking
			this.hasUnsavedChanges = false;
		} catch (e) {
			this.error = e instanceof Error ? e.message : String(e);
			throw e;
		} finally {
			this.isLoading = false;
		}
	}

	async loadManuscript() {
		const chaptersData = await chapterApi.getAll();
		this.chapters = chaptersData;

		const scenesMap = new SvelteMap<string, Scene[]>();
		for (const chapter of chaptersData) {
			const chapterScenes = await sceneApi.getByChapter(chapter.id);
			scenesMap.set(chapter.id, chapterScenes);
		}
		this.scenes = scenesMap;

		// Try to restore pending selection from previous session
		let restored = false;
		if (this._pendingChapterId && this._pendingSceneId) {
			const chapterExists = chaptersData.some((c) => c.id === this._pendingChapterId);
			const sceneExists =
				scenesMap.get(this._pendingChapterId)?.some((s) => s.id === this._pendingSceneId) ?? false;
			if (chapterExists && sceneExists) {
				this.selectedChapterId = this._pendingChapterId;
				this.selectedSceneId = this._pendingSceneId;
				restored = true;
			}
			this._pendingSceneId = null;
			this._pendingChapterId = null;
		}

		// Auto-select first chapter/scene if none selected and nothing restored
		if (!restored && chaptersData.length > 0 && !this.selectedChapterId) {
			this.selectedChapterId = chaptersData[0].id;
			const firstScenes = scenesMap.get(chaptersData[0].id);
			if (firstScenes && firstScenes.length > 0) {
				this.selectedSceneId = firstScenes[0].id;
			}
		}
	}

	async reloadScenes() {
		for (const chapter of this.chapters) {
			const chapterScenes = await sceneApi.getByChapter(chapter.id);
			this.scenes.set(chapter.id, chapterScenes);
		}
	}

	async loadBible() {
		const entries = await bibleApi.getAll();
		this.bibleEntries = entries;
	}

	async loadStats() {
		const stats = await statsApi.getWordCounts();
		this.wordCounts = stats;
	}

	async loadSceneHealth() {
		try {
			const healthData = await healthApi.getBatch();
			const map = new SvelteMap<string, SceneHealth>();
			for (const h of healthData) {
				map.set(h.scene_id, h);
			}
			this.sceneHealthMap = map;
		} catch (e) {
			console.error('Failed to load scene health:', e);
		}
	}

	/** Run issue detections in background (non-blocking). */
	async runDetections() {
		try {
			await issueApi.runDetections();
		} catch (e) {
			console.warn('Background detection failed:', e);
		}
	}

	setRevisionPass(pass: RevisionPassId | null) {
		this.revisionPass = pass;
	}

	// -------------------------------------------------------------------------
	// Actions - Chapters
	// -------------------------------------------------------------------------

	async createChapter(title: string, summary?: string) {
		const chapter = await chapterApi.create(title, summary);
		this.chapters = [...this.chapters, chapter];
		this.scenes.set(chapter.id, []);
		this.selectedChapterId = chapter.id;
		return chapter;
	}

	async updateChapter(id: string, data: Partial<Chapter>) {
		const chapter = await chapterApi.update(id, data);
		this.chapters = this.chapters.map((ch) => (ch.id === id ? chapter : ch));
		return chapter;
	}

	async deleteChapter(id: string) {
		await chapterApi.delete(id);
		this.chapters = this.chapters.filter((ch) => ch.id !== id);
		this.scenes.delete(id);
		if (this.selectedChapterId === id) {
			this.selectedChapterId = this.chapters.length > 0 ? this.chapters[0].id : null;
		}
	}

	async loadChapters() {
		const chaptersData = await chapterApi.getAll();
		this.chapters = chaptersData;

		const scenesMap = new SvelteMap<string, Scene[]>();
		for (const chapter of chaptersData) {
			const chapterScenes = await sceneApi.getByChapter(chapter.id);
			scenesMap.set(chapter.id, chapterScenes);
		}
		this.scenes = scenesMap;
	}

	// -------------------------------------------------------------------------
	// Actions - Scenes
	// -------------------------------------------------------------------------

	async createScene(chapterId: string, title: string, summary?: string) {
		const scene = await sceneApi.create(chapterId, title, summary);
		const existing = this.scenes.get(chapterId) || [];
		this.scenes.set(chapterId, [...existing, scene]);
		this.selectedSceneId = scene.id;
		return scene;
	}

	private _saveVersion = 0;

	/** Mark content as changed. Returns a version token for use with updateScene. */
	markUnsaved(): number {
		this.hasUnsavedChanges = true;
		return ++this._saveVersion;
	}

	async updateScene(id: string, data: Partial<Scene>, saveVersion?: number) {
		const scene = await sceneApi.update(id, data);
		for (const [chapterId, sceneList] of this.scenes.entries()) {
			const idx = sceneList.findIndex((sc) => sc.id === id);
			if (idx !== -1) {
				const newList = [...sceneList];
				newList[idx] = scene;
				this.scenes.set(chapterId, newList);
				break;
			}
		}
		// Only clear hasUnsavedChanges if no new changes happened since this save started
		if (saveVersion === undefined || saveVersion === this._saveVersion) {
			this.hasUnsavedChanges = false;
		}
		return scene;
	}

	async deleteScene(id: string) {
		await sceneApi.delete(id);
		for (const [chapterId, sceneList] of this.scenes.entries()) {
			const filtered = sceneList.filter((sc) => sc.id !== id);
			if (filtered.length !== sceneList.length) {
				this.scenes.set(chapterId, filtered);
				break;
			}
		}
		if (this.selectedSceneId === id) {
			this.selectedSceneId = null;
		}
	}

	selectScene(sceneId: string, chapterId?: string) {
		this.selectedSceneId = sceneId;
		if (chapterId) {
			this.selectedChapterId = chapterId;
		}
	}

	// -------------------------------------------------------------------------
	// Actions - Bible
	// -------------------------------------------------------------------------

	async createBibleEntry(data: Parameters<typeof bibleApi.create>[0]) {
		const entry = await bibleApi.create(data);
		this.bibleEntries = [...this.bibleEntries, entry];
		return entry;
	}

	async updateBibleEntry(id: string, data: Partial<BibleEntry>) {
		const entry = await bibleApi.update(id, data);
		this.bibleEntries = this.bibleEntries.map((en) => (en.id === id ? entry : en));
		return entry;
	}

	async deleteBibleEntry(id: string) {
		await bibleApi.delete(id);
		this.bibleEntries = this.bibleEntries.filter((en) => en.id !== id);
	}

	// -------------------------------------------------------------------------
	// Actions - UI Toggles
	// -------------------------------------------------------------------------

	toggleOutline() {
		this.showOutline = !this.showOutline;
	}

	toggleContextPanel() {
		this.showContextPanel = !this.showContextPanel;
	}

	toggleQuickOpen() {
		this.isQuickOpenVisible = !this.isQuickOpenVisible;
	}

	setViewMode(mode: ViewMode) {
		this.viewMode = mode;
	}

	/** Navigate to a specific entity, switching view and pushing to history. */
	navigateTo(
		type: 'scene' | 'bible' | 'arc' | 'event' | 'issue' | 'dashboard',
		id?: string,
		meta?: Record<string, string>
	) {
		// Push to history (truncate forward entries)
		this._navHistory = this._navHistory.slice(0, this._navIndex + 1);
		this._navHistory.push({ type, id: id || '', meta });
		this._navIndex = this._navHistory.length - 1;

		switch (type) {
			case 'scene':
				if (id) {
					for (const [chapterId, scenes] of this.scenes) {
						const scene = scenes.find((s) => s.id === id);
						if (scene) {
							this.selectedChapterId = chapterId;
							this.selectedSceneId = id;
							break;
						}
					}
				}
				this.viewMode = 'editor';
				break;
			case 'bible':
				if (id) this.selectedBibleEntryId = id;
				this.viewMode = 'bible';
				break;
			case 'arc':
				if (id) this.selectedArcId = id;
				this.viewMode = 'timeline';
				break;
			case 'event':
				if (id) this.selectedEventId = id;
				this.viewMode = 'timeline';
				break;
			case 'issue':
				if (id) this.selectedIssueId = id;
				this.viewMode = 'issues';
				break;
			case 'dashboard':
				this.viewMode = 'dashboard';
				break;
		}
	}

	/** Navigate back in history. */
	navigateBack() {
		if (this._navIndex <= 0) return;
		this._navIndex--;
		const entry = this._navHistory[this._navIndex];
		if (entry) {
			this._applyNavEntry(entry);
		}
	}

	/** Navigate forward in history. */
	navigateForward() {
		if (this._navIndex >= this._navHistory.length - 1) return;
		this._navIndex++;
		const entry = this._navHistory[this._navIndex];
		if (entry) {
			this._applyNavEntry(entry);
		}
	}

	get canNavigateBack(): boolean {
		return this._navIndex > 0;
	}

	get canNavigateForward(): boolean {
		return this._navIndex < this._navHistory.length - 1;
	}

	private _applyNavEntry(entry: { type: string; id: string; meta?: Record<string, string> }) {
		switch (entry.type) {
			case 'scene':
				if (entry.id) {
					for (const [chapterId, scenes] of this.scenes) {
						if (scenes.find((s) => s.id === entry.id)) {
							this.selectedChapterId = chapterId;
							this.selectedSceneId = entry.id;
							break;
						}
					}
				}
				this.viewMode = 'editor';
				break;
			case 'bible':
				if (entry.id) this.selectedBibleEntryId = entry.id;
				this.viewMode = 'bible';
				break;
			case 'arc':
				if (entry.id) this.selectedArcId = entry.id;
				this.viewMode = 'timeline';
				break;
			case 'event':
				if (entry.id) this.selectedEventId = entry.id;
				this.viewMode = 'timeline';
				break;
			case 'issue':
				if (entry.id) this.selectedIssueId = entry.id;
				this.viewMode = 'issues';
				break;
			case 'dashboard':
				this.viewMode = 'dashboard';
				break;
		}
	}

	toggleWorkMode() {
		this.workMode = this.workMode === 'writing' ? 'revision' : 'writing';
	}

	setColorMode(mode: 'light' | 'dark' | 'system') {
		this.colorMode = mode;
	}

	setThemePalette(palette: 'cool' | 'warm') {
		this.themePalette = palette;
	}

	/** Check if a keyboard event matches a named shortcut. */
	matchesShortcut(event: KeyboardEvent, action: keyof KeyboardShortcuts): boolean {
		const binding = this.keyboardShortcuts[action];
		if (!binding) return false;

		const isMac = navigator.platform.includes('Mac');
		const modPressed = isMac ? event.metaKey : event.ctrlKey;

		return (
			event.key === binding.key && modPressed === binding.mod && event.shiftKey === binding.shift
		);
	}

	/** Update a single keyboard shortcut binding. */
	setShortcut(action: keyof KeyboardShortcuts, binding: ShortcutBinding) {
		this.keyboardShortcuts = { ...this.keyboardShortcuts, [action]: binding };
	}

	/** Reset all keyboard shortcuts to defaults. */
	resetShortcuts() {
		this.keyboardShortcuts = { ...defaultKeyboardShortcuts };
	}

	/** @deprecated Use setColorMode instead */
	toggleDarkMode() {
		// Legacy support: cycle through light -> dark -> system
		if (this.colorMode === 'light') {
			this.colorMode = 'dark';
		} else if (this.colorMode === 'dark') {
			this.colorMode = 'system';
		} else {
			this.colorMode = 'light';
		}
	}

	/** Helper to check if currently in dark mode (resolved) */
	get isDarkMode(): boolean {
		if (typeof window === 'undefined') return false;
		if (this.colorMode === 'system') {
			return window.matchMedia('(prefers-color-scheme: dark)').matches;
		}
		return this.colorMode === 'dark';
	}

	showCharacterThread(bibleEntryId: string) {
		this.characterThreadId = bibleEntryId;
	}

	closeCharacterThread() {
		this.characterThreadId = null;
	}

	openExportDialog() {
		this.isExportDialogOpen = true;
	}

	closeExportDialog() {
		this.isExportDialogOpen = false;
	}

	openTrashView() {
		this.isTrashViewOpen = true;
	}

	closeTrashView() {
		this.isTrashViewOpen = false;
	}

	openCutLibrary() {
		this.isCutLibraryOpen = true;
	}

	closeCutLibrary() {
		this.isCutLibraryOpen = false;
	}

	openSnapshotsView() {
		this.isSnapshotsViewOpen = true;
	}

	closeSnapshotsView() {
		this.isSnapshotsViewOpen = false;
	}

	/** Show a transient message in the status bar. */
	showStatusMessage(text: string, type: 'info' | 'success' | 'warning' = 'info', duration = 3000) {
		if (this._statusTimer) clearTimeout(this._statusTimer);
		this.statusMessage = { text, type };
		this._statusTimer = setTimeout(() => {
			this.statusMessage = null;
			this._statusTimer = null;
		}, duration);
	}

	// -------------------------------------------------------------------------
	// Actions - Focus Mode
	// -------------------------------------------------------------------------

	toggleFocusMode() {
		this.isFocusMode = !this.isFocusMode;
	}

	setFocusSetting<K extends keyof FocusSettings>(key: K, value: FocusSettings[K]) {
		this.focusSettings = { ...this.focusSettings, [key]: value };
	}

	toggleTypewriterMode() {
		this.focusSettings = {
			...this.focusSettings,
			typewriterMode: !this.focusSettings.typewriterMode,
		};
	}

	toggleDimSurroundings() {
		this.focusSettings = {
			...this.focusSettings,
			dimSurroundings: !this.focusSettings.dimSurroundings,
		};
	}

	toggleFullscreenMode() {
		const newValue = !this.focusSettings.fullscreenMode;
		// Also toggle focus mode when entering/exiting fullscreen
		this.isFocusMode = newValue;
		this.focusSettings = { ...this.focusSettings, fullscreenMode: newValue };
	}
}

// =============================================================================
// Singleton Instance
// =============================================================================

export const appState = new AppState();
