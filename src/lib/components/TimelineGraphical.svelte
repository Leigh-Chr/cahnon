<!--
  Graphical timeline visualization.
  Horizontal axis shows time_point columns, scenes displayed as stacked bars.
  Scrolls horizontally for projects with many time points.
-->
<script lang="ts">
	import type { Scene, TimelineEvent } from '$lib/api';
	import { appState } from '$lib/stores';
	import { statusColors } from '$lib/utils';
	import { buildTimelineColumns } from '$lib/utils/timeline-layout';

	interface Props {
		scenes: Scene[];
		events: TimelineEvent[];
		conflicts?: { time_point: string | null }[];
	}

	let { scenes, events, conflicts = [] }: Props = $props();

	let columns = $derived(buildTimelineColumns(scenes, events));

	const COL_WIDTH = 140;
	const BAR_HEIGHT = 32;
	const BAR_GAP = 4;
	const HEADER_HEIGHT = 40;
	const PADDING = 16;

	let svgWidth = $derived(Math.max(columns.length * COL_WIDTH + PADDING * 2, 400));
	let maxLanes = $derived(Math.max(1, ...columns.map((c) => c.items.length)));
	let svgHeight = $derived(HEADER_HEIGHT + maxLanes * (BAR_HEIGHT + BAR_GAP) + PADDING * 2);

	function getBarColor(status?: string): string {
		if (!status) return 'var(--color-accent)';
		return statusColors[status] || 'var(--color-accent)';
	}

	function hasConflict(timePoint: string): boolean {
		return conflicts.some((c) => c.time_point === timePoint);
	}

	function truncateLabel(text: string, maxLen = 16): string {
		return text.length > maxLen ? text.slice(0, maxLen - 1) + '…' : text;
	}

	let selectedItemId = $state<string | null>(null);

	function handleItemClick(item: { id: string; type: 'scene' | 'event' }) {
		selectedItemId = item.id;
		if (item.type === 'scene') {
			appState.selectedSceneId = item.id;
			appState.viewMode = 'editor';
		} else {
			appState.navigateToEvent(item.id);
		}
	}
</script>

<div class="timeline-graphical">
	{#if columns.length === 0}
		<p class="empty-text">No scenes or events have time_point values set.</p>
	{:else}
		<div class="scroll-container">
			<svg width={svgWidth} height={svgHeight} class="timeline-svg">
				<!-- Column headers and gridlines -->
				{#each columns as col, colIndex (col.timePoint)}
					{@const x = PADDING + colIndex * COL_WIDTH}

					<!-- Gridline -->
					<line
						x1={x + COL_WIDTH / 2}
						y1={HEADER_HEIGHT}
						x2={x + COL_WIDTH / 2}
						y2={svgHeight}
						stroke="var(--color-border)"
						stroke-width="1"
						stroke-dasharray="4 4"
						opacity="0.5"
					/>

					<!-- Conflict indicator -->
					{#if hasConflict(col.timePoint)}
						<rect
							x={x + 2}
							y={HEADER_HEIGHT}
							width={COL_WIDTH - 4}
							height={svgHeight - HEADER_HEIGHT}
							fill="var(--color-error)"
							opacity="0.05"
							rx="4"
						/>
					{/if}

					<!-- Time point label -->
					<text
						x={x + COL_WIDTH / 2}
						y={HEADER_HEIGHT - 12}
						text-anchor="middle"
						class="col-label"
						class:conflict={hasConflict(col.timePoint)}>{truncateLabel(col.timePoint)}</text
					>

					<!-- Item bars -->
					{#each col.items as item, laneIndex (item.id)}
						{@const y = HEADER_HEIGHT + PADDING + laneIndex * (BAR_HEIGHT + BAR_GAP)}
						<g
							class="item-bar"
							class:selected={selectedItemId === item.id}
							onclick={() => handleItemClick(item)}
							onkeydown={(e) => e.key === 'Enter' && handleItemClick(item)}
							role="button"
							tabindex="0"
						>
							<rect
								{x}
								{y}
								width={COL_WIDTH - 8}
								height={BAR_HEIGHT}
								rx="4"
								fill={item.type === 'event' ? 'var(--color-info)' : getBarColor(item.status)}
								opacity="0.85"
								stroke={selectedItemId === item.id
									? 'var(--color-accent)'
									: hasConflict(col.timePoint)
										? 'var(--color-error)'
										: 'none'}
								stroke-width={selectedItemId === item.id || hasConflict(col.timePoint) ? 2 : 0}
							/>
							<text x={x + 6} y={y + BAR_HEIGHT / 2 + 4} class="bar-label"
								>{truncateLabel(item.title, 18)}</text
							>
							<title>{item.title} ({item.type}{item.status ? ` — ${item.status}` : ''})</title>
						</g>
					{/each}
				{/each}
			</svg>
		</div>
	{/if}
</div>

<style>
	.timeline-graphical {
		width: 100%;
		height: 100%;
	}

	.scroll-container {
		overflow-x: auto;
		overflow-y: auto;
		width: 100%;
		height: 100%;
		padding: var(--spacing-md);
	}

	.timeline-svg {
		display: block;
	}

	.empty-text {
		text-align: center;
		padding: var(--spacing-xl);
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
	}

	.col-label {
		font-size: 11px;
		fill: var(--color-text-secondary);
		font-weight: 500;
	}

	.col-label.conflict {
		fill: var(--color-error);
		font-weight: 600;
	}

	.bar-label {
		font-size: 11px;
		fill: var(--text-on-accent, #fff);
		pointer-events: none;
	}

	.item-bar {
		cursor: pointer;
	}

	.item-bar:hover rect {
		opacity: 1;
	}

	.item-bar:focus {
		outline: none;
	}

	.item-bar:focus rect {
		stroke: var(--color-accent);
		stroke-width: 2;
	}

	.item-bar.selected rect {
		opacity: 1;
	}
</style>
