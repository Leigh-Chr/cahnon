/**
 * Central application state using Svelte 5 runes.
 *
 * All state is reactive via $state(). Derived values use getters.
 * Actions are methods that mutate state.
 *
 * @module
 */

import { untrack } from 'svelte';
import type { Project, Chapter, Scene, BibleEntry, WordCounts } from '$lib/api';
import { projectApi, chapterApi, sceneApi, bibleApi, statsApi } from '$lib/api';
import { SvelteMap } from 'svelte/reactivity';
import type { ViewMode, WorkMode, EditorSettings, FocusSettings } from './types';
import { defaultEditorSettings, defaultFocusSettings } from './types';

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
	// Stats
	// -------------------------------------------------------------------------
	wordCounts = $state<WordCounts | null>(null);

	// -------------------------------------------------------------------------
	// Quick Open
	// -------------------------------------------------------------------------
	isQuickOpenVisible = $state(false);

	// -------------------------------------------------------------------------
	// Search
	// -------------------------------------------------------------------------
	searchQuery = $state('');

	// -------------------------------------------------------------------------
	// UI State - Dialogs
	// -------------------------------------------------------------------------
	isDarkMode = $state(false);
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
	// Initialization
	// -------------------------------------------------------------------------

	constructor() {
		if (typeof window !== 'undefined') {
			this.initializeFromLocalStorage();
			this.setupStorageSync();
			this.setupSystemColorSchemeListener();
		}
	}

	private initializeFromLocalStorage() {
		// Dark mode
		const savedDarkMode = localStorage.getItem('darkMode');
		if (savedDarkMode !== null) {
			this.isDarkMode = savedDarkMode === 'true';
		} else {
			this.isDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
		}
		this.applyDarkMode();

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
	}

	private setupStorageSync() {
		// Use $effect.root for effects outside components
		$effect.root(() => {
			// Sync dark mode
			$effect(() => {
				const darkMode = this.isDarkMode;
				localStorage.setItem('darkMode', String(darkMode));
				// Use untrack to avoid creating additional dependencies
				untrack(() => this.applyDarkMode());
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
		});
	}

	private setupSystemColorSchemeListener() {
		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		mediaQuery.addEventListener('change', (e) => {
			// Only auto-update if user hasn't manually set a preference
			const savedDarkMode = localStorage.getItem('darkMode');
			if (savedDarkMode === null) {
				this.isDarkMode = e.matches;
			}
		});
	}

	private applyDarkMode() {
		if (typeof document !== 'undefined') {
			if (this.isDarkMode) {
				document.documentElement.classList.add('dark');
			} else {
				document.documentElement.classList.remove('dark');
			}
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
		this.isLoading = true;
		this.error = null;
		try {
			// Check file status and lock before opening
			const fileStatus = await projectApi.checkFileStatus(path);
			if (fileStatus.has_lock && fileStatus.lock_info) {
				const lockInfo = fileStatus.lock_info;
				const proceed = confirm(
					`This project may be open on another device:\n\n` +
						`Machine: ${lockInfo.machine_name}\n` +
						`Since: ${new Date(lockInfo.timestamp).toLocaleString()}\n\n` +
						`Opening it here may cause conflicts. Continue anyway?`
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
			await this.loadManuscript();
			await this.loadBible();
			await this.loadStats();
		} catch (e) {
			this.error = e as string;
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
		} catch (e) {
			this.error = e as string;
			throw e;
		} finally {
			this.isLoading = false;
		}
	}

	async closeProject() {
		// Release lock before closing
		if (this.projectPath) {
			try {
				await projectApi.releaseLock(this.projectPath);
			} catch (e) {
				console.warn('Failed to release lock:', e);
			}
		}
		await projectApi.close();
		this.project = null;
		this.projectPath = null;
		this.chapters = [];
		this.scenes = new SvelteMap();
		this.bibleEntries = [];
		this.selectedChapterId = null;
		this.selectedSceneId = null;
		this.wordCounts = null;
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

		// Auto-select first chapter/scene if none selected
		if (chaptersData.length > 0 && !this.selectedChapterId) {
			this.selectedChapterId = chaptersData[0].id;
			const firstScenes = scenesMap.get(chaptersData[0].id);
			if (firstScenes && firstScenes.length > 0) {
				this.selectedSceneId = firstScenes[0].id;
			}
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

	async updateScene(id: string, data: Partial<Scene>) {
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
		this.hasUnsavedChanges = false;
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

	toggleWorkMode() {
		this.workMode = this.workMode === 'writing' ? 'revision' : 'writing';
	}

	toggleDarkMode() {
		this.isDarkMode = !this.isDarkMode;
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
