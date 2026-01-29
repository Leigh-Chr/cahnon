<!--
	AH1: Reusable Dialog component

	Provides a consistent modal dialog pattern with:
	- Focus trapping
	- Escape key handling
	- Backdrop click to close
	- Consistent header/body/footer layout
	- Size variants
	- Entry animation
-->
<script lang="ts">
	import type { Snippet } from 'svelte';

	import { trapFocus } from '$lib/utils/focus-trap';

	import { Icon } from './';

	interface Props {
		isOpen: boolean;
		title: string;
		size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
		onclose: () => void;
		children: Snippet;
		footer?: Snippet;
		/** Whether to show the close button in the header */
		showCloseButton?: boolean;
		/** Custom class for the dialog container */
		class?: string;
	}

	let {
		isOpen,
		title,
		size = 'md',
		onclose,
		children,
		footer,
		showCloseButton = true,
		class: className = '',
	}: Props = $props();

	const sizeClasses: Record<string, string> = {
		sm: 'dialog-sm',
		md: 'dialog-md',
		lg: 'dialog-lg',
		xl: 'dialog-xl',
		full: 'dialog-full',
	};

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onclose();
		}
	}
</script>

{#if isOpen}
	<div class="dialog-overlay" onclick={handleBackdropClick} role="presentation">
		<div
			class="dialog-container {sizeClasses[size]} {className} modal-enter"
			role="dialog"
			aria-modal="true"
			aria-labelledby="dialog-title"
			use:trapFocus={{ onEscape: onclose }}
		>
			<div class="dialog-header">
				<h2 id="dialog-title">{title}</h2>
				{#if showCloseButton}
					<button class="dialog-close" onclick={onclose} title="Close" aria-label="Close dialog">
						<Icon name="close" size={20} />
					</button>
				{/if}
			</div>
			<div class="dialog-body">
				{@render children()}
			</div>
			{#if footer}
				<div class="dialog-footer">
					{@render footer()}
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	/* Phase 3.3: Enhanced dialog animations */
	.dialog-overlay {
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
		padding: var(--spacing-md);
		animation: backdrop-fade 0.2s ease-out;
	}

	@keyframes backdrop-fade {
		from {
			background-color: transparent;
		}
		to {
			background-color: var(--overlay-backdrop);
		}
	}

	.dialog-container {
		position: relative;
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		max-height: calc(100vh - var(--spacing-lg) * 2);
		overflow: hidden;
	}

	/* Phase 3.3: Enhanced modal entrance with subtle bounce */
	.dialog-container.modal-enter {
		animation: dialog-enter 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
	}

	@keyframes dialog-enter {
		from {
			opacity: 0;
			transform: scale(0.92) translateY(8px);
		}
		to {
			opacity: 1;
			transform: scale(1) translateY(0);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.dialog-overlay {
			animation: none;
		}
		.dialog-container.modal-enter {
			animation: none;
		}
	}

	/* Size variants */
	.dialog-sm {
		width: 100%;
		max-width: 400px;
	}

	.dialog-md {
		width: 100%;
		max-width: 600px;
	}

	.dialog-lg {
		width: 100%;
		max-width: 800px;
	}

	.dialog-xl {
		width: 100%;
		max-width: 1000px;
	}

	.dialog-full {
		width: calc(100vw - var(--spacing-lg) * 2);
		height: calc(100vh - var(--spacing-lg) * 2);
		max-width: none;
		max-height: none;
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.dialog-header h2 {
		margin: 0;
		font-size: var(--font-size-lg);
		font-weight: 600;
		color: var(--color-text-primary);
		word-break: break-word;
		overflow-wrap: break-word;
	}

	.dialog-close {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.dialog-close:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.dialog-body {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
	}

	.dialog-footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--spacing-sm);
		padding: var(--spacing-md) var(--spacing-lg);
		border-top: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	/* Responsive dialog sizes for smaller screens */
	@media (max-width: 900px) {
		.dialog-sm {
			max-width: min(360px, calc(100vw - var(--spacing-md) * 2));
		}
		.dialog-md {
			max-width: min(500px, calc(100vw - var(--spacing-md) * 2));
		}
		.dialog-lg {
			max-width: min(650px, calc(100vw - var(--spacing-md) * 2));
		}
		.dialog-xl {
			max-width: min(800px, calc(100vw - var(--spacing-md) * 2));
		}
		.dialog-body {
			padding: var(--spacing-md);
		}
	}
</style>
