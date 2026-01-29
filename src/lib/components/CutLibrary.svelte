<script lang="ts">
	import { type Cut, cutApi } from '$lib/api';
	import { showError, showSuccess } from '$lib/toast';
	import { countWords, formatDateTime, formatWordCount } from '$lib/utils';
	import { trapFocus } from '$lib/utils/focus-trap';
	import { nativeConfirm } from '$lib/utils/native-dialog';

	import { Button, EmptyState, Icon, LoadingState } from './ui';

	interface Props {
		isOpen?: boolean;
		onInsert?: ((text: string) => void) | null;
	}

	let { isOpen = $bindable(false), onInsert = null }: Props = $props();

	let cuts = $state<Cut[]>([]);
	let isLoading = $state(true);
	let searchQuery = $state('');

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
			showError('Failed to load cut library');
		}
		isLoading = false;
	}

	async function handleDeleteCut(id: string) {
		const confirmed = await nativeConfirm('Delete this cut permanently?');
		if (!confirmed) return;
		try {
			await cutApi.delete(id);
			cuts = cuts.filter((c) => c.id !== id);
		} catch (e) {
			console.error('Failed to delete cut:', e);
			showError('Failed to delete cut');
		}
	}

	function insertCut(cut: Cut) {
		if (onInsert) {
			onInsert(cut.text);
		}
	}

	async function copyToClipboard(text: string) {
		try {
			await navigator.clipboard.writeText(text);
			showSuccess('Copied to clipboard');
		} catch (e) {
			console.error('Failed to copy to clipboard:', e);
			showError('Failed to copy to clipboard');
		}
	}

	function close() {
		isOpen = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!isOpen) return;
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

	let filteredCuts = $derived(
		searchQuery
			? cuts.filter((c) => c.text.toLowerCase().includes(searchQuery.toLowerCase()))
			: cuts
	);
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<div
		class="panel-overlay"
		onclick={handleOverlayClick}
		onkeydown={(e) => {
			if (e.key === 'Escape') close();
		}}
		role="presentation"
		tabindex="-1"
	>
		<div
			class="panel"
			onclick={handlePanelClick}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-labelledby="cuts-title"
			tabindex="-1"
			use:trapFocus={{ onEscape: close }}
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
					<div class="search-box">
						<input type="text" placeholder="Search cuts..." bind:value={searchQuery} />
						{#if searchQuery}
							<span class="result-count">{filteredCuts.length} of {cuts.length} cuts</span>
						{/if}
					</div>
					{#if filteredCuts.length === 0 && searchQuery}
						<div class="no-results">No cuts matching "{searchQuery}"</div>
					{/if}
					<div class="cuts-list">
						{#each filteredCuts as cut (cut.id)}
							<div class="cut-item">
								<div class="cut-content">
									<pre>{cut.text}</pre>
								</div>
								<div class="cut-footer">
									<div class="cut-meta">
										<span class="word-count">{formatWordCount(countWords(cut.text))} words</span>
										<span class="date">{formatDateTime(cut.created_at)}</span>
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
											onclick={() => handleDeleteCut(cut.id)}
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

	.search-box {
		padding: var(--spacing-md) var(--spacing-md) 0;
	}

	.result-count {
		display: block;
		margin-top: var(--spacing-xs);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.search-box input {
		width: 100%;
		padding: var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
	}

	.no-results {
		padding: var(--spacing-md);
		text-align: center;
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
		font-style: italic;
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
