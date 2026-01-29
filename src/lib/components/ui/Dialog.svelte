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
</style>
