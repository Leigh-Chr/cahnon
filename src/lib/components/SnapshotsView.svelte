<script lang="ts">
	import { snapshotApi, type Snapshot } from '$lib/api';
	import { showSuccess, showError } from '$lib/toast';
	import { Icon, Button, EmptyState, LoadingState, FormGroup, FormActions } from './ui';

	// Type for parsed snapshot data
	interface SnapshotData {
		chapters?: unknown[];
		scenes?: unknown[];
		bible_entries?: unknown[];
		arcs?: unknown[];
	}

	interface Props {
		isOpen?: boolean;
		onclose?: () => void;
	}

	let { isOpen = false, onclose }: Props = $props();

	let snapshots = $state<Snapshot[]>([]);
	let isLoading = $state(true);
	let isCreating = $state(false);
	let showCreateForm = $state(false);
	let selectedSnapshot = $state<Snapshot | null>(null);

	// Create form
	let newName = $state('');
	let newDescription = $state('');
	let newType = $state('manual');

	const snapshotTypes = [
		{ value: 'manual', label: 'Manual Backup' },
		{ value: 'milestone', label: 'Milestone' },
		{ value: 'pre_bulk', label: 'Pre-bulk Operation' },
	];

	$effect(() => {
		if (isOpen) {
			loadSnapshots();
		}
	});

	async function loadSnapshots() {
		isLoading = true;
		try {
			snapshots = await snapshotApi.getAll();
		} catch (e) {
			console.error('Failed to load snapshots:', e);
		}
		isLoading = false;
	}

	async function createSnapshot() {
		if (!newName.trim()) return;
		isCreating = true;
		try {
			const snapshot = await snapshotApi.create(
				newName.trim(),
				newDescription.trim() || undefined,
				newType
			);
			snapshots = [snapshot, ...snapshots];
			resetCreateForm();
			showSuccess('Snapshot created successfully');
		} catch (e) {
			console.error('Failed to create snapshot:', e);
			showError('Failed to create snapshot');
		}
		isCreating = false;
	}

	function resetCreateForm() {
		showCreateForm = false;
		newName = '';
		newDescription = '';
		newType = 'manual';
	}

	async function viewSnapshot(snapshot: Snapshot) {
		selectedSnapshot = snapshot;
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleString();
	}

	function formatRelativeDate(dateStr: string): string {
		const date = new Date(dateStr);
		const now = new Date();
		const diffMs = now.getTime() - date.getTime();
		const diffMins = Math.floor(diffMs / 60000);
		const diffHours = Math.floor(diffMs / 3600000);
		const diffDays = Math.floor(diffMs / 86400000);

		if (diffMins < 1) return 'Just now';
		if (diffMins < 60) return `${diffMins} min ago`;
		if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
		if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
		return date.toLocaleDateString();
	}

	function getTypeLabel(type: string): string {
		const t = snapshotTypes.find((st) => st.value === type);
		return t?.label || type;
	}

	function _getTypeIcon(type: string): string {
		const icons: Record<string, string> = {
			manual:
				'M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z M17 21v-8H7v8 M7 3v5h8',
			milestone: 'M3 3v18h18 M18.7 8l-5.1 5.2-2.8-2.7L7 14.3',
			pre_bulk:
				'M12 2v4 M12 18v4 M4.93 4.93l2.83 2.83 M16.24 16.24l2.83 2.83 M2 12h4 M18 12h4 M4.93 19.07l2.83-2.83 M16.24 7.76l2.83-2.83',
		};
		return icons[type] || icons.manual;
	}

	function close() {
		selectedSnapshot = null;
		onclose?.();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			if (selectedSnapshot) {
				selectedSnapshot = null;
			} else {
				close();
			}
		}
	}

	function parseSnapshotData(data: string): SnapshotData | null {
		try {
			return JSON.parse(data) as SnapshotData;
		} catch {
			return null;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="panel-overlay" onclick={close} role="presentation">
		<div
			class="panel"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="snapshots-title"
			tabindex="-1"
		>
			<div class="panel-header">
				<h2 id="snapshots-title">Snapshots</h2>
				<div class="header-actions">
					<Button variant="primary" onclick={() => (showCreateForm = true)}>
						<Icon name="plus" size={16} />
						New Snapshot
					</Button>
					<Button variant="icon" onclick={close} title="Close">
						<Icon name="close" size={20} />
					</Button>
				</div>
			</div>

			<div class="panel-content">
				{#if showCreateForm}
					<div class="create-form">
						<h3>Create Snapshot</h3>
						<FormGroup label="Name" id="snapshot-name">
							<input
								id="snapshot-name"
								type="text"
								bind:value={newName}
								placeholder="e.g., Before major revision..."
							/>
						</FormGroup>
						<FormGroup label="Description (optional)" id="snapshot-description">
							<textarea
								id="snapshot-description"
								bind:value={newDescription}
								placeholder="Describe what this snapshot captures..."
								rows="2"
							></textarea>
						</FormGroup>
						<FormGroup label="Type" id="snapshot-type">
							<select id="snapshot-type" bind:value={newType}>
								{#each snapshotTypes as type (type.value)}
									<option value={type.value}>{type.label}</option>
								{/each}
							</select>
						</FormGroup>
						<FormActions>
							<Button variant="ghost" onclick={resetCreateForm}>Cancel</Button>
							<Button
								variant="primary"
								onclick={createSnapshot}
								disabled={!newName.trim() || isCreating}
							>
								{isCreating ? 'Creating...' : 'Create Snapshot'}
							</Button>
						</FormActions>
					</div>
				{/if}

				{#if selectedSnapshot}
					<div class="snapshot-detail">
						<Button variant="ghost" onclick={() => (selectedSnapshot = null)}>
							<Icon name="chevron-left" size={16} />
							Back to list
						</Button>

						<div class="detail-header">
							<h3>{selectedSnapshot.name}</h3>
							<span class="detail-type">{getTypeLabel(selectedSnapshot.snapshot_type)}</span>
						</div>

						{#if selectedSnapshot.description}
							<p class="detail-description">{selectedSnapshot.description}</p>
						{/if}

						<p class="detail-date">Created: {formatDate(selectedSnapshot.created_at)}</p>

						{#if selectedSnapshot.data}
							{@const data = parseSnapshotData(selectedSnapshot.data)}
							{#if data}
								<div class="snapshot-contents">
									<h4>Contents</h4>
									<div class="contents-summary">
										{#if data.chapters}
											<div class="summary-item">
												<span class="count">{data.chapters.length}</span>
												<span class="label">Chapters</span>
											</div>
										{/if}
										{#if data.scenes}
											<div class="summary-item">
												<span class="count">{data.scenes.length}</span>
												<span class="label">Scenes</span>
											</div>
										{/if}
										{#if data.bible_entries}
											<div class="summary-item">
												<span class="count">{data.bible_entries.length}</span>
												<span class="label">Bible Entries</span>
											</div>
										{/if}
										{#if data.arcs}
											<div class="summary-item">
												<span class="count">{data.arcs.length}</span>
												<span class="label">Arcs</span>
											</div>
										{/if}
									</div>
								</div>
							{/if}
						{/if}

						<div class="detail-actions">
							<Button variant="primary" disabled>
								<Icon name="refresh" size={16} />
								Restore Full Project
							</Button>
						</div>
					</div>
				{:else if isLoading}
					<LoadingState message="Loading snapshots..." />
				{:else if snapshots.length === 0 && !showCreateForm}
					<EmptyState
						icon="image"
						title="No snapshots yet"
						description="Create snapshots to save your project state at important milestones."
						actionLabel="Create First Snapshot"
						onaction={() => (showCreateForm = true)}
					/>
				{:else if !showCreateForm}
					<div class="snapshots-list">
						{#each snapshots as snapshot (snapshot.id)}
							<button class="snapshot-item" onclick={() => viewSnapshot(snapshot)}>
								<div class="snapshot-icon">
									<Icon name="file" size={20} />
								</div>
								<div class="snapshot-info">
									<div class="snapshot-name">{snapshot.name}</div>
									<div class="snapshot-meta">
										<span class="type">{getTypeLabel(snapshot.snapshot_type)}</span>
										<span class="date">{formatRelativeDate(snapshot.created_at)}</span>
									</div>
									{#if snapshot.description}
										<div class="snapshot-description">{snapshot.description}</div>
									{/if}
								</div>
								<div class="snapshot-arrow">
									<Icon name="chevron-right" size={16} />
								</div>
							</button>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.panel-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: var(--overlay-backdrop);
		display: flex;
		justify-content: flex-end;
		z-index: 1000;
	}

	.panel {
		background-color: var(--color-bg-primary);
		width: 450px;
		max-width: 90%;
		height: 100%;
		display: flex;
		flex-direction: column;
		box-shadow: -4px 0 20px oklch(0% 0 0 / 10%);
	}

	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.panel-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.panel-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
	}

	.create-form {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-lg);
		margin-bottom: var(--spacing-lg);
	}

	.create-form h3 {
		font-size: var(--font-size-base);
		font-weight: 600;
		margin-bottom: var(--spacing-md);
	}

	.snapshots-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.snapshot-item {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-md);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		text-align: left;
		transition: all var(--transition-fast);
	}

	.snapshot-item:hover {
		border-color: var(--color-accent);
		background-color: var(--color-bg-hover);
	}

	.snapshot-icon {
		width: 40px;
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		flex-shrink: 0;
	}

	.snapshot-info {
		flex: 1;
		min-width: 0;
	}

	.snapshot-name {
		font-weight: 500;
		color: var(--color-text-primary);
		margin-bottom: var(--spacing-xs);
	}

	.snapshot-meta {
		display: flex;
		gap: var(--spacing-md);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.snapshot-description {
		margin-top: var(--spacing-xs);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.snapshot-arrow {
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	/* Snapshot detail view */
	.snapshot-detail {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.detail-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
	}

	.detail-header h3 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.detail-type {
		padding: 2px 8px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.detail-description {
		color: var(--color-text-secondary);
	}

	.detail-date {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.snapshot-contents {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
	}

	.snapshot-contents h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		margin-bottom: var(--spacing-md);
	}

	.contents-summary {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--spacing-md);
	}

	.summary-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
	}

	.summary-item .count {
		font-size: var(--font-size-xl);
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.summary-item .label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.detail-actions {
		margin-top: var(--spacing-lg);
	}
</style>
