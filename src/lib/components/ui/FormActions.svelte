<script lang="ts">
	/**
	 * FormActions Component
	 *
	 * Container for form action buttons (Save/Cancel).
	 * Replaces the repeated .form-actions pattern.
	 */

	import type { Snippet } from 'svelte';

	interface Props {
		align?: 'start' | 'center' | 'end';
		children: Snippet;
	}

	let { align = 'end', children }: Props = $props();
</script>

<div class="form-actions align-{align}">
	{@render children()}
</div>

<style>
	.form-actions {
		display: flex;
		gap: var(--spacing-sm);
		margin-top: var(--spacing-md);
	}

	.align-start {
		justify-content: flex-start;
	}

	.align-center {
		justify-content: center;
	}

	.align-end {
		justify-content: flex-end;
	}
</style>
