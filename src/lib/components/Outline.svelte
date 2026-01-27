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
	import { SvelteSet } from 'svelte/reactivity';

	import { chapterApi, sceneApi, trashApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';
	import { countWords, formatWordCount, statusColors } from '$lib/utils';

	import ImpactDialog from './ImpactDialog.svelte';
	import { Button, Icon } from './ui';
	import ContextMenu from './ui/ContextMenu.svelte';
	import ContextMenuItem from './ui/ContextMenuItem.svelte';
	import ContextMenuSeparator from './ui/ContextMenuSeparator.svelte';

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

	let expandedChapters = new SvelteSet<string>();

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

	$effect(() => {
		if (chapters.length > 0 && expandedChapters.size === 0) {
			// Auto-expand first chapter
			expandedChapters.add(chapters[0].id);
		}
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
		await appState.createScene(chapterId, title);
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
		try {
			if (impactDialog.entityType === 'scene') {
				await appState.deleteScene(impactDialog.entityId);
			} else if (impactDialog.entityType === 'chapter') {
				await appState.deleteChapter(impactDialog.entityId);
			}
		} catch (e) {
			console.error(`Failed to delete ${impactDialog.entityType}:`, e);
			showError(`Failed to delete ${impactDialog.entityType}`);
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
		try {
			await chapterApi.update(editingChapterId, { title: editingChapterTitle.trim() });
			await appState.loadChapters();
		} catch (e) {
			console.error('Failed to rename chapter:', e);
			showError('Failed to rename chapter');
		}
		editingChapterId = null;
	}

	function cancelRenamingChapter() {
		editingChapterId = null;
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

	// Context menu close is handled by the ContextMenu component
</script>

<div class="outline">
	<div class="outline-header">
		<h2>Manuscript</h2>
		<Button variant="icon" onclick={handleAddChapter} title="Add Chapter">
			<Icon name="plus" size={16} />
		</Button>
	</div>

	<div class="outline-tree">
		{#each chapters as chapter (chapter.id)}
			{@const chapterScenes = scenes.get(chapter.id) || []}
			{@const isExpanded = expandedChapters.has(chapter.id)}
			{@const isSelected = appState.selectedChapterId === chapter.id && !appState.selectedSceneId}
			{@const wordCount = getChapterWordCount(chapter.id)}

			<div class="chapter-group">
				<div
					class="chapter-item"
					class:selected={isSelected}
					class:drop-target={isDropTarget('chapter', chapter.id)}
					class:dragging={draggedItem?.type === 'chapter' && draggedItem?.id === chapter.id}
					role="button"
					tabindex="0"
					draggable="true"
					onclick={() => selectChapter(chapter.id)}
					onkeydown={(e) => e.key === 'Enter' && selectChapter(chapter.id)}
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
						style="background-color: {statusColors[chapter.status] || 'var(--color-text-muted)'}"
					></span>

					{#if editingChapterId === chapter.id && !showChapterDetails}
						<input
							type="text"
							class="inline-rename"
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
						<span class="chapter-title truncate">{chapter.title}</span>
					{/if}

					<span class="word-count">{formatWordCount(wordCount)}</span>

					<button
						class="add-scene-btn"
						onclick={(e) => {
							e.stopPropagation();
							handleAddScene(chapter.id);
						}}
						title="Add Scene"
					>
						<Icon name="plus" size={12} />
					</button>
				</div>

				{#if isExpanded}
					<div class="scenes-list">
						{#each chapterScenes as scene (scene.id)}
							{@const isSceneSelected = appState.selectedSceneId === scene.id}
							{@const health = appState.sceneHealthMap.get(scene.id)}
							<button
								class="scene-item"
								class:selected={isSceneSelected}
								class:drop-target={isDropTarget('scene', scene.id)}
								class:dragging={draggedItem?.type === 'scene' && draggedItem?.id === scene.id}
								draggable="true"
								onclick={() => selectScene(scene.id, chapter.id)}
								oncontextmenu={(e) => handleContextMenu(e, scene.id, chapter.id)}
								ondragstart={(e) => handleDragStart(e, 'scene', scene.id, chapter.id)}
								ondragend={handleDragEnd}
								ondragover={(e) => handleDragOver(e, 'scene', scene.id)}
								ondragleave={handleDragLeave}
								ondrop={(e) => handleDrop(e, 'scene', scene.id, chapter.id)}
							>
								<span
									class="status-dot small"
									style="background-color: {statusColors[scene.status] ||
										'var(--color-text-muted)'}"
								></span>
								<span class="scene-title truncate">{scene.title}</span>
								{#if health && health.score < 1.0}
									<span
										class="health-dot"
										title={health.checks
											.filter((c) => !c.passed)
											.map((c) => c.label)
											.join('\n')}
									></span>
								{/if}
								<span class="word-count">{formatWordCount(countWords(scene.text))}</span>
							</button>
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
						{/if}
					</div>
				{/if}
			</div>
		{/each}

		{#if chapters.length === 0}
			<div class="empty-outline">
				<p>No chapters yet</p>
				<button class="add-first-chapter" onclick={handleAddChapter}> + Add first chapter </button>
			</div>
		{/if}
	</div>

	{#if contextMenu}
		<ContextMenu x={contextMenu.x} y={contextMenu.y} onclose={closeContextMenu}>
			{#if contextMenu.menuType === 'chapter'}
				<ContextMenuItem
					label="Rename"
					onclick={() => {
						startRenamingChapter(contextMenu!.chapterId);
						closeContextMenu();
					}}
				/>
				<ContextMenuItem
					label="Edit Details…"
					onclick={() => {
						openChapterDetails(contextMenu!.chapterId);
						closeContextMenu();
					}}
				/>
				<ContextMenuSeparator />
				<ContextMenuItem
					label="Delete"
					danger
					onclick={() => {
						handleDeleteChapter(contextMenu!.chapterId);
						closeContextMenu();
					}}
				/>
			{:else}
				<ContextMenuItem
					label="Duplicate"
					onclick={() => {
						handleDuplicateScene(contextMenu!.sceneId, false);
						closeContextMenu();
					}}
				/>
				<ContextMenuItem
					label="Duplicate (structure only)"
					onclick={() => {
						handleDuplicateScene(contextMenu!.sceneId, true);
						closeContextMenu();
					}}
				/>
				<ContextMenuSeparator />
				<ContextMenuItem
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
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<div class="chapter-details-overlay" onclick={cancelChapterDetails} role="presentation">
			<div
				class="chapter-details-dialog"
				onclick={(e) => e.stopPropagation()}
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

	.word-count {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
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
		opacity: 0;
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

	.empty-chapter,
	.empty-outline {
		padding: var(--spacing-md);
		text-align: center;
	}

	.empty-outline p {
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-sm);
	}

	.add-first-scene,
	.add-first-chapter {
		font-size: var(--font-size-sm);
		color: var(--color-accent);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		transition: background-color var(--transition-fast);
	}

	.add-first-scene:hover,
	.add-first-chapter:hover {
		background-color: var(--color-accent-light);
	}

	/* Drag and drop styles */
	.chapter-item.dragging,
	.scene-item.dragging {
		opacity: 0.5;
	}

	.chapter-item.drop-target,
	.scene-item.drop-target {
		background-color: var(--color-accent-light);
		border-top: 2px solid var(--color-accent);
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
</style>
