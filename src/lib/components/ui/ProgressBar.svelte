<!--
  ProgressBar Component (AM2)

  Displays progress for operations with determinate or indeterminate state.

  Usage:
    <ProgressBar value={75} label="Exporting..." />
    <ProgressBar indeterminate label="Processing..." />
-->
<script lang="ts">
	interface Props {
		value?: number;
		indeterminate?: boolean;
		label?: string;
		showValue?: boolean;
	}

	let { value = 0, indeterminate = false, label, showValue = true }: Props = $props();

	let clampedValue = $derived(Math.max(0, Math.min(100, value)));
</script>

<div class="progress-container">
	{#if label || (showValue && !indeterminate)}
		<div class="progress-header">
			{#if label}
				<span class="progress-label">{label}</span>
			{/if}
			{#if showValue && !indeterminate}
				<span class="progress-value">{Math.round(clampedValue)}%</span>
			{/if}
		</div>
	{/if}
	<div
		class="progress-track"
		role="progressbar"
		aria-valuenow={indeterminate ? undefined : clampedValue}
		aria-valuemin={0}
		aria-valuemax={100}
		aria-label={label}
	>
		<div
			class="progress-fill"
			class:indeterminate
			style={!indeterminate ? `width: ${clampedValue}%` : undefined}
		></div>
	</div>
</div>

<style>
	/* Phase 4.4: Enriched progress bar */
	.progress-container {
		width: 100%;
	}

	.progress-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--spacing-xs);
		font-size: var(--font-size-sm);
	}

	.progress-label {
		color: var(--color-text-secondary);
	}

	.progress-value {
		color: var(--color-text-muted);
		font-family: var(--font-family-mono);
		font-size: var(--font-size-xs);
		letter-spacing: var(--tracking-wide, 0.025em);
	}

	.progress-track {
		height: 6px;
		background-color: var(--color-bg-tertiary);
		border-radius: 3px;
		overflow: hidden;
		/* Inset shadow for depth */
		box-shadow: inset 0 1px 2px oklch(0% 0 0 / 8%);
	}

	.progress-fill {
		height: 100%;
		/* Gradient for visual interest */
		background: linear-gradient(90deg, var(--color-accent) 0%, var(--accent-hover) 100%);
		border-radius: 3px;
		transition: width 0.3s ease;
		position: relative;
	}

	/* Subtle shine effect */
	.progress-fill::after {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 50%;
		background: linear-gradient(180deg, oklch(100% 0 0 / 20%) 0%, oklch(100% 0 0 / 0%) 100%);
		border-radius: 3px 3px 0 0;
	}

	.progress-fill.indeterminate {
		width: 30%;
		animation: indeterminate 1.5s ease-in-out infinite;
	}

	@keyframes indeterminate {
		0% {
			transform: translateX(-100%);
		}
		100% {
			transform: translateX(400%);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.progress-fill.indeterminate {
			animation: none;
			width: 100%;
			opacity: 0.6;
		}
	}
</style>
