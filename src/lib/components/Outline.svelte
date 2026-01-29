<!--
  Chapter and scene tree navigation panel.

  Features:
  - Hierarchical view of chapters and scenes
  - Drag-and-drop reordering (scenes within/between chapters)
  - Inline creation of chapters and scenes
  - Status color indicators
  - Word count display per scene
  - Expandable/collapsible chapter sections
  - Context menu for edit/delete operations
-->
<script lang="ts">
	import { untrack } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';

	import { chapterApi, type Scene, sceneApi, trashApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { getAllRecoveryDrafts } from '$lib/stores/recovery';
	import { showError, showSuccess } from '$lib/toast';
	import { countWords, formatWordCount, sceneStatuses, statusColors } from '$lib/utils';

	import ImpactDialog from './ImpactDialog.svelte';
	import { Button, EmptyState, Icon } from './ui';
	import ContextMenu from './ui/ContextMenu.svelte';
	import ContextMenuItem from './ui/ContextMenuItem.svelte';
	import ContextMenuSeparator from './ui/ContextMenuSeparator.svelte';

	// AJ1: Track scenes with recovery drafts
	let recoverySceneIds = $derived.by(() => {
		const drafts = getAllRecoveryDrafts();
		return new Set(drafts.map((d) => d.sceneId));
	});

	// UC2: Compact view toggle
	let compactView = $state(
		typeof localStorage !== 'undefined' && localStorage.getItem('outline-compact') === 'true'
	);

	$effect(() => {
		try {
			localStorage.setItem('outline-compact', String(compactView));
		} catch {
			// localStorage unavailable
		}
	});

	// Impact dialog state
	let impactDialog = $state<{
		entityType: 'scene' | 'chapter';
		entityId: string;
		entityName: string;
	} | null>(null);

	// Context menu state
	let contextMenu = $state<{
		x: number;
		y: number;
		menuType: 'chapter' | 'scene';
		sceneId: string;
		chapterId: string;
	} | null>(null);

	// Chapter inline editing state
	let editingChapterId = $state<string | null>(null);
	let editingChapterTitle = $state('');
	let showChapterDetails = $state(false);
	let editingChapterSummary = $state('');
	let editingChapterStatus = $state('draft');
	let editingChapterNotes = $state('');

	// Scene inline editing state
	let editingSceneId = $state<string | null>(null);
	let editingSceneTitle = $state('');

	let expandedChapters = new SvelteSet<string>(loadExpandedChapters());

	function getExpandedKey(): string {
		return `expandedChapters:${appState.projectPath || 'default'}`;
	}

	function loadExpandedChapters(): string[] {
		try {
			const stored = localStorage.getItem(getExpandedKey());
			return stored ? JSON.parse(stored) : [];
		} catch {
			return [];
		}
	}

	// Persist expanded chapters when they change
	$effect(() => {
		const ids = Array.from(expandedChapters);
		try {
			localStorage.setItem(getExpandedKey(), JSON.stringify(ids));
		} catch {
			// localStorage unavailable
		}
	});

	// Drag and drop state
	let draggedItem = $state<{ type: 'chapter' | 'scene'; id: string; chapterId?: string } | null>(
		null
	);
	let dropTarget = $state<{
		type: 'chapter' | 'scene' | 'chapter-end';
		id: string;
		position?: 'before' | 'after';
	} | null>(null);

	// Create derived values that track state reactively
	let chapters = $derived(appState.chapters);
	let scenes = $derived(appState.scenes);

	// CB1: Persist Outline filter in localStorage per project
	function getFilterKey(): string {
		return `outline-filter-${appState.projectPath || 'default'}`;
	}

	function loadPersistedFilter(): string {
		try {
			return localStorage.getItem(getFilterKey()) || '';
		} catch {
			return '';
		}
	}

	// Outline filter
	let filterQuery = $state(loadPersistedFilter());
	let isFiltering = $derived(filterQuery.trim().length > 0);
	// AW4: Store expanded state before filtering to restore after (non-reactive to avoid loops)
	let preFilterExpanded: Set<string> | null = null;

	// CB1: Persist filter query when it changes
	$effect(() => {
		try {
			localStorage.setItem(getFilterKey(), filterQuery);
		} catch {
			// localStorage unavailable
		}
	});

	let filteredChapters = $derived.by(() => {
		if (!isFiltering) return chapters;
		const q = filterQuery.trim().toLowerCase();
		return chapters.filter((chapter) => {
			if (chapter.title.toLowerCase().includes(q)) return true;
			const chapterScenes = scenes.get(chapter.id) || [];
			return chapterScenes.some((scene) => scene.title.toLowerCase().includes(q));
		});
	});

	function getFilteredScenes(chapterId: string) {
		const chapterScenes = scenes.get(chapterId) || [];
		if (!isFiltering) return chapterScenes;
		const q = filterQuery.trim().toLowerCase();
		// If chapter title matches, show all its scenes
		const chapter = chapters.find((c) => c.id === chapterId);
		if (chapter && chapter.title.toLowerCase().includes(q)) return chapterScenes;
		// Otherwise filter scenes by title
		return chapterScenes.filter((scene) => scene.title.toLowerCase().includes(q));
	}

	let filterResultCount = $derived.by(() => {
		if (!isFiltering) return 0;
		let count = 0;
		for (const chapter of filteredChapters) {
			count += getFilteredScenes(chapter.id).length;
		}
		return count;
	});

	let hasInitializedExpansion = false;

	$effect(() => {
		if (chapters.length > 0 && !hasInitializedExpansion) {
			hasInitializedExpansion = true;
			// Only auto-expand first chapter if nothing was persisted
			// Use untrack to avoid creating dependency on expandedChapters
			untrack(() => {
				if (expandedChapters.size === 0) {
					expandedChapters.add(chapters[0].id);
				}
			});
		}
	});

	// AW4: Save expanded state when starting to filter, restore when clearing
	$effect(() => {
		// Use untrack for reading/writing expandedChapters to avoid loops
		if (isFiltering && preFilterExpanded === null) {
			// Starting to filter - save current state
			preFilterExpanded = new Set(untrack(() => expandedChapters));
		} else if (!isFiltering && preFilterExpanded !== null) {
			// Filter cleared - restore previous state
			untrack(() => {
				expandedChapters.clear();
				for (const id of preFilterExpanded!) {
					expandedChapters.add(id);
				}
			});
			preFilterExpanded = null;
		}
	});

	// Auto-scroll to selected scene and expand its chapter
	$effect(() => {
		const sceneId = appState.selectedSceneId;
		const chapterId = appState.selectedChapterId;
		if (!sceneId) return;

		// Expand the chapter containing the selected scene
		// Use untrack to avoid creating dependency on expandedChapters
		untrack(() => {
			if (chapterId && !expandedChapters.has(chapterId)) {
				expandedChapters.add(chapterId);
			}
		});

		// AC2: Scroll to the scene item after DOM update (center in view)
		requestAnimationFrame(() => {
			const el = document.querySelector(`[data-scene-id="${sceneId}"]`);
			el?.scrollIntoView({ block: 'center', behavior: 'smooth' });
		});
	});

	function toggleChapter(id: string) {
		if (expandedChapters.has(id)) {
			expandedChapters.delete(id);
		} else {
			expandedChapters.add(id);
		}
	}

	function selectChapter(id: string) {
		appState.selectScene('', id);
		appState.selectedChapterId = id;
		if (!expandedChapters.has(id)) {
			expandedChapters.add(id);
		}
	}

	function selectScene(sceneId: string, chapterId: string) {
		appState.selectScene(sceneId, chapterId);
	}

	async function handleAddChapter() {
		const title = `Chapter ${chapters.length + 1}`;
		await appState.createChapter(title);
	}

	async function handleAddScene(chapterId: string) {
		const chapterScenes = scenes.get(chapterId) || [];
		const title = `Scene ${chapterScenes.length + 1}`;
		const newScene = await appState.createScene(chapterId, title);

		// AO3: Auto-select and focus the new scene
		// Ensure chapter is expanded
		if (!expandedChapters.has(chapterId)) {
			expandedChapters.add(chapterId);
		}

		// Switch to editor view if not already
		if (appState.viewMode !== 'editor') {
			appState.setViewMode('editor');
		}

		// AV2: Keep focus in Outline on the new scene item instead of jumping to editor
		// This allows the user to continue organizing scenes or rename immediately
		requestAnimationFrame(() => {
			const sceneItem = document.querySelector(`[data-scene-id="${newScene.id}"]`);
			if (sceneItem instanceof HTMLElement) {
				sceneItem.focus();
			}
		});

		showSuccess(`Created "${newScene.title}"`);
	}

	function getChapterWordCount(chapterId: string): number {
		const chapterScenes = scenes.get(chapterId) || [];
		return chapterScenes.reduce((sum, scene) => sum + countWords(scene.text), 0);
	}

	// Drag and drop handlers
	function handleDragStart(
		event: DragEvent,
		type: 'chapter' | 'scene',
		id: string,
		chapterId?: string
	) {
		if (!event.dataTransfer) return;
		event.dataTransfer.effectAllowed = 'move';
		event.dataTransfer.setData('text/plain', JSON.stringify({ type, id, chapterId }));
		draggedItem = { type, id, chapterId };
	}

	function handleDragEnd() {
		draggedItem = null;
		dropTarget = null;
	}

	function handleDragOver(
		event: DragEvent,
		targetType: 'chapter' | 'scene' | 'chapter-end',
		targetId: string,
		position?: 'before' | 'after'
	) {
		event.preventDefault();
		if (!event.dataTransfer) return;
		event.dataTransfer.dropEffect = 'move';
		dropTarget = { type: targetType, id: targetId, position };
	}

	function handleDragLeave() {
		dropTarget = null;
	}

	async function handleDrop(
		event: DragEvent,
		targetType: 'chapter' | 'scene' | 'chapter-end',
		targetId: string,
		targetChapterId?: string
	) {
		event.preventDefault();
		if (!draggedItem) return;

		try {
			if (draggedItem.type === 'chapter' && targetType === 'chapter') {
				// Reorder chapters
				const chapterIds = chapters.map((c) => c.id);
				const fromIndex = chapterIds.indexOf(draggedItem.id);
				const toIndex = chapterIds.indexOf(targetId);

				if (fromIndex !== -1 && toIndex !== -1 && fromIndex !== toIndex) {
					chapterIds.splice(fromIndex, 1);
					chapterIds.splice(toIndex, 0, draggedItem.id);
					await chapterApi.reorder(chapterIds);
					await appState.loadChapters();
				}
			} else if (draggedItem.type === 'scene') {
				const sourceChapterId = draggedItem.chapterId!;
				const sourceScenes = scenes.get(sourceChapterId) || [];

				if (targetType === 'scene' && targetChapterId) {
					// Reorder scenes within same chapter or move to different chapter
					if (sourceChapterId === targetChapterId) {
						// Same chapter - reorder
						const sceneIds = sourceScenes.map((s) => s.id);
						const fromIndex = sceneIds.indexOf(draggedItem.id);
						const toIndex = sceneIds.indexOf(targetId);

						if (fromIndex !== -1 && toIndex !== -1 && fromIndex !== toIndex) {
							sceneIds.splice(fromIndex, 1);
							sceneIds.splice(toIndex, 0, draggedItem.id);
							await sceneApi.reorder(sourceChapterId, sceneIds);
							await appState.loadChapters();
						}
					} else {
						// Different chapter - move scene
						const targetScenes = scenes.get(targetChapterId) || [];
						const toIndex = targetScenes.findIndex((s) => s.id === targetId);
						await sceneApi.moveToChapter(
							draggedItem.id,
							targetChapterId,
							toIndex >= 0 ? toIndex : 0
						);
						await appState.loadChapters();
					}
				} else if (targetType === 'chapter-end') {
					// Move scene to end of a chapter
					if (sourceChapterId !== targetId) {
						const targetScenes = scenes.get(targetId) || [];
						await sceneApi.moveToChapter(draggedItem.id, targetId, targetScenes.length);
						await appState.loadChapters();
					}
				}
			}
		} catch (e) {
			console.error('Failed to reorder:', e);
			showError('Failed to reorder items');
		}

		draggedItem = null;
		dropTarget = null;
	}

	function isDropTarget(type: string, id: string): boolean {
		return dropTarget?.type === type && dropTarget?.id === id;
	}

	function handleContextMenu(event: MouseEvent, sceneId: string, chapterId: string) {
		event.preventDefault();
		contextMenu = { x: event.clientX, y: event.clientY, menuType: 'scene', sceneId, chapterId };
	}

	function closeContextMenu() {
		contextMenu = null;
	}

	async function handleDuplicateScene(sceneId: string, structureOnly: boolean = false) {
		try {
			const duplicated = await trashApi.duplicateScene(sceneId, structureOnly);
			await appState.loadChapters();
			// Select the duplicated scene
			appState.selectedSceneId = duplicated.id;
		} catch (e) {
			console.error('Failed to duplicate scene:', e);
			showError('Failed to duplicate scene');
		}
		closeContextMenu();
	}

	async function handleDeleteScene(sceneId: string) {
		const scene = Array.from(scenes.values())
			.flat()
			.find((s) => s.id === sceneId);
		impactDialog = {
			entityType: 'scene',
			entityId: sceneId,
			entityName: scene?.title || 'Scene',
		};
		closeContextMenu();
	}

	async function confirmDelete() {
		if (!impactDialog) return;
		const { entityType, entityId, entityName } = impactDialog;
		try {
			if (entityType === 'scene') {
				await appState.deleteScene(entityId);
				// AF3: Show toast with undo action
				showSuccess(`Scene "${entityName}" deleted`, {
					action: {
						label: 'Undo',
						onClick: async () => {
							try {
								await appState.restoreScene(entityId);
								showSuccess(`Scene "${entityName}" restored`);
							} catch (e) {
								console.error('Failed to restore scene:', e);
								showError('Failed to restore scene');
							}
						},
					},
					duration: 10000,
				});
			} else if (entityType === 'chapter') {
				await appState.deleteChapter(entityId);
			}
		} catch (e) {
			console.error(`Failed to delete ${entityType}:`, e);
			showError(`Failed to delete ${entityType}`);
		}
		impactDialog = null;
	}

	function handleChapterContextMenu(event: MouseEvent, chapterId: string) {
		event.preventDefault();
		contextMenu = {
			x: event.clientX,
			y: event.clientY,
			menuType: 'chapter',
			sceneId: '',
			chapterId,
		};
	}

	function startRenamingChapter(chapterId: string) {
		const chapter = chapters.find((c) => c.id === chapterId);
		if (!chapter) return;
		editingChapterId = chapterId;
		editingChapterTitle = chapter.title;
		closeContextMenu();
	}

	async function finishRenamingChapter() {
		if (!editingChapterId || !editingChapterTitle.trim()) {
			editingChapterId = null;
			return;
		}
		const chapterId = editingChapterId;
		try {
			await chapterApi.update(chapterId, { title: editingChapterTitle.trim() });
			await appState.loadChapters();
			// AP5: Flash confirmation
			justSavedChapterId = chapterId;
			setTimeout(() => {
				if (justSavedChapterId === chapterId) justSavedChapterId = null;
			}, 1000);
		} catch (e) {
			console.error('Failed to rename chapter:', e);
			showError('Failed to rename chapter');
		}
		editingChapterId = null;
	}

	function cancelRenamingChapter() {
		editingChapterId = null;
	}

	function startRenamingScene(sceneId: string) {
		const scene = Array.from(scenes.values())
			.flat()
			.find((s) => s.id === sceneId);
		if (!scene) return;
		editingSceneTitle = scene.title;
		editingSceneId = sceneId;
	}

	async function finishRenamingScene() {
		if (!editingSceneId || !editingSceneTitle.trim()) {
			editingSceneId = null;
			return;
		}
		const sceneId = editingSceneId;
		try {
			await sceneApi.update(sceneId, { title: editingSceneTitle.trim() });
			await appState.loadChapters();
			// AP5: Flash confirmation
			justSavedSceneId = sceneId;
			setTimeout(() => {
				if (justSavedSceneId === sceneId) justSavedSceneId = null;
			}, 1000);
		} catch (e) {
			console.error('Failed to rename scene:', e);
			showError('Failed to rename scene');
		}
		editingSceneId = null;
	}

	function cancelRenamingScene() {
		editingSceneId = null;
	}

	function openChapterDetails(chapterId: string) {
		const chapter = chapters.find((c) => c.id === chapterId);
		if (!chapter) return;
		editingChapterId = chapterId;
		editingChapterSummary = chapter.summary || '';
		editingChapterStatus = chapter.status || 'draft';
		editingChapterNotes = chapter.notes || '';
		showChapterDetails = true;
		closeContextMenu();
	}

	async function saveChapterDetails() {
		if (!editingChapterId) return;
		try {
			await chapterApi.update(editingChapterId, {
				summary: editingChapterSummary.trim() || undefined,
				status: editingChapterStatus,
				notes: editingChapterNotes.trim() || undefined,
			});
			await appState.loadChapters();
		} catch (e) {
			console.error('Failed to update chapter:', e);
			showError('Failed to update chapter');
		}
		showChapterDetails = false;
		editingChapterId = null;
	}

	function cancelChapterDetails() {
		showChapterDetails = false;
		editingChapterId = null;
	}

	async function handleDeleteChapter(chapterId: string) {
		const chapter = chapters.find((c) => c.id === chapterId);
		impactDialog = {
			entityType: 'chapter',
			entityId: chapterId,
			entityName: chapter?.title || 'Chapter',
		};
		closeContextMenu();
	}

	// Keyboard accessibility for reordering
	let moveAnnouncement = $state('');
	// AI5: Track recently moved items for visual feedback
	let recentlyMovedId = $state<string | null>(null);

	// AP5: Track recently saved items for visual feedback
	let justSavedSceneId = $state<string | null>(null);
	let justSavedChapterId = $state<string | null>(null);

	function flashMovedItem(id: string) {
		recentlyMovedId = id;
		setTimeout(() => {
			if (recentlyMovedId === id) {
				recentlyMovedId = null;
			}
		}, 600);
	}

	async function moveChapter(chapterId: string, direction: 'up' | 'down') {
		const chapterIds = chapters.map((c) => c.id);
		const index = chapterIds.indexOf(chapterId);
		if (index === -1) return;
		const newIndex = direction === 'up' ? index - 1 : index + 1;
		if (newIndex < 0 || newIndex >= chapterIds.length) return;

		chapterIds.splice(index, 1);
		chapterIds.splice(newIndex, 0, chapterId);
		try {
			await chapterApi.reorder(chapterIds);
			await appState.loadChapters();
			const chapter = chapters.find((c) => c.id === chapterId);
			moveAnnouncement = `${chapter?.title || 'Chapter'} moved ${direction}`;
		} catch {
			showError('Failed to reorder chapter');
		}
	}

	async function moveScene(sceneId: string, chapterId: string, direction: 'up' | 'down') {
		const chapterScenes = scenes.get(chapterId) || [];
		const sceneIds = chapterScenes.map((s) => s.id);
		const index = sceneIds.indexOf(sceneId);
		if (index === -1) return;
		const newIndex = direction === 'up' ? index - 1 : index + 1;
		if (newIndex < 0 || newIndex >= sceneIds.length) return;

		sceneIds.splice(index, 1);
		sceneIds.splice(newIndex, 0, sceneId);
		try {
			await sceneApi.reorder(chapterId, sceneIds);
			await appState.loadChapters();
			const scene = chapterScenes.find((s) => s.id === sceneId);
			moveAnnouncement = `${scene?.title || 'Scene'} moved ${direction}`;
			// AI5: Visual feedback for keyboard move
			flashMovedItem(sceneId);
		} catch {
			showError('Failed to reorder scene');
		}
	}

	async function moveSceneBetweenChapters(
		sceneId: string,
		sourceChapterId: string,
		direction: 'prev' | 'next'
	) {
		const chapterIndex = chapters.findIndex((c) => c.id === sourceChapterId);
		if (chapterIndex === -1) return;
		const targetIndex = direction === 'prev' ? chapterIndex - 1 : chapterIndex + 1;
		if (targetIndex < 0 || targetIndex >= chapters.length) return;

		const targetChapter = chapters[targetIndex];
		const targetScenes = scenes.get(targetChapter.id) || [];
		try {
			await sceneApi.moveToChapter(sceneId, targetChapter.id, targetScenes.length);
			await appState.loadChapters();
			expandedChapters.add(targetChapter.id);
			const scene = (scenes.get(sourceChapterId) || []).find((s) => s.id === sceneId);
			moveAnnouncement = `${scene?.title || 'Scene'} moved to ${targetChapter.title}`;
		} catch {
			showError('Failed to move scene');
		}
	}

	function handleChapterKeydown(e: KeyboardEvent, chapterId: string) {
		if (e.key === 'Enter') {
			selectChapter(chapterId);
			return;
		}
		if (!e.ctrlKey && !e.metaKey) return;
		if (!e.shiftKey) return;
		if (e.key === 'ArrowUp') {
			e.preventDefault();
			moveChapter(chapterId, 'up');
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			moveChapter(chapterId, 'down');
		}
	}

	function handleSceneKeydown(e: KeyboardEvent, sceneId: string, chapterId: string) {
		if (e.key === 'F2') {
			e.preventDefault();
			startRenamingScene(sceneId);
			return;
		}
		if (!e.ctrlKey && !e.metaKey) return;
		if (!e.shiftKey) return;
		if (e.key === 'ArrowUp') {
			e.preventDefault();
			moveScene(sceneId, chapterId, 'up');
		} else if (e.key === 'ArrowDown') {
			e.preventDefault();
			moveScene(sceneId, chapterId, 'down');
		} else if (e.key === 'ArrowLeft') {
			e.preventDefault();
			moveSceneBetweenChapters(sceneId, chapterId, 'prev');
		} else if (e.key === 'ArrowRight') {
			e.preventDefault();
			moveSceneBetweenChapters(sceneId, chapterId, 'next');
		}
	}

	// Context menu close is handled by the ContextMenu component

	// BB3: Status legend popover state
	let showStatusLegend = $state(false);

	// CB2: Scene preview popover on hover
	let previewScene = $state<Scene | null>(null);
	let previewPos = $state({ x: 0, y: 0 });
	let previewTimer: ReturnType<typeof setTimeout> | null = null;

	function showPreview(scene: Scene, e: MouseEvent) {
		if (previewTimer) clearTimeout(previewTimer);
		previewTimer = setTimeout(() => {
			previewScene = scene;
			// Position below and to the right of cursor
			const x = Math.min(e.clientX + 10, window.innerWidth - 280);
			const y = Math.min(e.clientY + 10, window.innerHeight - 150);
			previewPos = { x, y };
		}, 400); // Delay to avoid flickering
	}

	function hidePreview() {
		if (previewTimer) clearTimeout(previewTimer);
		previewTimer = null;
		previewScene = null;
	}

	// CB5: Toggle hide scenes by status
	let hiddenStatuses = new SvelteSet<string>();

	function toggleHideStatus(status: string) {
		if (hiddenStatuses.has(status)) {
			hiddenStatuses.delete(status);
		} else {
			hiddenStatuses.add(status);
		}
	}

	function isSceneHidden(scene: Scene): boolean {
		return hiddenStatuses.has(scene.status);
	}
</script>

<div class="outline">
	<div class="outline-header">
		<h2>Manuscript</h2>
		<div class="header-actions">
			<!-- UC2: Compact view toggle -->
			<Button
				variant="icon"
				onclick={() => (compactView = !compactView)}
				title={compactView ? 'Expand view' : 'Compact view'}
			>
				<Icon name={compactView ? 'menu' : 'sort'} size={14} />
			</Button>
			<!-- BB3: Status legend trigger -->
			<div class="status-legend-trigger">
				<Button
					variant="icon"
					onclick={() => (showStatusLegend = !showStatusLegend)}
					title="Status legend"
				>
					<Icon name="help-circle" size={14} />
				</Button>
				{#if showStatusLegend}
					<div class="status-legend-popover">
						<div class="legend-header">Scene Status</div>
						<!-- CB5: Click to toggle visibility per status -->
						{#each sceneStatuses as status (status.value)}
							<button
								class="legend-item"
								class:hidden-status={hiddenStatuses.has(status.value)}
								onclick={() => toggleHideStatus(status.value)}
								title={hiddenStatuses.has(status.value) ? 'Show scenes' : 'Hide scenes'}
							>
								<span class="legend-dot" style="background-color: {statusColors[status.value]}"
								></span>
								<span class="legend-label">{status.label}</span>
								{#if hiddenStatuses.has(status.value)}
									<Icon name="eye-off" size={12} />
								{/if}
							</button>
						{/each}
						{#if hiddenStatuses.size > 0}
							<button class="legend-clear" onclick={() => hiddenStatuses.clear()}>
								Show all
							</button>
						{/if}
					</div>
				{/if}
			</div>
			<Button variant="icon" onclick={handleAddChapter} title="Add Chapter">
				<Icon name="plus" size={16} />
			</Button>
		</div>
	</div>

	{#if chapters.length > 0}
		<!-- AG5: More prominent filter with better placeholder -->
		<div class="outline-filter">
			<div class="filter-input-wrapper" class:has-value={filterQuery.length > 0}>
				<Icon name="search" size={14} />
				<input
					type="text"
					class="filter-input"
					placeholder="Search scenes..."
					bind:value={filterQuery}
					aria-label="Search scenes"
				/>
				{#if filterQuery}
					<button class="filter-clear" onclick={() => (filterQuery = '')} aria-label="Clear filter">
						<Icon name="close" size={12} />
					</button>
				{/if}
			</div>
			{#if isFiltering}
				<span class="filter-count"
					>{filterResultCount} scene{filterResultCount !== 1 ? 's' : ''}</span
				>
			{/if}
		</div>
	{/if}

	<div class="outline-tree" class:compact={compactView}>
		{#each filteredChapters as chapter (chapter.id)}
			{@const chapterScenes = getFilteredScenes(chapter.id)}
			{@const isExpanded = isFiltering || expandedChapters.has(chapter.id)}
			{@const isSelected = appState.selectedChapterId === chapter.id && !appState.selectedSceneId}
			{@const wordCount = getChapterWordCount(chapter.id)}

			<div class="chapter-group">
				<div
					class="chapter-item"
					class:selected={isSelected}
					class:drag-disabled={isFiltering}
					class:drop-target={!isFiltering && isDropTarget('chapter', chapter.id)}
					class:dragging={!isFiltering &&
						draggedItem?.type === 'chapter' &&
						draggedItem?.id === chapter.id}
					role="button"
					tabindex="0"
					draggable={!isFiltering}
					onclick={() => selectChapter(chapter.id)}
					onkeydown={(e) => handleChapterKeydown(e, chapter.id)}
					oncontextmenu={(e) => handleChapterContextMenu(e, chapter.id)}
					ondblclick={(e) => {
						e.preventDefault();
						startRenamingChapter(chapter.id);
					}}
					ondragstart={(e) => handleDragStart(e, 'chapter', chapter.id)}
					ondragend={handleDragEnd}
					ondragover={(e) => handleDragOver(e, 'chapter', chapter.id)}
					ondragleave={handleDragLeave}
					ondrop={(e) => handleDrop(e, 'chapter', chapter.id)}
				>
					<button
						class="expand-btn"
						onclick={(e) => {
							e.stopPropagation();
							toggleChapter(chapter.id);
						}}
						aria-label={isExpanded ? 'Collapse chapter' : 'Expand chapter'}
					>
						<span class:expanded={isExpanded}>
							<Icon name="chevron-right" size={12} />
						</span>
					</button>

					<span
						class="status-dot"
						role="img"
						aria-label="Chapter status: {chapter.status || 'draft'}"
						title={chapter.status || 'draft'}
						style="background-color: {statusColors[chapter.status] || 'var(--color-text-muted)'}"
					></span>

					{#if editingChapterId === chapter.id && !showChapterDetails}
						<!-- svelte-ignore a11y_autofocus -->
						<input
							type="text"
							class="inline-rename"
							maxlength={100}
							bind:value={editingChapterTitle}
							onkeydown={(e) => {
								if (e.key === 'Enter') {
									e.preventDefault();
									finishRenamingChapter();
								} else if (e.key === 'Escape') {
									cancelRenamingChapter();
								}
							}}
							onblur={finishRenamingChapter}
							onclick={(e) => e.stopPropagation()}
							autofocus
						/>
					{:else}
						<span
							class="chapter-title truncate"
							class:just-saved={justSavedChapterId === chapter.id}>{chapter.title}</span
						>
					{/if}

					<!-- CB4: Scene count per chapter -->
					<span class="scene-count">{chapterScenes.length}</span>
					<span class="word-count">{formatWordCount(wordCount)}</span>

					<button
						class="add-scene-btn"
						onclick={(e) => {
							e.stopPropagation();
							handleAddScene(chapter.id);
						}}
						title="Add Scene"
						aria-label="Add Scene"
					>
						<Icon name="plus" size={12} />
					</button>
				</div>

				{#if isExpanded}
					<div class="scenes-list">
						{#each chapterScenes as scene (scene.id)}
							{@const isSceneSelected = appState.selectedSceneId === scene.id}
							{@const health = appState.sceneHealthMap.get(scene.id)}
							<!-- CB5: Hide scene if its status is hidden -->
							{#if !isSceneHidden(scene)}
								<button
									class="scene-item"
									class:selected={isSceneSelected}
									class:drag-disabled={isFiltering}
									class:drop-target={!isFiltering && isDropTarget('scene', scene.id)}
									class:just-moved={recentlyMovedId === scene.id}
									class:dragging={!isFiltering &&
										draggedItem?.type === 'scene' &&
										draggedItem?.id === scene.id}
									draggable={!isFiltering}
									data-scene-id={scene.id}
									onclick={() => selectScene(scene.id, chapter.id)}
									onkeydown={(e) => handleSceneKeydown(e, scene.id, chapter.id)}
									oncontextmenu={(e) => handleContextMenu(e, scene.id, chapter.id)}
									ondragstart={(e) => handleDragStart(e, 'scene', scene.id, chapter.id)}
									ondragend={handleDragEnd}
									ondragover={(e) => handleDragOver(e, 'scene', scene.id)}
									ondragleave={handleDragLeave}
									ondrop={(e) => handleDrop(e, 'scene', scene.id, chapter.id)}
									onmouseenter={(e) => showPreview(scene, e)}
									onmouseleave={hidePreview}
								>
									<span
										class="status-dot small"
										role="img"
										aria-label="Scene status: {scene.status || 'draft'}"
										title={scene.status || 'draft'}
										style="background-color: {statusColors[scene.status] ||
											'var(--color-text-muted)'}"
									></span>
									{#if editingSceneId === scene.id}
										<!-- svelte-ignore a11y_autofocus -->
										<input
											type="text"
											class="inline-rename"
											maxlength={100}
											bind:value={editingSceneTitle}
											onkeydown={(e) => {
												if (e.key === 'Enter') {
													e.preventDefault();
													finishRenamingScene();
												} else if (e.key === 'Escape') {
													cancelRenamingScene();
												}
											}}
											onblur={finishRenamingScene}
											onclick={(e) => e.stopPropagation()}
											autofocus
										/>
									{:else}
										<span
											class="scene-title truncate"
											class:just-saved={justSavedSceneId === scene.id}>{scene.title}</span
										>
										<!-- CB3: POV initial inline -->
										{#if scene.pov}
											<span class="pov-initial" title="POV: {scene.pov}"
												>{scene.pov.charAt(0).toUpperCase()}</span
											>
										{/if}
										{#if appState.isFavorite(scene.id)}
											<span class="favorite-star" title="Pinned">&#9733;</span>
										{/if}
										{#if recoverySceneIds.has(scene.id)}
											<span class="recovery-badge" title="Has unsaved recovery draft">
												<Icon name="alert" size={12} />
											</span>
										{/if}
									{/if}
									{#if health && health.score < 1.0}
										<span
											class="health-dot"
											title="Health: {Math.round(health.score * 100)}% — Issues: {health.checks
												.filter((c) => !c.passed)
												.map((c) => c.label)
												.join(', ')}"
										></span>
									{/if}
									<span class="word-count">{formatWordCount(countWords(scene.text))}</span>
								</button>
							{/if}
						{/each}

						{#if chapterScenes.length === 0}
							<div
								class="empty-chapter"
								class:drop-target={isDropTarget('chapter-end', chapter.id)}
								ondragover={(e) => handleDragOver(e, 'chapter-end', chapter.id)}
								ondragleave={handleDragLeave}
								ondrop={(e) => handleDrop(e, 'chapter-end', chapter.id)}
								role="presentation"
							>
								<button class="add-first-scene" onclick={() => handleAddScene(chapter.id)}>
									+ Add first scene
								</button>
							</div>
						{:else}
							<div
								class="drop-zone"
								class:drop-target={isDropTarget('chapter-end', chapter.id)}
								ondragover={(e) => handleDragOver(e, 'chapter-end', chapter.id)}
								ondragleave={handleDragLeave}
								ondrop={(e) => handleDrop(e, 'chapter-end', chapter.id)}
								role="presentation"
							></div>
							<button class="add-scene-inline" onclick={() => handleAddScene(chapter.id)}>
								+ Add scene
							</button>
						{/if}
					</div>
				{/if}
			</div>
		{/each}

		{#if isFiltering && filteredChapters.length === 0}
			<!-- AJ3: Specific empty state when filter matches nothing -->
			<EmptyState
				compact
				icon="search"
				title="No matching scenes"
				description="No scenes match '{filterQuery}'"
				actionLabel="Clear filter"
				onaction={() => (filterQuery = '')}
			/>
		{:else if chapters.length === 0}
			<EmptyState
				compact
				icon="book"
				title="Start your manuscript"
				description="Chapters organize your story. Each chapter contains scenes."
				actionLabel="Create First Chapter"
				onaction={handleAddChapter}
			/>
		{/if}
	</div>

	{#if contextMenu}
		<ContextMenu x={contextMenu.x} y={contextMenu.y} onclose={closeContextMenu}>
			{#if contextMenu.menuType === 'chapter'}
				<ContextMenuItem
					icon="edit"
					label="Rename"
					shortcut="F2"
					onclick={() => {
						startRenamingChapter(contextMenu!.chapterId);
						closeContextMenu();
					}}
				/>
				<ContextMenuItem
					icon="settings"
					label="Edit Details…"
					onclick={() => {
						openChapterDetails(contextMenu!.chapterId);
						closeContextMenu();
					}}
				/>
				<ContextMenuSeparator />
				<ContextMenuItem
					label="Move Up"
					shortcut="⌘⇧↑"
					onclick={() => {
						moveChapter(contextMenu!.chapterId, 'up');
						closeContextMenu();
					}}
				/>
				<ContextMenuItem
					label="Move Down"
					shortcut="⌘⇧↓"
					onclick={() => {
						moveChapter(contextMenu!.chapterId, 'down');
						closeContextMenu();
					}}
				/>
				<ContextMenuSeparator />
				<ContextMenuItem
					icon="trash"
					label="Delete"
					danger
					onclick={() => {
						handleDeleteChapter(contextMenu!.chapterId);
						closeContextMenu();
					}}
				/>
			{:else}
				<ContextMenuItem
					icon="edit"
					label="Rename"
					shortcut="F2"
					onclick={() => {
						startRenamingScene(contextMenu!.sceneId);
						closeContextMenu();
					}}
				/>
				<ContextMenuSeparator />
				<ContextMenuItem
					label="Move Up"
					shortcut="⌘⇧↑"
					onclick={() => {
						moveScene(contextMenu!.sceneId, contextMenu!.chapterId, 'up');
						closeContextMenu();
					}}
				/>
				<ContextMenuItem
					label="Move Down"
					shortcut="⌘⇧↓"
					onclick={() => {
						moveScene(contextMenu!.sceneId, contextMenu!.chapterId, 'down');
						closeContextMenu();
					}}
				/>
				<ContextMenuSeparator />
				<ContextMenuItem
					icon="copy"
					label="Duplicate"
					onclick={() => {
						handleDuplicateScene(contextMenu!.sceneId, false);
						closeContextMenu();
					}}
				/>
				<ContextMenuItem
					icon="scissors"
					label="Duplicate (structure only)"
					onclick={() => {
						handleDuplicateScene(contextMenu!.sceneId, true);
						closeContextMenu();
					}}
				/>
				<ContextMenuItem
					icon="pin"
					label={appState.isFavorite(contextMenu!.sceneId) ? 'Unpin' : 'Pin'}
					onclick={() => {
						appState.toggleFavorite(contextMenu!.sceneId);
						closeContextMenu();
					}}
				/>
				<ContextMenuSeparator />
				<ContextMenuItem
					icon="trash"
					label="Delete"
					danger
					onclick={() => {
						handleDeleteScene(contextMenu!.sceneId);
						closeContextMenu();
					}}
				/>
			{/if}
		</ContextMenu>
	{/if}

	{#if showChapterDetails}
		<div
			class="chapter-details-overlay"
			onclick={cancelChapterDetails}
			onkeydown={(e) => {
				if (e.key === 'Escape') cancelChapterDetails();
			}}
			role="presentation"
			tabindex="-1"
		>
			<div
				class="chapter-details-dialog"
				onclick={(e) => e.stopPropagation()}
				onkeydown={(e) => e.stopPropagation()}
				role="dialog"
				aria-modal="true"
				tabindex="-1"
			>
				<h3>Edit Chapter Details</h3>
				<div class="chapter-details-form">
					<label class="form-label">
						Summary
						<textarea bind:value={editingChapterSummary} placeholder="Chapter summary..." rows="3"
						></textarea>
					</label>
					<label class="form-label">
						Status
						<select bind:value={editingChapterStatus}>
							<option value="draft">Draft</option>
							<option value="in_progress">In Progress</option>
							<option value="revised">Revised</option>
							<option value="done">Done</option>
						</select>
					</label>
					<label class="form-label">
						Notes
						<textarea bind:value={editingChapterNotes} placeholder="Private notes..." rows="3"
						></textarea>
					</label>
					<div class="chapter-details-actions">
						<Button variant="ghost" size="sm" onclick={cancelChapterDetails}>Cancel</Button>
						<Button variant="primary" size="sm" onclick={saveChapterDetails}>Save</Button>
					</div>
				</div>
			</div>
		</div>
	{/if}

	{#if impactDialog}
		<ImpactDialog
			entityType={impactDialog.entityType}
			entityId={impactDialog.entityId}
			entityName={impactDialog.entityName}
			onconfirm={confirmDelete}
			oncancel={() => (impactDialog = null)}
		/>
	{/if}

	<div aria-live="polite" class="sr-only">{moveAnnouncement}</div>

	<!-- CB2: Scene preview popover -->
	{#if previewScene}
		<div
			class="scene-preview-popover"
			style="left: {previewPos.x}px; top: {previewPos.y}px;"
			role="tooltip"
		>
			<div class="preview-header">
				<span
					class="preview-status-dot"
					style="background-color: {statusColors[previewScene.status] || 'var(--color-text-muted)'}"
				></span>
				<span class="preview-title">{previewScene.title}</span>
			</div>
			{#if previewScene.summary}
				<p class="preview-summary">{previewScene.summary}</p>
			{/if}
			<div class="preview-meta">
				{#if previewScene.pov}
					<span>POV: {previewScene.pov}</span>
				{/if}
				<span>{formatWordCount(countWords(previewScene.text))} words</span>
			</div>
		</div>
	{/if}
</div>

<style>
	.outline {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.outline-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-sm) var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
	}

	.outline-header h2 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
	}

	/* BB3: Status legend popover */
	.status-legend-trigger {
		position: relative;
	}

	.status-legend-popover {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: var(--spacing-xs);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-elevated);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-md);
		z-index: 100;
		min-width: 140px;
	}

	.legend-header {
		font-size: var(--font-size-xs);
		font-weight: 600;
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-xs);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.legend-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: 2px var(--spacing-xs);
		width: 100%;
		border-radius: var(--border-radius-sm);
		cursor: pointer;
		transition: background-color var(--transition-fast);
	}

	.legend-item:hover {
		background-color: var(--color-bg-hover);
	}

	/* CB5: Hidden status styling */
	.legend-item.hidden-status {
		opacity: 0.5;
	}

	.legend-item.hidden-status .legend-dot {
		opacity: 0.3;
	}

	.legend-clear {
		margin-top: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-accent);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
	}

	.legend-clear:hover {
		background-color: var(--color-accent-light);
	}

	.legend-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.legend-label {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.outline-filter {
		padding: var(--spacing-xs) var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.filter-input-wrapper {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-muted);
		transition: all var(--transition-fast);
	}

	.filter-input-wrapper:focus-within {
		border-color: var(--color-accent);
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-accent) 20%, transparent);
	}

	/* AG5: Highlight when filter has value */
	.filter-input-wrapper.has-value {
		background-color: color-mix(in srgb, var(--color-accent) 10%, var(--color-bg-primary));
		border-color: var(--color-accent);
	}

	.filter-input {
		flex: 1;
		border: none;
		background: none;
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		outline: none;
		min-width: 0;
	}

	.filter-input::placeholder {
		color: var(--color-text-muted);
	}

	.filter-clear {
		display: flex;
		align-items: center;
		padding: 2px;
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.filter-clear:hover {
		color: var(--color-text-primary);
		background-color: var(--color-bg-hover);
	}

	.filter-count {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		padding-left: var(--spacing-sm);
	}

	.outline-tree {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-sm) 0;
	}

	.chapter-group {
		margin-bottom: var(--spacing-xs);
	}

	.chapter-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-md);
		text-align: left;
		font-size: var(--font-size-sm);
		font-weight: 500;
		transition: background-color var(--transition-fast);
	}

	.chapter-item:hover {
		background-color: var(--color-bg-hover);
	}

	.chapter-item.selected {
		background-color: var(--color-accent-light);
	}

	.expand-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		height: 16px;
		flex-shrink: 0;
		color: var(--color-text-muted);
	}

	.expand-btn span {
		display: flex;
		transition: transform var(--transition-fast);
	}

	.expand-btn span.expanded {
		transform: rotate(90deg);
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.status-dot.small {
		width: 6px;
		height: 6px;
	}

	.health-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
		background-color: var(--color-warning);
	}

	.chapter-title,
	.scene-title {
		flex: 1;
		min-width: 0;
	}

	/* AP5: Flash confirmation when title is saved */
	.chapter-title.just-saved,
	.scene-title.just-saved {
		animation: save-flash 0.5s ease-out;
		border-radius: var(--border-radius-sm);
	}

	@keyframes save-flash {
		0% {
			background-color: var(--color-success-light, rgba(34, 197, 94, 0.3));
		}
		100% {
			background-color: transparent;
		}
	}

	.favorite-star {
		color: var(--color-warning);
		font-size: var(--font-size-xs);
		flex-shrink: 0;
	}

	/* AJ1: Recovery draft indicator */
	.recovery-badge {
		color: var(--color-warning);
		flex-shrink: 0;
		display: flex;
		align-items: center;
	}

	/* CB4: Scene count per chapter */
	.scene-count {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		flex-shrink: 0;
		padding: 0 var(--spacing-xs);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
	}

	.word-count {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	/* CB3: POV initial indicator */
	.pov-initial {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 16px;
		height: 16px;
		font-size: var(--font-size-xs);
		font-weight: 600;
		color: var(--color-accent);
		background-color: var(--color-accent-light);
		border-radius: 50%;
		flex-shrink: 0;
	}

	.add-scene-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: var(--border-radius-sm);
		color: var(--color-text-muted);
		opacity: 0.4;
		transition: all var(--transition-fast);
	}

	.chapter-item:hover .add-scene-btn {
		opacity: 1;
	}

	.add-scene-btn:hover {
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-primary);
	}

	.scenes-list {
		padding-left: calc(var(--spacing-md) + 16px);
	}

	.scene-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-md);
		text-align: left;
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		transition: background-color var(--transition-fast);
	}

	.scene-item:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.scene-item.selected {
		background-color: var(--color-accent-light);
		color: var(--color-text-primary);
	}

	/* AI5: Visual feedback when item is moved via keyboard */
	.scene-item.just-moved {
		animation: move-flash 0.6s ease-out;
	}

	@keyframes move-flash {
		0% {
			background-color: var(--color-accent-light);
			transform: scale(1.02);
		}
		100% {
			background-color: transparent;
			transform: scale(1);
		}
	}

	.empty-chapter {
		padding: var(--spacing-md);
		text-align: center;
	}

	.add-first-scene,
	.add-scene-inline {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.add-first-scene {
		color: var(--color-accent);
	}

	.add-scene-inline {
		display: block;
		width: 100%;
		text-align: left;
		padding: var(--spacing-xs) var(--spacing-md);
		opacity: 0.5;
	}

	.add-first-scene:hover,
	.add-scene-inline:hover {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
		opacity: 1;
	}

	/* AK3: Improved drag and drop visual feedback */
	.chapter-item.dragging,
	.scene-item.dragging {
		opacity: 0.5;
		border: 1px dashed var(--color-accent);
	}

	.chapter-item.drop-target,
	.scene-item.drop-target {
		background-color: var(--color-accent-light);
		position: relative;
	}

	/* AK3: Drop indicator line */
	.scene-item.drop-target::before {
		content: '';
		position: absolute;
		top: -2px;
		left: 0;
		right: 0;
		height: 3px;
		background-color: var(--color-accent);
		border-radius: 2px;
	}

	.empty-chapter.drop-target,
	.drop-zone.drop-target {
		background-color: var(--color-accent-light);
		border: 2px dashed var(--color-accent);
	}

	.drop-zone {
		height: 4px;
		margin: 0 var(--spacing-md);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.drop-zone.drop-target {
		height: 24px;
		margin: var(--spacing-xs) var(--spacing-md);
	}

	[draggable='true'] {
		cursor: grab;
	}

	[draggable='true']:active {
		cursor: grabbing;
	}

	/* Inline rename */
	.inline-rename {
		flex: 1;
		min-width: 0;
		padding: 0 var(--spacing-xs);
		font-size: var(--font-size-sm);
		font-weight: 500;
		border: 1px solid var(--color-accent);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		outline: none;
	}

	/* Chapter details dialog */
	.chapter-details-overlay {
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

	.chapter-details-dialog {
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-lg);
		width: 400px;
		max-width: 90%;
	}

	.chapter-details-dialog h3 {
		font-size: var(--font-size-lg);
		margin: 0 0 var(--spacing-md) 0;
		text-transform: none;
		letter-spacing: normal;
		color: var(--color-text-primary);
	}

	.chapter-details-form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.form-label {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-secondary);
	}

	.form-label textarea,
	.form-label select {
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-sm);
		font-family: inherit;
		resize: vertical;
	}

	.chapter-details-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
	}

	.scene-item.drag-disabled,
	.chapter-item.drag-disabled {
		cursor: default;
	}

	.scene-item.drag-disabled:active {
		cursor: default;
	}

	/* CB2: Scene preview popover */
	.scene-preview-popover {
		position: fixed;
		z-index: 1000;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-sm);
		width: 250px;
		max-width: 90vw;
		pointer-events: none;
	}

	.preview-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		margin-bottom: var(--spacing-xs);
	}

	.preview-status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.preview-title {
		font-weight: 500;
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.preview-summary {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		margin: 0 0 var(--spacing-xs);
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	.preview-meta {
		display: flex;
		gap: var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	/* UC2: Compact view mode */
	.outline-tree.compact .chapter-item {
		padding: 2px var(--spacing-md);
	}

	.outline-tree.compact .scene-item {
		padding: 2px var(--spacing-md);
	}

	.outline-tree.compact .word-count,
	.outline-tree.compact .pov-initial,
	.outline-tree.compact .health-dot {
		display: none;
	}

	.outline-tree.compact .scenes-list {
		padding-left: calc(var(--spacing-sm) + 16px);
	}

	.outline-tree.compact .add-scene-inline {
		padding: 2px var(--spacing-md);
		font-size: var(--font-size-xs);
	}

	/* UC3: Enhanced drag and drop feedback */
	.chapter-item.dragging,
	.scene-item.dragging {
		opacity: 0.5;
		transform: scale(0.98);
		box-shadow: inset 0 0 0 2px var(--color-accent);
	}

	.chapter-item.drop-target,
	.scene-item.drop-target {
		background-color: var(--color-accent-light);
		box-shadow: inset 0 0 0 2px var(--color-accent);
	}

	/* UC3: Drop indicator line */
	.chapter-item.drop-target::after,
	.scene-item.drop-target::after {
		content: '';
		position: absolute;
		left: var(--spacing-md);
		right: var(--spacing-md);
		bottom: -1px;
		height: 2px;
		background-color: var(--color-accent);
		border-radius: 1px;
	}

	.drop-zone.drop-target {
		height: 8px;
		background-color: var(--color-accent-light);
		border: 2px dashed var(--color-accent);
		margin-top: var(--spacing-xs);
	}
</style>
