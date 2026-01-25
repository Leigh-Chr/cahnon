<script lang="ts">
	import { onMount } from 'svelte';
	import { arcApi, type Arc } from '$lib/api';

	let arcs = $state<Arc[]>([]);
	let isLoading = $state(true);
	let editingArc = $state<Arc | null>(null);
	let showNewArcForm = $state(false);

	let newArcName = $state('');
	let newArcDescription = $state('');
	let newArcStakes = $state('');
	let newArcColor = $state('#525252');

	// Use onMount for one-time initialization instead of $effect
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
		newArcColor = '#525252';
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
			active: 'var(--color-text-primary)',
			planned: 'var(--color-text-muted)',
			completed: 'var(--color-text-secondary)',
			abandoned: 'var(--color-border)',
		};
		return colors[status] || 'var(--color-text-muted)';
	}

	const arcStatuses = ['planned', 'active', 'completed', 'abandoned'];
</script>

<div class="arcs-view">
	<div class="arcs-header">
		<h2>Story Arcs</h2>
		<button class="add-btn" onclick={() => (showNewArcForm = true)}>
			<svg
				width="16"
				height="16"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<line x1="12" y1="5" x2="12" y2="19" />
				<line x1="5" y1="12" x2="19" y2="12" />
			</svg>
			New Arc
		</button>
	</div>

	{#if isLoading}
		<div class="loading">Loading arcs...</div>
	{:else if arcs.length === 0 && !showNewArcForm}
		<div class="empty-state">
			<svg
				width="48"
				height="48"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
				<path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
			</svg>
			<h3>No story arcs yet</h3>
			<p>Create arcs to track major plotlines and character journeys through your story.</p>
			<button class="primary-btn" onclick={() => (showNewArcForm = true)}
				>Create Your First Arc</button
			>
		</div>
	{:else}
		<div class="arcs-list">
			{#if showNewArcForm}
				<div class="arc-card new-arc-form">
					<div class="form-group">
						<label for="arc-name">Name</label>
						<!-- svelte-ignore a11y_autofocus -->
						<input
							id="arc-name"
							type="text"
							bind:value={newArcName}
							placeholder="Arc name..."
							autofocus
						/>
					</div>
					<div class="form-group">
						<label for="arc-description">Description</label>
						<textarea
							id="arc-description"
							bind:value={newArcDescription}
							placeholder="What is this arc about?"
							rows="3"
						></textarea>
					</div>
					<div class="form-group">
						<label for="arc-stakes">Stakes</label>
						<textarea
							id="arc-stakes"
							bind:value={newArcStakes}
							placeholder="What's at stake?"
							rows="2"
						></textarea>
					</div>
					<div class="form-group">
						<label for="arc-color">Color</label>
						<input id="arc-color" type="color" bind:value={newArcColor} />
					</div>
					<div class="form-actions">
						<button class="cancel-btn" onclick={resetNewArcForm}>Cancel</button>
						<button class="save-btn" onclick={createArc} disabled={!newArcName.trim()}
							>Create Arc</button
						>
					</div>
				</div>
			{/if}

			{#each arcs as arc (arc.id)}
				<div class="arc-card" style="--arc-color: {arc.color || '#525252'}">
					{#if editingArc?.id === arc.id}
						<div class="form-group">
							<label for="edit-arc-name">Name</label>
							<input id="edit-arc-name" type="text" bind:value={editingArc.name} />
						</div>
						<div class="form-group">
							<label for="edit-arc-description">Description</label>
							<textarea id="edit-arc-description" bind:value={editingArc.description} rows="3"
							></textarea>
						</div>
						<div class="form-group">
							<label for="edit-arc-stakes">Stakes</label>
							<textarea id="edit-arc-stakes" bind:value={editingArc.stakes} rows="2"></textarea>
						</div>
						<div class="form-row">
							<div class="form-group">
								<label for="edit-arc-status">Status</label>
								<select id="edit-arc-status" bind:value={editingArc.status}>
									{#each arcStatuses as status (status)}
										<option value={status}>{status}</option>
									{/each}
								</select>
							</div>
							<div class="form-group">
								<label for="edit-arc-color">Color</label>
								<input id="edit-arc-color" type="color" bind:value={editingArc.color} />
							</div>
						</div>
						<div class="form-actions">
							<button class="cancel-btn" onclick={() => (editingArc = null)}>Cancel</button>
							<button class="save-btn" onclick={() => updateArc(editingArc!)}>Save</button>
						</div>
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
								<button class="icon-btn" onclick={() => (editingArc = { ...arc })} title="Edit">
									<svg
										width="16"
										height="16"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"
									>
										<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
										<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
									</svg>
								</button>
								<button class="icon-btn danger" onclick={() => deleteArc(arc.id)} title="Delete">
									<svg
										width="16"
										height="16"
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
								</button>
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

	.add-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-md);
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-md);
		font-size: var(--font-size-sm);
		font-weight: 500;
	}

	.add-btn:hover {
		background-color: var(--color-accent-hover);
	}

	.loading,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-xl);
	}

	.empty-state svg {
		opacity: 0.5;
		margin-bottom: var(--spacing-md);
	}

	.empty-state h3 {
		font-size: var(--font-size-lg);
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-sm);
	}

	.primary-btn {
		margin-top: var(--spacing-lg);
		padding: var(--spacing-sm) var(--spacing-lg);
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-md);
		font-weight: 500;
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

	.icon-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.icon-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.icon-btn.danger:hover {
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

	.form-group {
		margin-bottom: var(--spacing-md);
	}

	.form-group label {
		display: block;
		font-size: var(--font-size-xs);
		font-weight: 500;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-xs);
	}

	.form-group input[type='text'],
	.form-group textarea,
	.form-group select {
		width: 100%;
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.form-group input[type='color'] {
		width: 60px;
		height: 32px;
		padding: 2px;
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
	}

	.form-row {
		display: flex;
		gap: var(--spacing-md);
	}

	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
		margin-top: var(--spacing-md);
	}

	.cancel-btn {
		padding: var(--spacing-xs) var(--spacing-md);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
	}

	.cancel-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.save-btn {
		padding: var(--spacing-xs) var(--spacing-md);
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-sm);
		font-weight: 500;
	}

	.save-btn:hover:not(:disabled) {
		background-color: var(--color-accent-hover);
	}

	.save-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
