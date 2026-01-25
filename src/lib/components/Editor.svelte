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
	import { onDestroy, untrack } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import Placeholder from '@tiptap/extension-placeholder';
	import CharacterCount from '@tiptap/extension-character-count';
	import Typography from '@tiptap/extension-typography';
	import Highlight from '@tiptap/extension-highlight';
	import { appState, saveRecoveryDraft, getRecoveryDraft, clearRecoveryDraft } from '$lib/stores';
	import { countWords, debounce, sceneStatuses, statusColors } from '$lib/utils';
	import SceneHistoryModal from './SceneHistoryModal.svelte';
	import CutLibrary from './CutLibrary.svelte';
	import FindReplace from './FindReplace.svelte';
	import { cutApi, sceneApi } from '$lib/api';
	import { isModKey } from '$lib/utils';
	import { showSuccess, showError } from '$lib/toast';

	// Derived values for proper reactivity tracking in templates
	let selectedScene = $derived(appState.selectedScene);
	let selectedChapter = $derived(appState.selectedChapter);

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

		if (
			!confirm(
				`Merge "${appState.selectedScene.title}" with "${nextScene.title}"? This cannot be undone.`
			)
		) {
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
	let isUpdating = $state(false);

	// Scene metadata editing
	let editingTitle = $state(false);
	let titleInput = $state<HTMLInputElement | null>(null);

	// History modal
	let showHistoryModal = $state(false);

	// Cut library
	let showCutLibrary = $state(false);

	// Find and Replace
	let showFindReplace = $state(false);
	let showReplace = $state(false);
	let findReplaceHandle = $state<
		{ updateMatchInfo: (current: number, total: number) => void } | undefined
	>(undefined);
	let searchMarks = $state<Array<{ from: number; to: number }>>([]);
	let currentSearchIndex = $state(0);

	function handleKeydown(event: KeyboardEvent) {
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
		// Undo: Cmd/Ctrl + Z (handled by TipTap, but ensure focus)
		// Redo: Cmd/Ctrl + Shift + Z or Cmd/Ctrl + Y (handled by TipTap)
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

	const saveScene = debounce(async (text: string) => {
		if (appState.selectedScene && !isUpdating) {
			// Save recovery draft to localStorage (in case of crash)
			saveRecoveryDraft(appState.selectedScene.id, text);
			await appState.updateScene(appState.selectedScene.id, { text });
			// Clear recovery draft after successful save
			clearRecoveryDraft();
		}
	}, 1000);

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
			],
			content: appState.selectedScene?.text || '',
			editorProps: {
				attributes: {
					class: 'prose-editor',
				},
			},
			onUpdate: ({ editor }) => {
				if (!isUpdating) {
					appState.hasUnsavedChanges = true;
					saveScene(editor.getHTML());
				}
				updateCanStates();
			},
			onTransaction: () => {
				updateCanStates();
			},
		});
	}

	// Track current scene ID to detect when we switch scenes
	let currentSceneId = $state<string | null>(null);

	async function updateTitle() {
		if (appState.selectedScene && titleInput) {
			await appState.updateScene(appState.selectedScene.id, { title: titleInput.value });
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

	function handleFind(data: { query: string; caseSensitive: boolean; wholeWord: boolean }) {
		if (!editor) return;

		const { query, caseSensitive, wholeWord } = data;

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
			const pattern = wholeWord ? `\\b${query}\\b` : query;
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

	function handleReplaceAll(data: {
		find: string;
		replace: string;
		caseSensitive: boolean;
		wholeWord: boolean;
	}) {
		if (!editor) return;

		const { find, replace, caseSensitive, wholeWord } = data;

		const content = editor.getHTML();
		let regex: RegExp;
		try {
			const pattern = wholeWord ? `\\b${find}\\b` : find;
			regex = new RegExp(pattern, caseSensitive ? 'g' : 'gi');
		} catch {
			return;
		}

		const newContent = content.replace(regex, replace);
		isUpdating = true;
		editor.commands.setContent(newContent);
		isUpdating = false;
		appState.hasUnsavedChanges = true;
		saveScene(newContent);

		// Clear search
		searchMarks = [];
		currentSearchIndex = 0;
		findReplaceHandle?.updateMatchInfo(0, 0);
	}

	function handleFindClose() {
		showFindReplace = false;
		editor?.commands.unsetHighlight();
		searchMarks = [];
	}

	// Autosave on window blur
	function handleWindowBlur() {
		if (editor && appState.selectedScene && appState.hasUnsavedChanges) {
			saveScene(editor.getHTML());
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
			editorElement.scrollTo({ top: scrollTop, behavior: 'smooth' });
		}
	}

	// Cleanup on destroy
	onDestroy(() => {
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
			// Scene changed - update editor content
			currentSceneId = sceneId;
			isUpdating = true;
			currentEditor.commands.setContent(sceneText || '');
			isUpdating = false;

			// Check for crash recovery draft
			const recoveryDraft = getRecoveryDraft();
			if (recoveryDraft && recoveryDraft.sceneId === sceneId) {
				const shouldRecover = confirm(
					'A recovery draft was found from a previous session. Would you like to restore it?\n\n' +
						'Click OK to restore the draft, or Cancel to discard it.'
				);
				if (shouldRecover) {
					isUpdating = true;
					currentEditor.commands.setContent(recoveryDraft.text);
					isUpdating = false;
					appState.hasUnsavedChanges = true;
					showSuccess('Draft recovered');
				}
				clearRecoveryDraft();
			}
		} else {
			// No editor yet - initialize it
			currentSceneId = sceneId;
			initEditor();
		}
	});
</script>

<svelte:window onkeydown={handleKeydown} onblur={handleWindowBlur} />

<div class="editor-container" class:revision-mode={appState.workMode === 'revision'}>
	{#if selectedScene}
		<div class="editor-header">
			<div class="scene-info">
				{#if selectedChapter}
					<span class="chapter-name">{selectedChapter.title}</span>
					<span class="separator">/</span>
				{/if}

				{#if editingTitle}
					<input
						bind:this={titleInput}
						type="text"
						class="title-input"
						value={selectedScene.title}
						onblur={updateTitle}
						onkeydown={(e) => e.key === 'Enter' && updateTitle()}
					/>
				{:else}
					<button
						class="scene-title"
						onclick={() => {
							editingTitle = true;
						}}
					>
						{selectedScene.title}
					</button>
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

			{#if appState.workMode === 'revision'}
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
						{countWords(selectedScene.text)} words
					</div>

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
				</div>
			{/if}
		</div>

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
		{/if}

		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="editor-content"
			class:focus-mode={dimSurroundings}
			class:typewriter-mode={typewriterMode}
			bind:this={editorElement}
			onkeyup={handleEditorScroll}
			onclick={handleEditorScroll}
			style="--editor-font-family: {appState.editorSettings
				.fontFamily}; --editor-font-size: {appState.editorSettings
				.fontSize}px; --editor-line-height: {appState.editorSettings
				.lineHeight}; --editor-text-width: {appState.editorSettings.textWidth}px;"
		></div>

		<FindReplace
			bind:handle={findReplaceHandle}
			bind:isOpen={showFindReplace}
			bind:showReplace
			onfind={handleFind}
			onnext={handleFindNext}
			onprev={handleFindPrev}
			onreplace={handleReplace}
			onreplaceAll={handleReplaceAll}
			onclose={handleFindClose}
		/>
	{:else}
		<div class="no-scene">
			<div class="no-scene-content">
				<svg
					width="48"
					height="48"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
					<polyline points="14 2 14 8 20 8" />
					<line x1="16" y1="13" x2="8" y2="13" />
					<line x1="16" y1="17" x2="8" y2="17" />
					<polyline points="10 9 9 9 8 9" />
				</svg>
				<h3>No scene selected</h3>
				<p>Select a scene from the outline or create a new one to start writing.</p>
			</div>
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
		transition: background-color var(--transition-fast);
	}

	.scene-title:hover {
		background-color: var(--color-bg-hover);
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
		cursor: pointer;
	}

	.word-count {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
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
		line-height: var(--line-height-normal);
	}

	.editor-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
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

	.no-scene {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding: var(--spacing-xl);
	}

	.no-scene-content {
		text-align: center;
		color: var(--color-text-muted);
	}

	.no-scene-content svg {
		margin-bottom: var(--spacing-md);
		opacity: 0.5;
	}

	.no-scene-content h3 {
		font-size: var(--font-size-lg);
		font-weight: 500;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-sm);
	}

	.no-scene-content p {
		font-size: var(--font-size-sm);
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

	.editor-content.focus-mode :global(.prose-editor p:focus-within),
	.editor-content.focus-mode :global(.prose-editor p:has(.ProseMirror-selectednode)),
	.editor-content.focus-mode :global(.prose-editor h1:focus-within),
	.editor-content.focus-mode :global(.prose-editor h2:focus-within),
	.editor-content.focus-mode :global(.prose-editor h3:focus-within),
	.editor-content.focus-mode :global(.prose-editor blockquote:focus-within) {
		opacity: 1;
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
</style>
