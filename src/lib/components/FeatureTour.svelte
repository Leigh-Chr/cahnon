<!--
  FeatureTour Component (AL5)

  An interactive tour system for guiding users through features.
  Highlights elements on the page and shows explanatory tooltips.

  Usage:
    <FeatureTour
      steps={[
        { target: '.outline-panel', title: 'Outline', description: 'Navigate your scenes here', position: 'right' },
        { target: '.editor-container', title: 'Editor', description: 'Write your story here', position: 'bottom' },
      ]}
      onComplete={() => console.log('Tour completed')}
      onSkip={() => console.log('Tour skipped')}
    />
-->
<script lang="ts">
	import { tick } from 'svelte';

	import { Button, Icon } from './ui';

	interface TourStep {
		target: string;
		title: string;
		description: string;
		position?: 'top' | 'bottom' | 'left' | 'right';
	}

	interface Props {
		steps: TourStep[];
		onComplete: () => void;
		onSkip: () => void;
	}

	let { steps, onComplete, onSkip }: Props = $props();

	let currentStep = $state(0);
	let targetRect = $state<DOMRect | null>(null);
	let tooltipStyle = $state<string>('');

	$effect(() => {
		const step = steps[currentStep];
		if (step) {
			tick().then(() => {
				const element = document.querySelector(step.target);
				if (element) {
					targetRect = element.getBoundingClientRect();
					element.scrollIntoView({ behavior: 'smooth', block: 'center' });
					calculateTooltipPosition(targetRect, step.position || 'bottom');
				}
			});
		}
	});

	function calculateTooltipPosition(rect: DOMRect, position: string) {
		const offset = 12;
		const tooltipWidth = 320;
		const tooltipHeight = 160; // Approximate

		let top: number;
		let left: number;

		switch (position) {
			case 'top':
				top = rect.top - tooltipHeight - offset;
				left = rect.left + rect.width / 2 - tooltipWidth / 2;
				break;
			case 'bottom':
				top = rect.bottom + offset;
				left = rect.left + rect.width / 2 - tooltipWidth / 2;
				break;
			case 'left':
				top = rect.top + rect.height / 2 - tooltipHeight / 2;
				left = rect.left - tooltipWidth - offset;
				break;
			case 'right':
				top = rect.top + rect.height / 2 - tooltipHeight / 2;
				left = rect.right + offset;
				break;
			default:
				top = rect.bottom + offset;
				left = rect.left + rect.width / 2 - tooltipWidth / 2;
		}

		// Keep tooltip on screen
		top = Math.max(16, Math.min(top, window.innerHeight - tooltipHeight - 16));
		left = Math.max(16, Math.min(left, window.innerWidth - tooltipWidth - 16));

		tooltipStyle = `top: ${top}px; left: ${left}px;`;
	}

	function next() {
		if (currentStep < steps.length - 1) {
			currentStep++;
		} else {
			onComplete();
		}
	}

	function prev() {
		if (currentStep > 0) {
			currentStep--;
		}
	}

	function handleSkip() {
		onSkip();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			handleSkip();
		} else if (event.key === 'ArrowRight' || event.key === 'Enter') {
			next();
		} else if (event.key === 'ArrowLeft') {
			prev();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if steps[currentStep] && targetRect}
	{@const step = steps[currentStep]}

	<div class="tour-overlay">
		<!-- Spotlight mask - 4 rectangles to create a hole -->
		<div class="tour-mask tour-mask-top" style="height: {targetRect.top - 4}px;"></div>
		<div
			class="tour-mask tour-mask-left"
			style="top: {targetRect.top - 4}px; height: {targetRect.height +
				8}px; width: {targetRect.left - 4}px;"
		></div>
		<div
			class="tour-mask tour-mask-right"
			style="top: {targetRect.top - 4}px; height: {targetRect.height +
				8}px; left: {targetRect.right + 4}px;"
		></div>
		<div class="tour-mask tour-mask-bottom" style="top: {targetRect.bottom + 4}px;"></div>

		<!-- Spotlight border -->
		<div
			class="tour-spotlight"
			style="
				top: {targetRect.top - 4}px;
				left: {targetRect.left - 4}px;
				width: {targetRect.width + 8}px;
				height: {targetRect.height + 8}px;
			"
		></div>

		<!-- Tooltip -->
		<div class="tour-tooltip" style={tooltipStyle}>
			<div class="tooltip-header">
				<h4>{step.title}</h4>
				<button class="close-btn" onclick={handleSkip} title="Skip tour">
					<Icon name="close" size={16} />
				</button>
			</div>
			<p class="tooltip-description">{step.description}</p>
			<div class="tooltip-footer">
				<span class="step-counter">{currentStep + 1} of {steps.length}</span>
				<div class="tooltip-actions">
					{#if currentStep > 0}
						<Button variant="ghost" size="sm" onclick={prev}>Back</Button>
					{/if}
					<Button variant="primary" size="sm" onclick={next}>
						{currentStep < steps.length - 1 ? 'Next' : 'Done'}
					</Button>
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.tour-overlay {
		position: fixed;
		inset: 0;
		z-index: 9999;
		pointer-events: none;
	}

	.tour-mask {
		position: fixed;
		background-color: rgba(0, 0, 0, 0.5);
		pointer-events: auto;
	}

	.tour-mask-top {
		top: 0;
		left: 0;
		right: 0;
	}

	.tour-mask-left {
		left: 0;
	}

	.tour-mask-right {
		right: 0;
	}

	.tour-mask-bottom {
		left: 0;
		right: 0;
		bottom: 0;
	}

	.tour-spotlight {
		position: fixed;
		border: 2px solid var(--color-accent);
		border-radius: var(--border-radius-md);
		box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0);
		pointer-events: none;
	}

	.tour-tooltip {
		position: fixed;
		width: 320px;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-md);
		pointer-events: auto;
		z-index: 10000;
	}

	.tooltip-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: var(--spacing-sm);
	}

	.tooltip-header h4 {
		margin: 0;
		font-size: var(--font-size-md);
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.tooltip-description {
		margin: 0 0 var(--spacing-md);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		line-height: var(--line-height-relaxed);
	}

	.tooltip-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.step-counter {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.tooltip-actions {
		display: flex;
		gap: var(--spacing-xs);
	}
</style>
