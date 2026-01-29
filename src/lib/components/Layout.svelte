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
	import { listen } from '@tauri-apps/api/event';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { confirm, message, open } from '@tauri-apps/plugin-dialog';
	import { onMount, untrack } from 'svelte';

	import type { Scene } from '$lib/api';
	import { appState, undoStack } from '$lib/stores';
	import { getOnboardingState, markTipShown } from '$lib/stores/onboarding';
	import {
		clearRecoveryDraft,
		getAllRecoveryDrafts,
		type RecoveryDraft,
	} from '$lib/stores/recovery';
	import { showInfo } from '$lib/toast';
	import { countWords, formatTimeAgo } from '$lib/utils';
	import { trapFocus } from '$lib/utils/focus-trap';

	import ArcsManager from './ArcsManager.svelte';
	import BibleView from './BibleView.svelte';
	import ContextPanel from './ContextPanel.svelte';
	import Corkboard from './Corkboard.svelte';
	import Dashboard from './Dashboard.svelte';
	import Editor from './Editor.svelte';
	import EventsManager from './EventsManager.svelte';
	import ExportDialog from './ExportDialog.svelte';
	import FeatureTour from './FeatureTour.svelte';
	import ImportDialog from './ImportDialog.svelte';
	import IssuesView from './IssuesView.svelte';
	import KeyboardShortcutsDialog from './KeyboardShortcutsDialog.svelte';
	import Outline from './Outline.svelte';
	import QuickOpen from './QuickOpen.svelte';
	import ReviewGrid from './ReviewGrid.svelte';
	import SettingsDialog from './SettingsDialog.svelte';
	import SnapshotsView from './SnapshotsView.svelte';
	import StatusBar from './StatusBar.svelte';
	import TemplatesManager from './TemplatesManager.svelte';
	import TimelineView from './TimelineView.svelte';
	import ToastNotifications from './ToastNotifications.svelte';
	import Toolbar from './Toolbar.svelte';
	import TrashView from './TrashView.svelte';
	import { Button, Dialog, Icon } from './ui';
	import Splitter from './ui/Splitter.svelte';

	// Modal stack: tracks open modals in order for Escape-key priority
	let modalStack = $state<string[]>([]);

	function openModal(id: string) {
		// Use untrack to avoid creating dependencies when called from effects
		untrack(() => {
			if (!modalStack.includes(id)) {
				modalStack = [...modalStack, id];
			}
		});
	}

	function closeModal(id: string) {
		// Use untrack to avoid creating dependencies when called from effects
		untrack(() => {
			modalStack = modalStack.filter((m) => m !== id);
		});
	}

	function closeTopModal(): boolean {
		if (modalStack.length === 0) return false;
		const top = modalStack[modalStack.length - 1];
		closeModal(top);
		// Also close the underlying state
		switch (top) {
			case 'reviewGrid':
				showReviewGrid = false;
				break;
			case 'importDialog':
				showImportDialog = false;
				break;
			case 'settingsDialog':
				showSettingsDialog = false;
				break;
			case 'arcsManager':
				showArcsManager = false;
				break;
			case 'eventsManager':
				showEventsManager = false;
				break;
			case 'templatesManager':
				showTemplatesManager = false;
				break;
			case 'keyboardShortcuts':
				showKeyboardShortcuts = false;
				break;
			case 'exportDialog':
				appState.closeExportDialog();
				break;
			case 'trashView':
				appState.closeTrashView();
				break;
			case 'snapshotsView':
				appState.closeSnapshotsView();
				break;
			case 'cutLibrary':
				appState.closeCutLibrary();
				break;
		}
		return true;
	}

	let showReviewGrid = $state(false);
	let showImportDialog = $state(false);
	let showSettingsDialog = $state(false);
	let showArcsManager = $state(false);
	let showEventsManager = $state(false);
	let showTemplatesManager = $state(false);
	let showKeyboardShortcuts = $state(false);
	// AF2: Recovery modal state
	let showRecoveryModal = $state(false);
	let recoveryDrafts = $state<RecoveryDraft[]>([]);

	// AV4: Feature tour state
	let showFeatureTour = $state(false);
	const featureTourSteps = [
		{
			target: '.sidebar',
			title: 'Outline',
			description:
				'Navigate your chapters and scenes here. Drag to reorder, right-click for options.',
			position: 'right' as const,
		},
		{
			target: '.content',
			title: 'Editor',
			description: 'Write your story here. Your work saves automatically every 10 seconds.',
			position: 'bottom' as const,
		},
		{
			target: '.context-panel',
			title: 'Context Panel',
			description:
				'See writing stats, linked characters, and analysis. Toggle tabs for different views.',
			position: 'left' as const,
		},
		{
			target: '.toolbar',
			title: 'Toolbar',
			description:
				'Switch views, open Quick Open (Cmd+K), and access settings. Use keyboard shortcuts for speed.',
			position: 'bottom' as const,
		},
	];

	// Sync boolean states with modal stack
	$effect(() => {
		if (showReviewGrid) openModal('reviewGrid');
		else closeModal('reviewGrid');
	});
	$effect(() => {
		if (showImportDialog) openModal('importDialog');
		else closeModal('importDialog');
	});
	$effect(() => {
		if (showSettingsDialog) openModal('settingsDialog');
		else closeModal('settingsDialog');
	});
	$effect(() => {
		if (showArcsManager) openModal('arcsManager');
		else closeModal('arcsManager');
	});
	$effect(() => {
		if (showEventsManager) openModal('eventsManager');
		else closeModal('eventsManager');
	});
	// Consume navigation requests from appState
	// These one-shot effects: read request, act if true, then clear
	$effect(() => {
		if (appState.requestOpenArcsManager) {
			showArcsManager = true;
			untrack(() => {
				appState.requestOpenArcsManager = false;
			});
		}
	});
	$effect(() => {
		if (appState.requestOpenEventsManager) {
			showEventsManager = true;
			untrack(() => {
				appState.requestOpenEventsManager = false;
			});
		}
	});
	$effect(() => {
		if (appState.requestOpenTemplatesManager) {
			showTemplatesManager = true;
			untrack(() => {
				appState.requestOpenTemplatesManager = false;
			});
		}
	});
	$effect(() => {
		if (showTemplatesManager) openModal('templatesManager');
		else closeModal('templatesManager');
	});
	$effect(() => {
		if (showKeyboardShortcuts) openModal('keyboardShortcuts');
		else closeModal('keyboardShortcuts');
	});
	$effect(() => {
		if (appState.isExportDialogOpen) openModal('exportDialog');
		else closeModal('exportDialog');
	});
	$effect(() => {
		if (appState.isTrashViewOpen) openModal('trashView');
		else closeModal('trashView');
	});
	$effect(() => {
		if (appState.isSnapshotsViewOpen) openModal('snapshotsView');
		else closeModal('snapshotsView');
	});
	$effect(() => {
		if (appState.isCutLibraryOpen) openModal('cutLibrary');
		else closeModal('cutLibrary');
	});

	// AF2: Check for recovery drafts when project loads
	$effect(() => {
		if (appState.project && appState.chapters.length > 0) {
			const drafts = getAllRecoveryDrafts();
			// Only show drafts for scenes that exist in current project
			const allSceneIds = new Set(
				Array.from(appState.scenes.values())
					.flat()
					.map((s) => s.id)
			);
			const relevantDrafts = drafts.filter((d) => allSceneIds.has(d.sceneId));
			if (relevantDrafts.length > 0) {
				recoveryDrafts = relevantDrafts;
				showRecoveryModal = true;
			}
		}
	});

	// AV4: Show feature tour for first-time users (after they have content to see)
	$effect(() => {
		if (
			appState.project &&
			appState.chapters.length > 0 &&
			!appState.isDemo &&
			!showRecoveryModal
		) {
			const onboarding = getOnboardingState();
			if (!onboarding.featureTourShown && !onboarding.completed) {
				// Delay slightly to let UI settle
				setTimeout(() => {
					showFeatureTour = true;
				}, 500);
			}
		}
	});

	function handleFeatureTourComplete() {
		showFeatureTour = false;
		markTipShown('featureTourShown');
	}

	function handleFeatureTourSkip() {
		showFeatureTour = false;
		markTipShown('featureTourShown');
	}

	// Scroll lock when modals are open
	$effect(() => {
		if (modalStack.length > 0) {
			const prev = document.body.style.overflow;
			document.body.style.overflow = 'hidden';
			return () => {
				document.body.style.overflow = prev;
			};
		}
		return undefined;
	});

	// BA6: Tauri close confirmation with save option
	onMount(() => {
		let unlisten: (() => void) | null = null;

		(async () => {
			try {
				const appWindow = getCurrentWindow();
				unlisten = await appWindow.onCloseRequested(async (event) => {
					if (appState.hasUnsavedChanges) {
						event.preventDefault();
						const shouldSave = await confirm('You have unsaved changes. Save before closing?', {
							title: 'Unsaved Changes',
							kind: 'warning',
						});
						if (shouldSave) {
							// Force save and then close
							await appState.forceSave();
							await appWindow.close();
						} else {
							// User chose to discard changes - close without saving
							await appWindow.destroy();
						}
					}
				});
			} catch {
				// Not in Tauri environment or API not available
			}
		})();

		return () => {
			if (unlisten) unlisten();
		};
	});

	// BD1: Save and restore scroll position per view
	let contentElement: HTMLElement | null = null;
	let previousViewMode: string | null = null;

	// Separate effect for scroll position restoration (runs when viewMode changes)
	$effect(() => {
		const currentView = appState.viewMode;
		if (!contentElement) return;

		// Check if view actually changed (using untracked previous value)
		if (previousViewMode !== null && previousViewMode !== currentView) {
			// Restore scroll position for new view
			const savedPosition = appState.getScrollPosition(currentView);
			contentElement.scrollTop = savedPosition;
		}
		// Update previous without triggering re-run
		previousViewMode = currentView;
	});

	// Separate effect for scroll listener setup
	$effect(() => {
		if (!contentElement) return;

		const handleScroll = () => {
			if (contentElement) {
				appState.saveScrollPosition(appState.viewMode, contentElement.scrollTop);
			}
		};
		contentElement.addEventListener('scroll', handleScroll);
		return () => contentElement?.removeEventListener('scroll', handleScroll);
	});

	// Resizable panel widths
	const DEFAULT_SIDEBAR_WIDTH = 260;
	const DEFAULT_CONTEXT_PANEL_WIDTH = 300;
	let sidebarWidth = $state(
		parseInt(localStorage.getItem('sidebarWidth') || String(DEFAULT_SIDEBAR_WIDTH), 10)
	);
	let contextPanelWidth = $state(
		parseInt(localStorage.getItem('contextPanelWidth') || String(DEFAULT_CONTEXT_PANEL_WIDTH), 10)
	);

	function handleSidebarResize(newWidth: number) {
		sidebarWidth = newWidth;
		localStorage.setItem('sidebarWidth', String(newWidth));
	}

	function handleContextPanelResize(newWidth: number) {
		contextPanelWidth = newWidth;
		localStorage.setItem('contextPanelWidth', String(newWidth));
	}

	function handleKeydown(event: KeyboardEvent) {
		// UD1: Global undo (Ctrl/Cmd+Shift+Z for app-level undo)
		// This handles deletion undo, not editor undo
		if ((event.metaKey || event.ctrlKey) && event.shiftKey && event.key === 'z') {
			event.preventDefault();
			undoStack.undo();
			return;
		}

		if (appState.matchesShortcut(event, 'quickOpen')) {
			event.preventDefault();
			appState.toggleQuickOpen();
			return;
		}

		if (appState.matchesShortcut(event, 'toggleContextPanel')) {
			event.preventDefault();
			appState.toggleContextPanel();
			return;
		}

		if (appState.matchesShortcut(event, 'toggleOutline')) {
			event.preventDefault();
			appState.toggleOutline();
			return;
		}

		// View switching
		const viewActions = [
			'viewEditor',
			'viewCorkboard',
			'viewTimeline',
			'viewBible',
			'viewIssues',
			'viewDashboard',
		] as const;
		const viewModes = ['editor', 'corkboard', 'timeline', 'bible', 'issues', 'dashboard'] as const;
		for (let i = 0; i < viewActions.length; i++) {
			if (appState.matchesShortcut(event, viewActions[i])) {
				event.preventDefault();
				appState.setViewMode(viewModes[i]);
				return;
			}
		}

		if (appState.matchesShortcut(event, 'toggleWorkMode')) {
			event.preventDefault();
			appState.toggleWorkMode();
			return;
		}

		if (appState.matchesShortcut(event, 'nextScene')) {
			event.preventDefault();
			navigateScene('next');
			return;
		}

		if (appState.matchesShortcut(event, 'prevScene')) {
			event.preventDefault();
			navigateScene('prev');
			return;
		}

		// AD4: Chapter navigation shortcuts
		if (appState.matchesShortcut(event, 'nextChapter')) {
			event.preventDefault();
			navigateChapter('next');
			return;
		}

		if (appState.matchesShortcut(event, 'prevChapter')) {
			event.preventDefault();
			navigateChapter('prev');
			return;
		}

		// V1: New chapter/scene shortcuts
		if (appState.matchesShortcut(event, 'newChapter')) {
			event.preventDefault();
			handleNewChapter();
			return;
		}

		if (appState.matchesShortcut(event, 'newScene')) {
			event.preventDefault();
			handleNewScene();
			return;
		}

		if (appState.matchesShortcut(event, 'save')) {
			event.preventDefault();
			if (appState.hasUnsavedChanges) {
				appState.triggerImmediateSave();
			}
			return;
		}

		if (appState.matchesShortcut(event, 'export')) {
			event.preventDefault();
			appState.openExportDialog();
			return;
		}

		if (appState.matchesShortcut(event, 'reviewGrid')) {
			event.preventDefault();
			showReviewGrid = !showReviewGrid;
			return;
		}

		if (appState.matchesShortcut(event, 'importDialog')) {
			event.preventDefault();
			showImportDialog = !showImportDialog;
			return;
		}

		if (appState.matchesShortcut(event, 'arcsManager')) {
			event.preventDefault();
			showArcsManager = !showArcsManager;
			return;
		}

		if (appState.matchesShortcut(event, 'eventsManager')) {
			event.preventDefault();
			showEventsManager = !showEventsManager;
			return;
		}

		if (appState.matchesShortcut(event, 'templatesManager')) {
			event.preventDefault();
			showTemplatesManager = !showTemplatesManager;
			return;
		}

		if (
			appState.matchesShortcut(event, 'fullscreen') ||
			appState.matchesShortcut(event, 'focusMode')
		) {
			event.preventDefault();
			appState.toggleFullscreenMode();
			return;
		}

		if (appState.matchesShortcut(event, 'showShortcuts')) {
			event.preventDefault();
			showKeyboardShortcuts = !showKeyboardShortcuts;
			return;
		}

		// Alt+Left: Navigate back
		if (event.altKey && event.key === 'ArrowLeft') {
			event.preventDefault();
			appState.navigateBack();
			return;
		}

		// Alt+Right: Navigate forward
		if (event.altKey && event.key === 'ArrowRight') {
			event.preventDefault();
			appState.navigateForward();
			return;
		}

		// Escape key priority chain
		if (event.key === 'Escape') {
			// 1. Close QuickOpen
			if (appState.isQuickOpenVisible) {
				event.preventDefault();
				appState.isQuickOpenVisible = false;
				return;
			}

			// 2. Close topmost modal from the stack
			if (modalStack.length > 0) {
				event.preventDefault();
				closeTopModal();
				return;
			}

			// 3. Exit focus mode
			if (appState.isFocusMode) {
				event.preventDefault();
				appState.isFocusMode = false;
				appState.focusSettings = { ...appState.focusSettings, fullscreenMode: false };
				return;
			}

			// 4. Deselect current scene
			if (appState.selectedSceneId) {
				event.preventDefault();
				appState.selectedSceneId = null;
				return;
			}
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

	/** AD4: Navigate to next/previous chapter */
	function navigateChapter(direction: 'next' | 'prev') {
		const chapters = appState.chapters;
		if (chapters.length === 0) return;

		const currentIndex = chapters.findIndex((c) => c.id === appState.selectedChapterId);

		let newIndex: number;
		if (direction === 'next') {
			newIndex = currentIndex === -1 ? 0 : Math.min(currentIndex + 1, chapters.length - 1);
		} else {
			newIndex = currentIndex === -1 ? 0 : Math.max(currentIndex - 1, 0);
		}

		const targetChapter = chapters[newIndex];
		if (targetChapter) {
			appState.selectedChapterId = targetChapter.id;
			// Select first scene in the chapter
			const chapterScenes = appState.scenes.get(targetChapter.id);
			if (chapterScenes && chapterScenes.length > 0) {
				appState.selectedSceneId = chapterScenes[0].id;
			} else {
				appState.selectedSceneId = null;
			}
		}
	}

	/** V1: Create a new chapter */
	async function handleNewChapter() {
		const title = `Chapter ${appState.chapters.length + 1}`;
		await appState.createChapter(title);
	}

	/** V1: Create a new scene in the current chapter */
	async function handleNewScene() {
		const chapterId = appState.selectedChapterId;
		if (!chapterId) {
			// AA3: Improved feedback for scene creation
			if (appState.chapters.length === 0) {
				// No chapters exist - create one automatically
				await handleNewChapter();
				// Then create a scene in the new chapter
				const newChapterId = appState.selectedChapterId;
				if (newChapterId) {
					await appState.createScene(newChapterId, 'Scene 1');
				}
			} else {
				// Chapters exist but none selected - inform user
				showInfo('Select a chapter first, or use Cmd+Shift+N to create a new chapter.');
			}
			return;
		}
		const chapterScenes = appState.scenes.get(chapterId) || [];
		const title = `Scene ${chapterScenes.length + 1}`;
		await appState.createScene(chapterId, title);
	}

	$effect(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => {
			window.removeEventListener('keydown', handleKeydown);
		};
	});

	// Listen for native menu events from Tauri
	$effect(() => {
		const unlistenPromise = listen<string>('menu-event', (event) => {
			const id = event.payload;
			switch (id) {
				case 'new_project':
					appState.closeProject();
					break;
				case 'open_project':
					handleMenuOpenProject();
					break;
				case 'save':
					if (appState.hasUnsavedChanges) appState.triggerImmediateSave();
					break;
				case 'export':
					appState.openExportDialog();
					break;
				case 'import':
					showImportDialog = true;
					break;
				case 'close_project':
					appState.closeProject();
					break;
				case 'about':
					handleMenuAbout();
					break;
				case 'view_editor':
					appState.setViewMode('editor');
					break;
				case 'view_corkboard':
					appState.setViewMode('corkboard');
					break;
				case 'view_timeline':
					appState.setViewMode('timeline');
					break;
				case 'view_bible':
					appState.setViewMode('bible');
					break;
				case 'view_issues':
					appState.setViewMode('issues');
					break;
				case 'view_dashboard':
					appState.setViewMode('dashboard');
					break;
				case 'toggle_outline':
					appState.toggleOutline();
					break;
				case 'toggle_context_panel':
					appState.toggleContextPanel();
					break;
				case 'focus_mode':
					appState.toggleFullscreenMode();
					break;
				case 'review_grid':
					showReviewGrid = !showReviewGrid;
					break;
				case 'quick_open':
					appState.toggleQuickOpen();
					break;
			}
		});

		return () => {
			unlistenPromise.then((fn) => fn());
		};
	});

	async function handleMenuOpenProject() {
		const selected = await open({
			filters: [{ name: 'Cahnon Project', extensions: ['cahnon'] }],
			multiple: false,
		});
		if (selected) {
			await appState.loadProject(selected);
		}
	}

	async function handleMenuAbout() {
		await message('Write freely. Stay consistent.\n\nA desktop application for fiction writers.', {
			title: 'Cahnon',
			kind: 'info',
		});
	}

	function handleBeforeUnload(event: BeforeUnloadEvent) {
		// UB7: Warn if writing session is active
		if (appState.writingSessionActive) {
			event.preventDefault();
			event.returnValue = 'Writing session in progress. Are you sure you want to leave?';
			return event.returnValue;
		}
		if (appState.hasUnsavedChanges) {
			event.preventDefault();
			event.returnValue = 'You have unsaved changes.';
			return event.returnValue;
		}
	}

	// AF2: Recovery modal helpers
	function getSceneTitleForRecovery(sceneId: string): string {
		for (const [, scenes] of appState.scenes) {
			const scene = scenes.find((s) => s.id === sceneId);
			if (scene) return scene.title;
		}
		return 'Unknown Scene';
	}

	function handleReviewRecovery() {
		// Navigate to the first recovery scene
		if (recoveryDrafts.length > 0) {
			const firstDraft = recoveryDrafts[0];
			for (const [chapterId, scenes] of appState.scenes) {
				const scene = scenes.find((s) => s.id === firstDraft.sceneId);
				if (scene) {
					appState.selectScene(scene.id, chapterId);
					appState.setViewMode('editor');
					break;
				}
			}
		}
		showRecoveryModal = false;
	}

	function handleDiscardAllRecovery() {
		clearRecoveryDraft();
		recoveryDrafts = [];
		showRecoveryModal = false;
	}
</script>

<svelte:window onbeforeunload={handleBeforeUnload} />

<!-- X3: Skip link for keyboard accessibility -->
<a href="#main-content" class="skip-link">Skip to content</a>

<!-- AI1: Hidden h1 for proper heading hierarchy -->
<h1 class="sr-only">{appState.project?.title || 'Cahnon'}</h1>

<div class="layout" class:focus-mode={appState.isFocusMode}>
	{#if appState.isLoading}
		<div class="global-loading-bar"></div>
		{#if appState.loadingStage}
			<div class="loading-stage">{appState.loadingStage}</div>
		{/if}
	{/if}

	{#if !appState.isFocusMode}
		<Toolbar
			onOpenReviewGrid={openReviewGrid}
			onOpenImportDialog={openImportDialog}
			onOpenSettings={openSettings}
			onOpenSnapshots={openSnapshots}
			onOpenKeyboardShortcuts={() => (showKeyboardShortcuts = true)}
		/>
	{/if}

	<div class="main">
		{#if appState.showOutline && !appState.isFocusMode}
			<aside class="sidebar" style="width: {sidebarWidth}px; min-width: {sidebarWidth}px">
				<Outline />
			</aside>
			<Splitter
				position={sidebarWidth}
				min={180}
				max={450}
				side="left"
				onresize={handleSidebarResize}
			/>
		{/if}

		<main id="main-content" class="content" bind:this={contentElement}>
			{#if appState.viewMode === 'editor'}
				<Editor />
			{:else if appState.viewMode === 'corkboard'}
				<Corkboard />
			{:else if appState.viewMode === 'bible'}
				<BibleView />
			{:else if appState.viewMode === 'timeline'}
				<TimelineView />
			{:else if appState.viewMode === 'issues'}
				<IssuesView />
			{:else if appState.viewMode === 'dashboard'}
				<Dashboard />
			{/if}
		</main>

		{#if appState.showContextPanel && (appState.viewMode === 'editor' || appState.viewMode === 'corkboard' || appState.viewMode === 'timeline' || appState.viewMode === 'bible') && !appState.isFocusMode}
			<Splitter
				position={contextPanelWidth}
				min={200}
				max={500}
				side="right"
				onresize={handleContextPanelResize}
			/>
			<aside
				class="context-panel"
				style="width: {contextPanelWidth}px; min-width: {contextPanelWidth}px"
			>
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

		<!-- AO1: Minimal save indicator in focus mode -->
		<div
			class="focus-save-indicator"
			class:saving={appState.isSaving}
			class:failed={appState.saveFailed}
			class:unsaved={appState.hasUnsavedChanges && !appState.isSaving}
			title={appState.saveFailed
				? 'Save failed - click to retry'
				: appState.isSaving
					? 'Saving...'
					: appState.hasUnsavedChanges
						? 'Unsaved changes'
						: 'All changes saved'}
		>
			{#if appState.saveFailed}
				<button
					class="save-dot failed"
					onclick={() => appState.retrySave()}
					title="Click to retry save"
				></button>
				<span class="save-label">Save failed</span>
			{:else if appState.isSaving}
				<span class="save-spinner-mini"></span>
			{:else if appState.hasUnsavedChanges}
				<span class="save-dot unsaved"></span>
			{:else}
				<span class="save-dot saved"></span>
			{/if}
		</div>
	{/if}

	{#if appState.isQuickOpenVisible}
		<QuickOpen />
	{/if}

	<ExportDialog isOpen={appState.isExportDialogOpen} onclose={() => appState.closeExportDialog()} />

	{#if appState.isTrashViewOpen}
		<div
			class="modal-overlay"
			onclick={() => appState.closeTrashView()}
			onkeydown={(e) => {
				if (e.key === 'Escape') appState.closeTrashView();
			}}
			role="presentation"
			tabindex="-1"
		>
			<div
				class="modal-container modal-enter"
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
				role="dialog"
				aria-modal="true"
				tabindex="-1"
				use:trapFocus={{ onEscape: () => appState.closeTrashView() }}
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
	<ArcsManager isOpen={showArcsManager} onclose={() => (showArcsManager = false)} />
	<EventsManager isOpen={showEventsManager} onclose={() => (showEventsManager = false)} />
	<TemplatesManager isOpen={showTemplatesManager} onclose={() => (showTemplatesManager = false)} />
	<KeyboardShortcutsDialog
		bind:isOpen={showKeyboardShortcuts}
		onclose={() => (showKeyboardShortcuts = false)}
	/>
	<ToastNotifications />

	<SnapshotsView
		isOpen={appState.isSnapshotsViewOpen}
		onclose={() => appState.closeSnapshotsView()}
	/>

	<!-- AV4: Feature tour for first-time users -->
	{#if showFeatureTour}
		<FeatureTour
			steps={featureTourSteps}
			onComplete={handleFeatureTourComplete}
			onSkip={handleFeatureTourSkip}
		/>
	{/if}

	<!-- BA2: Save Failed Modal - appears after 60s of persistent save failure -->
	<Dialog
		isOpen={appState.showSaveFailedModal}
		title="Save Failed"
		size="sm"
		onclose={() => appState.dismissSaveFailedModal()}
	>
		<div class="save-failed-content">
			<div class="save-failed-icon">
				<Icon name="alert-triangle" size={48} />
			</div>
			<p>Your changes haven't been saved for over a minute.</p>
			<p class="save-failed-warning">Don't close the app without saving or exporting your work.</p>
		</div>

		{#snippet footer()}
			<Button variant="ghost" onclick={() => appState.exportBackup()}>Export Backup</Button>
			<Button variant="primary" onclick={() => appState.retrySave()}>Retry Save</Button>
		{/snippet}
	</Dialog>

	<!-- AF2, AO5: Recovery modal with positive language -->
	{#if showRecoveryModal && recoveryDrafts.length > 0}
		<div class="modal-overlay" role="presentation" tabindex="-1">
			<div
				class="modal-container recovery-modal modal-enter"
				role="dialog"
				aria-modal="true"
				aria-labelledby="recovery-title"
				tabindex="-1"
				use:trapFocus={{ onEscape: () => (showRecoveryModal = false) }}
			>
				<!-- AO5: Positive header with success icon -->
				<div class="recovery-header">
					<svg
						class="recovery-icon"
						width="24"
						height="24"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
						<polyline points="22 4 12 14.01 9 11.01"></polyline>
					</svg>
					<h2 id="recovery-title">Your writing is safe!</h2>
				</div>
				<p class="recovery-description">
					Found {recoveryDrafts.length} unsaved draft{recoveryDrafts.length > 1 ? 's' : ''} ready to restore.
				</p>
				<ul class="recovery-list">
					{#each recoveryDrafts as draft (draft.sceneId)}
						<li class="recovery-item">
							<strong>{getSceneTitleForRecovery(draft.sceneId)}</strong>
							<span class="recovery-meta">
								{formatTimeAgo(new Date(draft.timestamp))} &middot; {countWords(draft.text)} words
							</span>
							<p class="recovery-preview">
								{draft.text.slice(0, 150)}{draft.text.length > 150 ? '...' : ''}
							</p>
						</li>
					{/each}
				</ul>
				<!-- AO5: Reordered buttons - positive action first (visually on right for LTR) -->
				<div class="recovery-actions">
					<button class="btn btn-ghost" onclick={handleDiscardAllRecovery}>Discard all</button>
					<button class="btn btn-primary" onclick={handleReviewRecovery}>
						Review &amp; Restore
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	/* X3: Skip link for keyboard accessibility */
	.skip-link {
		position: absolute;
		top: -40px;
		left: 0;
		background: var(--surface-elevated);
		color: var(--text-primary);
		padding: var(--spacing-sm) var(--spacing-md);
		z-index: 10000;
		text-decoration: none;
		border: 2px solid var(--accent-default);
		border-radius: var(--border-radius-sm);
		transition: top 0.2s;
	}

	.skip-link:focus {
		top: var(--spacing-sm);
		left: var(--spacing-sm);
	}

	.layout {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
		position: relative;
	}

	.global-loading-bar {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 2px;
		background: var(--color-accent);
		z-index: 9999;
		animation: loading-slide 1.2s ease-in-out infinite;
	}

	@keyframes loading-slide {
		0% {
			transform: translateX(-100%);
		}
		50% {
			transform: translateX(0%);
		}
		100% {
			transform: translateX(100%);
		}
	}

	.loading-stage {
		position: fixed;
		top: 8px;
		left: 50%;
		transform: translateX(-50%);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-align: center;
		z-index: 1001;
		background: var(--color-bg-primary);
		padding: 2px var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		box-shadow: var(--shadow-sm);
	}

	.main {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.sidebar {
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
		background-color: var(--overlay-backdrop);
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

	/* BA2: Save failed modal styles */
	.save-failed-content {
		text-align: center;
	}

	.save-failed-icon {
		color: var(--color-warning);
		margin-bottom: var(--spacing-md);
	}

	.save-failed-warning {
		color: var(--color-warning);
		font-weight: 500;
		margin-top: var(--spacing-sm);
	}

	/* AF2, AO5: Recovery modal styles */
	.recovery-modal {
		max-width: 500px;
		height: auto;
		padding: var(--spacing-lg);
	}

	/* AO5: Positive header with icon */
	.recovery-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		margin-bottom: var(--spacing-sm);
	}

	.recovery-icon {
		color: var(--color-success);
		flex-shrink: 0;
	}

	.recovery-modal h2 {
		margin: 0;
		font-size: var(--font-size-lg);
	}

	.recovery-description {
		margin: 0 0 var(--spacing-md) 0;
		color: var(--color-text-secondary);
	}

	.recovery-list {
		list-style: none;
		padding: 0;
		margin: 0 0 var(--spacing-lg) 0;
		max-height: 300px;
		overflow-y: auto;
	}

	.recovery-item {
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		margin-bottom: var(--spacing-sm);
	}

	.recovery-item strong {
		display: block;
		margin-bottom: var(--spacing-xs);
	}

	.recovery-meta {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.recovery-preview {
		margin: var(--spacing-xs) 0 0 0;
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.recovery-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
	}

	.btn {
		padding: var(--spacing-sm) var(--spacing-md);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		font-weight: 500;
		cursor: pointer;
	}

	.btn-ghost {
		background: transparent;
		color: var(--color-text-secondary);
	}

	.btn-ghost:hover {
		background: var(--color-bg-hover);
	}

	.btn-primary {
		background: var(--color-accent);
		color: var(--text-on-accent);
	}

	.btn-primary:hover {
		background: var(--color-accent-hover);
	}

	/* AO1: Focus mode save indicator */
	.focus-save-indicator {
		position: fixed;
		bottom: var(--spacing-md);
		left: var(--spacing-md);
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		opacity: 0.6;
		transition: opacity var(--transition-fast);
		z-index: 100;
	}

	.focus-save-indicator:hover {
		opacity: 1;
	}

	.focus-save-indicator .save-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		border: none;
		padding: 0;
	}

	.focus-save-indicator .save-dot.saved {
		background-color: var(--color-success);
	}

	.focus-save-indicator .save-dot.unsaved {
		background-color: var(--color-warning);
	}

	.focus-save-indicator .save-dot.failed {
		background-color: var(--color-error);
		cursor: pointer;
		animation: pulse 1s ease-in-out infinite;
	}

	.focus-save-indicator.failed {
		opacity: 1;
		color: var(--color-error);
	}

	.focus-save-indicator .save-label {
		font-size: var(--font-size-xs);
	}

	.save-spinner-mini {
		width: 8px;
		height: 8px;
		border: 1.5px solid var(--color-text-muted);
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes pulse {
		0%,
		100% {
			opacity: 0.7;
		}
		50% {
			opacity: 1;
		}
	}
</style>
