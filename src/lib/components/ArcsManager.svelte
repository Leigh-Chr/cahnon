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
	import { arcApi, type Arc } from '$lib/api';
	import { appState } from '$lib/stores';
	import { Button, FormGroup, FormActions, Icon, EmptyState } from './ui';

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
	let formCharacters = $state('');

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

	async function loadArcs() {
		isLoading = true;
		try {
			arcs = await arcApi.getAll();
		} catch (e) {
			console.error('Failed to load arcs:', e);
		} finally {
			isLoading = false;
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
		formCharacters = arc.characters || '';
	}

	function resetForm() {
		formName = '';
		formDescription = '';
		formStakes = '';
		formStatus = 'setup';
		formColor = '#6366f1';
		formCharacters = '';
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
					characters: formCharacters.trim() || undefined,
				});
				arcs = [...arcs, newArc];
				selectedArcId = newArc.id;
			} else if (isEditing && selectedArcId) {
				const updated = await arcApi.update(selectedArcId, {
					name: formName.trim(),
					description: formDescription.trim() || null,
					stakes: formStakes.trim() || null,
					status: formStatus,
					color: formColor,
					characters: formCharacters.trim() || null,
				});
				arcs = arcs.map((a) => (a.id === updated.id ? updated : a));
			}
			isCreating = false;
			isEditing = false;
		} catch (e) {
			console.error('Failed to save arc:', e);
		}
	}

	async function deleteArc(arcId: string) {
		if (!confirm('Delete this arc? This action cannot be undone.')) return;

		try {
			await arcApi.delete(arcId);
			arcs = arcs.filter((a) => a.id !== arcId);
			if (selectedArcId === arcId) {
				selectedArcId = null;
			}
		} catch (e) {
			console.error('Failed to delete arc:', e);
		}
	}

	function getStatusLabel(status: string): string {
		return arcStatuses.find((s) => s.value === status)?.label || status;
	}

	function getCharacterNames(characterIds: string | null): string[] {
		if (!characterIds) return [];
		return characterIds
			.split(',')
			.map((id) => id.trim())
			.filter((id) => id)
			.map((id) => {
				const entry = appState.bibleEntries.find((e) => e.id === id);
				return entry?.name || id;
			});
	}

	function handleOverlayClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			onclose();
		}
	}
</script>

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="modal-overlay" onclick={handleOverlayClick} role="presentation">
		<div
			class="modal-container"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="arcs-title"
			tabindex="-1"
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
							<div class="loading">Loading arcs...</div>
						{:else if arcs.length === 0}
							<EmptyState title="No arcs yet" />
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
								>
									<span class="arc-color" style="background-color: {arc.color || '#6366f1'}"></span>
									<div class="arc-info">
										<span class="arc-name">{arc.name}</span>
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
								<input type="text" bind:value={formName} placeholder="Arc name" required />
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
									{#each arcStatuses as status}
										<option value={status.value}>{status.label}</option>
									{/each}
								</select>
							</FormGroup>

							<FormGroup label="Color">
								<div class="color-picker">
									{#each arcColors as color}
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

							<FormGroup label="Key Characters (Bible entry IDs, comma-separated)">
								<input
									type="text"
									bind:value={formCharacters}
									placeholder="Optional: character IDs"
								/>
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
									<Button size="sm" onclick={() => startEdit(selectedArc!)}>
										<Icon name="edit" size={14} />
										Edit
									</Button>
									<Button size="sm" onclick={() => deleteArc(selectedArc!.id)}>
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

							{#if selectedArc.characters}
								<div class="detail-section">
									<h4>Key Characters</h4>
									<div class="characters-list">
										{#each getCharacterNames(selectedArc.characters) as name}
											<span class="character-tag">{name}</span>
										{/each}
									</div>
								</div>
							{/if}

							<div class="detail-section">
								<h4>Metadata</h4>
								<div class="meta-info">
									<span>Created: {new Date(selectedArc.created_at).toLocaleDateString()}</span>
									<span>Updated: {new Date(selectedArc.updated_at).toLocaleDateString()}</span>
								</div>
							</div>
						</div>
					{:else}
						<EmptyState title="Select an arc or create a new one" />
					{/if}
				</div>
			</div>
		</div>
	</div>
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

	.loading {
		padding: var(--spacing-lg);
		text-align: center;
		color: var(--color-text-secondary);
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
		cursor: pointer;
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
		cursor: pointer;
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
</style>
