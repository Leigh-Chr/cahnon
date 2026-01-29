<script lang="ts">
	/**
	 * FormGroup Component
	 *
	 * Wrapper for form fields with consistent label styling.
	 * Replaces the repeated .form-group pattern across 9+ components.
	 *
	 * AM6: Enhanced accessibility for validation errors:
	 * - Error icon (not just color) for visibility
	 * - aria-describedby links input to error/hint message
	 * - role="alert" on error for screen reader announcement
	 */

	import type { Snippet } from 'svelte';

	import Icon from './Icon.svelte';

	interface Props {
		label: string;
		id?: string;
		hint?: string;
		error?: string;
		required?: boolean;
		children: Snippet;
	}

	let { label, id, hint, error, required = false, children }: Props = $props();

	// Generate IDs for accessibility
	let errorId = $derived(id ? `${id}-error` : undefined);
	let hintId = $derived(id ? `${id}-hint` : undefined);
</script>

<div class="form-group" class:has-error={error}>
	<label for={id} class="form-label">
		{label}
		{#if required}
			<span class="required" aria-hidden="true">*</span>
		{/if}
	</label>
	<div class="form-input-wrapper">
		<div class="form-control">
			{@render children()}
		</div>
		{#if error}
			<span class="form-error-icon" aria-hidden="true">
				<Icon name="alert" size={16} />
			</span>
		{/if}
	</div>
	{#if error}
		<div class="form-error" id={errorId} role="alert">
			<Icon name="alert" size={12} />
			<span>{error}</span>
		</div>
	{:else if hint}
		<span class="form-hint" id={hintId}>{hint}</span>
	{/if}
</div>

<style>
	.form-group {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
		margin-bottom: var(--spacing-md);
	}

	.form-label {
		display: block;
		font-size: var(--font-size-xs);
		font-weight: 500;
		color: var(--color-text-secondary);
	}

	.required {
		color: var(--color-error);
		margin-left: 2px;
	}

	.form-input-wrapper {
		position: relative;
	}

	.form-control {
		width: 100%;
	}

	/* AM6: Error icon positioned inside the input */
	.form-error-icon {
		position: absolute;
		right: var(--spacing-sm);
		top: 50%;
		transform: translateY(-50%);
		color: var(--color-error);
		pointer-events: none;
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

	/* Phase 3.5: Enhanced focus state with inset shadow and glow */
	.form-control :global(input:focus),
	.form-control :global(textarea:focus),
	.form-control :global(select:focus) {
		outline: none;
		border-color: var(--color-accent);
		box-shadow:
			0 0 0 3px var(--focus-glow),
			inset 0 1px 2px var(--accent-subtle);
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
	}

	.has-error .form-control :global(input),
	.has-error .form-control :global(textarea),
	.has-error .form-control :global(select) {
		border-color: var(--color-error);
		padding-right: calc(var(--spacing-sm) * 2 + 16px);
	}

	/* Error focus state */
	.has-error .form-control :global(input:focus),
	.has-error .form-control :global(textarea:focus) {
		box-shadow:
			0 0 0 3px oklch(55% 0.2 25 / 15%),
			inset 0 1px 2px var(--danger-subtle);
	}

	.form-hint {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	/* AM6: Error with icon for accessibility (not just color) */
	.form-error {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-xs);
		color: var(--color-error);
	}
</style>
