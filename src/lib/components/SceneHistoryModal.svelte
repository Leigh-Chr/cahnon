<script lang="ts">
	import { historyApi, type SceneHistoryEntry } from '$lib/api';
	import { countWords, formatWordCount } from '$lib/utils';
	import { showSuccess, showError } from '$lib/toast';

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
		onclose?.();
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
		return date.toLocaleString();
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

	function handleKeydown(event: KeyboardEvent) {
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
				<h2 id="history-title">Scene History</h2>
				<button class="close-btn" onclick={close} aria-label="Close">
					<svg
						width="20"
						height="20"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<line x1="18" y1="6" x2="6" y2="18" />
						<line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>

			<div class="modal-content">
				{#if isLoading}
					<div class="loading">Loading history...</div>
				{:else if history.length === 0}
					<div class="empty-state">
						<svg
							width="48"
							height="48"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
						>
							<circle cx="12" cy="12" r="10" />
							<polyline points="12 6 12 12 16 14" />
						</svg>
						<h3>No history yet</h3>
						<p>Scene versions are saved automatically as you write.</p>
					</div>
				{:else}
					<div class="history-layout">
						<div class="history-list">
							<div class="list-header">Versions ({history.length})</div>
							{#each history as entry (entry.id)}
								<button
									class="history-item"
									class:selected={selectedEntry?.id === entry.id}
									onclick={() => (selectedEntry = entry)}
								>
									<div class="item-date">{formatRelativeDate(entry.created_at)}</div>
									<div class="item-meta">
										<span>{formatWordCount(countWords(entry.text))} words</span>
										<span class="full-date">{formatDate(entry.created_at)}</span>
									</div>
								</button>
							{/each}
						</div>

						<div class="preview-panel">
							{#if selectedEntry}
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
					<button class="cancel-btn" onclick={close}>Cancel</button>
					<button class="restore-btn" onclick={restoreVersion} disabled={isRestoring}>
						{#if isRestoring}
							Restoring...
						{:else}
							<svg
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<polyline points="1 4 1 10 7 10" />
								<path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
							</svg>
							Restore This Version
						{/if}
					</button>
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

	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.modal-content {
		flex: 1;
		overflow: hidden;
		display: flex;
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

	.cancel-btn {
		padding: var(--spacing-sm) var(--spacing-lg);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-md);
	}

	.cancel-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.restore-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-sm) var(--spacing-lg);
		background-color: var(--color-accent);
		color: var(--text-on-accent);
		border-radius: var(--border-radius-md);
		font-weight: 500;
	}

	.restore-btn:hover:not(:disabled) {
		background-color: var(--color-accent-hover);
	}

	.restore-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
