<script lang="ts">
	/**
	 * Button Component
	 *
	 * Unified button component with multiple variants to replace the scattered
	 * button patterns across the codebase (.add-btn, .icon-btn, .save-btn, etc.)
	 *
	 * Variants:
	 * - primary: Main CTA buttons (accent background)
	 * - secondary: Cancel/dismiss buttons (subtle)
	 * - ghost: Minimal buttons (no background until hover)
	 * - icon: Icon-only buttons (square)
	 * - danger: Destructive actions
	 */

	import type { Snippet } from 'svelte';

	type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'icon' | 'danger';
	type ButtonSize = 'sm' | 'md' | 'lg';

	interface Props {
		variant?: ButtonVariant;
		size?: ButtonSize;
		disabled?: boolean;
		type?: 'button' | 'submit' | 'reset';
		title?: string;
		class?: string;
		onclick?: (event: MouseEvent) => void;
		children: Snippet;
	}

	let {
		variant = 'primary',
		size = 'md',
		disabled = false,
		type = 'button',
		title,
		class: className = '',
		onclick,
		children,
	}: Props = $props();
</script>

<button class="btn btn-{variant} btn-{size} {className}" {type} {disabled} {title} {onclick}>
	{@render children()}
</button>

<style>
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--spacing-xs);
		font-family: inherit;
		font-weight: 500;
		border: none;
		border-radius: var(--border-radius-md);
		transition:
			background-color var(--transition-fast),
			color var(--transition-fast),
			opacity var(--transition-fast);
		white-space: nowrap;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* Size variants */
	.btn-sm {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
	}

	.btn-md {
		padding: var(--spacing-xs) var(--spacing-md);
		font-size: var(--font-size-sm);
	}

	.btn-lg {
		padding: var(--spacing-sm) var(--spacing-lg);
		font-size: var(--font-size-base);
	}

	/* Icon button sizes (square) */
	.btn-icon.btn-sm {
		width: 24px;
		height: 24px;
		padding: var(--spacing-xs);
	}

	.btn-icon.btn-md {
		width: 32px;
		height: 32px;
		padding: var(--spacing-xs);
	}

	.btn-icon.btn-lg {
		width: 40px;
		height: 40px;
		padding: var(--spacing-sm);
	}

	/* Primary variant */
	.btn-primary {
		background-color: var(--color-accent);
		color: var(--text-on-accent);
	}

	.btn-primary:hover:not(:disabled) {
		background-color: var(--color-accent-hover);
	}

	/* Secondary variant */
	.btn-secondary {
		background-color: var(--color-bg-secondary);
		color: var(--color-text-secondary);
		border: 1px solid var(--color-border);
	}

	.btn-secondary:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	/* Ghost variant */
	.btn-ghost {
		background-color: transparent;
		color: var(--color-text-secondary);
	}

	.btn-ghost:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	/* Icon variant */
	.btn-icon {
		background-color: transparent;
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.btn-icon:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	/* Danger variant */
	.btn-danger {
		background-color: var(--color-error);
		color: var(--text-on-accent);
	}

	.btn-danger:hover:not(:disabled) {
		background-color: var(--color-error);
		opacity: 0.9;
	}

	/* Icon button danger state (for icon-only delete buttons) */
	.btn-icon.btn-danger {
		background-color: transparent;
		color: var(--color-text-muted);
	}

	.btn-icon.btn-danger:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
		color: var(--color-error);
	}
</style>
