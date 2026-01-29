<script lang="ts">
	/**
	 * LoadingState Component
	 *
	 * Consistent loading indicator with optional message.
	 * Supports spinner (default) and skeleton variants.
	 */

	interface Props {
		message?: string;
		size?: 'sm' | 'md' | 'lg';
		inline?: boolean;
		variant?: 'spinner' | 'skeleton';
		lines?: number;
	}

	let {
		message = 'Loading...',
		size = 'md',
		inline = false,
		variant = 'spinner',
		lines = 4,
	}: Props = $props();
</script>

{#if variant === 'skeleton'}
	<div class="skeleton-container" class:sm={size === 'sm'} class:lg={size === 'lg'}>
		{#each Array(lines) as _, i (i)}
			<div class="skeleton-line" style="width: {i === lines - 1 ? '60%' : '100%'}"></div>
		{/each}
	</div>
{:else}
	<div class="loading-state" class:inline class:sm={size === 'sm'} class:lg={size === 'lg'}>
		<div class="spinner"></div>
		{#if message}
			<span class="loading-message">{message}</span>
		{/if}
	</div>
{/if}

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
		width: var(--spinner-size-lg);
		height: var(--spinner-size-lg);
		border: var(--spinner-width) solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin var(--spinner-speed) linear infinite;
	}

	.loading-state.sm .spinner {
		width: var(--spinner-size-md);
		height: var(--spinner-size-md);
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

	.skeleton-container {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		padding: var(--spacing-lg);
	}

	.skeleton-line {
		height: 14px;
		border-radius: var(--border-radius-sm);
		background: linear-gradient(
			90deg,
			var(--color-bg-tertiary) 25%,
			var(--color-bg-secondary) 50%,
			var(--color-bg-tertiary) 75%
		);
		background-size: 200% 100%;
		animation: skeleton-shimmer 1.5s infinite;
	}

	.skeleton-container.sm .skeleton-line {
		height: 10px;
	}

	.skeleton-container.lg .skeleton-line {
		height: 18px;
	}

	@keyframes skeleton-shimmer {
		0% {
			background-position: 200% 0;
		}
		100% {
			background-position: -200% 0;
		}
	}
</style>
