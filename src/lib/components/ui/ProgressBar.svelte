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
	}

	.progress-track {
		height: 6px;
		background-color: var(--color-bg-tertiary);
		border-radius: 3px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: var(--color-accent);
		border-radius: 3px;
		transition: width 0.3s ease;
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
</style>
