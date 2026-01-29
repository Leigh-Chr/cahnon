<!--
  Rich text editor for scene content using TipTap.

  Features:
  - Auto-save with debouncing (2 seconds after typing stops)
  - Crash recovery drafts stored in localStorage
  - Typewriter mode (cursor stays vertically centered)
  - Dim surroundings mode (fades non-focused paragraphs)
  - Character/word count display
  - Scene split at cursor position
  - Cut library integration (save deleted text)
  - Find & replace
  - Scene version history
  - Configurable typography (font, size, line height, width)
-->
<script lang="ts">
	import { Editor } from '@tiptap/core';
	import CharacterCount from '@tiptap/extension-character-count';
	import Highlight from '@tiptap/extension-highlight';
	import Placeholder from '@tiptap/extension-placeholder';
	import Typography from '@tiptap/extension-typography';
	import StarterKit from '@tiptap/starter-kit';
	import { onDestroy, tick, untrack } from 'svelte';

	import type { Annotation, BibleEntry } from '$lib/api';
	import { annotationApi, associationApi, cutApi, issueApi, sceneApi, searchApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import {
		clearRecoveryDraftForScene,
		getRecoveryDraft,
		saveRecoveryDraft,
	} from '$lib/stores/recovery';
	import { AnnotationMark } from '$lib/tiptap/annotation-mark';
	import { showError, showSuccess, showWarning } from '$lib/toast';
	import { countWords, debounce, formatShortcut, sceneStatuses, statusColors } from '$lib/utils';
	import { isModKey } from '$lib/utils';
	import { truncate } from '$lib/utils';
	import {
		annotationTypes,
		getAnnotationStatusColor,
		getAnnotationType,
	} from '$lib/utils/annotations';
	import { nativeConfirm } from '$lib/utils/native-dialog';

	import CutLibrary from './CutLibrary.svelte';
	import EditorToolbar from './EditorToolbar.svelte';
	import type { FindReplaceScope } from './FindReplace.svelte';
	import FindReplace from './FindReplace.svelte';
	import NewProjectWelcome from './NewProjectWelcome.svelte';
	import SceneHistoryModal from './SceneHistoryModal.svelte';
	import { EmptyState } from './ui';

	// Derived values for proper reactivity tracking in templates
	let selectedScene = $derived(appState.selectedScene);
	let selectedChapter = $derived(appState.selectedChapter);

	// UB4: Scene position within chapter
	let scenePosition = $derived.by(() => {
		if (!selectedScene || !selectedChapter) return null;
		const chapterScenes = appState.scenes.get(selectedChapter.id) || [];
		const index = chapterScenes.findIndex((s) => s.id === selectedScene.id);
		if (index === -1) return null;
		return { current: index + 1, total: chapterScenes.length };
	});

	// Scene operations
	async function splitSceneAtCursor() {
		if (!editor || !appState.selectedScene || !appState.selectedChapter) return;

		const { from } = editor.state.selection;
		// Get cursor position in the text content
		const textBefore = editor.state.doc.textBetween(0, from);
		const splitPosition = textBefore.length;

		if (splitPosition === 0) {
			showError('Cannot split at the beginning of the scene');
			return;
		}

		// Z3: Confirmation before Split Scene
		const confirmed = await nativeConfirm(
			'Split scene at cursor?\n\nThis will create a new scene with the text after the cursor.',
			'Split Scene'
		);
		if (!confirmed) return;

		const newTitle = `${appState.selectedScene.title} (Part 2)`;

		try {
			const result = await sceneApi.split(appState.selectedScene.id, splitPosition, newTitle);
			await appState.loadChapters();
			// Select the first scene (which retains original content up to split)
			appState.selectScene(result.first_scene.id, appState.selectedChapter.id);
			showSuccess('Scene split successfully');
		} catch (e) {
			console.error('Failed to split scene:', e);
			showError('Failed to split scene');
		}
	}

	async function mergeWithNextScene() {
		if (!appState.selectedScene || !appState.selectedChapter) return;

		// Get scenes in this chapter
		const chapterScenes = await sceneApi.getByChapter(appState.selectedChapter.id);
		const currentIndex = chapterScenes.findIndex((s) => s.id === appState.selectedScene!.id);

		if (currentIndex === -1 || currentIndex >= chapterScenes.length - 1) {
			showError('No next scene to merge with');
			return;
		}

		const nextScene = chapterScenes[currentIndex + 1];

		// Z4: Improved merge scene confirmation message
		const confirmed = await nativeConfirm(
			`Merge "${appState.selectedScene.title}" with "${nextScene.title}"?\n\n` +
				`The content of "${nextScene.title}" will be appended to this scene, and the second scene will be deleted.\n\n` +
				`This cannot be undone.`,
			'Merge Scenes'
		);
		if (!confirmed) {
			return;
		}

		try {
			const mergedScene = await sceneApi.merge([appState.selectedScene.id, nextScene.id]);
			await appState.loadChapters();
			appState.selectScene(mergedScene.id, appState.selectedChapter.id);
			showSuccess('Scenes merged successfully');
		} catch (e) {
			console.error('Failed to merge scenes:', e);
			showError('Failed to merge scenes');
		}
	}

	let editorElement = $state<HTMLElement | null>(null);
	let editor = $state<Editor | null>(null);
	// UB1: Smooth scene transition
	let isTransitioning = $state(false);
	let isUpdating = $state(false);

	// T1: Live word count (updated in TipTap onUpdate)
	let liveWordCount = $state(0);

	// CA5: Session stats tracking
	let sessionStartTime = $state<Date | null>(null);
	let sessionStartWordCount = $state(0);

	// Initialize session on first scene load
	$effect(() => {
		// Track selectedScene but read sessionStartTime without tracking
		if (selectedScene && untrack(() => sessionStartTime) === null) {
			sessionStartTime = new Date();
			sessionStartWordCount = countWords(selectedScene.text);
		}
	});

	// Derived session stats
	let sessionWordsWritten = $derived(
		liveWordCount > sessionStartWordCount ? liveWordCount - sessionStartWordCount : 0
	);
	let sessionDuration = $derived.by(() => {
		if (!sessionStartTime) return 0;
		return Math.floor((Date.now() - sessionStartTime.getTime()) / 60000); // minutes
	});
	let sessionWPM = $derived(
		sessionDuration > 0 ? Math.round(sessionWordsWritten / sessionDuration) : 0
	);

	// Scene metadata editing
	let editingTitle = $state(false);
	let titleInput = $state<HTMLInputElement | null>(null);
	let titleJustSaved = $state(false); // AD3: Visual confirmation for title save

	// Auto-select title text when entering edit mode
	$effect(() => {
		if (editingTitle && titleInput) {
			tick().then(() => titleInput?.select());
		}
	});

	// History modal
	let showHistoryModal = $state(false);

	// AD1: Summary section expanded state (collapsed by default in writing mode)
	let summaryExpanded = $state(false);

	// Cut library
	let showCutLibrary = $state(false);

	// Find and Replace
	let showFindReplace = $state(false);
	let showReplace = $state(false);
	let findReplaceScope = $state<FindReplaceScope>('scene');
	let findReplaceHandle = $state<
		{ updateMatchInfo: (current: number, total: number) => void } | undefined
	>(undefined);
	let searchMarks = $state<Array<{ from: number; to: number }>>([]);
	let currentSearchIndex = $state(0);

	// AC3: Store last search parameters for re-execution on scene change
	let lastSearchParams = $state<{
		query: string;
		caseSensitive: boolean;
		wholeWord: boolean;
	} | null>(null);

	function handleKeydown(event: KeyboardEvent) {
		// Close context menu or annotation popover on Escape
		if (event.key === 'Escape' && contextMenu) {
			event.preventDefault();
			contextMenu = null;
			return;
		}
		if (event.key === 'Escape' && annotationPopover) {
			event.preventDefault();
			closeAnnotationPopover();
			return;
		}
		// Find: Cmd/Ctrl + F
		if (isModKey(event) && event.key === 'f') {
			event.preventDefault();
			showFindReplace = true;
			showReplace = event.shiftKey;
			return;
		}
		// Find & Replace: Cmd/Ctrl + H
		if (isModKey(event) && event.key === 'h') {
			event.preventDefault();
			showFindReplace = true;
			showReplace = true;
			return;
		}
		// Quick-add Bible entry: Cmd/Ctrl + Shift + B
		if (isModKey(event) && event.shiftKey && event.key === 'B') {
			event.preventDefault();
			openQuickAddBible();
			return;
		}
		// Add annotation: Cmd/Ctrl + Shift + A
		if (appState.matchesShortcut(event, 'addAnnotation')) {
			event.preventDefault();
			triggerAnnotationCreation();
			return;
		}
		// AC4: Context menu via keyboard (Shift+F10 or ContextMenu key)
		if ((event.shiftKey && event.key === 'F10') || event.key === 'ContextMenu') {
			event.preventDefault();
			openContextMenuAtCursor();
			return;
		}
		// Undo: Cmd/Ctrl + Z (handled by TipTap, but ensure focus)
		// Redo: Cmd/Ctrl + Shift + Z or Cmd/Ctrl + Y (handled by TipTap)
	}

	/** AC4: Open context menu at cursor position for keyboard users */
	function openContextMenuAtCursor() {
		if (!editor) return;
		const { empty } = editor.state.selection;
		if (empty) return;

		// Get cursor position from editor
		const { from } = editor.state.selection;
		const coords = editor.view.coordsAtPos(from);
		contextMenu = { x: coords.left, y: coords.bottom + 8 };
	}

	function undo() {
		editor?.chain().focus().undo().run();
	}

	function redo() {
		editor?.chain().focus().redo().run();
	}

	let canUndo = $state(false);
	let canRedo = $state(false);

	// Update can states when editor updates
	function updateCanStates() {
		canUndo = editor?.can().undo() ?? false;
		canRedo = editor?.can().redo() ?? false;
	}

	const saveScene = debounce(async (sceneId: string, text: string, saveVersion?: number) => {
		if (!isUpdating && sceneId === currentSceneId) {
			// Save recovery draft to localStorage (in case of crash)
			saveRecoveryDraft(sceneId, text);
			const saved = await appState.saveWithRetry(async () => {
				await appState.updateScene(sceneId, { text }, saveVersion);
			}, 'save scene');
			if (saved) {
				// Clear recovery draft after successful save
				clearRecoveryDraftForScene(sceneId);
			}
		}
	}, 1000);

	// Register autosave callback so the periodic timer can trigger saves
	appState.registerAutosaveCallback(async () => {
		if (editor && currentSceneId && appState.hasUnsavedChanges) {
			const version = appState.markUnsaved();
			await saveScene(currentSceneId, editor.getHTML(), version);
		}
	});

	// Recovery bar state - Z2: with enriched context
	let recoveryDraftAvailable = $state<{
		sceneId: string;
		text: string;
		timestamp: number;
		wordCount: number;
		preview: string;
	} | null>(null);

	/** Strip HTML tags from text for preview */
	function stripHtmlTags(html: string): string {
		return html
			.replace(/<[^>]*>/g, ' ')
			.replace(/\s+/g, ' ')
			.trim();
	}

	/** Check for a recovery draft for the given scene */
	function checkRecoveryDraft(sceneId: string, _targetEditor: Editor) {
		const recoveryDraft = getRecoveryDraft(sceneId);
		if (recoveryDraft) {
			// Z2: Enrich with timestamp, word count, and preview
			const plainText = stripHtmlTags(recoveryDraft.text);
			const words = countWords(plainText);
			const preview = plainText.slice(0, 50) + (plainText.length > 50 ? '...' : '');
			recoveryDraftAvailable = {
				sceneId: recoveryDraft.sceneId,
				text: recoveryDraft.text,
				timestamp: recoveryDraft.timestamp,
				wordCount: words,
				preview,
			};
			// T3: Show toast to alert user
			showWarning('Unsaved changes recovered — see the bar above the editor.');
		} else {
			recoveryDraftAvailable = null;
		}
	}

	function restoreRecoveryDraft() {
		if (!recoveryDraftAvailable || !editor) return;
		isUpdating = true;
		editor.commands.setContent(recoveryDraftAvailable.text);
		isUpdating = false;
		appState.markUnsaved();
		clearRecoveryDraftForScene(recoveryDraftAvailable.sceneId);
		recoveryDraftAvailable = null;
		showSuccess('Draft recovered');
	}

	function discardRecoveryDraft() {
		if (!recoveryDraftAvailable) return;
		clearRecoveryDraftForScene(recoveryDraftAvailable.sceneId);
		recoveryDraftAvailable = null;
	}

	function initEditor() {
		if (editor) {
			editor.destroy();
		}

		editor = new Editor({
			element: editorElement,
			extensions: [
				StarterKit.configure({
					heading: {
						levels: [1, 2, 3],
					},
				}),
				Placeholder.configure({
					placeholder: 'Start writing...',
				}),
				CharacterCount,
				Typography,
				Highlight,
				AnnotationMark,
			],
			content: appState.selectedScene?.text || '',
			editorProps: {
				attributes: {
					class: 'prose-editor',
				},
			},
			onUpdate: ({ editor: ed, transaction }) => {
				// T1: Update live word count
				liveWordCount = countWords(ed.getText());

				// UB2: Update session word count if session active
				if (appState.writingSessionActive) {
					appState.updateSessionWordCount(liveWordCount);
				}

				if (!isUpdating && currentSceneId) {
					// BA1: Track typing state with character delta
					const charsDelta = transaction.steps.reduce((acc, step) => {
						// Rough estimate of chars changed
						if ('slice' in step && step.slice) {
							const slice = step.slice as { content?: { size?: number } };
							return acc + (slice.content?.size ?? 0);
						}
						return acc;
					}, 0);
					appState.markTyping(charsDelta);
					const version = appState.markUnsaved();
					// Capture scene ID at edit time to prevent saving to wrong scene
					saveScene(currentSceneId, ed.getHTML(), version);
				}
				updateCanStates();
			},
			onSelectionUpdate: ({ editor }) => {
				// Mark the active paragraph for focus mode
				const container = editor.view.dom;
				container.querySelectorAll('.active-paragraph').forEach((el) => {
					el.classList.remove('active-paragraph');
				});
				const { from } = editor.state.selection;
				const resolved = editor.state.doc.resolve(from);
				for (let depth = resolved.depth; depth >= 1; depth--) {
					const node = resolved.node(depth);
					if (node.isBlock) {
						const domNode = editor.view.nodeDOM(resolved.before(depth));
						if (domNode instanceof HTMLElement) {
							domNode.classList.add('active-paragraph');
						}
						break;
					}
				}
			},
			onTransaction: () => {
				updateCanStates();
			},
		});
	}

	// Track current scene ID to detect when we switch scenes
	let currentSceneId = $state<string | null>(null);

	// Remember cursor positions per scene (persisted in sessionStorage)
	const CURSOR_STORAGE_KEY = 'cahnon-cursor-positions';
	const MAX_CURSOR_ENTRIES = 50;

	function getCursorPositions(): Record<string, number> {
		try {
			return JSON.parse(sessionStorage.getItem(CURSOR_STORAGE_KEY) || '{}');
		} catch {
			return {};
		}
	}

	function setCursorPosition(sceneId: string, pos: number) {
		const positions = getCursorPositions();
		positions[sceneId] = pos;
		// Enforce max entries (FIFO)
		const keys = Object.keys(positions);
		if (keys.length > MAX_CURSOR_ENTRIES) {
			for (const key of keys.slice(0, keys.length - MAX_CURSOR_ENTRIES)) {
				delete positions[key];
			}
		}
		try {
			sessionStorage.setItem(CURSOR_STORAGE_KEY, JSON.stringify(positions));
		} catch {
			// Storage full or unavailable — ignore
		}
	}

	function getSavedCursorPosition(sceneId: string): number | undefined {
		return getCursorPositions()[sceneId];
	}

	async function updateTitle() {
		if (appState.selectedScene && titleInput) {
			await appState.updateScene(appState.selectedScene.id, { title: titleInput.value });
			// AD3: Visual confirmation when title is saved
			titleJustSaved = true;
			setTimeout(() => {
				titleJustSaved = false;
			}, 1500);
		}
		editingTitle = false;
	}

	async function updateStatus(status: string) {
		if (appState.selectedScene) {
			await appState.updateScene(appState.selectedScene.id, { status });
		}
	}

	async function updateSummary(event: Event) {
		const target = event.target as HTMLTextAreaElement;
		if (appState.selectedScene) {
			await appState.updateScene(appState.selectedScene.id, { summary: target.value });
		}
	}

	async function handleHistoryRestored() {
		// Reload the scene after restoring from history
		if (appState.selectedScene) {
			await appState.loadChapters();
		}
	}

	function insertFromCutLibrary(text: string) {
		if (editor) {
			editor.chain().focus().insertContent(text).run();
		}
		showCutLibrary = false;
	}

	async function cutSelectedText() {
		if (!editor || !appState.selectedScene) return;
		const { from, to, empty } = editor.state.selection;
		if (empty) return;

		const text = editor.state.doc.textBetween(from, to);
		if (text) {
			try {
				await cutApi.create(text, appState.selectedScene.id);
				editor.chain().focus().deleteSelection().run();
				showSuccess('Text moved to cut library');
			} catch (_e) {
				showError('Failed to save cut text');
			}
		}
	}

	function escapeRegex(str: string): string {
		return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	}

	function handleFind(data: { query: string; caseSensitive: boolean; wholeWord: boolean }) {
		if (!editor) return;

		const { query, caseSensitive, wholeWord } = data;

		// AC3: Store search parameters for re-execution on scene change
		lastSearchParams = query ? { query, caseSensitive, wholeWord } : null;

		// Clear previous highlights
		editor.commands.unsetHighlight();
		searchMarks = [];
		currentSearchIndex = 0;

		if (!query) {
			findReplaceHandle?.updateMatchInfo(0, 0);
			return;
		}

		// Find all matches
		const content = editor.state.doc.textContent;
		let regex: RegExp;
		try {
			const escaped = escapeRegex(query);
			const pattern = wholeWord ? `\\b${escaped}\\b` : escaped;
			regex = new RegExp(pattern, caseSensitive ? 'g' : 'gi');
		} catch {
			findReplaceHandle?.updateMatchInfo(0, 0);
			return;
		}

		let match;
		const matches: Array<{ from: number; to: number }> = [];
		while ((match = regex.exec(content)) !== null) {
			matches.push({ from: match.index, to: match.index + match[0].length });
		}

		searchMarks = matches;
		findReplaceHandle?.updateMatchInfo(matches.length > 0 ? 1 : 0, matches.length);
	}

	function handleFindNext() {
		if (searchMarks.length === 0 || !editor) return;
		currentSearchIndex = (currentSearchIndex + 1) % searchMarks.length;
		const mark = searchMarks[currentSearchIndex];
		// Scroll to and select the match
		editor.commands.setTextSelection({ from: mark.from + 1, to: mark.to + 1 });
		findReplaceHandle?.updateMatchInfo(currentSearchIndex + 1, searchMarks.length);
	}

	function handleFindPrev() {
		if (searchMarks.length === 0 || !editor) return;
		currentSearchIndex = (currentSearchIndex - 1 + searchMarks.length) % searchMarks.length;
		const mark = searchMarks[currentSearchIndex];
		editor.commands.setTextSelection({ from: mark.from + 1, to: mark.to + 1 });
		findReplaceHandle?.updateMatchInfo(currentSearchIndex + 1, searchMarks.length);
	}

	function handleReplace(data: {
		find: string;
		replace: string;
		caseSensitive: boolean;
		wholeWord: boolean;
	}) {
		if (!editor || searchMarks.length === 0) return;

		const { find, replace, caseSensitive, wholeWord } = data;

		// Replace current selection
		const { from, to } = editor.state.selection;
		editor.chain().focus().deleteRange({ from, to }).insertContent(replace).run();

		// Refresh search
		handleFind({ query: find, caseSensitive, wholeWord });
	}

	async function handleReplaceAll(data: {
		find: string;
		replace: string;
		caseSensitive: boolean;
		wholeWord: boolean;
		scope: FindReplaceScope;
	}) {
		if (!editor) return;

		const { find, replace, caseSensitive, wholeWord, scope } = data;

		if (scope === 'chapter' || scope === 'manuscript') {
			// Multi-scene replace via backend
			try {
				const chapterId =
					scope === 'chapter' ? (appState.selectedChapterId ?? undefined) : undefined;
				const count = await searchApi.findReplaceInScenes({
					find,
					replace,
					caseSensitive,
					wholeWord,
					chapterId,
				});
				if (count > 0) {
					// Reload scenes since backend changed them
					await appState.reloadScenes();
					// Refresh editor content if current scene was affected
					if (currentSceneId) {
						const scene = appState.selectedScene;
						if (scene) {
							isUpdating = true;
							editor.commands.setContent(scene.text);
							isUpdating = false;
						}
					}
					showSuccess(`Replaced in ${count} scene${count !== 1 ? 's' : ''}`);
				} else {
					showSuccess('No matches found');
				}
			} catch (e) {
				showError(e instanceof Error ? e.message : 'Replace failed');
			}
			// Clear search
			searchMarks = [];
			currentSearchIndex = 0;
			findReplaceHandle?.updateMatchInfo(0, 0);
			return;
		}

		// Scene-level replace (existing behavior)
		const content = editor.getHTML();
		let regex: RegExp;
		try {
			const escaped = escapeRegex(find);
			const pattern = wholeWord ? `\\b${escaped}\\b` : escaped;
			regex = new RegExp(pattern, caseSensitive ? 'g' : 'gi');
		} catch {
			return;
		}

		const newContent = content.replace(regex, replace);
		isUpdating = true;
		editor.commands.setContent(newContent);
		isUpdating = false;
		const version = appState.markUnsaved();
		if (currentSceneId) {
			saveScene(currentSceneId, newContent, version);
		}

		// Clear search
		searchMarks = [];
		currentSearchIndex = 0;
		findReplaceHandle?.updateMatchInfo(0, 0);
	}

	function handleFindClose() {
		showFindReplace = false;
		editor?.commands.unsetHighlight();
		searchMarks = [];
		// AC3: Clear stored search params on close
		lastSearchParams = null;
	}

	// Autosave on window blur
	function handleWindowBlur() {
		if (editor && currentSceneId && appState.hasUnsavedChanges) {
			saveScene(currentSceneId, editor.getHTML());
		}
	}

	// -------------------------------------------------------------------------
	// Annotation rendering in editor
	// -------------------------------------------------------------------------

	/** Convert a plain-text offset to a ProseMirror document position. */
	function textOffsetToDocPos(doc: import('@tiptap/pm/model').Node, offset: number): number {
		let textSeen = 0;
		let result = 0;
		doc.descendants((node, pos) => {
			if (result > 0) return false;
			if (node.isText && node.text) {
				if (textSeen + node.text.length >= offset) {
					result = pos + (offset - textSeen);
					return false;
				}
				textSeen += node.text.length;
			}
			return true;
		});
		return result || 1;
	}

	async function applyAnnotationMarks() {
		if (!editor || !currentSceneId) return;
		let annotations: Annotation[];
		try {
			annotations = await annotationApi.getByScene(currentSceneId);
		} catch {
			return;
		}

		// Update the local cache for tooltip lookups
		cachedAnnotations = annotations;

		if (!editor || annotations.length === 0) return;

		// First, remove existing annotation marks
		const { tr } = editor.state;
		const markType = editor.schema.marks.annotationMark;
		if (!markType) return;

		tr.removeMark(0, editor.state.doc.content.size, markType);

		for (const ann of annotations) {
			if (ann.status === 'resolved') continue;
			const from = textOffsetToDocPos(editor.state.doc, ann.start_offset);
			const to = textOffsetToDocPos(editor.state.doc, ann.end_offset);
			if (from > 0 && to > from) {
				tr.addMark(
					from,
					to,
					markType.create({
						annotationId: ann.id,
						annotationType: ann.annotation_type,
					})
				);
			}
		}
		if (tr.steps.length > 0) {
			// Apply without triggering onUpdate (to avoid marking as unsaved)
			isUpdating = true;
			editor.view.dispatch(tr);
			isUpdating = false;
		}
	}

	// -------------------------------------------------------------------------
	// T2: Viewport clamping for popovers
	// -------------------------------------------------------------------------

	function clampToViewport(
		x: number,
		y: number,
		popWidth: number,
		popHeight: number
	): { x: number; y: number } {
		const vw = window.innerWidth;
		const vh = window.innerHeight;
		let clampedX = Math.min(x, vw - popWidth - 16);
		let clampedY = y;
		if (y + popHeight > vh - 16) {
			clampedY = y - popHeight - 16;
		}
		return { x: Math.max(8, clampedX), y: Math.max(8, clampedY) };
	}

	// -------------------------------------------------------------------------
	// Quick-add Bible entry (Ctrl+Shift+B)
	// -------------------------------------------------------------------------

	let showQuickAddBible = $state(false);
	let quickAddPosition = $state({ x: 0, y: 0 });
	let quickAddSelectedText = $state('');
	let quickAddSearchQuery = $state('');
	// AB4: Added 'created' mode to allow continuing
	let quickAddMode = $state<'choose' | 'create' | 'link' | 'created'>('choose');
	let quickAddCreatedName = $state('');
	let quickAddEntryType = $state('character');
	let quickAddFilteredEntries = $derived(
		quickAddSearchQuery
			? appState.bibleEntries.filter(
					(e) =>
						e.name.toLowerCase().includes(quickAddSearchQuery.toLowerCase()) ||
						(e.aliases && e.aliases.toLowerCase().includes(quickAddSearchQuery.toLowerCase()))
				)
			: appState.bibleEntries.slice(0, 10)
	);

	function openQuickAddBible() {
		if (!editor || !appState.selectedScene) return;
		const { from, to, empty } = editor.state.selection;
		if (empty) {
			showError('Select text first to create or link a bible entry');
			return;
		}
		const text = editor.state.doc.textBetween(from, to);
		if (!text.trim()) return;

		// Position popup near selection (T2: clamped to viewport)
		const coords = editor.view.coordsAtPos(from);
		quickAddSelectedText = text.trim();
		quickAddSearchQuery = text.trim();
		quickAddPosition = clampToViewport(coords.left, coords.bottom + 8, 300, 200);
		quickAddMode = 'choose';
		showQuickAddBible = true;
	}

	async function quickAddCreateEntry() {
		if (!appState.selectedSceneId || !quickAddSelectedText) return;
		try {
			const entry = await appState.createBibleEntry({
				entry_type: quickAddEntryType,
				name: quickAddSelectedText,
			});
			await associationApi.create(appState.selectedSceneId, entry.id);
			// AB4: Show success state with option to continue
			quickAddCreatedName = entry.name;
			quickAddMode = 'created';
		} catch (e) {
			console.error('Failed to create bible entry:', e);
			showError('Failed to create bible entry');
		}
	}

	/** AB4: Close Quick Add after creation */
	function quickAddClose() {
		showQuickAddBible = false;
		quickAddMode = 'choose';
		quickAddCreatedName = '';
	}

	/** AB4: Continue creating after success */
	function quickAddCreateAnother() {
		quickAddMode = 'create';
		quickAddCreatedName = '';
	}

	async function quickAddLinkEntry(entry: BibleEntry) {
		if (!appState.selectedSceneId) return;
		try {
			await associationApi.create(appState.selectedSceneId, entry.id);
			showSuccess(`Linked "${entry.name}" to scene`);
			showQuickAddBible = false;
		} catch (e) {
			console.error('Failed to link bible entry:', e);
			showError('Failed to link bible entry');
		}
	}

	// -------------------------------------------------------------------------
	// Annotation tooltip, click-to-focus, and context menu
	// -------------------------------------------------------------------------

	let cachedAnnotations = $state<Annotation[]>([]);
	let tooltipAnnotation = $state<{ annotation: Annotation; x: number; y: number } | null>(null);
	let tooltipTimer = $state<ReturnType<typeof setTimeout> | null>(null);
	const prefersReducedMotion =
		typeof window !== 'undefined'
			? window.matchMedia('(prefers-reduced-motion: reduce)').matches
			: false;
	const TOOLTIP_ENTER_DELAY = prefersReducedMotion ? 0 : 150;
	const TOOLTIP_EXIT_DELAY = prefersReducedMotion ? 0 : 100;

	// Context menu state
	let contextMenu = $state<{ x: number; y: number } | null>(null);

	// Annotation creation popover
	let annotationPopover = $state<{
		x: number;
		y: number;
		startOffset: number;
		endOffset: number;
	} | null>(null);
	let popoverType = $state('comment');
	let popoverContent = $state('');

	function getAnnotationById(id: string): Annotation | undefined {
		return cachedAnnotations.find((a) => a.id === id);
	}

	function handleEditorMouseOver(e: MouseEvent) {
		const target = e.target as HTMLElement;
		const mark = target.closest('mark.annotation-highlight') as HTMLElement | null;
		if (!mark) {
			clearTooltipTimer();
			return;
		}
		const annotationId = mark.getAttribute('data-annotation-id');
		if (!annotationId) return;

		// Don't restart timer if we're already showing this tooltip
		if (tooltipAnnotation && tooltipAnnotation.annotation.id === annotationId) return;

		clearTooltipTimer();
		tooltipTimer = setTimeout(() => {
			const annotation = getAnnotationById(annotationId);
			if (annotation) {
				const rect = mark.getBoundingClientRect();
				tooltipAnnotation = {
					annotation,
					x: rect.left + rect.width / 2,
					y: rect.top - 4,
				};
			}
		}, TOOLTIP_ENTER_DELAY);
	}

	function handleEditorMouseOut(e: MouseEvent) {
		const target = e.relatedTarget as HTMLElement | null;
		if (target?.closest('.annotation-tooltip')) return;
		if (target?.closest('mark.annotation-highlight')) return;
		clearTooltipTimer();
		// Small exit delay to prevent flickering
		tooltipTimer = setTimeout(() => {
			tooltipAnnotation = null;
		}, TOOLTIP_EXIT_DELAY);
	}

	function handleEditorFocus(e: FocusEvent) {
		const target = e.target as HTMLElement;
		const mark = target.closest('mark.annotation-highlight') as HTMLElement | null;
		if (!mark) {
			clearTooltipTimer();
			return;
		}
		const annotationId = mark.getAttribute('data-annotation-id');
		if (!annotationId) return;

		// Don't restart timer if we're already showing this tooltip
		if (tooltipAnnotation && tooltipAnnotation.annotation.id === annotationId) return;

		clearTooltipTimer();
		tooltipTimer = setTimeout(() => {
			const annotation = getAnnotationById(annotationId);
			if (annotation) {
				const rect = mark.getBoundingClientRect();
				tooltipAnnotation = {
					annotation,
					x: rect.left + rect.width / 2,
					y: rect.top - 4,
				};
			}
		}, TOOLTIP_ENTER_DELAY);
	}

	function handleEditorBlur(e: FocusEvent) {
		const target = e.relatedTarget as HTMLElement | null;
		if (target?.closest('.annotation-tooltip')) return;
		if (target?.closest('mark.annotation-highlight')) return;
		clearTooltipTimer();
		tooltipTimer = setTimeout(() => {
			tooltipAnnotation = null;
		}, TOOLTIP_EXIT_DELAY);
	}

	function clearTooltipTimer() {
		if (tooltipTimer) {
			clearTimeout(tooltipTimer);
			tooltipTimer = null;
		}
	}

	function handleEditorClick(e: MouseEvent) {
		// Close context menu and tooltip on any click
		contextMenu = null;
		tooltipAnnotation = null;
		clearTooltipTimer();

		const target = e.target as HTMLElement;
		const mark = target.closest('mark.annotation-highlight') as HTMLElement | null;
		if (!mark) return;

		const annotationId = mark.getAttribute('data-annotation-id');
		if (!annotationId) return;

		// Set focused annotation in store for panel to scroll to
		appState.focusedAnnotationId = annotationId;

		// Open context panel if closed
		if (!appState.showContextPanel) {
			appState.showContextPanel = true;
		}
	}

	function handleEditorContextMenu(e: MouseEvent) {
		if (!editor) return;
		const { empty } = editor.state.selection;
		if (empty) return;

		e.preventDefault();
		contextMenu = { x: e.clientX, y: e.clientY };
	}

	function triggerAnnotationCreation() {
		if (!editor) return;
		const { from, to, empty } = editor.state.selection;
		if (empty) return;

		const startOffset = editor.state.doc.textBetween(0, from).length;
		const endOffset = startOffset + editor.state.doc.textBetween(from, to).length;

		// Apply a temporary "pending" highlight so the user sees the selected range
		const markType = editor.schema.marks.annotationMark;
		if (markType) {
			const { tr } = editor.state;
			tr.addMark(
				from,
				to,
				markType.create({ annotationId: '__pending__', annotationType: 'pending' })
			);
			isUpdating = true;
			editor.view.dispatch(tr);
			isUpdating = false;
		}

		// Position the popover below the end of the selection (T2: clamped to viewport)
		const coords = editor.view.coordsAtPos(to);
		const clamped = clampToViewport(coords.left, coords.bottom + 8, 300, 200);
		annotationPopover = {
			x: clamped.x,
			y: clamped.y,
			startOffset,
			endOffset,
		};
		popoverType = 'comment';
		popoverContent = '';
	}

	async function submitAnnotationPopover() {
		if (!annotationPopover || !popoverContent.trim() || !currentSceneId) return;
		try {
			await annotationApi.create({
				scene_id: currentSceneId,
				start_offset: annotationPopover.startOffset,
				end_offset: annotationPopover.endOffset,
				annotation_type: popoverType,
				content: popoverContent.trim(),
			});
			appState.annotationVersion++;
			closeAnnotationPopover();
		} catch (e) {
			console.error('Failed to create annotation:', e);
			showError('Failed to create annotation');
		}
	}

	function closeAnnotationPopover() {
		annotationPopover = null;
		popoverContent = '';
		popoverType = 'comment';
		// Clear pending mark
		appState.annotationVersion++;
	}

	// AC1: Tooltip action functions for uniform annotation interactions
	function tooltipEditAnnotation(annotationId: string) {
		// Focus the annotation in the panel where inline editing is available
		appState.focusedAnnotationId = annotationId;
		if (!appState.showContextPanel) {
			appState.showContextPanel = true;
		}
		appState.contextPanelTab = 'analysis';
		tooltipAnnotation = null;
	}

	async function tooltipDeleteAnnotation(annotationId: string) {
		try {
			await annotationApi.delete(annotationId);
			appState.annotationVersion++;
			tooltipAnnotation = null;
		} catch (e) {
			console.error('Failed to delete annotation:', e);
			showError('Failed to delete annotation');
		}
	}

	async function tooltipResolveAnnotation(annotationId: string, currentStatus: string) {
		const newStatus = currentStatus === 'resolved' ? 'open' : 'resolved';
		try {
			await annotationApi.update(annotationId, { status: newStatus });
			appState.annotationVersion++;
			tooltipAnnotation = null;
		} catch (e) {
			console.error('Failed to update annotation:', e);
			showError('Failed to update status');
		}
	}

	function contextMenuAddAnnotation() {
		triggerAnnotationCreation();
		contextMenu = null;
	}

	function contextMenuCutToLibrary() {
		cutSelectedText();
		contextMenu = null;
	}

	function contextMenuLinkBible() {
		openQuickAddBible();
		contextMenu = null;
	}

	// UB3: Quick add selection to Bible/Codex
	function contextMenuAddToBible() {
		if (!editor) {
			contextMenu = null;
			return;
		}
		const { from, to, empty } = editor.state.selection;
		if (empty) {
			contextMenu = null;
			return;
		}

		const text = editor.state.doc.textBetween(from, to).trim();
		if (!text) {
			contextMenu = null;
			return;
		}

		// Position popup near selection
		const coords = editor.view.coordsAtPos(from);
		quickAddSelectedText = text;
		quickAddSearchQuery = text;
		quickAddPosition = clampToViewport(coords.left, coords.bottom + 8, 300, 200);
		quickAddMode = 'create'; // Go directly to create mode
		showQuickAddBible = true;
		contextMenu = null;
	}

	// CD4: Create issue from selected text in editor
	async function contextMenuCreateIssue() {
		const selectedText = editor?.state.selection.empty
			? ''
			: editor?.state.doc.textBetween(editor.state.selection.from, editor.state.selection.to, ' ');

		contextMenu = null;

		if (!selectedScene) return;

		const title = selectedText
			? `Issue: "${selectedText.slice(0, 50)}${selectedText.length > 50 ? '...' : ''}"`
			: 'New issue from scene';

		try {
			const issue = await issueApi.create({
				issue_type: 'continuity_error',
				title,
				description: selectedText ? `Selected text: "${selectedText}"` : '',
				severity: 'warning',
			});

			// Link the issue to the current scene
			await issueApi.linkScene(issue.id, selectedScene.id);

			showSuccess('Issue created and linked to scene');

			// Navigate to issues view
			appState.setViewMode('issues');
		} catch (e) {
			console.error('Failed to create issue:', e);
			showError('Failed to create issue');
		}
	}

	// Focus mode settings - use from store
	let typewriterMode = $derived(appState.focusSettings.typewriterMode);
	let dimSurroundings = $derived(appState.focusSettings.dimSurroundings);

	function toggleTypewriterMode() {
		appState.toggleTypewriterMode();
	}

	function toggleDimSurroundings() {
		appState.toggleDimSurroundings();
	}

	// Typewriter scroll - keep cursor centered
	function handleEditorScroll() {
		if (!typewriterMode || !editor || !editorElement) return;

		const selection = window.getSelection();
		if (!selection || selection.rangeCount === 0) return;

		const range = selection.getRangeAt(0);
		const rect = range.getBoundingClientRect();
		const editorRect = editorElement.getBoundingClientRect();
		const editorCenter = editorRect.top + editorRect.height / 2;

		if (Math.abs(rect.top - editorCenter) > 50) {
			const scrollTop = editorElement.scrollTop + (rect.top - editorCenter);
			editorElement.scrollTo({ top: scrollTop, behavior: 'instant' });
		}
	}

	// Re-apply annotation marks when annotationVersion changes
	$effect(() => {
		// Track dependency on annotationVersion
		void appState.annotationVersion;
		// Only re-apply if we have an editor and scene loaded
		if (editor && currentSceneId) {
			applyAnnotationMarks();
		}
	});

	// Cleanup on destroy
	onDestroy(() => {
		clearTooltipTimer();
		if (editor) {
			editor.destroy();
		}
	});

	// Effect that watches both editorElement and scene changes
	$effect(() => {
		// Track these dependencies
		const element = editorElement;
		const sceneId = appState.selectedScene?.id;
		const sceneText = appState.selectedScene?.text;

		// Can't do anything without an element or scene
		if (!element || !sceneId) return;

		// Use untrack to read mutable state without creating circular dependencies
		const prevSceneId = untrack(() => currentSceneId);
		const currentEditor = untrack(() => editor);

		if (currentEditor && prevSceneId === sceneId) {
			// Editor exists and scene hasn't changed - nothing to do
			return;
		}

		if (currentEditor && prevSceneId !== sceneId) {
			// Save cursor position for the scene we're leaving
			if (prevSceneId) {
				try {
					setCursorPosition(prevSceneId, currentEditor.state.selection.from);
				} catch {
					// Ignore if selection state is invalid
				}
			}

			// UB1: Trigger transition animation
			isTransitioning = true;

			// Scene changed - update editor content
			currentSceneId = sceneId;
			isUpdating = true;
			currentEditor.commands.setContent(sceneText || '');
			isUpdating = false;

			// UB1: End transition after content update
			requestAnimationFrame(() => {
				isTransitioning = false;
			});

			// T1: Initialize live word count for new scene
			liveWordCount = countWords(sceneText || '');

			// UB2: Reset session baseline for new scene
			appState.resetSessionSceneBaseline(liveWordCount);

			// Restore cursor position for the scene we're entering
			const savedPos = getSavedCursorPosition(sceneId);
			if (savedPos !== undefined) {
				try {
					const docSize = currentEditor.state.doc.content.size;
					const pos = Math.min(savedPos, docSize);
					currentEditor.commands.setTextSelection(pos);
				} catch {
					// Position invalid, ignore
				}
			}

			// Check for crash recovery draft
			checkRecoveryDraft(sceneId, currentEditor);

			// Apply annotation marks (fire-and-forget)
			applyAnnotationMarks();

			// T5: Auto-focus editor on scene open
			tick().then(() => {
				if (currentEditor && !editingTitle) {
					currentEditor.commands.focus('end');
				}
				// AC3: Re-execute search when Find/Replace is open and we change scenes
				if (showFindReplace && lastSearchParams) {
					handleFind(lastSearchParams);
				}
			});
		} else {
			// No editor yet - initialize it
			currentSceneId = sceneId;
			// T1: Initialize live word count
			liveWordCount = countWords(sceneText || '');
			initEditor();

			// Check for crash recovery draft on initial load
			if (editor) {
				checkRecoveryDraft(sceneId, editor);
			}

			// Apply annotation marks after init (fire-and-forget)
			applyAnnotationMarks();
		}
	});
</script>

<svelte:window onkeydown={handleKeydown} onblur={handleWindowBlur} />

<div class="editor-container" class:revision-mode={appState.workMode === 'revision'}>
	{#if selectedScene}
		<div class="editor-header">
			<!-- UC4: Enhanced Breadcrumb navigation in editor -->
			<div class="scene-info">
				<button
					class="breadcrumb-link"
					onclick={() => appState.setViewMode('dashboard')}
					title="Go to Dashboard"
				>
					{appState.project?.title || 'Project'}
				</button>
				<span class="separator">/</span>
				{#if selectedChapter}
					<button
						class="chapter-name breadcrumb-link"
						onclick={() => appState.setViewMode('corkboard')}
						title="View chapter in Corkboard"
					>
						{selectedChapter.title}
					</button>
					<span class="separator">/</span>
				{/if}

				{#if editingTitle}
					<input
						bind:this={titleInput}
						type="text"
						class="title-input"
						maxlength={200}
						value={selectedScene.title}
						onblur={updateTitle}
						onkeydown={(e) => e.key === 'Enter' && updateTitle()}
					/>
				{:else}
					<!-- AB1: Title with pencil icon on hover, AD3: just-saved flash -->
					<span
						class="scene-title"
						class:just-saved={titleJustSaved}
						role="button"
						tabindex="0"
						ondblclick={() => {
							editingTitle = true;
						}}
						onkeydown={(e) => {
							if (e.key === 'F2') {
								e.preventDefault();
								editingTitle = true;
							}
						}}
						title="Double-click or press F2 to rename"
					>
						{selectedScene.title}
						<svg
							class="edit-pencil"
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
							<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
						</svg>
					</span>
				{/if}

				<!-- UB4: Scene position indicator -->
				{#if scenePosition}
					<span
						class="scene-position"
						title="Scene {scenePosition.current} of {scenePosition.total} in this chapter"
					>
						{scenePosition.current}/{scenePosition.total}
					</span>
				{/if}

				<!-- Z1: Inline save indicator -->
				{#if appState.hasUnsavedChanges}
					<span class="inline-save-indicator" title="Unsaved changes">
						<span class="pulse-dot"></span>
					</span>
				{/if}

				<div class="undo-redo-buttons">
					<button class="header-btn" onclick={undo} disabled={!canUndo} title="Undo (Cmd+Z)">
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M3 7v6h6" />
							<path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13" />
						</svg>
					</button>
					<button class="header-btn" onclick={redo} disabled={!canRedo} title="Redo (Cmd+Shift+Z)">
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M21 7v6h-6" />
							<path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3L21 13" />
						</svg>
					</button>
				</div>

				{#if appState.workMode === 'writing'}
					<div class="focus-controls">
						<button
							class="header-btn"
							class:active={typewriterMode}
							onclick={toggleTypewriterMode}
							title="Typewriter mode (keep cursor centered)"
						>
							<svg
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<rect x="2" y="4" width="20" height="16" rx="2" />
								<line x1="6" y1="8" x2="18" y2="8" />
								<line x1="6" y1="12" x2="18" y2="12" />
								<line x1="6" y1="16" x2="12" y2="16" />
							</svg>
						</button>
						<button
							class="header-btn"
							class:active={dimSurroundings}
							onclick={toggleDimSurroundings}
							title="Focus mode (dim surrounding paragraphs)"
						>
							<svg
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<circle cx="12" cy="12" r="3" />
								<path
									d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"
								/>
							</svg>
						</button>
					</div>
				{/if}
			</div>

			<div class="scene-meta">
				<select
					class="status-select"
					value={selectedScene.status}
					onchange={(e) => updateStatus(e.currentTarget.value)}
					style="border-color: {statusColors[selectedScene.status] || 'var(--color-border)'}"
				>
					{#each sceneStatuses as status (status.value)}
						<option value={status.value}>{status.label}</option>
					{/each}
				</select>

				<div class="word-count">
					{liveWordCount.toLocaleString()} words
				</div>
				<!-- CA5: Session stats -->
				{#if sessionWordsWritten > 0}
					<div class="session-stats" title="Session: {sessionDuration}min, {sessionWPM} WPM">
						<span class="session-words">+{sessionWordsWritten}</span>
						{#if sessionWPM > 0}
							<span class="session-wpm">{sessionWPM} wpm</span>
						{/if}
					</div>
				{/if}

				<!-- AJ2: Find/Replace active indicator -->
				{#if showFindReplace}
					<div class="find-replace-indicator" title="Find/Replace is open">
						<svg
							width="12"
							height="12"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<circle cx="11" cy="11" r="8" />
							<line x1="21" y1="21" x2="16.65" y2="16.65" />
						</svg>
						<span>Find active</span>
					</div>
				{/if}

				{#if appState.workMode === 'revision'}
					<button
						class="header-btn"
						onclick={() => (showHistoryModal = true)}
						title="View scene history"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<circle cx="12" cy="12" r="10" />
							<polyline points="12 6 12 12 16 14" />
						</svg>
					</button>

					<button class="header-btn" onclick={() => (showCutLibrary = true)} title="Cut library">
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<circle cx="6" cy="6" r="3" />
							<circle cx="6" cy="18" r="3" />
							<line x1="20" y1="4" x2="8.12" y2="15.88" />
							<line x1="14.47" y1="14.48" x2="20" y2="20" />
							<line x1="8.12" y1="8.12" x2="12" y2="12" />
						</svg>
					</button>

					<button class="header-btn" onclick={cutSelectedText} title="Cut selection to library">
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
							<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
						</svg>
					</button>

					<span class="divider"></span>

					<button
						class="header-btn"
						onclick={splitSceneAtCursor}
						title="Split scene at cursor position"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<line x1="12" y1="2" x2="12" y2="22" />
							<path d="M8 6l-4 6 4 6" />
							<path d="M16 6l4 6-4 6" />
						</svg>
					</button>

					<button class="header-btn" onclick={mergeWithNextScene} title="Merge with next scene">
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M8 6l4 6-4 6" />
							<path d="M16 6l-4 6 4 6" />
						</svg>
					</button>
				{/if}
			</div>
		</div>

		<!-- AD1: Summary accessible in both modes -->
		{#if appState.workMode === 'revision'}
			<div class="summary-section">
				<label for="summary">Summary</label>
				<textarea
					id="summary"
					placeholder="Brief summary of this scene..."
					value={selectedScene.summary || ''}
					onblur={updateSummary}
					rows="2"
				></textarea>
			</div>
		{:else}
			<!-- Writing mode: collapsible summary -->
			<div class="summary-section collapsed" class:expanded={summaryExpanded}>
				<button
					class="summary-toggle"
					onclick={() => (summaryExpanded = !summaryExpanded)}
					title={summaryExpanded ? 'Collapse summary' : 'Expand summary'}
				>
					<svg
						width="12"
						height="12"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						class:rotated={summaryExpanded}
					>
						<polyline points="6 9 12 15 18 9" />
					</svg>
					<span>Summary</span>
					{#if selectedScene.summary && !summaryExpanded}
						<span class="summary-preview"
							>{selectedScene.summary.slice(0, 40)}{selectedScene.summary.length > 40
								? '...'
								: ''}</span
						>
					{/if}
				</button>
				{#if summaryExpanded}
					<textarea
						id="summary-writing"
						placeholder="Brief summary of this scene..."
						value={selectedScene.summary || ''}
						onblur={updateSummary}
						rows="2"
					></textarea>
				{/if}
			</div>
		{/if}

		{#if recoveryDraftAvailable}
			{@const recoveryTime = new Date(recoveryDraftAvailable.timestamp).toLocaleTimeString([], {
				hour: 'numeric',
				minute: '2-digit',
			})}
			<div class="recovery-bar">
				<!-- Z2: Enriched recovery context -->
				<span class="recovery-info">
					Recovered draft from {recoveryTime} ({recoveryDraftAvailable.wordCount} words)
					{#if recoveryDraftAvailable.preview}
						<span class="recovery-preview">— "{recoveryDraftAvailable.preview}"</span>
					{/if}
				</span>
				<button class="recovery-btn restore" onclick={restoreRecoveryDraft}>Restore</button>
				<button class="recovery-btn discard" onclick={discardRecoveryDraft}>Discard</button>
			</div>
		{/if}

		{#if !appState.isFocusMode}
			<EditorToolbar {editor} />
		{/if}

		<!-- UB2: Writing Session Progress Bar -->
		{#if appState.writingSessionActive}
			<div class="session-progress-container">
				<div class="session-progress-bar">
					<div
						class="session-progress-fill"
						class:complete={appState.sessionProgress >= 100}
						style="width: {appState.sessionProgress}%"
					></div>
				</div>
				<div class="session-progress-info">
					<span class="session-progress-count">
						{appState.writingSessionWordsWritten} / {appState.writingSessionGoal} words
					</span>
					<button class="session-end-btn" onclick={() => appState.endWritingSession()}>
						End Session
					</button>
				</div>
			</div>
		{/if}

		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="editor-content"
			class:focus-mode={dimSurroundings}
			class:typewriter-mode={typewriterMode}
			class:transitioning={isTransitioning}
			data-editor-theme={appState.editorSettings.theme !== 'default'
				? appState.editorSettings.theme
				: undefined}
			bind:this={editorElement}
			onkeyup={handleEditorScroll}
			onclick={(e) => {
				handleEditorScroll();
				handleEditorClick(e);
			}}
			onmouseover={handleEditorMouseOver}
			onmouseout={handleEditorMouseOut}
			onfocus={handleEditorFocus}
			onblur={handleEditorBlur}
			oncontextmenu={handleEditorContextMenu}
			style="--editor-font-family: {appState.editorSettings
				.fontFamily}; --editor-font-size: {appState.editorSettings
				.fontSize}px; --editor-line-height: {appState.editorSettings
				.lineHeight}; --editor-text-width: {appState.editorSettings.textWidth}px;"
		></div>
		<!-- CA2: Flash "Saved" indicator in editor content area -->
		{#if appState.justSaved}
			<div class="content-saved-flash" aria-live="polite">Saved</div>
		{/if}

		{#if tooltipAnnotation}
			{@const typeInfo = getAnnotationType(tooltipAnnotation.annotation.annotation_type)}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="annotation-tooltip"
				style="left: {tooltipAnnotation.x}px; top: {tooltipAnnotation.y}px; border-top: 3px solid {typeInfo.color};"
				onmouseout={(e) => {
					const related = e.relatedTarget as HTMLElement | null;
					if (
						!related?.closest('mark.annotation-highlight') &&
						!related?.closest('.annotation-tooltip')
					) {
						tooltipAnnotation = null;
					}
				}}
				onblur={(e) => {
					const related = e.relatedTarget as HTMLElement | null;
					if (
						!related?.closest('mark.annotation-highlight') &&
						!related?.closest('.annotation-tooltip')
					) {
						tooltipAnnotation = null;
					}
				}}
			>
				<div class="tooltip-header">
					<span class="tooltip-type-icon">{typeInfo.icon}</span>
					<span class="tooltip-type-label" style="color: {typeInfo.color}">{typeInfo.label}</span>
					<span
						class="tooltip-status-badge"
						style="color: {getAnnotationStatusColor(tooltipAnnotation.annotation.status)}"
					>
						{tooltipAnnotation.annotation.status.replace('_', ' ')}
					</span>
				</div>
				<p class="tooltip-content">{truncate(tooltipAnnotation.annotation.content, 80)}</p>
				<!-- AC1: Uniform annotation actions -->
				<div class="tooltip-actions">
					<button
						class="tooltip-action-btn"
						onclick={() => tooltipEditAnnotation(tooltipAnnotation!.annotation.id)}
						title="Edit annotation"
					>
						Edit
					</button>
					<button
						class="tooltip-action-btn"
						onclick={() =>
							tooltipResolveAnnotation(
								tooltipAnnotation!.annotation.id,
								tooltipAnnotation!.annotation.status
							)}
						title={tooltipAnnotation.annotation.status === 'resolved' ? 'Reopen' : 'Resolve'}
					>
						{tooltipAnnotation.annotation.status === 'resolved' ? 'Reopen' : 'Resolve'}
					</button>
					<button
						class="tooltip-action-btn tooltip-action-delete"
						onclick={() => tooltipDeleteAnnotation(tooltipAnnotation!.annotation.id)}
						title="Delete annotation"
					>
						Delete
					</button>
				</div>
			</div>
		{/if}

		{#if contextMenu}
			<div
				class="context-menu-overlay"
				onclick={() => (contextMenu = null)}
				onkeydown={(e) => {
					if (e.key === 'Escape') contextMenu = null;
				}}
				role="presentation"
				tabindex="-1"
			>
				<div
					class="context-menu"
					style="left: {contextMenu.x}px; top: {contextMenu.y}px;"
					onclick={(e) => e.stopPropagation()}
					onkeydown={(e) => e.stopPropagation()}
					role="menu"
					tabindex="-1"
				>
					<!-- AB3: Show shortcut for annotation -->
					<button class="context-menu-item" onclick={contextMenuAddAnnotation} role="menuitem">
						<span>Add Annotation</span>
						<span class="context-menu-shortcut">{formatShortcut('M', true, true)}</span>
					</button>
					<button class="context-menu-item" onclick={contextMenuCutToLibrary} role="menuitem">
						Cut to Library
					</button>
					<button class="context-menu-item" onclick={contextMenuLinkBible} role="menuitem">
						Link Bible Entry
					</button>
					<!-- UB3: Add to Codex from selection -->
					<button class="context-menu-item" onclick={contextMenuAddToBible} role="menuitem">
						Add to Codex
					</button>
					<!-- CD4: Create issue from editor -->
					<button class="context-menu-item" onclick={contextMenuCreateIssue} role="menuitem">
						Create Issue
					</button>
				</div>
			</div>
		{/if}

		{#if annotationPopover}
			<div
				class="annotation-popover-overlay"
				onclick={closeAnnotationPopover}
				onkeydown={(e) => {
					if (e.key === 'Escape') closeAnnotationPopover();
				}}
				role="presentation"
				tabindex="-1"
			>
				<div
					class="annotation-popover"
					style="left: {annotationPopover.x}px; top: {annotationPopover.y}px;"
					onclick={(e) => e.stopPropagation()}
					onkeydown={(e) => {
						e.stopPropagation();
						if (e.key === 'Escape') closeAnnotationPopover();
						if (e.key === 'Enter' && isModKey(e)) {
							e.preventDefault();
							submitAnnotationPopover();
						}
					}}
					role="dialog"
					tabindex="-1"
				>
					<div class="popover-header">Add Annotation</div>
					<select bind:value={popoverType} class="popover-type-select">
						{#each annotationTypes as type (type.value)}
							<option value={type.value}>{type.icon} {type.label}</option>
						{/each}
					</select>
					<!-- svelte-ignore a11y_autofocus -->
					<textarea
						bind:value={popoverContent}
						placeholder="Add your note..."
						rows="3"
						class="popover-textarea"
						autofocus
					></textarea>
					<div class="popover-actions">
						<button class="popover-btn" onclick={closeAnnotationPopover}>Cancel</button>
						<button
							class="popover-btn popover-btn-primary"
							onclick={submitAnnotationPopover}
							disabled={!popoverContent.trim()}
						>
							Add
						</button>
					</div>
				</div>
			</div>
		{/if}

		<FindReplace
			bind:handle={findReplaceHandle}
			bind:isOpen={showFindReplace}
			bind:showReplace
			bind:scope={findReplaceScope}
			onfind={handleFind}
			onnext={handleFindNext}
			onprev={handleFindPrev}
			onreplace={handleReplace}
			onreplaceAll={handleReplaceAll}
			onclose={handleFindClose}
		/>
		{#if showQuickAddBible}
			<div
				class="quick-add-overlay"
				onclick={() => (showQuickAddBible = false)}
				onkeydown={(e) => {
					if (e.key === 'Escape') showQuickAddBible = false;
				}}
				role="presentation"
				tabindex="-1"
			>
				<div
					class="quick-add-popup"
					style="left: {quickAddPosition.x}px; top: {quickAddPosition.y}px;"
					onclick={(e) => e.stopPropagation()}
					onkeydown={(e) => e.stopPropagation()}
					role="dialog"
					tabindex="-1"
				>
					{#if quickAddMode === 'choose'}
						<div class="quick-add-header">
							<strong>"{quickAddSelectedText}"</strong>
						</div>
						<div class="quick-add-actions">
							<button class="quick-add-btn" onclick={() => (quickAddMode = 'create')}>
								Create Bible entry
							</button>
							<button class="quick-add-btn" onclick={() => (quickAddMode = 'link')}>
								Link to existing
							</button>
						</div>
					{:else if quickAddMode === 'created'}
						<!-- AB4: Success state with option to continue -->
						<div class="quick-add-header quick-add-success">
							<svg
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<polyline points="20 6 9 17 4 12" />
							</svg>
							<span>Created "{quickAddCreatedName}"</span>
						</div>
						<div class="quick-add-actions">
							<button class="quick-add-btn" onclick={quickAddCreateAnother}>
								Create Another
							</button>
							<button class="quick-add-btn primary" onclick={quickAddClose}> Done </button>
						</div>
					{:else if quickAddMode === 'create'}
						<div class="quick-add-header">
							<span>Create: "{quickAddSelectedText}"</span>
						</div>
						<select class="quick-add-select" bind:value={quickAddEntryType}>
							<option value="character">Character</option>
							<option value="location">Location</option>
							<option value="object">Object</option>
							<option value="faction">Faction</option>
							<option value="concept">Concept</option>
							<option value="glossary">Glossary</option>
						</select>
						<div class="quick-add-actions">
							<button class="quick-add-btn secondary" onclick={() => (quickAddMode = 'choose')}>
								Back
							</button>
							<button class="quick-add-btn primary" onclick={quickAddCreateEntry}> Create </button>
						</div>
					{:else if quickAddMode === 'link'}
						<div class="quick-add-header">
							<span>Link to existing entry</span>
						</div>
						<input
							type="text"
							class="quick-add-search"
							placeholder="Search entries..."
							bind:value={quickAddSearchQuery}
						/>
						<div class="quick-add-results">
							{#each quickAddFilteredEntries.slice(0, 8) as entry (entry.id)}
								<button class="quick-add-result" onclick={() => quickAddLinkEntry(entry)}>
									<span class="entry-type-badge">{entry.entry_type.charAt(0).toUpperCase()}</span>
									{entry.name}
								</button>
							{:else}
								<span class="quick-add-empty">No entries found</span>
							{/each}
						</div>
						<button class="quick-add-btn secondary" onclick={() => (quickAddMode = 'choose')}>
							Back
						</button>
					{/if}
				</div>
			</div>
		{/if}
	{:else}
		<!-- AB5: Contextual empty states -->
		<div class="no-scene">
			{#if appState.chapters.length === 0}
				<NewProjectWelcome />
			{:else}
				{@const totalScenes = Array.from(appState.scenes.values()).flat().length}
				{#if totalScenes === 0}
					<EmptyState
						icon="file"
						title="No scenes yet"
						description="Add a scene to your first chapter to start writing."
						actionLabel="Create first scene"
						onaction={() => {
							if (appState.chapters[0]) {
								appState.createScene(appState.chapters[0].id, 'Scene 1');
							}
						}}
					/>
				{:else}
					<EmptyState
						icon="file"
						title="No scene selected"
						description="Select a scene from the outline to start writing."
					/>
				{/if}
			{/if}
		</div>
	{/if}

	<!-- AD2: Template hint only when scene is completely empty -->
	{#if selectedScene && countWords(selectedScene.text) === 0 && !editingTitle}
		<div class="template-hint">
			<span>Start writing or</span>
			<button class="template-link" onclick={() => (appState.requestOpenTemplatesManager = true)}>
				apply a template
			</button>
		</div>
	{/if}
</div>

{#if selectedScene}
	<SceneHistoryModal
		bind:isOpen={showHistoryModal}
		sceneId={selectedScene.id}
		currentText={selectedScene.text}
		onrestored={handleHistoryRestored}
	/>
{/if}

<CutLibrary bind:isOpen={showCutLibrary} onInsert={insertFromCutLibrary} />

<style>
	.editor-container {
		display: flex;
		flex-direction: column;
		height: 100%;
		background-color: var(--color-bg-primary);
	}

	.editor-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border-light);
		gap: var(--spacing-md);
	}

	.scene-info {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.chapter-name {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	/* BB5: Clickable breadcrumb link */
	.breadcrumb-link {
		cursor: pointer;
		transition: color var(--transition-fast);
	}

	.breadcrumb-link:hover {
		color: var(--color-accent);
		text-decoration: underline;
		text-underline-offset: 2px;
	}

	.separator {
		color: var(--color-text-muted);
	}

	.scene-title {
		font-size: var(--font-size-lg);
		font-weight: 600;
		color: var(--color-text-primary);
		padding: var(--spacing-xs);
		margin: calc(-1 * var(--spacing-xs));
		border-radius: var(--border-radius-sm);
		border-bottom: 1px dashed transparent;
		transition:
			background-color var(--transition-fast),
			border-color var(--transition-fast);
		cursor: text;
		display: inline-flex;
		align-items: center;
		gap: var(--spacing-xs);
	}

	/* AB1: Pencil icon on hover */
	.scene-title .edit-pencil {
		opacity: 0;
		color: var(--color-text-muted);
		transition: opacity var(--transition-fast);
	}

	.scene-title:hover {
		background-color: var(--color-bg-hover);
		border-bottom-color: var(--color-border);
	}

	.scene-title:hover .edit-pencil {
		opacity: 1;
	}

	/* AD3: Visual confirmation when title is saved */
	.scene-title.just-saved {
		background-color: var(--success-subtle);
		animation: title-saved-flash 1.5s ease-out;
	}

	@keyframes title-saved-flash {
		0% {
			background-color: var(--success-subtle);
		}
		100% {
			background-color: transparent;
		}
	}

	/* UB4: Scene position indicator */
	.scene-position {
		margin-left: var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		padding: 2px var(--spacing-xs);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
	}

	/* Z1: Inline save indicator */
	.inline-save-indicator {
		margin-left: var(--spacing-sm);
		display: inline-flex;
		align-items: center;
	}

	.pulse-dot {
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: var(--warning-default, oklch(75% 0.15 75));
		animation: pulse-subtle 2s ease-in-out infinite;
	}

	@keyframes pulse-subtle {
		0%,
		100% {
			opacity: 0.5;
		}
		50% {
			opacity: 1;
		}
	}

	.undo-redo-buttons {
		display: flex;
		gap: var(--spacing-xs);
		margin-left: var(--spacing-md);
	}

	.undo-redo-buttons .header-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	.title-input {
		font-size: var(--font-size-lg);
		font-weight: 600;
		padding: var(--spacing-xs);
		border: 1px solid var(--color-accent);
		border-radius: var(--border-radius-sm);
	}

	.scene-meta {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
	}

	.status-select {
		font-size: var(--font-size-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		border-width: 2px;
		background-color: var(--color-bg-secondary);
	}

	.word-count {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	/* CA5: Session stats display */
	.session-stats {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-sm);
		background-color: var(--color-accent-light);
		border-radius: var(--border-radius-sm);
	}

	.session-words {
		color: var(--color-accent);
		font-weight: 500;
	}

	.session-wpm {
		color: var(--color-text-muted);
	}

	/* AJ2: Find/Replace active indicator */
	.find-replace-indicator {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-xs);
		color: var(--color-accent);
		background-color: var(--color-accent-light);
		padding: 2px var(--spacing-sm);
		border-radius: var(--border-radius-sm);
	}

	.header-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.header-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.scene-meta .divider {
		width: 1px;
		height: 16px;
		background-color: var(--color-border);
		margin: 0 var(--spacing-xs);
	}

	.summary-section {
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border-light);
	}

	.summary-section label {
		display: block;
		font-size: var(--font-size-xs);
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-xs);
	}

	.summary-section textarea {
		width: 100%;
		resize: none;
		font-size: var(--font-size-sm);
	}

	/* AD1: Collapsible summary in writing mode */
	.summary-section.collapsed {
		padding: var(--spacing-xs) var(--spacing-lg);
	}

	.summary-toggle {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		background: none;
		border: none;
		padding: var(--spacing-xs) 0;
		width: 100%;
		text-align: left;
		cursor: pointer;
	}

	.summary-toggle:hover {
		color: var(--color-text-secondary);
	}

	.summary-toggle svg {
		transition: transform var(--transition-fast);
	}

	.summary-toggle svg.rotated {
		transform: rotate(180deg);
	}

	.summary-preview {
		color: var(--color-text-muted);
		font-style: italic;
		margin-left: var(--spacing-sm);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		flex: 1;
	}

	.summary-section.collapsed.expanded {
		padding: var(--spacing-sm) var(--spacing-lg);
	}

	.summary-section.collapsed textarea {
		margin-top: var(--spacing-sm);
		line-height: var(--line-height-normal);
	}

	/* UB2: Writing Session Progress Bar */
	.session-progress-container {
		padding: var(--spacing-sm) var(--spacing-lg);
		background-color: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border);
	}

	.session-progress-bar {
		height: 6px;
		background-color: var(--color-border);
		border-radius: 3px;
		overflow: hidden;
		margin-bottom: var(--spacing-xs);
	}

	.session-progress-fill {
		height: 100%;
		background-color: var(--color-accent);
		border-radius: 3px;
		transition: width 0.3s ease-out;
	}

	.session-progress-fill.complete {
		background-color: var(--color-success);
		animation: progress-pulse 1s ease-in-out;
	}

	@keyframes progress-pulse {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.7;
		}
	}

	.session-progress-info {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.session-progress-count {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.session-end-btn {
		padding: 2px var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--color-border);
		background: transparent;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.session-end-btn:hover {
		color: var(--color-text-primary);
		border-color: var(--color-text-muted);
	}

	.editor-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
		position: relative;
		transition: opacity 0.15s ease-out;
	}

	/* UB1: Scene transition animation */
	.editor-content.transitioning {
		opacity: 0.7;
	}

	/* CA4: Editor theme support */
	.editor-content[data-editor-theme] {
		background-color: var(--editor-bg);
		color: var(--editor-text);
	}

	.editor-content[data-editor-theme] :global(.prose-editor) {
		color: var(--editor-text);
	}

	/* CA2: Flash "Saved" indicator in editor content area */
	.content-saved-flash {
		position: absolute;
		top: var(--spacing-lg);
		right: var(--spacing-lg);
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-success);
		color: white;
		font-size: var(--font-size-xs);
		font-weight: 500;
		border-radius: var(--border-radius-sm);
		animation: saved-flash-appear 2s ease-out forwards;
		pointer-events: none;
		z-index: 10;
	}

	@keyframes saved-flash-appear {
		0% {
			opacity: 1;
			transform: translateY(0);
		}
		70% {
			opacity: 1;
			transform: translateY(0);
		}
		100% {
			opacity: 0;
			transform: translateY(-10px);
		}
	}

	.editor-content :global(.prose-editor) {
		max-width: var(--editor-text-width, 700px);
		margin: 0 auto;
		font-family: var(--editor-font-family, var(--font-family-editor));
		font-size: var(--editor-font-size, var(--font-size-md));
		line-height: var(--editor-line-height, var(--line-height-relaxed));
		outline: none;
		min-height: 100%;
	}

	.editor-content :global(.prose-editor p) {
		margin-bottom: 1em;
	}

	.editor-content :global(.prose-editor h1),
	.editor-content :global(.prose-editor h2),
	.editor-content :global(.prose-editor h3) {
		font-family: var(--font-family-ui);
		font-weight: 600;
		margin-top: 1.5em;
		margin-bottom: 0.5em;
	}

	.editor-content :global(.prose-editor h1) {
		font-size: var(--font-size-2xl);
	}

	.editor-content :global(.prose-editor h2) {
		font-size: var(--font-size-xl);
	}

	.editor-content :global(.prose-editor h3) {
		font-size: var(--font-size-lg);
	}

	.editor-content :global(.prose-editor blockquote) {
		border-left: 3px solid var(--color-border);
		padding-left: var(--spacing-md);
		color: var(--color-text-secondary);
		font-style: italic;
		margin: 1em 0;
	}

	.editor-content :global(.prose-editor code) {
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 0.9em;
		background-color: var(--color-bg-tertiary);
		padding: 0.1em 0.3em;
		border-radius: var(--border-radius-sm);
	}

	.editor-content :global(.prose-editor mark) {
		background-color: var(--color-bg-tertiary);
		padding: 0.1em 0;
	}

	.editor-content :global(.prose-editor .is-empty::before) {
		content: attr(data-placeholder);
		color: var(--color-text-muted);
		float: left;
		pointer-events: none;
		height: 0;
	}

	/* T3: More visible recovery bar */
	.recovery-bar {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-lg);
		background-color: var(--warning-subtle, oklch(90% 0.08 80));
		border-bottom: 2px solid var(--warning-border, oklch(75% 0.12 80));
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		font-weight: 500;
	}

	/* Z2: Recovery bar enriched info */
	.recovery-info {
		flex: 1;
	}

	.recovery-preview {
		color: var(--color-text-muted);
		font-style: italic;
		font-weight: 400;
	}

	.recovery-btn {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--color-border);
		background-color: var(--color-bg-primary);
	}

	.recovery-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.recovery-btn.restore {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	.recovery-btn.discard {
		color: var(--color-text-muted);
	}

	.template-hint {
		position: absolute;
		bottom: var(--spacing-lg);
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		opacity: 0.7;
	}

	.template-link {
		color: var(--color-accent);
		text-decoration: underline;
		text-underline-offset: 2px;
		font-size: var(--font-size-sm);
		cursor: pointer;
	}

	.template-link:hover {
		color: var(--color-accent-hover);
	}

	.no-scene {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
	}

	/* Writing mode - even more minimal */
	.editor-container:not(.revision-mode) .editor-header {
		border-bottom: none;
	}

	.editor-container:not(.revision-mode) .scene-meta {
		display: none;
	}

	/* Focus controls */
	.focus-controls {
		display: flex;
		gap: var(--spacing-xs);
		margin-left: var(--spacing-md);
	}

	.focus-controls .header-btn.active {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}

	/* Focus mode - dim non-current paragraphs */
	.editor-content.focus-mode :global(.prose-editor p),
	.editor-content.focus-mode :global(.prose-editor h1),
	.editor-content.focus-mode :global(.prose-editor h2),
	.editor-content.focus-mode :global(.prose-editor h3),
	.editor-content.focus-mode :global(.prose-editor blockquote) {
		opacity: 0.3;
		transition: opacity var(--transition-normal);
	}

	.editor-content.focus-mode :global(.prose-editor .active-paragraph),
	.editor-content.focus-mode :global(.prose-editor p:has(.ProseMirror-selectednode)) {
		opacity: 1;
		background-color: var(--color-bg-hover);
		border-radius: var(--border-radius-sm);
		margin-inline: calc(-1 * var(--spacing-sm));
		padding-inline: var(--spacing-sm);
	}

	/* Use hover as fallback for current paragraph detection */
	.editor-content.focus-mode:hover :global(.prose-editor p),
	.editor-content.focus-mode:hover :global(.prose-editor h1),
	.editor-content.focus-mode:hover :global(.prose-editor h2),
	.editor-content.focus-mode:hover :global(.prose-editor h3),
	.editor-content.focus-mode:hover :global(.prose-editor blockquote) {
		opacity: 0.3;
	}

	.editor-content.focus-mode :global(.prose-editor .is-editor-empty:first-child::before) {
		opacity: 1;
	}

	/* Typewriter mode - center content vertically */
	.editor-content.typewriter-mode {
		padding-top: 40vh;
		padding-bottom: 40vh;
	}

	/* Annotation highlight styles */
	.editor-content :global(.annotation-highlight) {
		padding: 0.1em 0;
		border-radius: 2px;
		cursor: pointer;
		border-bottom: 2px solid transparent;
		transition: filter var(--transition-fast);
	}

	.editor-content :global(.annotation-highlight:hover) {
		filter: brightness(1.15);
	}

	/* W1: Annotation colors adapted for light/dark mode */
	.editor-content :global(.annotation-comment) {
		background-color: var(--annotation-comment);
		border-bottom-color: oklch(65% 0.15 250);
	}

	.editor-content :global(.annotation-question) {
		background-color: var(--annotation-question);
		border-bottom-color: oklch(55% 0.15 145);
	}

	.editor-content :global(.annotation-todo) {
		background-color: var(--annotation-todo);
		border-bottom-color: oklch(65% 0.15 65);
	}

	.editor-content :global(.annotation-research) {
		background-color: var(--annotation-research);
		border-bottom-color: oklch(55% 0.15 300);
	}

	.editor-content :global(.annotation-revision) {
		background-color: var(--annotation-note);
		border-bottom-color: oklch(55% 0.12 145);
	}

	.editor-content :global(.annotation-pending) {
		background-color: rgba(59, 130, 246, 0.18);
		border-bottom: 2px dashed rgba(59, 130, 246, 0.7);
		animation: pending-pulse 1.5s ease-in-out infinite;
	}

	@keyframes pending-pulse {
		0%,
		100% {
			background-color: rgba(59, 130, 246, 0.18);
		}
		50% {
			background-color: rgba(59, 130, 246, 0.32);
		}
	}

	/* Quick-add Bible popup */
	.quick-add-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 500;
	}

	.quick-add-popup {
		position: fixed;
		min-width: 220px;
		max-width: 300px;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-sm);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		z-index: 501;
	}

	.quick-add-header {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		padding-bottom: var(--spacing-xs);
		border-bottom: 1px solid var(--color-border-light);
	}

	.quick-add-header strong {
		color: var(--color-text-primary);
	}

	.quick-add-actions {
		display: flex;
		gap: var(--spacing-xs);
	}

	.quick-add-btn {
		flex: 1;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border-radius: var(--border-radius-sm);
		border: 1px solid var(--color-border);
		background-color: var(--color-bg-secondary);
		color: var(--color-text-primary);
		transition: all var(--transition-fast);
	}

	.quick-add-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.quick-add-btn.primary {
		background-color: var(--color-accent);
		color: var(--text-on-accent, #fff);
		border-color: var(--color-accent);
	}

	.quick-add-btn.primary:hover {
		opacity: 0.9;
	}

	.quick-add-btn.secondary {
		background-color: transparent;
	}

	/* AB4: Quick Add success state */
	.quick-add-success {
		color: var(--color-success);
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
	}

	.quick-add-select {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
	}

	.quick-add-search {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.quick-add-results {
		max-height: 200px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.quick-add-result {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border-radius: var(--border-radius-sm);
		text-align: left;
		transition: background-color var(--transition-fast);
	}

	.quick-add-result:hover {
		background-color: var(--color-bg-hover);
	}

	.entry-type-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-tertiary);
		font-size: var(--font-size-xs);
		font-weight: 600;
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.quick-add-empty {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		padding: var(--spacing-sm);
		text-align: center;
	}

	/* Annotation tooltip */
	.annotation-tooltip {
		position: fixed;
		transform: translate(-50%, -100%);
		max-width: 280px;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-sm);
		z-index: 600;
		pointer-events: auto;
	}

	.tooltip-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		margin-bottom: var(--spacing-xs);
	}

	.tooltip-type-icon {
		font-size: var(--font-size-sm);
	}

	.tooltip-type-label {
		font-size: var(--font-size-xs);
		font-weight: 600;
		color: var(--color-text-secondary);
	}

	.tooltip-status-badge {
		margin-left: auto;
		font-size: var(--font-size-xs);
		font-weight: 500;
		text-transform: capitalize;
	}

	.tooltip-content {
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		line-height: var(--line-height-normal);
		margin: 0;
	}

	/* AC1: Tooltip action buttons */
	.tooltip-actions {
		display: flex;
		gap: var(--spacing-xs);
		margin-top: var(--spacing-sm);
		padding-top: var(--spacing-sm);
		border-top: 1px solid var(--color-border-light);
	}

	.tooltip-action-btn {
		flex: 1;
		padding: var(--spacing-xs);
		font-size: var(--font-size-xs);
		font-weight: 500;
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		transition: all var(--transition-fast);
	}

	.tooltip-action-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.tooltip-action-delete:hover {
		background-color: var(--color-error);
		color: var(--text-on-accent);
	}

	/* Context menu */
	.context-menu-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 500;
	}

	.context-menu {
		position: fixed;
		min-width: 180px;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-xs);
		z-index: 501;
	}

	.context-menu-item {
		display: block;
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		text-align: left;
		border-radius: var(--border-radius-sm);
		color: var(--color-text-primary);
		transition: background-color var(--transition-fast);
	}

	.context-menu-item:hover {
		background-color: var(--color-bg-hover);
	}

	/* AB3: Shortcut hint in context menu */
	.context-menu-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.context-menu-shortcut {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-left: var(--spacing-md);
	}

	/* Annotation creation popover */
	.annotation-popover-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 500;
	}

	.annotation-popover {
		position: fixed;
		width: 300px;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-md);
		z-index: 501;
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.popover-header {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
	}

	.popover-type-select {
		font-size: var(--font-size-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.popover-textarea {
		font-size: var(--font-size-sm);
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		resize: none;
		font-family: inherit;
	}

	.popover-textarea:focus {
		border-color: var(--color-accent);
		outline: none;
	}

	.popover-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-xs);
	}

	.popover-btn {
		font-size: var(--font-size-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
	}

	.popover-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.popover-btn-primary {
		background-color: var(--color-accent);
		color: white;
		font-weight: 500;
	}

	.popover-btn-primary:hover {
		filter: brightness(1.1);
	}

	.popover-btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
