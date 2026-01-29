<!--
  Arc management dialog.

  Features:
  - View all arcs in the project
  - Create new arcs with name, description, stakes, status, color
  - Edit existing arcs
  - Delete arcs
  - Link/unlink characters to arcs
  - View scenes associated with each arc
-->
<script lang="ts">
	import { slide } from 'svelte/transition';

	import { type Arc, arcApi, type Scene } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';
	import { formatDate } from '$lib/utils';
	import { trapFocus } from '$lib/utils/focus-trap';
	import { nativeConfirm } from '$lib/utils/native-dialog';

	import { Button, EmptyState, FormActions, FormGroup, Icon, LoadingState } from './ui';
	import ContextMenu from './ui/ContextMenu.svelte';
	import ContextMenuItem from './ui/ContextMenuItem.svelte';
	import ContextMenuSeparator from './ui/ContextMenuSeparator.svelte';

	let arcContextMenu = $state<{ x: number; y: number; arcId: string } | null>(null);

	function handleArcContextMenu(event: MouseEvent, arcId: string) {
		event.preventDefault();
		arcContextMenu = { x: event.clientX, y: event.clientY, arcId };
	}

	function closeArcContextMenu() {
		arcContextMenu = null;
	}

	interface Props {
		isOpen: boolean;
		onclose: () => void;
	}

	let { isOpen, onclose }: Props = $props();

	let arcs = $state<Arc[]>([]);
	let selectedArcId = $state<string | null>(null);
	let isCreating = $state(false);
	let isEditing = $state(false);
	let isLoading = $state(false);

	// Form state
	let formName = $state('');
	let formDescription = $state('');
	let formStakes = $state('');
	let formStatus = $state('setup');
	let formColor = $state('#6366f1');
	let formCharacterIds = $state<string[]>([]);

	// Linked scenes
	let arcScenes = $state<Scene[]>([]);

	// Scene linking
	let isAddingSceneToArc = $state(false);
	let sceneSearchQuery = $state('');

	// Character selector
	let characterSearch = $state('');
	let showCharacterDropdown = $state(false);

	let availableCharacters = $derived(
		appState.bibleEntries
			.filter((e) => e.entry_type === 'character')
			.filter((e) => !formCharacterIds.includes(e.id))
			.filter(
				(e) => !characterSearch || e.name.toLowerCase().includes(characterSearch.toLowerCase())
			)
	);

	let availableScenesForArc = $derived.by(() => {
		const linkedIds = new Set(arcScenes.map((s) => s.id));
		const result: Array<{ id: string; title: string; chapterTitle: string }> = [];
		for (const chapter of appState.chapters) {
			const scenes = appState.scenes.get(chapter.id) || [];
			for (const scene of scenes) {
				if (
					!linkedIds.has(scene.id) &&
					(!sceneSearchQuery || scene.title.toLowerCase().includes(sceneSearchQuery.toLowerCase()))
				) {
					result.push({ id: scene.id, title: scene.title, chapterTitle: chapter.title });
				}
			}
		}
		return result;
	});

	const arcStatuses = [
		{ value: 'setup', label: 'Setup' },
		{ value: 'active', label: 'Active' },
		{ value: 'climax', label: 'Climax' },
		{ value: 'resolved', label: 'Resolved' },
		{ value: 'abandoned', label: 'Abandoned' },
	];

	const arcColors = [
		'#ef4444', // red
		'#f97316', // orange
		'#eab308', // yellow
		'#22c55e', // green
		'#06b6d4', // cyan
		'#3b82f6', // blue
		'#6366f1', // indigo
		'#8b5cf6', // violet
		'#ec4899', // pink
	];

	let selectedArc = $derived(
		selectedArcId ? arcs.find((a) => a.id === selectedArcId) || null : null
	);

	$effect(() => {
		if (isOpen && appState.project) {
			loadArcs();
		}
	});

	$effect(() => {
		if (selectedArcId && !isCreating && !isEditing) {
			loadArcScenes(selectedArcId);
		} else {
			arcScenes = [];
		}
		isAddingSceneToArc = false;
		sceneSearchQuery = '';
	});

	async function loadArcs() {
		isLoading = true;
		try {
			arcs = await arcApi.getAll();
		} catch (e) {
			console.error('Failed to load arcs:', e);
			showError('Failed to load arcs');
		} finally {
			isLoading = false;
		}
	}

	async function loadArcScenes(arcId: string) {
		try {
			arcScenes = await arcApi.getArcScenes(arcId);
		} catch (e) {
			console.error('Failed to load arc scenes:', e);
			arcScenes = [];
		}
	}

	function navigateToScene(sceneId: string) {
		for (const [chapterId, scenes] of appState.scenes) {
			const scene = scenes.find((s) => s.id === sceneId);
			if (scene) {
				appState.selectedChapterId = chapterId;
				appState.selectedSceneId = sceneId;
				appState.setViewMode('editor');
				onclose();
				return;
			}
		}
	}

	async function linkSceneToArc(sceneId: string) {
		if (!selectedArc) return;
		try {
			await arcApi.linkScene(sceneId, selectedArc.id);
			arcScenes = await arcApi.getArcScenes(selectedArc.id);
			isAddingSceneToArc = false;
			sceneSearchQuery = '';
		} catch (e) {
			console.error('Failed to link scene to arc:', e);
			showError('Failed to link scene to arc');
		}
	}

	async function unlinkSceneFromArc(sceneId: string) {
		if (!selectedArc) return;
		try {
			await arcApi.unlinkScene(sceneId, selectedArc.id);
			arcScenes = arcScenes.filter((s) => s.id !== sceneId);
		} catch (e) {
			console.error('Failed to unlink scene from arc:', e);
			showError('Failed to unlink scene from arc');
		}
	}

	function startCreate() {
		isCreating = true;
		isEditing = false;
		selectedArcId = null;
		resetForm();
	}

	function startEdit(arc: Arc) {
		isEditing = true;
		isCreating = false;
		selectedArcId = arc.id;
		formName = arc.name;
		formDescription = arc.description || '';
		formStakes = arc.stakes || '';
		formStatus = arc.status;
		formColor = arc.color || '#6366f1';
		formCharacterIds = [...arc.characters];
	}

	function resetForm() {
		formName = '';
		formDescription = '';
		formStakes = '';
		formStatus = 'setup';
		formColor = '#6366f1';
		formCharacterIds = [];
	}

	function cancelForm() {
		isCreating = false;
		isEditing = false;
		resetForm();
	}

	async function saveArc() {
		if (!formName.trim()) return;

		try {
			if (isCreating) {
				const newArc = await arcApi.create({
					name: formName.trim(),
					description: formDescription.trim() || undefined,
					stakes: formStakes.trim() || undefined,
					status: formStatus,
					color: formColor,
				});
				await arcApi.setCharacters(newArc.id, formCharacterIds);
				newArc.characters = [...formCharacterIds];
				arcs = [...arcs, newArc];
				selectedArcId = newArc.id;
			} else if (isEditing && selectedArcId) {
				const updated = await arcApi.update(selectedArcId, {
					name: formName.trim(),
					description: formDescription.trim() || null,
					stakes: formStakes.trim() || null,
					status: formStatus,
					color: formColor,
				});
				await arcApi.setCharacters(selectedArcId, formCharacterIds);
				updated.characters = [...formCharacterIds];
				arcs = arcs.map((a) => (a.id === updated.id ? updated : a));
			}
			isCreating = false;
			isEditing = false;
		} catch (e) {
			console.error('Failed to save arc:', e);
			showError('Failed to save arc');
		}
	}

	async function deleteArc(arcId: string) {
		if (!(await nativeConfirm('Delete this arc? This action cannot be undone.', 'Delete Arc')))
			return;

		try {
			await arcApi.delete(arcId);
			arcs = arcs.filter((a) => a.id !== arcId);
			if (selectedArcId === arcId) {
				selectedArcId = null;
			}
		} catch (e) {
			console.error('Failed to delete arc:', e);
			showError('Failed to delete arc');
		}
	}

	function getStatusLabel(status: string): string {
		return arcStatuses.find((s) => s.value === status)?.label || status;
	}

	function getCharacterNames(characterIds: string[]): string[] {
		return characterIds.map((id) => {
			const entry = appState.bibleEntries.find((e) => e.id === id);
			return entry?.name || id;
		});
	}

	function addCharacter(id: string) {
		if (!formCharacterIds.includes(id)) {
			formCharacterIds = [...formCharacterIds, id];
		}
		characterSearch = '';
		showCharacterDropdown = false;
	}

	function removeCharacter(id: string) {
		formCharacterIds = formCharacterIds.filter((cid) => cid !== id);
	}

	function handleOverlayClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			onclose();
		}
	}
</script>

{#if isOpen}
	<div
		class="modal-overlay"
		onclick={handleOverlayClick}
		onkeydown={(e) => {
			if (e.key === 'Escape') onclose();
		}}
		role="presentation"
		tabindex="-1"
	>
		<!-- AE1: Focus trap -->
		<div
			class="modal-container modal-enter"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="arcs-title"
			tabindex="-1"
			use:trapFocus={{ onEscape: onclose }}
		>
			<div class="modal-header">
				<h2 id="arcs-title">Story Arcs</h2>
				<button class="close-btn" onclick={onclose} aria-label="Close">
					<Icon name="close" size={20} />
				</button>
			</div>

			<div class="modal-body">
				<div class="arcs-sidebar">
					<div class="sidebar-header">
						<Button size="sm" variant="primary" onclick={startCreate}>
							<Icon name="plus" size={14} />
							New Arc
						</Button>
					</div>

					<div class="arcs-list">
						{#if isLoading}
							<LoadingState message="Loading arcs..." size="sm" />
						{:else if arcs.length === 0}
							<EmptyState
								compact
								icon="bookmark"
								title="No arcs yet"
								description="Story arcs track the major narrative threads in your manuscript."
							/>
						{:else}
							{#each arcs as arc (arc.id)}
								<button
									class="arc-item"
									class:selected={selectedArcId === arc.id && !isCreating}
									onclick={() => {
										selectedArcId = arc.id;
										isCreating = false;
										isEditing = false;
									}}
									oncontextmenu={(e) => handleArcContextMenu(e, arc.id)}
								>
									<span class="arc-color" style="background-color: {arc.color || '#6366f1'}"></span>
									<div class="arc-info">
										<span class="arc-name" title={arc.name}>{arc.name}</span>
										<span class="arc-status">{getStatusLabel(arc.status)}</span>
									</div>
								</button>
							{/each}
						{/if}
					</div>
				</div>

				<div class="arcs-detail">
					{#if isCreating || isEditing}
						<form
							onsubmit={(e) => {
								e.preventDefault();
								saveArc();
							}}
						>
							<h3>{isCreating ? 'Create New Arc' : 'Edit Arc'}</h3>

							<FormGroup label="Name">
								<input
									type="text"
									bind:value={formName}
									placeholder="Arc name"
									maxlength={100}
									required
								/>
							</FormGroup>

							<FormGroup label="Description">
								<textarea
									bind:value={formDescription}
									rows="3"
									placeholder="What is this arc about?"
								></textarea>
							</FormGroup>

							<FormGroup label="Stakes">
								<textarea bind:value={formStakes} rows="2" placeholder="What's at risk?"></textarea>
							</FormGroup>

							<FormGroup label="Status">
								<select bind:value={formStatus}>
									{#each arcStatuses as status (status.value)}
										<option value={status.value}>{status.label}</option>
									{/each}
								</select>
							</FormGroup>

							<FormGroup label="Color">
								<div class="color-picker">
									{#each arcColors as color (color)}
										<button
											type="button"
											class="color-option"
											class:selected={formColor === color}
											style="background-color: {color}"
											onclick={() => (formColor = color)}
											aria-label="Select color {color}"
										></button>
									{/each}
								</div>
							</FormGroup>

							<FormGroup label="Key Characters">
								<div class="character-selector">
									{#if formCharacterIds.length > 0}
										<div class="character-chips">
											{#each formCharacterIds as charId (charId)}
												{@const entry = appState.bibleEntries.find((e) => e.id === charId)}
												<span class="character-chip">
													{entry?.name || charId}
													<button
														type="button"
														class="chip-remove"
														onclick={() => removeCharacter(charId)}
													>
														<Icon name="close" size={10} />
													</button>
												</span>
											{/each}
										</div>
									{/if}
									<div class="character-search-wrapper">
										<input
											type="text"
											bind:value={characterSearch}
											placeholder="Search characters..."
											onfocus={() => (showCharacterDropdown = true)}
											onfocusout={(e) => {
												const related = (e as FocusEvent).relatedTarget as Node | null;
												if (
													!related ||
													!(e.currentTarget as HTMLElement).parentElement?.contains(related)
												) {
													showCharacterDropdown = false;
												}
											}}
											onkeydown={(e) => {
												if (e.key === 'Escape') {
													e.stopPropagation();
													showCharacterDropdown = false;
													characterSearch = '';
												}
											}}
										/>
										{#if showCharacterDropdown && availableCharacters.length > 0}
											<div class="character-dropdown" transition:slide={{ duration: 150 }}>
												{#each availableCharacters.slice(0, 10) as entry (entry.id)}
													<button
														type="button"
														class="character-dropdown-item"
														onmousedown={() => addCharacter(entry.id)}
													>
														{entry.name}
													</button>
												{/each}
											</div>
										{/if}
									</div>
								</div>
							</FormGroup>

							<FormActions>
								<Button onclick={cancelForm}>Cancel</Button>
								<Button variant="primary" type="submit">
									{isCreating ? 'Create Arc' : 'Save Changes'}
								</Button>
							</FormActions>
						</form>
					{:else if selectedArc}
						<div class="arc-detail">
							<div class="detail-header">
								<span
									class="arc-color-large"
									style="background-color: {selectedArc.color || '#6366f1'}"
								></span>
								<div class="detail-title">
									<h3>{selectedArc.name}</h3>
									<span class="status-badge" data-status={selectedArc.status}>
										{getStatusLabel(selectedArc.status)}
									</span>
								</div>
								<div class="detail-actions">
									<Button
										size="sm"
										onclick={() => {
											if (selectedArc) startEdit(selectedArc);
										}}
									>
										<Icon name="edit" size={14} />
										Edit
									</Button>
									<Button
										size="sm"
										onclick={() => {
											if (selectedArc) deleteArc(selectedArc.id);
										}}
									>
										<Icon name="trash" size={14} />
										Delete
									</Button>
								</div>
							</div>

							{#if selectedArc.description}
								<div class="detail-section">
									<h4>Description</h4>
									<p>{selectedArc.description}</p>
								</div>
							{/if}

							{#if selectedArc.stakes}
								<div class="detail-section">
									<h4>Stakes</h4>
									<p>{selectedArc.stakes}</p>
								</div>
							{/if}

							{#if selectedArc.characters.length > 0}
								<div class="detail-section">
									<h4>Key Characters</h4>
									<div class="characters-list">
										{#each getCharacterNames(selectedArc.characters) as name, i (i)}
											<span class="character-tag">{name}</span>
										{/each}
									</div>
								</div>
							{/if}

							<div class="detail-section">
								<div class="section-header">
									<h4>Scenes ({arcScenes.length})</h4>
									<Button
										variant="secondary"
										size="sm"
										onclick={() => (isAddingSceneToArc = !isAddingSceneToArc)}
									>
										<Icon name="plus" size={14} />
										Link
									</Button>
								</div>
								{#if isAddingSceneToArc}
									<div class="scene-search" transition:slide={{ duration: 150 }}>
										<input
											type="text"
											placeholder="Search scenes..."
											bind:value={sceneSearchQuery}
											class="scene-search-input"
											onkeydown={(e) => {
												if (e.key === 'Escape') {
													e.stopPropagation();
													isAddingSceneToArc = false;
													sceneSearchQuery = '';
												}
											}}
										/>
										<div class="scene-search-results">
											{#each availableScenesForArc.slice(0, 10) as scene (scene.id)}
												<button class="scene-search-item" onclick={() => linkSceneToArc(scene.id)}>
													<span class="scene-search-title">{scene.title}</span>
													<span class="scene-search-chapter">{scene.chapterTitle}</span>
												</button>
											{:else}
												<span class="no-results">No matching scenes</span>
											{/each}
										</div>
									</div>
								{/if}
								{#if arcScenes.length > 0}
									<div class="scenes-list">
										{#each arcScenes as scene (scene.id)}
											<div class="scene-link-item">
												<button class="scene-link" onclick={() => navigateToScene(scene.id)}>
													{scene.title}
												</button>
												<button
													class="unlink-btn"
													onclick={() => unlinkSceneFromArc(scene.id)}
													title="Unlink scene"
												>
													<Icon name="close" size={12} />
												</button>
											</div>
										{/each}
									</div>
								{:else if !isAddingSceneToArc}
									<p class="no-scenes">No scenes linked to this arc.</p>
								{/if}
							</div>

							<div class="detail-section">
								<h4>Metadata</h4>
								<div class="meta-info">
									<span>Created: {formatDate(selectedArc.created_at)}</span>
									<span>Updated: {formatDate(selectedArc.updated_at)}</span>
								</div>
							</div>
						</div>
					{:else}
						<EmptyState
							icon="bookmark"
							title="Select an arc or create a new one"
							description="Choose an arc from the list to view its details, or create a new arc to track a narrative thread."
						/>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}

{#if arcContextMenu}
	<ContextMenu x={arcContextMenu.x} y={arcContextMenu.y} onclose={closeArcContextMenu}>
		<ContextMenuItem
			label="Edit"
			onclick={() => {
				selectedArcId = arcContextMenu!.arcId;
				isEditing = true;
				isCreating = false;
				closeArcContextMenu();
			}}
		/>
		<ContextMenuSeparator />
		<ContextMenuItem
			label="Delete"
			danger
			onclick={() => {
				deleteArc(arcContextMenu!.arcId);
				closeArcContextMenu();
			}}
		/>
	</ContextMenu>
{/if}

<style>
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
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		width: 90%;
		max-width: 900px;
		height: 80vh;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.modal-header h2 {
		margin: 0;
		font-size: var(--font-size-lg);
	}

	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.modal-body {
		flex: 1;
		display: flex;
		overflow: hidden;
	}

	.arcs-sidebar {
		width: 280px;
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
	}

	.sidebar-header {
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
	}

	.arcs-list {
		flex: 1;
		overflow-y: auto;
	}

	.arc-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-md);
		border-bottom: 1px solid var(--color-border-light);
		text-align: left;
		background: none;
		transition: background-color var(--transition-fast);
	}

	.arc-item:hover {
		background-color: var(--color-bg-hover);
	}

	.arc-item.selected {
		background-color: var(--color-accent-light);
	}

	.arc-color {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.arc-info {
		flex: 1;
		min-width: 0;
	}

	.arc-name {
		display: block;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.arc-status {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
	}

	.arcs-detail {
		flex: 1;
		padding: var(--spacing-lg);
		overflow-y: auto;
	}

	.arcs-detail h3 {
		margin: 0 0 var(--spacing-lg) 0;
		font-size: var(--font-size-lg);
	}

	.arcs-detail form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
		max-width: 500px;
	}

	.arcs-detail input,
	.arcs-detail select,
	.arcs-detail textarea {
		width: 100%;
		padding: var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-base);
	}

	.arcs-detail textarea {
		resize: vertical;
	}

	.color-picker {
		display: flex;
		gap: var(--spacing-xs);
		flex-wrap: wrap;
	}

	.color-option {
		width: 28px;
		height: 28px;
		border-radius: 50%;
		border: 2px solid transparent;
		transition: transform var(--transition-fast);
	}

	.color-option:hover {
		transform: scale(1.1);
	}

	.color-option.selected {
		border-color: var(--color-text-primary);
	}

	.arc-detail {
		max-width: 600px;
	}

	.detail-header {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-md);
		margin-bottom: var(--spacing-lg);
	}

	.arc-color-large {
		width: 24px;
		height: 24px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.detail-title {
		flex: 1;
	}

	.detail-title h3 {
		margin: 0 0 var(--spacing-xs) 0;
	}

	.status-badge {
		display: inline-block;
		padding: 2px 8px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
		text-transform: capitalize;
	}

	.status-badge[data-status='active'] {
		background-color: var(--color-success-light);
		color: var(--color-success);
	}

	.status-badge[data-status='climax'] {
		background-color: var(--color-warning-light);
		color: var(--color-warning);
	}

	.status-badge[data-status='resolved'] {
		background-color: var(--color-info-light);
		color: var(--color-info);
	}

	.detail-actions {
		display: flex;
		gap: var(--spacing-xs);
	}

	.detail-section {
		margin-bottom: var(--spacing-lg);
	}

	.detail-section h4 {
		margin: 0 0 var(--spacing-xs) 0;
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.detail-section p {
		margin: 0;
		color: var(--color-text-primary);
		line-height: 1.6;
	}

	.characters-list {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
	}

	.character-tag {
		padding: 2px 8px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.meta-info {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.scenes-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.scene-link {
		display: block;
		text-align: left;
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		color: var(--color-accent);
		transition: background-color var(--transition-fast);
	}

	.scene-link:hover {
		background-color: var(--color-bg-hover);
		text-decoration: underline;
	}

	/* Character Selector */
	.character-selector {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.character-chips {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
	}

	.character-chip {
		display: inline-flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: 2px 8px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.chip-remove {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		padding: 1px;
		border-radius: 50%;
		color: var(--color-text-muted);
	}

	.chip-remove:hover {
		color: var(--color-error);
		background-color: var(--color-bg-hover);
	}

	.character-search-wrapper {
		position: relative;
	}

	.character-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		box-shadow: var(--shadow-md);
		z-index: 10;
		max-height: min(200px, 40vh);
		overflow-y: auto;
	}

	.character-dropdown-item {
		width: 100%;
		padding: var(--spacing-sm);
		text-align: left;
		font-size: var(--font-size-sm);
	}

	.character-dropdown-item:hover {
		background-color: var(--color-bg-hover);
	}

	/* Scene Linking */
	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-xs);
	}

	.section-header h4 {
		margin: 0;
	}

	.scene-link-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
	}

	.scene-link-item .scene-link {
		flex: 1;
	}

	.unlink-btn {
		padding: 2px;
		color: var(--color-text-muted);
		cursor: pointer;
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
	}

	.unlink-btn:hover {
		color: var(--color-error);
		background-color: var(--color-bg-hover);
	}

	.scene-search {
		margin-bottom: var(--spacing-sm);
	}

	.scene-search-input {
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background: var(--color-bg-primary);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		margin-bottom: var(--spacing-xs);
	}

	.scene-search-input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.scene-search-results {
		max-height: 150px;
		overflow-y: auto;
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.scene-search-item {
		width: 100%;
		display: flex;
		justify-content: space-between;
		padding: var(--spacing-xs) var(--spacing-sm);
		text-align: left;
		cursor: pointer;
	}

	.scene-search-item:hover {
		background-color: var(--color-bg-hover);
	}

	.scene-search-title {
		font-size: var(--font-size-sm);
	}

	.scene-search-chapter {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.no-results {
		display: block;
		padding: var(--spacing-sm);
		text-align: center;
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
	}

	.no-scenes {
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
		font-style: italic;
	}

	/* Responsive adjustments */
	@media (max-width: 900px) {
		.modal-container {
			max-width: calc(100vw - var(--spacing-md) * 2);
		}
		.arcs-sidebar {
			width: 220px;
		}
	}

	@media (max-width: 800px) {
		.arcs-sidebar {
			width: 180px;
		}
	}
</style>
