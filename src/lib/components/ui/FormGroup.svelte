<script lang="ts">
	/**
	 * FormGroup Component
	 *
	 * Wrapper for form fields with consistent label styling.
	 * Replaces the repeated .form-group pattern across 9+ components.
	 */

	import type { Snippet } from 'svelte';

	interface Props {
		label: string;
		id?: string;
		hint?: string;
		error?: string;
		required?: boolean;
		children: Snippet;
	}

	let { label, id, hint, error, required = false, children }: Props = $props();
</script>

<div class="form-group" class:has-error={error}>
	<label for={id} class="form-label">
		{label}
		{#if required}
			<span class="required">*</span>
		{/if}
	</label>
	<div class="form-control">
		{@render children()}
	</div>
	{#if error}
		<span class="form-error">{error}</span>
	{:else if hint}
		<span class="form-hint">{hint}</span>
	{/if}
</div>

<style>
	.form-group {
		margin-bottom: var(--spacing-md);
	}

	.form-label {
		display: block;
		font-size: var(--font-size-xs);
		font-weight: 500;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-xs);
	}

	.required {
		color: var(--color-error);
		margin-left: 2px;
	}

	.form-control {
		width: 100%;
	}

	/* Style native inputs within form-control */
	.form-control :global(input[type='text']),
	.form-control :global(input[type='email']),
	.form-control :global(input[type='password']),
	.form-control :global(input[type='number']),
	.form-control :global(input[type='url']),
	.form-control :global(input[type='search']),
	.form-control :global(textarea),
	.form-control :global(select) {
		width: 100%;
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		font-family: inherit;
		color: var(--color-text-primary);
		transition: border-color var(--transition-fast);
	}

	.form-control :global(input:focus),
	.form-control :global(textarea:focus),
	.form-control :global(select:focus) {
		outline: none;
		border-color: var(--color-accent);
	}

	.form-control :global(textarea) {
		resize: vertical;
		min-height: 60px;
	}

	.form-control :global(input[type='color']) {
		width: 60px;
		height: 32px;
		padding: 2px;
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
	}

	.has-error .form-control :global(input),
	.has-error .form-control :global(textarea),
	.has-error .form-control :global(select) {
		border-color: var(--color-error);
	}

	.form-hint {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-top: var(--spacing-xs);
	}

	.form-error {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-error);
		margin-top: var(--spacing-xs);
	}
</style>
