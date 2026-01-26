<script lang="ts">
	import { historyApi, type SceneHistoryEntry } from '$lib/api';
	import { showError, showSuccess } from '$lib/toast';
	import { countWords, formatWordCount } from '$lib/utils';

	import { Button, EmptyState, Icon, LoadingState } from './ui';

	interface Props {
		isOpen?: boolean;
		sceneId: string;
		currentText: string;
		onclose?: () => void;
		onrestored?: () => void;
	}

	let { isOpen = $bindable(false), sceneId, currentText, onclose, onrestored }: Props = $props();

	let history = $state<SceneHistoryEntry[]>([]);
	let isLoading = $state(true);
	let selectedEntry = $state<SceneHistoryEntry | null>(null);
	let isRestoring = $state(false);
	let compareMode = $state(false);
	let compareEntryA = $state<SceneHistoryEntry | null>(null);
	let compareEntryB = $state<SceneHistoryEntry | null>(null);
	let diffResult = $state<{ text_a: string; text_b: string } | null>(null);
	let isLoadingDiff = $state(false);

	$effect(() => {
		if (isOpen && sceneId) {
			loadHistory();
		}
	});

	async function loadHistory() {
		isLoading = true;
		try {
			history = await historyApi.getSceneHistory(sceneId);
		} catch (e) {
			console.error('Failed to load history:', e);
		}
		isLoading = false;
	}

	function close() {
		isOpen = false;
		selectedEntry = null;
		compareMode = false;
		compareEntryA = null;
		compareEntryB = null;
		diffResult = null;
		onclose?.();
	}

	function toggleCompareMode() {
		compareMode = !compareMode;
		if (!compareMode) {
			compareEntryA = null;
			compareEntryB = null;
			diffResult = null;
		}
	}

	function selectForCompare(entry: SceneHistoryEntry) {
		if (!compareEntryA) {
			compareEntryA = entry;
		} else if (!compareEntryB && entry.id !== compareEntryA.id) {
			compareEntryB = entry;
			loadDiff();
		} else {
			// Reset selection
			compareEntryA = entry;
			compareEntryB = null;
			diffResult = null;
		}
	}

	async function loadDiff() {
		if (!compareEntryA || !compareEntryB) return;
		isLoadingDiff = true;
		try {
			diffResult = await historyApi.compareVersions(sceneId, compareEntryA.id, compareEntryB.id);
		} catch (e) {
			console.error('Failed to compare versions:', e);
			showError('Failed to compare versions');
		}
		isLoadingDiff = false;
	}

	async function restoreVersion() {
		if (!selectedEntry) return;
		isRestoring = true;
		try {
			await historyApi.restoreVersion(sceneId, selectedEntry.id);
			onrestored?.();
			showSuccess('Scene restored to previous version');
			close();
		} catch (e) {
			console.error('Failed to restore version:', e);
			showError('Failed to restore version');
		}
		isRestoring = false;
	}

	function formatDate(dateStr: string): string {
		const date = new Date(dateStr);
		if (isNaN(date.getTime())) return dateStr;
		return date.toLocaleString();
	}

	function formatRelativeDate(dateStr: string): string {
		const date = new Date(dateStr);
		if (isNaN(date.getTime())) return dateStr;
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

	function handleKeydown(event: KeyboardEvent) {
		if (!isOpen) return;
		if (event.key === 'Escape') {
			close();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="modal-overlay" onclick={close} role="presentation">
		<div
			class="modal"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="history-title"
			tabindex="-1"
		>
			<div class="modal-header">
				<div class="header-left">
					<h2 id="history-title">Scene History</h2>
					{#if history.length >= 2}
						<Button
							variant={compareMode ? 'primary' : 'ghost'}
							size="sm"
							onclick={toggleCompareMode}
						>
							{compareMode ? 'Exit Compare' : 'Compare'}
						</Button>
					{/if}
				</div>
				<Button variant="icon" onclick={close} title="Close">
					<Icon name="close" size={20} />
				</Button>
			</div>

			<div class="modal-content">
				{#if isLoading}
					<LoadingState message="Loading history..." />
				{:else if history.length === 0}
					<EmptyState
						icon="clock"
						title="No history yet"
						description="Scene versions are saved automatically as you write."
					/>
				{:else}
					<div class="history-layout">
						<div class="history-list">
							<div class="list-header">
								{#if compareMode}
									Select two versions to compare
								{:else}
									Versions ({history.length})
								{/if}
							</div>
							{#each history as entry (entry.id)}
								<button
									class="history-item"
									class:selected={!compareMode && selectedEntry?.id === entry.id}
									class:compare-a={compareMode && compareEntryA?.id === entry.id}
									class:compare-b={compareMode && compareEntryB?.id === entry.id}
									onclick={() => {
										if (compareMode) {
											selectForCompare(entry);
										} else {
											selectedEntry = entry;
										}
									}}
								>
									{#if compareMode}
										<span class="compare-badge">
											{#if compareEntryA?.id === entry.id}
												A
											{:else if compareEntryB?.id === entry.id}
												B
											{/if}
										</span>
									{/if}
									<div class="item-date">{formatRelativeDate(entry.created_at)}</div>
									<div class="item-meta">
										<span>{formatWordCount(countWords(entry.text))} words</span>
										<span class="full-date">{formatDate(entry.created_at)}</span>
									</div>
								</button>
							{/each}
						</div>

						<div class="preview-panel">
							{#if compareMode}
								{#if isLoadingDiff}
									<LoadingState message="Comparing versions..." />
								{:else if diffResult}
									<div class="preview-header">
										<div class="preview-info">
											<span class="preview-date">Version A vs Version B</span>
											<span class="preview-words">
												{formatWordCount(countWords(diffResult.text_a))} vs {formatWordCount(
													countWords(diffResult.text_b)
												)} words
											</span>
										</div>
									</div>
									<div class="diff-view">
										<div class="diff-column">
											<div class="diff-column-header">Version A</div>
											<div class="diff-column-content">
												<pre>{diffResult.text_a || '(empty)'}</pre>
											</div>
										</div>
										<div class="diff-column">
											<div class="diff-column-header">Version B</div>
											<div class="diff-column-content">
												<pre>{diffResult.text_b || '(empty)'}</pre>
											</div>
										</div>
									</div>
								{:else}
									<div class="no-selection">
										<p>
											{#if !compareEntryA}
												Select the first version (A)
											{:else}
												Now select the second version (B)
											{/if}
										</p>
									</div>
								{/if}
							{:else if selectedEntry}
								<div class="preview-header">
									<div class="preview-info">
										<span class="preview-date">{formatDate(selectedEntry.created_at)}</span>
										<span class="preview-words"
											>{formatWordCount(countWords(selectedEntry.text))} words</span
										>
									</div>
									<div class="preview-diff">
										{#if countWords(selectedEntry.text) !== countWords(currentText)}
											<span
												class:positive={countWords(selectedEntry.text) > countWords(currentText)}
												class:negative={countWords(selectedEntry.text) < countWords(currentText)}
											>
												{countWords(selectedEntry.text) > countWords(currentText)
													? '+'
													: ''}{countWords(selectedEntry.text) - countWords(currentText)} words vs current
											</span>
										{:else}
											<span>Same as current</span>
										{/if}
									</div>
								</div>
								<div class="preview-content">
									<pre>{selectedEntry.text || '(empty)'}</pre>
								</div>
							{:else}
								<div class="no-selection">
									<p>Select a version to preview</p>
								</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>

			{#if selectedEntry}
				<div class="modal-footer">
					<Button variant="ghost" onclick={close}>Cancel</Button>
					<Button variant="primary" onclick={restoreVersion} disabled={isRestoring}>
						{#if isRestoring}
							Restoring...
						{:else}
							<Icon name="restore" size={16} />
							Restore This Version
						{/if}
					</Button>
				</div>
			{/if}
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

	.modal {
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		width: 90%;
		max-width: 900px;
		height: 80vh;
		display: flex;
		flex-direction: column;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.modal-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
	}

	.modal-content {
		flex: 1;
		overflow: hidden;
		display: flex;
	}

	.history-layout {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.history-list {
		width: 280px;
		border-right: 1px solid var(--color-border);
		overflow-y: auto;
		flex-shrink: 0;
	}

	.list-header {
		padding: var(--spacing-md);
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		border-bottom: 1px solid var(--color-border);
		background-color: var(--color-bg-secondary);
		position: sticky;
		top: 0;
	}

	.history-item {
		display: block;
		width: 100%;
		padding: var(--spacing-md);
		text-align: left;
		border-bottom: 1px solid var(--color-border-light);
		transition: background-color var(--transition-fast);
	}

	.history-item:hover {
		background-color: var(--color-bg-hover);
	}

	.history-item.selected {
		background-color: var(--color-accent-light);
	}

	.item-date {
		font-weight: 500;
		color: var(--color-text-primary);
		margin-bottom: var(--spacing-xs);
	}

	.item-meta {
		display: flex;
		justify-content: space-between;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.full-date {
		display: none;
	}

	.history-item:hover .full-date,
	.history-item.selected .full-date {
		display: inline;
	}

	.preview-panel {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.preview-header {
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
		background-color: var(--color-bg-secondary);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.preview-info {
		display: flex;
		gap: var(--spacing-md);
		font-size: var(--font-size-sm);
	}

	.preview-date {
		color: var(--color-text-primary);
		font-weight: 500;
	}

	.preview-words {
		color: var(--color-text-muted);
	}

	.preview-diff {
		font-size: var(--font-size-sm);
	}

	.preview-diff .positive {
		color: var(--color-success);
	}

	.preview-diff .negative {
		color: var(--color-error);
	}

	.preview-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
	}

	.preview-content pre {
		font-family: var(--font-serif);
		font-size: var(--font-size-base);
		line-height: 1.8;
		color: var(--color-text-primary);
		white-space: pre-wrap;
		word-break: break-word;
		margin: 0;
	}

	.no-selection {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
		color: var(--color-text-muted);
	}

	.modal-footer {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
		padding: var(--spacing-md) var(--spacing-lg);
		border-top: 1px solid var(--color-border);
	}

	/* Compare mode styles */
	.compare-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		font-size: var(--font-size-xs);
		font-weight: 700;
		flex-shrink: 0;
	}

	.history-item.compare-a {
		background-color: var(--color-accent-light);
	}

	.history-item.compare-a .compare-badge {
		background-color: var(--color-accent);
		color: white;
	}

	.history-item.compare-b {
		background-color: oklch(80% 0.1 150);
	}

	.history-item.compare-b .compare-badge {
		background-color: oklch(50% 0.15 150);
		color: white;
	}

	.diff-view {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.diff-column {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.diff-column:first-child {
		border-right: 1px solid var(--color-border);
	}

	.diff-column-header {
		padding: var(--spacing-sm) var(--spacing-md);
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		background-color: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border-light);
	}

	.diff-column-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-md);
	}

	.diff-column-content pre {
		font-family: var(--font-serif);
		font-size: var(--font-size-sm);
		line-height: 1.8;
		color: var(--color-text-primary);
		white-space: pre-wrap;
		word-break: break-word;
		margin: 0;
	}
</style>
