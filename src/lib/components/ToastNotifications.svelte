<script lang="ts">
	import { removeToast, type Toast, toasts } from '$lib/toast';

	import { Icon } from './ui';

	function getIconName(type: Toast['type']) {
		const iconMap = {
			success: 'check',
			warning: 'warning',
			error: 'error',
			info: 'info',
		} as const;
		return iconMap[type] ?? 'info';
	}
</script>

<div class="toast-container" role="region" aria-live="polite" aria-label="Notifications">
	{#each $toasts as toast (toast.id)}
		<div class="toast toast-{toast.type}" role="alert">
			<div class="toast-content">
				<span class="toast-icon"><Icon name={getIconName(toast.type)} size={16} /></span>
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
			<!-- AM3: Countdown bar for toasts with actions -->
			{#if toast.action && toast.duration}
				<div class="toast-countdown">
					<div class="countdown-bar" style="animation-duration: {toast.duration}ms"></div>
				</div>
			{/if}
		</div>
	{/each}
</div>

<style>
	.toast-container {
		position: fixed;
		bottom: calc(var(--height-statusbar) + var(--spacing-md));
		right: var(--spacing-lg);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		z-index: 2000;
		max-width: 400px;
	}

	.toast {
		position: relative;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		border-left: 4px solid;
		animation: slideIn 0.3s ease-out;
		overflow: hidden;
	}

	.toast-content {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-md);
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
		border-left-color: var(--color-info);
	}

	.toast-success {
		border-left-color: var(--color-success);
	}

	.toast-warning {
		border-left-color: var(--color-warning);
	}

	.toast-error {
		border-left-color: var(--color-error);
	}

	.toast-icon {
		display: flex;
		align-items: center;
		flex-shrink: 0;
	}

	.toast-info .toast-icon {
		color: var(--color-info);
	}

	.toast-success .toast-icon {
		color: var(--color-success);
	}

	.toast-warning .toast-icon {
		color: var(--color-warning);
	}

	.toast-error .toast-icon {
		color: var(--color-error);
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

	/* AM3: Countdown bar for undo actions */
	.toast-countdown {
		height: 3px;
		background-color: rgba(0, 0, 0, 0.1);
	}

	.countdown-bar {
		height: 100%;
		background-color: currentColor;
		opacity: 0.3;
		animation: countdown linear forwards;
	}

	@keyframes countdown {
		from {
			width: 100%;
		}
		to {
			width: 0%;
		}
	}
</style>
