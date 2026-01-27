<script lang="ts">
	import { type Snapshot, snapshotApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError, showSuccess } from '$lib/toast';
	import { nativeConfirm } from '$lib/utils/native-dialog';

	import { Button, EmptyState, FormActions, FormGroup, Icon, LoadingState } from './ui';

	// Type for parsed snapshot data
	interface SnapshotData {
		chapters?: Array<{ id: string; title: string }>;
		scenes?: Array<{ id: string; title: string; chapter_id: string }>;
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
	let isRestoring = $state(false);
	let showCreateForm = $state(false);
	let selectedSnapshot = $state<Snapshot | null>(null);
	let showScenePicker = $state(false);
	let isRestoringScene = $state(false);

	// Compare mode
	let compareMode = $state(false);
	let compareBase = $state<Snapshot | null>(null);
	let compareTarget = $state<Snapshot | null>(null);

	interface DiffResult {
		addedScenes: Array<{ id: string; title: string }>;
		removedScenes: Array<{ id: string; title: string }>;
		modifiedScenes: Array<{ id: string; title: string }>;
		addedChapters: Array<{ id: string; title: string }>;
		removedChapters: Array<{ id: string; title: string }>;
	}

	let diffResult = $state<DiffResult | null>(null);

	function computeDiff(baseData: SnapshotData, targetData: SnapshotData): DiffResult {
		const baseScenes = new Map((baseData.scenes ?? []).map((s) => [s.id, s]));
		const targetScenes = new Map((targetData.scenes ?? []).map((s) => [s.id, s]));
		const baseChapters = new Map(
			(baseData.chapters ?? []).map((c) => [c.id, c as { id: string; title: string }])
		);
		const targetChapters = new Map(
			(targetData.chapters ?? []).map((c) => [c.id, c as { id: string; title: string }])
		);

		const addedScenes: Array<{ id: string; title: string }> = [];
		const removedScenes: Array<{ id: string; title: string }> = [];
		const modifiedScenes: Array<{ id: string; title: string }> = [];

		for (const [id, scene] of targetScenes) {
			if (!baseScenes.has(id)) {
				addedScenes.push({ id, title: scene.title });
			}
		}
		for (const [id, scene] of baseScenes) {
			if (!targetScenes.has(id)) {
				removedScenes.push({ id, title: scene.title });
			} else {
				const target = targetScenes.get(id)!;
				if (scene.title !== target.title) {
					modifiedScenes.push({ id, title: `${scene.title} → ${target.title}` });
				}
			}
		}

		const addedChapters: Array<{ id: string; title: string }> = [];
		const removedChapters: Array<{ id: string; title: string }> = [];

		for (const [id, chapter] of targetChapters) {
			if (!baseChapters.has(id)) {
				addedChapters.push({ id, title: chapter.title });
			}
		}
		for (const [id, chapter] of baseChapters) {
			if (!targetChapters.has(id)) {
				removedChapters.push({ id, title: chapter.title });
			}
		}

		return { addedScenes, removedScenes, modifiedScenes, addedChapters, removedChapters };
	}

	function startCompare() {
		compareMode = true;
		compareBase = null;
		compareTarget = null;
		diffResult = null;
	}

	function selectForCompare(snapshot: Snapshot) {
		if (!compareBase) {
			compareBase = snapshot;
		} else if (!compareTarget && snapshot.id !== compareBase.id) {
			compareTarget = snapshot;
			// Compute diff
			const baseData = parseSnapshotData(compareBase.data);
			const targetData = parseSnapshotData(snapshot.data);
			if (baseData && targetData) {
				diffResult = computeDiff(baseData, targetData);
			}
		}
	}

	function exitCompare() {
		compareMode = false;
		compareBase = null;
		compareTarget = null;
		diffResult = null;
	}

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
			showError('Failed to load snapshots');
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

	function close() {
		selectedSnapshot = null;
		onclose?.();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!isOpen) return;
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

	async function deleteSnapshot(snapshot: Snapshot) {
		const confirmed = await nativeConfirm(
			`Are you sure you want to delete the snapshot "${snapshot.name}"?\n\nThis action cannot be undone.`,
			'Delete Snapshot'
		);
		if (!confirmed) {
			return;
		}

		try {
			await snapshotApi.delete(snapshot.id);
			snapshots = snapshots.filter((s) => s.id !== snapshot.id);
			selectedSnapshot = null;
			showSuccess('Snapshot deleted');
		} catch (e) {
			console.error('Failed to delete snapshot:', e);
			showError('Failed to delete snapshot');
		}
	}

	async function restoreSceneFromSnapshot(sceneId: string, sceneTitle: string) {
		if (!selectedSnapshot) return;
		const confirmed = await nativeConfirm(
			`Restore scene "${sceneTitle}" from this snapshot?\n\n` +
				'This will replace the current scene content with the snapshot version.',
			'Restore Scene'
		);
		if (!confirmed) {
			return;
		}

		isRestoringScene = true;
		try {
			await snapshotApi.restoreScene(selectedSnapshot.id, sceneId);
			await appState.reloadScenes();
			showSuccess(`Scene "${sceneTitle}" restored from snapshot`);
			showScenePicker = false;
		} catch (e) {
			console.error('Failed to restore scene:', e);
			showError('Failed to restore scene from snapshot');
		}
		isRestoringScene = false;
	}

	async function cleanupExpired() {
		if (
			!(await nativeConfirm(
				'Remove all expired snapshots? This cannot be undone.',
				'Cleanup Snapshots'
			))
		)
			return;
		try {
			const removed = await snapshotApi.cleanupExpired();
			if (removed > 0) {
				await loadSnapshots();
				selectedSnapshot = null;
				showSuccess(`Removed ${removed} expired snapshot${removed > 1 ? 's' : ''}`);
			} else {
				showSuccess('No expired snapshots to clean up');
			}
		} catch (e) {
			console.error('Failed to cleanup snapshots:', e);
			showError('Failed to cleanup expired snapshots');
		}
	}

	async function restoreSnapshot(snapshot: Snapshot) {
		const confirmed = await nativeConfirm(
			`Are you sure you want to restore the snapshot "${snapshot.name}"?\n\n` +
				'This will replace ALL current project data (chapters, scenes, bible entries, arcs, events) ' +
				'with the data from this snapshot.\n\n' +
				'An automatic backup will be created before restoring.',
			'Restore Snapshot'
		);
		if (!confirmed) {
			return;
		}

		isRestoring = true;
		try {
			await snapshotApi.restore(snapshot.id);
			// Reload all project data
			await appState.loadManuscript();
			await appState.loadBible();
			await appState.loadStats();
			// Reload snapshots list (will include the auto-backup)
			await loadSnapshots();
			selectedSnapshot = null;
			showSuccess('Snapshot restored successfully');
			close();
		} catch (e) {
			console.error('Failed to restore snapshot:', e);
			showError('Failed to restore snapshot');
		}
		isRestoring = false;
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
					{#if compareMode}
						<Button variant="ghost" onclick={exitCompare}>Exit Compare</Button>
					{:else}
						{#if snapshots.length > 0}
							<Button variant="ghost" onclick={cleanupExpired} title="Remove expired snapshots">
								Cleanup
							</Button>
						{/if}
						<Button variant="secondary" onclick={startCompare} disabled={snapshots.length < 2}>
							Compare
						</Button>
						<Button variant="primary" onclick={() => (showCreateForm = true)}>
							<Icon name="plus" size={16} />
							New Snapshot
						</Button>
					{/if}
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

						{#if showScenePicker && selectedSnapshot?.data}
							{@const pickerData = parseSnapshotData(selectedSnapshot.data)}
							{#if pickerData?.scenes}
								<div class="scene-picker">
									<h4>Select scene to restore</h4>
									<div class="scene-picker-list">
										{#each pickerData.scenes as scene (scene.id)}
											<button
												class="scene-pick-btn"
												onclick={() => restoreSceneFromSnapshot(scene.id, scene.title)}
												disabled={isRestoringScene}
											>
												<span class="scene-pick-title">{scene.title}</span>
												<Icon name="refresh" size={14} />
											</button>
										{/each}
									</div>
									<Button variant="ghost" onclick={() => (showScenePicker = false)}>Cancel</Button>
								</div>
							{/if}
						{/if}

						<div class="detail-actions">
							<Button
								variant="primary"
								onclick={() => selectedSnapshot && restoreSnapshot(selectedSnapshot)}
								disabled={isRestoring}
							>
								<Icon name="refresh" size={16} />
								{isRestoring ? 'Restoring...' : 'Restore Full Project'}
							</Button>
							<Button
								variant="secondary"
								onclick={() => (showScenePicker = !showScenePicker)}
								disabled={isRestoring}
							>
								<Icon name="file" size={16} />
								Restore Single Scene
							</Button>
							<Button
								variant="danger"
								onclick={() => selectedSnapshot && deleteSnapshot(selectedSnapshot)}
								disabled={isRestoring}
							>
								<Icon name="trash" size={16} />
								Delete Snapshot
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
				{:else if compareMode}
					<div class="compare-view">
						{#if !diffResult}
							<div class="compare-instructions">
								{#if !compareBase}
									<p>Select the <strong>base</strong> snapshot to compare from:</p>
								{:else}
									<p>
										Base: <strong>{compareBase.name}</strong><br />
										Now select the <strong>target</strong> snapshot:
									</p>
								{/if}
							</div>
							<div class="snapshots-list">
								{#each snapshots as snapshot (snapshot.id)}
									<button
										class="snapshot-item"
										class:selected-compare={compareBase?.id === snapshot.id}
										onclick={() => selectForCompare(snapshot)}
										disabled={compareBase?.id === snapshot.id}
									>
										<div class="snapshot-icon">
											<Icon name="file" size={20} />
										</div>
										<div class="snapshot-info">
											<div class="snapshot-name">{snapshot.name}</div>
											<div class="snapshot-meta">
												<span class="type">{getTypeLabel(snapshot.snapshot_type)}</span>
												<span class="date">{formatRelativeDate(snapshot.created_at)}</span>
											</div>
										</div>
										{#if compareBase?.id === snapshot.id}
											<span class="compare-badge">Base</span>
										{/if}
									</button>
								{/each}
							</div>
						{:else}
							<div class="diff-header">
								<h3>Comparison</h3>
								<p class="diff-subtitle">
									{compareBase?.name} → {compareTarget?.name}
								</p>
							</div>
							<div class="diff-sections">
								{#if diffResult.addedChapters.length > 0}
									<div class="diff-section">
										<h4 class="diff-added">+ Added Chapters ({diffResult.addedChapters.length})</h4>
										{#each diffResult.addedChapters as ch (ch.id)}
											<div class="diff-item added">{ch.title}</div>
										{/each}
									</div>
								{/if}
								{#if diffResult.removedChapters.length > 0}
									<div class="diff-section">
										<h4 class="diff-removed">
											- Removed Chapters ({diffResult.removedChapters.length})
										</h4>
										{#each diffResult.removedChapters as ch (ch.id)}
											<div class="diff-item removed">{ch.title}</div>
										{/each}
									</div>
								{/if}
								{#if diffResult.addedScenes.length > 0}
									<div class="diff-section">
										<h4 class="diff-added">+ Added Scenes ({diffResult.addedScenes.length})</h4>
										{#each diffResult.addedScenes as scene (scene.id)}
											<div class="diff-item added">{scene.title}</div>
										{/each}
									</div>
								{/if}
								{#if diffResult.removedScenes.length > 0}
									<div class="diff-section">
										<h4 class="diff-removed">
											- Removed Scenes ({diffResult.removedScenes.length})
										</h4>
										{#each diffResult.removedScenes as scene (scene.id)}
											<div class="diff-item removed">{scene.title}</div>
										{/each}
									</div>
								{/if}
								{#if diffResult.modifiedScenes.length > 0}
									<div class="diff-section">
										<h4 class="diff-modified">
											~ Modified Scenes ({diffResult.modifiedScenes.length})
										</h4>
										{#each diffResult.modifiedScenes as scene (scene.id)}
											<div class="diff-item modified">{scene.title}</div>
										{/each}
									</div>
								{/if}
								{#if diffResult.addedChapters.length === 0 && diffResult.removedChapters.length === 0 && diffResult.addedScenes.length === 0 && diffResult.removedScenes.length === 0 && diffResult.modifiedScenes.length === 0}
									<p class="no-diff">No structural differences found between snapshots.</p>
								{/if}
							</div>
							<Button variant="ghost" onclick={exitCompare}>Done</Button>
						{/if}
					</div>
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

	.snapshot-item:hover:not(:disabled) {
		border-color: var(--color-accent);
		background-color: var(--color-bg-hover);
	}

	.snapshot-item:disabled {
		opacity: 0.6;
		cursor: default;
	}

	.snapshot-item.selected-compare {
		border-color: var(--color-accent);
		background-color: var(--color-accent-light);
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
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-sm);
	}

	.scene-picker {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
	}

	.scene-picker h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		margin-bottom: var(--spacing-sm);
	}

	.scene-picker-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 200px;
		overflow-y: auto;
		margin-bottom: var(--spacing-sm);
	}

	.scene-pick-btn {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--spacing-sm);
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		text-align: left;
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		transition: background-color var(--transition-fast);
	}

	.scene-pick-btn:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
	}

	.scene-pick-btn:disabled {
		opacity: 0.5;
	}

	.scene-pick-title {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Compare mode */
	.compare-view {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.compare-instructions {
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.compare-badge {
		padding: 2px 8px;
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		font-weight: 600;
	}

	.diff-header h3 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.diff-subtitle {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.diff-sections {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.diff-section {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
	}

	.diff-section h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		margin-bottom: var(--spacing-sm);
	}

	.diff-added {
		color: var(--color-success);
	}

	.diff-removed {
		color: var(--color-error);
	}

	.diff-modified {
		color: var(--color-warning, #e09100);
	}

	.diff-item {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border-radius: var(--border-radius-sm);
	}

	.diff-item.added {
		background-color: oklch(0.95 0.05 145);
	}

	.diff-item.removed {
		background-color: oklch(0.95 0.05 25);
	}

	.diff-item.modified {
		background-color: oklch(0.95 0.05 85);
	}

	.no-diff {
		text-align: center;
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
		padding: var(--spacing-lg);
	}
</style>
