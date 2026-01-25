<script lang="ts">
	import { cutApi, type Cut } from '$lib/api';
	import { countWords, formatWordCount } from '$lib/utils';
	import { Icon, Button, EmptyState, LoadingState } from './ui';

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
				<Button variant="icon" onclick={close} title="Close">
					<Icon name="close" size={20} />
				</Button>
			</div>

			<div class="panel-content">
				{#if isLoading}
					<LoadingState message="Loading cuts..." />
				{:else if cuts.length === 0}
					<EmptyState
						icon="scissors"
						title="No cuts yet"
						description="Text you cut from scenes will be saved here for later use."
					/>
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
										<Button
											variant="icon"
											size="sm"
											onclick={() => copyToClipboard(cut.text)}
											title="Copy to clipboard"
										>
											<Icon name="copy" size={16} />
										</Button>
										{#if onInsert}
											<Button
												variant="icon"
												size="sm"
												class="primary"
												onclick={() => insertCut(cut)}
												title="Insert at cursor"
											>
												<Icon name="plus" size={16} />
											</Button>
										{/if}
										<Button
											variant="icon"
											size="sm"
											class="danger"
											onclick={() => deleteCut(cut.id)}
											title="Delete permanently"
										>
											<Icon name="delete" size={16} />
										</Button>
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

	.panel-content {
		flex: 1;
		overflow-y: auto;
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

	.cut-actions :global(.btn.primary) {
		color: var(--color-accent);
	}

	.cut-actions :global(.btn.primary:hover) {
		background-color: var(--color-accent-light);
	}

	.cut-actions :global(.btn.danger:hover) {
		color: var(--color-error);
	}
</style>
