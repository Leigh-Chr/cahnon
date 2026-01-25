<script lang="ts">
	/**
	 * ArcsView Component
	 *
	 * Displays and manages story arcs - major plotlines and character journeys.
	 * Supports CRUD operations with inline editing.
	 */

	import { onMount } from 'svelte';
	import { arcApi, type Arc } from '$lib/api';
	import { DEFAULT_CUSTOM_COLOR } from '$lib/utils';
	import { Icon, Button, EmptyState, LoadingState, FormGroup, FormRow, FormActions } from './ui';

	let arcs = $state<Arc[]>([]);
	let isLoading = $state(true);
	let editingArc = $state<Arc | null>(null);
	let showNewArcForm = $state(false);

	let newArcName = $state('');
	let newArcDescription = $state('');
	let newArcStakes = $state('');
	let newArcColor = $state(DEFAULT_CUSTOM_COLOR);

	onMount(() => {
		loadArcs();
	});

	async function loadArcs() {
		isLoading = true;
		try {
			arcs = await arcApi.getAll();
		} catch (e) {
			console.error('Failed to load arcs:', e);
		}
		isLoading = false;
	}

	async function createArc() {
		if (!newArcName.trim()) return;
		try {
			const arc = await arcApi.create({
				name: newArcName.trim(),
				description: newArcDescription.trim() || undefined,
				stakes: newArcStakes.trim() || undefined,
				color: newArcColor,
			});
			arcs = [...arcs, arc];
			resetNewArcForm();
		} catch (e) {
			console.error('Failed to create arc:', e);
		}
	}

	function resetNewArcForm() {
		showNewArcForm = false;
		newArcName = '';
		newArcDescription = '';
		newArcStakes = '';
		newArcColor = DEFAULT_CUSTOM_COLOR;
	}

	async function updateArc(arc: Arc) {
		try {
			const updated = await arcApi.update(arc.id, {
				name: arc.name,
				description: arc.description || undefined,
				stakes: arc.stakes || undefined,
				status: arc.status,
				color: arc.color || undefined,
			});
			arcs = arcs.map((a) => (a.id === updated.id ? updated : a));
			editingArc = null;
		} catch (e) {
			console.error('Failed to update arc:', e);
		}
	}

	async function deleteArc(arcId: string) {
		if (!confirm('Delete this arc? This cannot be undone.')) return;
		try {
			await arcApi.delete(arcId);
			arcs = arcs.filter((a) => a.id !== arcId);
		} catch (e) {
			console.error('Failed to delete arc:', e);
		}
	}

	function getStatusColor(status: string): string {
		const colors: Record<string, string> = {
			active: 'var(--color-info)',
			planned: 'var(--color-text-muted)',
			completed: 'var(--color-success)',
			abandoned: 'var(--color-text-disabled)',
		};
		return colors[status] || 'var(--color-text-muted)';
	}

	const arcStatuses = ['planned', 'active', 'completed', 'abandoned'];
</script>

<div class="arcs-view">
	<div class="arcs-header">
		<h2>Story Arcs</h2>
		<Button variant="primary" onclick={() => (showNewArcForm = true)}>
			<Icon name="plus" size={16} />
			New Arc
		</Button>
	</div>

	{#if isLoading}
		<LoadingState message="Loading arcs..." />
	{:else if arcs.length === 0 && !showNewArcForm}
		<EmptyState
			icon="book"
			title="No story arcs yet"
			description="Create arcs to track major plotlines and character journeys through your story."
			actionLabel="Create Your First Arc"
			onaction={() => (showNewArcForm = true)}
		/>
	{:else}
		<div class="arcs-list">
			{#if showNewArcForm}
				<div class="arc-card new-arc-form">
					<FormGroup label="Name" id="arc-name">
						<!-- svelte-ignore a11y_autofocus -->
						<input
							id="arc-name"
							type="text"
							bind:value={newArcName}
							placeholder="Arc name..."
							autofocus
						/>
					</FormGroup>
					<FormGroup label="Description" id="arc-description">
						<textarea
							id="arc-description"
							bind:value={newArcDescription}
							placeholder="What is this arc about?"
							rows="3"
						></textarea>
					</FormGroup>
					<FormGroup label="Stakes" id="arc-stakes">
						<textarea
							id="arc-stakes"
							bind:value={newArcStakes}
							placeholder="What's at stake?"
							rows="2"
						></textarea>
					</FormGroup>
					<FormGroup label="Color" id="arc-color">
						<input id="arc-color" type="color" bind:value={newArcColor} />
					</FormGroup>
					<FormActions>
						<Button variant="ghost" onclick={resetNewArcForm}>Cancel</Button>
						<Button variant="primary" onclick={createArc} disabled={!newArcName.trim()}>
							Create Arc
						</Button>
					</FormActions>
				</div>
			{/if}

			{#each arcs as arc (arc.id)}
				<div class="arc-card" style="--arc-color: {arc.color || 'var(--color-neutral)'}">
					{#if editingArc?.id === arc.id}
						<FormGroup label="Name" id="edit-arc-name">
							<input id="edit-arc-name" type="text" bind:value={editingArc.name} />
						</FormGroup>
						<FormGroup label="Description" id="edit-arc-description">
							<textarea id="edit-arc-description" bind:value={editingArc.description} rows="3"
							></textarea>
						</FormGroup>
						<FormGroup label="Stakes" id="edit-arc-stakes">
							<textarea id="edit-arc-stakes" bind:value={editingArc.stakes} rows="2"></textarea>
						</FormGroup>
						<FormRow>
							<FormGroup label="Status" id="edit-arc-status">
								<select id="edit-arc-status" bind:value={editingArc.status}>
									{#each arcStatuses as status (status)}
										<option value={status}>{status}</option>
									{/each}
								</select>
							</FormGroup>
							<FormGroup label="Color" id="edit-arc-color">
								<input id="edit-arc-color" type="color" bind:value={editingArc.color} />
							</FormGroup>
						</FormRow>
						<FormActions>
							<Button variant="ghost" onclick={() => (editingArc = null)}>Cancel</Button>
							<Button variant="primary" onclick={() => updateArc(editingArc!)}>Save</Button>
						</FormActions>
					{:else}
						<div class="arc-header">
							<div class="arc-color-bar"></div>
							<div class="arc-info">
								<h3>{arc.name}</h3>
								<span class="arc-status" style="color: {getStatusColor(arc.status)}"
									>{arc.status}</span
								>
							</div>
							<div class="arc-actions">
								<Button variant="icon" onclick={() => (editingArc = { ...arc })} title="Edit">
									<Icon name="edit" size={16} />
								</Button>
								<Button
									variant="icon"
									class="danger"
									onclick={() => deleteArc(arc.id)}
									title="Delete"
								>
									<Icon name="delete" size={16} />
								</Button>
							</div>
						</div>
						{#if arc.description}
							<p class="arc-description">{arc.description}</p>
						{/if}
						{#if arc.stakes}
							<div class="arc-stakes">
								<strong>Stakes:</strong>
								{arc.stakes}
							</div>
						{/if}
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.arcs-view {
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-primary);
	}

	.arcs-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.arcs-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.arcs-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.arc-card {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
	}

	.arc-header {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-md);
	}

	.arc-color-bar {
		width: 4px;
		height: 40px;
		border-radius: 2px;
		background-color: var(--arc-color);
		flex-shrink: 0;
	}

	.arc-info {
		flex: 1;
	}

	.arc-info h3 {
		font-size: var(--font-size-base);
		font-weight: 600;
		margin-bottom: var(--spacing-xs);
	}

	.arc-status {
		font-size: var(--font-size-xs);
		font-weight: 500;
		text-transform: capitalize;
	}

	.arc-actions {
		display: flex;
		gap: var(--spacing-xs);
	}

	/* Danger variant for icon buttons in this context */
	.arc-actions :global(.btn-icon.danger:hover) {
		color: var(--color-error);
	}

	.arc-description {
		margin-top: var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		padding-left: calc(4px + var(--spacing-md));
	}

	.arc-stakes {
		margin-top: var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		padding-left: calc(4px + var(--spacing-md));
	}
</style>
