<!--
  Card-based scene organization view (corkboard).

  Features:
  - Visual scene cards showing title, summary, status, word count
  - Drag-and-drop reordering
  - Filtering by status, POV, arc, tags, bible entries
  - Bulk selection and operations (status change, deletion)
  - Grouping by chapter, status, or POV
  - Double-click to open scene in editor
  - Automatic snapshots before bulk operations
-->
<script lang="ts">
	import { SvelteMap, SvelteSet } from 'svelte/reactivity';

	import type { Arc, Scene } from '$lib/api';
	import type { SavedFilter } from '$lib/api';
	import { arcApi, savedFilterApi, sceneApi, snapshotApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError, showSuccess } from '$lib/toast';
	import { countWords, formatWordCount, sceneStatuses, statusColors } from '$lib/utils';

	// Create automatic snapshot before bulk operations
	async function createPreBulkSnapshot(operation: string) {
		try {
			await snapshotApi.create(
				`Pre-bulk: ${operation}`,
				`Automatic snapshot before bulk ${operation} on ${selectedSceneIds.size} scenes`,
				'pre_bulk'
			);
		} catch (e) {
			console.warn('Failed to create pre-bulk snapshot:', e);
			// Don't block the operation if snapshot fails
		}
	}

	type GroupBy = 'chapter' | 'status' | 'pov' | 'arc';
	let groupBy = $state<GroupBy>('chapter');
	let showFilters = $state(false);

	// Filter state
	let filterStatus = $state('');
	let filterPov = $state('');
	let filterTag = $state('');
	let filterArc = $state('');

	// Arc data for filtering and display
	let allArcs = $state<Arc[]>([]);
	let sceneArcMap = new SvelteMap<string, Arc[]>();

	$effect(() => {
		if (appState.project) {
			loadArcs();
		}
	});

	async function loadArcs() {
		try {
			allArcs = await arcApi.getAll();
			// Load arcs for all scenes
			sceneArcMap.clear();
			for (const { scene } of getAllScenes(appState.chapters, appState.scenes)) {
				try {
					const arcs = await arcApi.getSceneArcs(scene.id);
					if (arcs.length > 0) {
						sceneArcMap.set(scene.id, arcs);
					}
				} catch {
					// Ignore individual failures
				}
			}
		} catch {
			// Arcs are optional
		}
	}

	// Multi-select state
	let selectedSceneIds = new SvelteSet<string>();
	let isMultiSelectMode = $state(false);

	// Drag and drop state
	let draggedSceneId = $state<string | null>(null);
	let dragOverSceneId = $state<string | null>(null);

	let currentFilters = $derived({
		status: filterStatus,
		pov: filterPov,
		tag: filterTag,
		arc: filterArc,
	});

	let allScenes = $derived(getAllScenes(appState.chapters, appState.scenes));
	let filteredScenes = $derived(applyFilters(allScenes, currentFilters));
	let groupedScenes = $derived(getGroupedScenes(filteredScenes, groupBy));

	// Get unique POVs
	let uniquePovs = $derived([
		...new Set(allScenes.map((s) => s.scene.pov).filter(Boolean) as string[]),
	]);

	// Get unique tags
	let uniqueTags = $derived([
		...new Set(
			allScenes.flatMap(
				(s) =>
					s.scene.tags
						?.split(',')
						.map((t) => t.trim())
						.filter(Boolean) || []
			)
		),
	]);

	function getAllScenes(
		chaptersList: typeof appState.chapters,
		scenesMap: typeof appState.scenes
	): Array<{ scene: Scene; chapterId: string; chapterTitle: string }> {
		const result: Array<{ scene: Scene; chapterId: string; chapterTitle: string }> = [];
		for (const chapter of chaptersList) {
			const chapterScenes = scenesMap.get(chapter.id) || [];
			for (const scene of chapterScenes) {
				result.push({ scene, chapterId: chapter.id, chapterTitle: chapter.title });
			}
		}
		return result;
	}

	function applyFilters(
		scenesList: Array<{ scene: Scene; chapterId: string; chapterTitle: string }>,
		filters: typeof currentFilters
	): Array<{ scene: Scene; chapterId: string; chapterTitle: string }> {
		return scenesList.filter(({ scene }) => {
			if (filters.status && scene.status !== filters.status) return false;
			if (filters.pov && scene.pov !== filters.pov) return false;
			if (
				filters.tag &&
				!scene.tags
					?.split(',')
					.map((t) => t.trim())
					.includes(filters.tag)
			)
				return false;
			if (filters.arc) {
				const arcs = sceneArcMap.get(scene.id);
				if (!arcs || !arcs.some((a) => a.id === filters.arc)) return false;
			}
			return true;
		});
	}

	function getGroupedScenes(
		scenesList: Array<{ scene: Scene; chapterId: string; chapterTitle: string }>,
		group: GroupBy
	) {
		const groups = new SvelteMap<
			string,
			{ title: string; scenes: Array<{ scene: Scene; chapterId: string }> }
		>();

		for (const { scene, chapterId, chapterTitle } of scenesList) {
			let groupKey: string;
			let groupTitle: string;

			switch (group) {
				case 'chapter':
					groupKey = chapterId;
					groupTitle = chapterTitle;
					break;
				case 'status':
					groupKey = scene.status;
					groupTitle = sceneStatuses.find((s) => s.value === scene.status)?.label || scene.status;
					break;
				case 'pov':
					groupKey = scene.pov || 'No POV';
					groupTitle = scene.pov || 'No POV';
					break;
				case 'arc': {
					const arcs = sceneArcMap.get(scene.id);
					if (arcs && arcs.length > 0) {
						// Scene can appear in multiple arc groups
						for (const arc of arcs) {
							const key = arc.id;
							if (!groups.has(key)) {
								groups.set(key, { title: arc.name, scenes: [] });
							}
							groups.get(key)!.scenes.push({ scene, chapterId });
						}
						continue;
					}
					groupKey = '__no_arc__';
					groupTitle = 'No Arc';
					break;
				}
			}

			if (!groups.has(groupKey)) {
				groups.set(groupKey, { title: groupTitle, scenes: [] });
			}
			groups.get(groupKey)!.scenes.push({ scene, chapterId });
		}

		return groups;
	}

	function clearFilters() {
		filterStatus = '';
		filterPov = '';
		filterTag = '';
		filterArc = '';
	}

	let hasActiveFilters = $derived(filterStatus || filterPov || filterTag || filterArc);

	// Saved filters
	let savedFilters = $state<SavedFilter[]>([]);
	let showSaveFilterInput = $state(false);
	let newFilterName = $state('');

	$effect(() => {
		if (appState.project) {
			loadSavedFilters();
		}
	});

	async function loadSavedFilters() {
		try {
			savedFilters = await savedFilterApi.getAll('corkboard');
		} catch {
			// Non-critical
		}
	}

	async function saveCurrentFilter() {
		if (!newFilterName.trim() || !hasActiveFilters) return;
		try {
			const filterData = JSON.stringify({
				status: filterStatus,
				pov: filterPov,
				tag: filterTag,
				arc: filterArc,
			});
			await savedFilterApi.create({
				name: newFilterName.trim(),
				filter_type: 'corkboard',
				filter_data: filterData,
			});
			await loadSavedFilters();
			showSaveFilterInput = false;
			newFilterName = '';
			showSuccess('Filter saved');
		} catch {
			showError('Failed to save filter');
		}
	}

	function applySavedFilter(filter: SavedFilter) {
		try {
			const data = JSON.parse(filter.filter_data);
			filterStatus = data.status || '';
			filterPov = data.pov || '';
			filterTag = data.tag || '';
			filterArc = data.arc || '';
		} catch {
			showError('Invalid filter data');
		}
	}

	async function deleteSavedFilter(id: string) {
		try {
			await savedFilterApi.delete(id);
			await loadSavedFilters();
			showSuccess('Filter deleted');
		} catch {
			showError('Failed to delete filter');
		}
	}

	// Clear selection when filters change
	$effect(() => {
		if (currentFilters) {
			selectedSceneIds.clear();
		}
	});

	function selectScene(sceneId: string, chapterId: string, event: MouseEvent) {
		// Handle multi-select with Ctrl/Cmd or Shift
		if (event.ctrlKey || event.metaKey) {
			isMultiSelectMode = true;
			if (selectedSceneIds.has(sceneId)) {
				selectedSceneIds.delete(sceneId);
			} else {
				selectedSceneIds.add(sceneId);
			}
			return;
		}

		// Regular click - deselect all and go to editor
		if (!isMultiSelectMode || selectedSceneIds.size === 0) {
			selectedSceneIds.clear();
			isMultiSelectMode = false;
			appState.selectScene(sceneId, chapterId);
			appState.setViewMode('editor');
		} else {
			// In multi-select mode, toggle selection
			if (selectedSceneIds.has(sceneId)) {
				selectedSceneIds.delete(sceneId);
			} else {
				selectedSceneIds.add(sceneId);
			}
		}
	}

	function toggleMultiSelect() {
		isMultiSelectMode = !isMultiSelectMode;
		if (!isMultiSelectMode) {
			selectedSceneIds.clear();
		}
	}

	function selectAll() {
		selectedSceneIds.clear();
		for (const s of filteredScenes) {
			selectedSceneIds.add(s.scene.id);
		}
		isMultiSelectMode = true;
	}

	function deselectAll() {
		selectedSceneIds.clear();
		isMultiSelectMode = false;
	}

	// Bulk operations
	async function bulkUpdateStatus(newStatus: string) {
		if (selectedSceneIds.size === 0) return;
		try {
			// Create snapshot before bulk operation
			await createPreBulkSnapshot('status update');

			for (const sceneId of selectedSceneIds) {
				await sceneApi.update(sceneId, { status: newStatus });
			}
			await appState.loadChapters();
			showSuccess(`Updated ${selectedSceneIds.size} scenes to "${newStatus}"`);
			deselectAll();
		} catch (_e) {
			showError('Failed to update scenes');
		}
	}

	async function bulkDelete() {
		if (selectedSceneIds.size === 0) return;
		if (!confirm(`Delete ${selectedSceneIds.size} scenes? They will be moved to trash.`)) return;
		try {
			// Create snapshot before bulk delete
			await createPreBulkSnapshot('delete');

			for (const sceneId of selectedSceneIds) {
				await sceneApi.delete(sceneId);
			}
			await appState.loadChapters();
			showSuccess(`Moved ${selectedSceneIds.size} scenes to trash`);
			deselectAll();
		} catch (_e) {
			showError('Failed to delete scenes');
		}
	}

	// Drag and drop handlers
	function handleDragStart(event: DragEvent, sceneId: string) {
		if (groupBy !== 'chapter') return; // Only allow reordering when grouped by chapter
		draggedSceneId = sceneId;
		if (event.dataTransfer) {
			event.dataTransfer.effectAllowed = 'move';
			event.dataTransfer.setData('text/plain', sceneId);
		}
	}

	function handleDragEnd() {
		draggedSceneId = null;
		dragOverSceneId = null;
	}

	function handleDragOver(event: DragEvent, sceneId: string, _chapterId: string) {
		if (groupBy !== 'chapter' || !draggedSceneId || draggedSceneId === sceneId) return;
		event.preventDefault();
		dragOverSceneId = sceneId;
	}

	function handleDragLeave() {
		dragOverSceneId = null;
	}

	async function handleDrop(event: DragEvent, targetSceneId: string, targetChapterId: string) {
		event.preventDefault();
		if (groupBy !== 'chapter' || !draggedSceneId || draggedSceneId === targetSceneId) {
			handleDragEnd();
			return;
		}

		try {
			// Find source scene and its chapter
			const sourceInfo = allScenes.find((s) => s.scene.id === draggedSceneId);
			const targetInfo = allScenes.find((s) => s.scene.id === targetSceneId);
			if (!sourceInfo || !targetInfo) return;

			const sourceChapterId = sourceInfo.chapterId;
			const targetScene = targetInfo.scene;

			// Update the scene's position
			if (sourceChapterId === targetChapterId) {
				// Same chapter - reorder
				await sceneApi.update(draggedSceneId, { position: targetScene.position });
			} else {
				// Different chapter - move to new chapter
				await sceneApi.moveToChapter(draggedSceneId, targetChapterId, targetScene.position);
			}

			await appState.loadChapters();
			showSuccess('Scene moved');
		} catch (_e) {
			showError('Failed to move scene');
		}

		handleDragEnd();
	}

	function getStatusColor(status: string): string {
		return statusColors[status] || 'var(--color-text-muted)';
	}
</script>

<div class="corkboard">
	<div class="corkboard-header">
		<div class="header-left">
			<h2>Corkboard</h2>
			<span class="scene-count">{filteredScenes.length} scenes</span>
			{#if selectedSceneIds.size > 0}
				<span class="selection-count">({selectedSceneIds.size} selected)</span>
			{/if}
		</div>
		<div class="header-right">
			<!-- Multi-select toggle -->
			<button
				class="multi-select-toggle"
				class:active={isMultiSelectMode}
				onclick={toggleMultiSelect}
				title="Toggle multi-select mode (Ctrl/Cmd+click to select multiple)"
			>
				<svg
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<rect x="3" y="3" width="7" height="7" />
					<rect x="14" y="3" width="7" height="7" />
					<rect x="14" y="14" width="7" height="7" />
					<rect x="3" y="14" width="7" height="7" />
				</svg>
				Multi-select
			</button>

			{#if isMultiSelectMode}
				<button class="select-all-btn" onclick={selectAll}>Select All</button>
				<button class="deselect-btn" onclick={deselectAll}>Deselect</button>
			{/if}

			<button
				class="filter-toggle"
				class:active={showFilters || hasActiveFilters}
				onclick={() => (showFilters = !showFilters)}
			>
				<svg
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
				</svg>
				Filters
				{#if hasActiveFilters}
					<span class="filter-badge"></span>
				{/if}
			</button>
			<div class="group-selector">
				<span class="label">Group by:</span>
				<select bind:value={groupBy}>
					<option value="chapter">Chapter</option>
					<option value="status">Status</option>
					<option value="pov">POV</option>
					<option value="arc">Arc</option>
				</select>
			</div>
		</div>
	</div>

	<!-- Bulk actions bar -->
	{#if selectedSceneIds.size > 0}
		<div class="bulk-actions-bar">
			<span class="bulk-label">Bulk actions:</span>
			<div class="bulk-status-select">
				<label for="bulk-status-select">Set status:</label>
				<select id="bulk-status-select" onchange={(e) => bulkUpdateStatus(e.currentTarget.value)}>
					<option value="">Choose...</option>
					{#each sceneStatuses as status (status.value)}
						<option value={status.value}>{status.label}</option>
					{/each}
				</select>
			</div>
			<button class="bulk-delete-btn" onclick={bulkDelete}>
				<svg
					width="14"
					height="14"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<polyline points="3 6 5 6 21 6" />
					<path
						d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
					/>
				</svg>
				Delete Selected
			</button>
		</div>
	{/if}

	{#if showFilters}
		<div class="filters-bar">
			<div class="filter-controls">
				<div class="filter-group">
					<label for="filter-status">Status</label>
					<select id="filter-status" bind:value={filterStatus}>
						<option value="">All</option>
						{#each sceneStatuses as status (status.value)}
							<option value={status.value}>{status.label}</option>
						{/each}
					</select>
				</div>
				<div class="filter-group">
					<label for="filter-pov">POV</label>
					<select id="filter-pov" bind:value={filterPov}>
						<option value="">All</option>
						{#each uniquePovs as pov (pov)}
							<option value={pov}>{pov}</option>
						{/each}
					</select>
				</div>
				<div class="filter-group">
					<label for="filter-tag">Tag</label>
					<select id="filter-tag" bind:value={filterTag}>
						<option value="">All</option>
						{#each uniqueTags as tag (tag)}
							<option value={tag}>{tag}</option>
						{/each}
					</select>
				</div>
				{#if allArcs.length > 0}
					<div class="filter-group">
						<label for="filter-arc">Arc</label>
						<select id="filter-arc" bind:value={filterArc}>
							<option value="">All</option>
							{#each allArcs as arc (arc.id)}
								<option value={arc.id}>{arc.name}</option>
							{/each}
						</select>
					</div>
				{/if}
				{#if hasActiveFilters}
					<button class="clear-filters-btn" onclick={clearFilters}>Clear</button>
					<button
						class="save-filter-btn"
						onclick={() => (showSaveFilterInput = !showSaveFilterInput)}
						title="Save current filter"
					>
						<svg
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
							<polyline points="17 21 17 13 7 13 7 21" />
							<polyline points="7 3 7 8 15 8" />
						</svg>
						Save
					</button>
				{/if}
			</div>

			{#if showSaveFilterInput}
				<div class="save-filter-row">
					<input
						type="text"
						class="save-filter-input"
						placeholder="Filter name..."
						bind:value={newFilterName}
						onkeydown={(e) => {
							if (e.key === 'Enter') saveCurrentFilter();
							if (e.key === 'Escape') {
								showSaveFilterInput = false;
								newFilterName = '';
							}
						}}
					/>
					<button
						class="save-filter-confirm"
						onclick={saveCurrentFilter}
						disabled={!newFilterName.trim()}
					>
						Save
					</button>
					<button
						class="save-filter-cancel"
						onclick={() => {
							showSaveFilterInput = false;
							newFilterName = '';
						}}
					>
						Cancel
					</button>
				</div>
			{/if}

			{#if savedFilters.length > 0}
				<div class="saved-filters-row">
					<span class="saved-filters-label">Saved:</span>
					{#each savedFilters as filter (filter.id)}
						<span class="saved-filter-chip">
							<button
								class="saved-filter-name"
								onclick={() => applySavedFilter(filter)}
								title="Apply filter: {filter.name}"
							>
								{filter.name}
							</button>
							<button
								class="saved-filter-delete"
								onclick={() => deleteSavedFilter(filter.id)}
								title="Delete filter"
							>
								<svg
									width="10"
									height="10"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<line x1="18" y1="6" x2="6" y2="18" />
									<line x1="6" y1="6" x2="18" y2="18" />
								</svg>
							</button>
						</span>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<div class="corkboard-content">
		{#each [...groupedScenes.entries()] as [groupKey, group] (groupKey)}
			<div class="scene-group">
				<h3 class="group-title">{group.title}</h3>
				<div class="cards-grid">
					{#each group.scenes as { scene, chapterId } (scene.id)}
						{@const health = appState.sceneHealthMap.get(scene.id)}
						<button
							class="scene-card"
							class:selected={appState.selectedSceneId === scene.id}
							class:multi-selected={selectedSceneIds.has(scene.id)}
							class:dragging={draggedSceneId === scene.id}
							class:drag-over={dragOverSceneId === scene.id}
							onclick={(e) => selectScene(scene.id, chapterId, e)}
							style="--status-color: {getStatusColor(scene.status)}"
							draggable={groupBy === 'chapter'}
							ondragstart={(e) => handleDragStart(e, scene.id)}
							ondragend={handleDragEnd}
							ondragover={(e) => handleDragOver(e, scene.id, chapterId)}
							ondragleave={handleDragLeave}
							ondrop={(e) => handleDrop(e, scene.id, chapterId)}
						>
							{#if isMultiSelectMode}
								<div class="select-checkbox" class:checked={selectedSceneIds.has(scene.id)}>
									{#if selectedSceneIds.has(scene.id)}
										<svg
											width="12"
											height="12"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="3"
										>
											<polyline points="20 6 9 17 4 12" />
										</svg>
									{/if}
								</div>
							{/if}

							<div class="card-header">
								<span class="card-title truncate">{scene.title}</span>
								{#if health}
									<span
										class="health-badge"
										class:health-good={health.score >= 0.8}
										class:health-warning={health.score >= 0.5 && health.score < 0.8}
										class:health-bad={health.score < 0.5}
										title="Health: {Math.round(health.score * 100)}%"
										>{Math.round(health.score * 100)}%</span
									>
								{/if}
								<span class="status-badge">{scene.status}</span>
							</div>

							{#if scene.summary}
								<p class="card-summary">{scene.summary}</p>
							{/if}

							<div class="card-footer">
								{#if scene.pov}
									<span class="pov-tag">{scene.pov}</span>
								{/if}
								{#if sceneArcMap.get(scene.id)?.length}
									<span class="arc-badges">
										{#each sceneArcMap.get(scene.id) ?? [] as arc (arc.id)}
											<span
												class="arc-badge"
												style="background-color: {arc.color || 'var(--color-text-muted)'}"
												title={arc.name}
											></span>
										{/each}
									</span>
								{/if}
								<span class="word-count">{formatWordCount(countWords(scene.text))} words</span>
							</div>

							{#if groupBy === 'chapter'}
								<div class="drag-handle" title="Drag to reorder">
									<svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor">
										<circle cx="9" cy="5" r="2" />
										<circle cx="15" cy="5" r="2" />
										<circle cx="9" cy="12" r="2" />
										<circle cx="15" cy="12" r="2" />
										<circle cx="9" cy="19" r="2" />
										<circle cx="15" cy="19" r="2" />
									</svg>
								</div>
							{/if}
						</button>
					{/each}
				</div>
			</div>
		{:else}
			<div class="empty-corkboard">
				<p>No scenes to display</p>
				<p class="hint">Create chapters and scenes in the outline to see them here.</p>
			</div>
		{/each}
	</div>
</div>

<style>
	.corkboard {
		display: flex;
		flex-direction: column;
		height: 100%;
		background-color: var(--color-bg-secondary);
	}

	.corkboard-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		background-color: var(--color-bg-primary);
		border-bottom: 1px solid var(--color-border);
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
	}

	.corkboard-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.scene-count {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.selection-count {
		font-size: var(--font-size-sm);
		color: var(--color-accent);
		font-weight: 500;
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
	}

	.filter-toggle {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
		position: relative;
	}

	.filter-toggle:hover {
		background-color: var(--color-bg-hover);
	}

	.filter-toggle.active {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}

	.multi-select-toggle {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
	}

	.multi-select-toggle:hover {
		background-color: var(--color-bg-hover);
	}

	.multi-select-toggle.active {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}

	.select-all-btn,
	.deselect-btn {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
	}

	.select-all-btn:hover,
	.deselect-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.bulk-actions-bar {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		padding: var(--spacing-sm) var(--spacing-lg);
		background-color: var(--color-accent-light);
		border-bottom: 1px solid var(--color-border);
	}

	.bulk-label {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-accent);
	}

	.bulk-status-select {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
	}

	.bulk-status-select label {
		color: var(--color-text-secondary);
	}

	.bulk-status-select select {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.bulk-delete-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-error);
		border-radius: var(--border-radius-sm);
		margin-left: auto;
	}

	.bulk-delete-btn:hover {
		background-color: var(--danger-subtle);
	}

	.filter-badge {
		position: absolute;
		top: -2px;
		right: -2px;
		width: 8px;
		height: 8px;
		background-color: var(--color-accent);
		border-radius: 50%;
	}

	.filters-bar {
		display: flex;
		gap: var(--spacing-lg);
		padding: var(--spacing-md) var(--spacing-lg);
		background-color: var(--color-bg-primary);
		border-bottom: 1px solid var(--color-border);
	}

	.filter-controls {
		display: flex;
		align-items: flex-end;
		gap: var(--spacing-md);
		flex-wrap: wrap;
	}

	.filter-group {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.filter-group label {
		font-size: var(--font-size-xs);
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.filter-group select {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		min-width: 120px;
	}

	.clear-filters-btn {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
	}

	.clear-filters-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.group-selector {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.group-selector .label {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.group-selector select {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		background-color: var(--color-bg-secondary);
	}

	.corkboard-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
	}

	.scene-group {
		margin-bottom: var(--spacing-xl);
	}

	.group-title {
		font-size: var(--font-size-sm);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-md);
	}

	.cards-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
		gap: var(--spacing-md);
	}

	.scene-card {
		position: relative;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-left: 4px solid var(--status-color);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
		text-align: left;
		transition: all var(--transition-fast);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		min-height: 120px;
	}

	.scene-card:hover {
		border-color: var(--color-accent);
		box-shadow: var(--shadow-md);
		transform: translateY(-2px);
	}

	.scene-card.selected {
		border-color: var(--color-accent);
		background-color: var(--color-accent-light);
	}

	.scene-card.multi-selected {
		border-color: var(--color-accent);
		background-color: var(--color-accent-light);
		box-shadow: 0 0 0 2px var(--color-accent);
	}

	.scene-card.dragging {
		opacity: 0.5;
		transform: scale(0.95);
	}

	.scene-card.drag-over {
		border-color: var(--color-accent);
		border-style: dashed;
		background-color: var(--color-accent-light);
	}

	.scene-card[draggable='true'] {
		cursor: grab;
	}

	.scene-card[draggable='true']:active {
		cursor: grabbing;
	}

	.select-checkbox {
		position: absolute;
		top: var(--spacing-sm);
		left: var(--spacing-sm);
		width: 18px;
		height: 18px;
		border: 2px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all var(--transition-fast);
	}

	.select-checkbox.checked {
		background-color: var(--color-accent);
		border-color: var(--color-accent);
		color: var(--text-on-accent);
	}

	.drag-handle {
		position: absolute;
		bottom: var(--spacing-sm);
		right: var(--spacing-sm);
		color: var(--color-text-muted);
		opacity: 0;
		transition: opacity var(--transition-fast);
	}

	.scene-card:hover .drag-handle {
		opacity: 0.5;
	}

	.scene-card:hover .drag-handle:hover {
		opacity: 1;
	}

	.card-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--spacing-sm);
	}

	.card-title {
		font-weight: 500;
		color: var(--color-text-primary);
		flex: 1;
		min-width: 0;
	}

	.status-badge {
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-xs);
		background-color: var(--status-color);
		color: var(--text-on-accent);
		border-radius: var(--border-radius-sm);
		white-space: nowrap;
	}

	.health-badge {
		font-size: 10px;
		font-weight: 700;
		padding: 1px 4px;
		border-radius: var(--border-radius-sm);
		white-space: nowrap;
	}

	.health-badge.health-good {
		color: var(--color-success, #22c55e);
		background-color: color-mix(in srgb, var(--color-success, #22c55e) 15%, transparent);
	}

	.health-badge.health-warning {
		color: var(--color-warning);
		background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
	}

	.health-badge.health-bad {
		color: var(--color-error);
		background-color: color-mix(in srgb, var(--color-error) 15%, transparent);
	}

	.card-summary {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		line-height: var(--line-height-tight);
		flex: 1;
		overflow: hidden;
		display: -webkit-box;
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
	}

	.card-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--spacing-sm);
		margin-top: auto;
	}

	.pov-tag {
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-xs);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
	}

	.arc-badges {
		display: flex;
		gap: 3px;
		align-items: center;
	}

	.arc-badge {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.word-count {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-left: auto;
	}

	.empty-corkboard {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-text-muted);
		text-align: center;
	}

	.empty-corkboard .hint {
		font-size: var(--font-size-sm);
		margin-top: var(--spacing-sm);
	}

	.save-filter-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
	}

	.save-filter-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-accent);
	}

	.save-filter-row {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
		background-color: var(--color-accent-light);
	}

	.save-filter-input {
		flex: 1;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
	}

	.save-filter-confirm {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--text-on-accent);
		background-color: var(--color-accent);
		border-radius: var(--border-radius-sm);
	}

	.save-filter-confirm:disabled {
		opacity: 0.5;
	}

	.save-filter-cancel {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
	}

	.save-filter-cancel:hover {
		background-color: var(--color-bg-hover);
	}

	.saved-filters-row {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
		flex-wrap: wrap;
	}

	.saved-filters-label {
		font-size: var(--font-size-xs);
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.saved-filter-chip {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		overflow: hidden;
	}

	.saved-filter-name {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
	}

	.saved-filter-name:hover {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}

	.saved-filter-delete {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		display: flex;
		align-items: center;
	}

	.saved-filter-delete:hover {
		background-color: var(--danger-subtle);
		color: var(--color-error);
	}
</style>
