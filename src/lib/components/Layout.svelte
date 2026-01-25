<!--
  Main application layout component.

  Handles:
  - View mode switching (editor, corkboard, timeline, bible)
  - Keyboard shortcuts (Cmd+K, Cmd+N, Cmd+S, etc.)
  - Panel visibility (outline, context panel)
  - Focus mode and fullscreen
  - Dialogs (export, import, settings, trash, snapshots)
-->
<script lang="ts">
	import Toolbar from './Toolbar.svelte';
	import Outline from './Outline.svelte';
	import Editor from './Editor.svelte';
	import ContextPanel from './ContextPanel.svelte';
	import StatusBar from './StatusBar.svelte';
	import QuickOpen from './QuickOpen.svelte';
	import Corkboard from './Corkboard.svelte';
	import BibleView from './BibleView.svelte';
	import TimelineView from './TimelineView.svelte';
	import ExportDialog from './ExportDialog.svelte';
	import TrashView from './TrashView.svelte';
	import SnapshotsView from './SnapshotsView.svelte';
	import ReviewGrid from './ReviewGrid.svelte';
	import ImportDialog from './ImportDialog.svelte';
	import SettingsDialog from './SettingsDialog.svelte';
	import ToastNotifications from './ToastNotifications.svelte';

	let showReviewGrid = $state(false);
	let showImportDialog = $state(false);
	let showSettingsDialog = $state(false);
	import { appState } from '$lib/stores';
	import type { Scene } from '$lib/api';
	import { isModKey } from '$lib/utils';

	function handleKeydown(event: KeyboardEvent) {
		// Quick Open: Cmd/Ctrl + K
		if (isModKey(event) && event.key === 'k') {
			event.preventDefault();
			appState.toggleQuickOpen();
			return;
		}

		// Toggle Outline: Cmd/Ctrl + \
		if (isModKey(event) && event.key === '\\') {
			event.preventDefault();
			if (event.shiftKey) {
				appState.toggleContextPanel();
			} else {
				appState.toggleOutline();
			}
			return;
		}

		// View switching: Cmd/Ctrl + 1-4
		if (isModKey(event) && ['1', '2', '3', '4'].includes(event.key)) {
			event.preventDefault();
			const modes = ['editor', 'corkboard', 'timeline', 'bible'] as const;
			appState.setViewMode(modes[parseInt(event.key) - 1]);
			return;
		}

		// Focus mode: Cmd/Ctrl + D
		if (isModKey(event) && event.key === 'd') {
			event.preventDefault();
			appState.toggleWorkMode();
			return;
		}

		// Navigate to next scene: Cmd/Ctrl + Down
		if (isModKey(event) && event.key === 'ArrowDown') {
			event.preventDefault();
			navigateScene('next');
			return;
		}

		// Navigate to previous scene: Cmd/Ctrl + Up
		if (isModKey(event) && event.key === 'ArrowUp') {
			event.preventDefault();
			navigateScene('prev');
			return;
		}

		// Save: Cmd/Ctrl + S (prevent default, autosave handles it)
		if (isModKey(event) && event.key === 's') {
			event.preventDefault();
			// Autosave is already handling saves
			return;
		}

		// Export: Cmd/Ctrl + E
		if (isModKey(event) && event.key === 'e') {
			event.preventDefault();
			appState.openExportDialog();
			return;
		}

		// Review Grid: Cmd/Ctrl + G
		if (isModKey(event) && event.key === 'g') {
			event.preventDefault();
			showReviewGrid = !showReviewGrid;
			return;
		}

		// Import: Cmd/Ctrl + I
		if (isModKey(event) && event.key === 'i') {
			event.preventDefault();
			showImportDialog = !showImportDialog;
			return;
		}

		// Fullscreen focus mode: F11 or Cmd/Ctrl+Shift+F
		if (event.key === 'F11' || (isModKey(event) && event.shiftKey && event.key === 'f')) {
			event.preventDefault();
			appState.toggleFullscreenMode();
			return;
		}

		// Escape exits focus mode
		if (event.key === 'Escape' && appState.isFocusMode) {
			event.preventDefault();
			appState.isFocusMode = false;
			appState.focusSettings = { ...appState.focusSettings, fullscreenMode: false };
			return;
		}
	}

	function openReviewGrid() {
		showReviewGrid = true;
	}

	function openImportDialog() {
		showImportDialog = true;
	}

	function openSettings() {
		showSettingsDialog = true;
	}

	function openSnapshots() {
		appState.openSnapshotsView();
	}

	function navigateScene(direction: 'next' | 'prev') {
		const allScenesArr: Array<{ chapterId: string; scene: Scene }> = [];

		// Build flat list of all scenes in order
		for (const chapter of appState.chapters) {
			const chapterScenes = appState.scenes.get(chapter.id) || [];
			for (const scene of chapterScenes) {
				allScenesArr.push({ chapterId: chapter.id, scene });
			}
		}

		if (allScenesArr.length === 0) return;

		const currentIndex = allScenesArr.findIndex((s) => s.scene.id === appState.selectedSceneId);

		let newIndex: number;
		if (direction === 'next') {
			newIndex = currentIndex === -1 ? 0 : Math.min(currentIndex + 1, allScenesArr.length - 1);
		} else {
			newIndex = currentIndex === -1 ? 0 : Math.max(currentIndex - 1, 0);
		}

		const target = allScenesArr[newIndex];
		if (target) {
			appState.selectedChapterId = target.chapterId;
			appState.selectedSceneId = target.scene.id;
		}
	}

	$effect(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
		};
	});
</script>

<div class="layout" class:focus-mode={appState.isFocusMode}>
	{#if !appState.isFocusMode}
		<Toolbar
			onOpenReviewGrid={openReviewGrid}
			onOpenImportDialog={openImportDialog}
			onOpenSettings={openSettings}
			onOpenSnapshots={openSnapshots}
		/>
	{/if}

	<div class="main">
		{#if appState.showOutline && !appState.isFocusMode}
			<aside class="sidebar">
				<Outline />
			</aside>
		{/if}

		<main class="content">
			{#if appState.viewMode === 'editor'}
				<Editor />
			{:else if appState.viewMode === 'corkboard'}
				<Corkboard />
			{:else if appState.viewMode === 'bible'}
				<BibleView />
			{:else if appState.viewMode === 'timeline'}
				<TimelineView />
			{/if}
		</main>

		{#if appState.showContextPanel && appState.viewMode === 'editor' && !appState.isFocusMode}
			<aside class="context-panel">
				<ContextPanel />
			</aside>
		{/if}
	</div>

	{#if !appState.isFocusMode}
		<StatusBar />
	{/if}

	{#if appState.isFocusMode}
		<button
			class="exit-focus-btn"
			onclick={() => {
				appState.isFocusMode = false;
				appState.focusSettings = { ...appState.focusSettings, fullscreenMode: false };
			}}
			title="Exit focus mode (Escape)"
		>
			<svg
				width="20"
				height="20"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					d="M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3"
				/>
			</svg>
		</button>
	{/if}

	{#if appState.isQuickOpenVisible}
		<QuickOpen />
	{/if}

	<ExportDialog isOpen={appState.isExportDialogOpen} onclose={() => appState.closeExportDialog()} />

	{#if appState.isTrashViewOpen}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="modal-overlay" onclick={() => appState.closeTrashView()} role="presentation">
			<div
				class="modal-container"
				onclick={(e) => e.stopPropagation()}
				role="dialog"
				aria-modal="true"
				tabindex="-1"
			>
				<TrashView />
				<button class="modal-close" onclick={() => appState.closeTrashView()} aria-label="Close">
					<svg
						width="24"
						height="24"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<line x1="18" y1="6" x2="6" y2="18" />
						<line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>
		</div>
	{/if}

	<ReviewGrid bind:isOpen={showReviewGrid} />
	<ImportDialog bind:isOpen={showImportDialog} />
	<SettingsDialog bind:isOpen={showSettingsDialog} />
	<ToastNotifications />

	<SnapshotsView
		isOpen={appState.isSnapshotsViewOpen}
		onclose={() => appState.closeSnapshotsView()}
	/>
</div>

<style>
	.layout {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}

	.main {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.sidebar {
		width: var(--sidebar-width);
		min-width: var(--sidebar-width);
		border-right: 1px solid var(--color-border);
		background-color: var(--color-bg-secondary);
		overflow-y: auto;
	}

	.content {
		flex: 1;
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	.context-panel {
		width: var(--context-panel-width);
		min-width: var(--context-panel-width);
		border-left: 1px solid var(--color-border);
		background-color: var(--color-bg-secondary);
		overflow-y: auto;
	}

	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal-container {
		position: relative;
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		width: 90%;
		max-width: 600px;
		height: 70vh;
		overflow: hidden;
	}

	.modal-close {
		position: absolute;
		top: var(--spacing-md);
		right: var(--spacing-md);
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		z-index: 10;
	}

	.modal-close:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	/* Focus/Fullscreen mode */
	.layout.focus-mode {
		background-color: var(--color-bg-primary);
	}

	.layout.focus-mode .main {
		display: flex;
		justify-content: center;
	}

	.layout.focus-mode .content {
		max-width: 800px;
		width: 100%;
	}

	.exit-focus-btn {
		position: fixed;
		top: var(--spacing-md);
		right: var(--spacing-md);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		color: var(--color-text-muted);
		opacity: 0.5;
		transition:
			opacity var(--transition-fast),
			background-color var(--transition-fast);
		z-index: 100;
	}

	.exit-focus-btn:hover {
		opacity: 1;
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}
</style>
