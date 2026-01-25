<script lang="ts">
	import { cutApi, type Cut } from '$lib/api';
	import { countWords, formatWordCount } from '$lib/utils';

	interface Props {
		isOpen?: boolean;
		onInsert?: ((text: string) => void) | null;
	}

	let { isOpen = $bindable(false), onInsert = null }: Props = $props();

	let cuts = $state<Cut[]>([]);
	let isLoading = $state(true);

	$effect(() => {
		if (isOpen) {
			loadCuts();
		}
	});

	async function loadCuts() {
		isLoading = true;
		try {
			cuts = await cutApi.getAll();
		} catch (e) {
			console.error('Failed to load cuts:', e);
		}
		isLoading = false;
	}

	async function deleteCut(cutId: string) {
		try {
			await cutApi.delete(cutId);
			cuts = cuts.filter((c) => c.id !== cutId);
		} catch (e) {
			console.error('Failed to delete cut:', e);
		}
	}

	function insertCut(cut: Cut) {
		if (onInsert) {
			onInsert(cut.text);
		}
	}

	function copyToClipboard(text: string) {
		navigator.clipboard.writeText(text);
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleString();
	}

	function close() {
		isOpen = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			close();
		}
	}

	function handleOverlayClick() {
		close();
	}

	function handlePanelClick(event: MouseEvent) {
		event.stopPropagation();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<div class="panel-overlay" onclick={handleOverlayClick} role="presentation">
		<div
			class="panel"
			onclick={handlePanelClick}
			role="dialog"
			aria-modal="true"
			aria-labelledby="cuts-title"
			tabindex="-1"
		>
			<div class="panel-header">
				<h2 id="cuts-title">Cut Library</h2>
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

			<div class="panel-content">
				{#if isLoading}
					<div class="loading">Loading cuts...</div>
				{:else if cuts.length === 0}
					<div class="empty-state">
						<svg
							width="48"
							height="48"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.5"
						>
							<circle cx="12" cy="12" r="3" />
							<path
								d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
							/>
						</svg>
						<h3>No cuts yet</h3>
						<p>Text you cut from scenes will be saved here for later use.</p>
					</div>
				{:else}
					<div class="cuts-list">
						{#each cuts as cut (cut.id)}
							<div class="cut-item">
								<div class="cut-content">
									<pre>{cut.text}</pre>
								</div>
								<div class="cut-footer">
									<div class="cut-meta">
										<span class="word-count">{formatWordCount(countWords(cut.text))} words</span>
										<span class="date">{formatDate(cut.created_at)}</span>
									</div>
									<div class="cut-actions">
										<button
											class="action-btn"
											onclick={() => copyToClipboard(cut.text)}
											title="Copy to clipboard"
										>
											<svg
												width="16"
												height="16"
												viewBox="0 0 24 24"
												fill="none"
												stroke="currentColor"
												stroke-width="2"
											>
												<rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
												<path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
											</svg>
										</button>
										{#if onInsert}
											<button
												class="action-btn primary"
												onclick={() => insertCut(cut)}
												title="Insert at cursor"
											>
												<svg
													width="16"
													height="16"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="2"
												>
													<path d="M12 5v14" />
													<path d="M5 12h14" />
												</svg>
											</button>
										{/if}
										<button
											class="action-btn danger"
											onclick={() => deleteCut(cut.id)}
											title="Delete permanently"
										>
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
							</div>
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
		width: 400px;
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

	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.panel-content {
		flex: 1;
		overflow-y: auto;
	}

	.loading,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
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

	.cuts-list {
		padding: var(--spacing-md);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.cut-item {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		overflow: hidden;
	}

	.cut-content {
		padding: var(--spacing-md);
		max-height: 200px;
		overflow-y: auto;
	}

	.cut-content pre {
		font-family: var(--font-serif);
		font-size: var(--font-size-sm);
		line-height: 1.6;
		color: var(--color-text-primary);
		white-space: pre-wrap;
		word-break: break-word;
		margin: 0;
	}

	.cut-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--color-bg-tertiary);
		border-top: 1px solid var(--color-border-light);
	}

	.cut-meta {
		display: flex;
		gap: var(--spacing-md);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.cut-actions {
		display: flex;
		gap: var(--spacing-xs);
	}

	.action-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.action-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.action-btn.primary {
		color: var(--color-accent);
	}

	.action-btn.primary:hover {
		background-color: var(--color-accent-light);
	}

	.action-btn.danger:hover {
		color: var(--color-error);
	}
</style>
