<script lang="ts">
	import { toasts, removeToast, type Toast } from '$lib/toast';

	function getIcon(type: Toast['type']): string {
		switch (type) {
			case 'success':
				return '✓';
			case 'warning':
				return '⚠';
			case 'error':
				return '✕';
			default:
				return 'ℹ';
		}
	}
</script>

<div class="toast-container" role="region" aria-live="polite" aria-label="Notifications">
	{#each $toasts as toast (toast.id)}
		<div class="toast toast-{toast.type}" role="alert">
			<span class="toast-icon">{getIcon(toast.type)}</span>
			<span class="toast-message">{toast.message}</span>
			{#if toast.action}
				<button class="toast-action" onclick={toast.action.onClick}>
					{toast.action.label}
				</button>
			{/if}
			<button class="toast-close" onclick={() => removeToast(toast.id)} aria-label="Dismiss">
				<svg
					width="14"
					height="14"
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
	{/each}
</div>

<style>
	.toast-container {
		position: fixed;
		bottom: var(--spacing-lg);
		right: var(--spacing-lg);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		z-index: 2000;
		max-width: 400px;
	}

	.toast {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		border-left: 4px solid;
		animation: slideIn 0.3s ease-out;
	}

	@keyframes slideIn {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}

	.toast-info {
		border-left-color: var(--color-accent);
	}

	.toast-success {
		border-left-color: var(--color-text-primary);
	}

	.toast-warning {
		border-left-color: var(--color-text-secondary);
	}

	.toast-error {
		border-left-color: var(--color-text-muted);
	}

	.toast-icon {
		font-size: var(--font-size-md);
		flex-shrink: 0;
	}

	.toast-info .toast-icon {
		color: var(--color-accent);
	}

	.toast-success .toast-icon {
		color: var(--color-text-primary);
	}

	.toast-warning .toast-icon {
		color: var(--color-text-secondary);
	}

	.toast-error .toast-icon {
		color: var(--color-text-muted);
	}

	.toast-message {
		flex: 1;
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.toast-action {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
		color: var(--color-accent);
		border-radius: var(--border-radius-sm);
	}

	.toast-action:hover {
		background-color: var(--color-accent-light);
	}

	.toast-close {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
	}

	.toast-close:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}
</style>
