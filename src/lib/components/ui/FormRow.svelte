<script lang="ts">
	/**
	 * FormRow Component
	 *
	 * Horizontal layout for side-by-side form fields.
	 * Replaces the repeated .form-row pattern.
	 */

	import type { Snippet } from 'svelte';

	interface Props {
		gap?: 'sm' | 'md' | 'lg';
		children: Snippet;
	}

	let { gap = 'md', children }: Props = $props();
</script>

<div class="form-row gap-{gap}">
	{@render children()}
</div>

<style>
	.form-row {
		display: flex;
		flex-wrap: wrap;
	}

	.gap-sm {
		gap: var(--spacing-sm);
	}

	.gap-md {
		gap: var(--spacing-md);
	}

	.gap-lg {
		gap: var(--spacing-lg);
	}

	/* Children (FormGroup) should flex */
	.form-row :global(.form-group) {
		flex: 1;
		min-width: 120px;
	}
</style>
