<script lang="ts">
	/**
	 * LoadingState Component
	 *
	 * Consistent loading indicator with optional message.
	 * Replaces the repeated .loading pattern across components.
	 */

	interface Props {
		message?: string;
		size?: 'sm' | 'md' | 'lg';
		inline?: boolean;
	}

	let { message = 'Loading...', size = 'md', inline = false }: Props = $props();
</script>

<div class="loading-state" class:inline class:sm={size === 'sm'} class:lg={size === 'lg'}>
	<div class="spinner"></div>
	{#if message}
		<span class="loading-message">{message}</span>
	{/if}
</div>

<style>
	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		gap: var(--spacing-sm);
		padding: var(--spacing-lg);
		color: var(--color-text-muted);
	}

	.loading-state.inline {
		flex-direction: row;
		flex: none;
		padding: var(--spacing-sm);
	}

	.spinner {
		width: 24px;
		height: 24px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.loading-state.sm .spinner {
		width: 16px;
		height: 16px;
		border-width: 2px;
	}

	.loading-state.lg .spinner {
		width: 32px;
		height: 32px;
		border-width: 3px;
	}

	.loading-message {
		font-size: var(--font-size-sm);
	}

	.loading-state.sm .loading-message {
		font-size: var(--font-size-xs);
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
