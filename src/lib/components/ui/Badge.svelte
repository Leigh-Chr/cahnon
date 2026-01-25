<script lang="ts">
	/**
	 * Badge Component
	 *
	 * Colored badge/pill for displaying status, type, or category information.
	 * Replaces various badge patterns across the codebase.
	 */

	type BadgeVariant = 'default' | 'success' | 'warning' | 'error' | 'info' | 'muted';

	interface Props {
		variant?: BadgeVariant;
		size?: 'sm' | 'md';
		/** Custom color (CSS color value) - overrides variant */
		color?: string;
		/** Use outline style instead of filled */
		outline?: boolean;
		class?: string;
		children: import('svelte').Snippet;
	}

	let {
		variant = 'default',
		size = 'sm',
		color,
		outline = false,
		class: className = '',
		children,
	}: Props = $props();
</script>

<span
	class="badge badge-{variant} badge-{size} {className}"
	class:outline
	style={color ? `--badge-color: ${color}` : undefined}
>
	{@render children()}
</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		font-weight: 500;
		text-transform: capitalize;
		white-space: nowrap;
		border-radius: var(--border-radius-sm);
	}

	/* Sizes */
	.badge-sm {
		padding: 2px 6px;
		font-size: var(--font-size-xs);
	}

	.badge-md {
		padding: 4px 8px;
		font-size: var(--font-size-sm);
	}

	/* Custom color (via style prop) */
	.badge[style*='--badge-color'] {
		background-color: var(--badge-color);
		color: var(--text-on-accent);
	}

	.badge.outline[style*='--badge-color'] {
		background-color: transparent;
		color: var(--badge-color);
		border: 1px solid var(--badge-color);
	}

	/* Variant: default */
	.badge-default {
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-secondary);
	}

	.badge-default.outline {
		background-color: transparent;
		border: 1px solid var(--color-border);
	}

	/* Variant: success */
	.badge-success {
		background-color: var(--color-success);
		color: var(--text-on-accent);
	}

	.badge-success.outline {
		background-color: var(--success-subtle);
		color: var(--color-success);
		border: 1px solid var(--success-border);
	}

	/* Variant: warning */
	.badge-warning {
		background-color: var(--color-warning);
		color: var(--text-on-accent);
	}

	.badge-warning.outline {
		background-color: var(--warning-subtle);
		color: var(--color-warning);
		border: 1px solid var(--warning-border);
	}

	/* Variant: error */
	.badge-error {
		background-color: var(--color-error);
		color: var(--text-on-accent);
	}

	.badge-error.outline {
		background-color: transparent;
		color: var(--color-error);
		border: 1px solid var(--color-error);
	}

	/* Variant: info */
	.badge-info {
		background-color: var(--color-info);
		color: var(--text-on-accent);
	}

	.badge-info.outline {
		background-color: var(--info-subtle);
		color: var(--color-info);
		border: 1px solid var(--color-accent);
	}

	/* Variant: muted */
	.badge-muted {
		background-color: var(--color-bg-secondary);
		color: var(--color-text-muted);
	}

	.badge-muted.outline {
		background-color: transparent;
		color: var(--color-text-muted);
		border: 1px solid var(--color-border-light);
	}
</style>
