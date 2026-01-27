<!--
  Arc Swimlane visualization.

  Horizontal swimlane diagram showing each arc as a labeled color band.
  Arc name on left, colored band stretching across the width.
  Uses arcApi to load arcs on mount.
-->
<script lang="ts">
	import { type Arc, arcApi } from '$lib/api';
	import { appState } from '$lib/stores';

	let arcs = $state<Arc[]>([]);
	let isLoading = $state(false);

	const LANE_HEIGHT = 36;
	const LABEL_WIDTH = 140;
	const padding = { top: 10, right: 20, bottom: 10, left: 10 };
	const viewWidth = 600;

	let viewHeight = $derived(padding.top + padding.bottom + arcs.length * LANE_HEIGHT);
	let bandWidth = $derived(viewWidth - LABEL_WIDTH - padding.left - padding.right);

	// Load arcs when project is available
	$effect(() => {
		if (appState.project) {
			loadArcs();
		}
	});

	async function loadArcs() {
		isLoading = true;
		try {
			arcs = await arcApi.getAll();
		} catch (e) {
			console.error('Failed to load arcs:', e);
			arcs = [];
		} finally {
			isLoading = false;
		}
	}

	// Status indicator
	const statusSymbols: Record<string, string> = {
		setup: 'Setup',
		active: 'Active',
		climax: 'Climax',
		resolved: 'Resolved',
		abandoned: 'Abandoned',
	};

	function getStatusLabel(status: string): string {
		return statusSymbols[status] || status;
	}

	function defaultColor(index: number): string {
		const colors = [
			'#6366f1',
			'#ef4444',
			'#22c55e',
			'#f97316',
			'#06b6d4',
			'#8b5cf6',
			'#ec4899',
			'#eab308',
		];
		return colors[index % colors.length];
	}
</script>

<div class="arc-swimlane">
	{#if isLoading}
		<p class="loading-msg">Loading arcs...</p>
	{:else if arcs.length === 0}
		<p class="empty-msg">No arcs defined.</p>
	{:else}
		<svg
			viewBox="0 0 {viewWidth} {viewHeight}"
			preserveAspectRatio="xMidYMid meet"
			class="swimlane-svg"
			role="img"
			aria-label="Arc swimlane diagram"
		>
			{#each arcs as arc, i (arc.id)}
				{@const y = padding.top + i * LANE_HEIGHT}
				{@const color = arc.color || defaultColor(i)}

				<!-- Lane background (alternating subtle stripes) -->
				{#if i % 2 === 1}
					<rect
						x="0"
						{y}
						width={viewWidth}
						height={LANE_HEIGHT}
						fill="var(--color-bg-secondary)"
						opacity="0.3"
					/>
				{/if}

				<!-- Arc name label -->
				<text
					x={padding.left + LABEL_WIDTH - 8}
					y={y + LANE_HEIGHT / 2}
					text-anchor="end"
					dominant-baseline="middle"
					class="arc-label"
				>
					{arc.name.length > 18 ? arc.name.slice(0, 16) + '..' : arc.name}
				</text>

				<!-- Color band -->
				<rect
					x={padding.left + LABEL_WIDTH}
					y={y + 6}
					width={bandWidth}
					height={LANE_HEIGHT - 12}
					rx="4"
					fill={color}
					opacity="0.7"
					class="arc-band"
				>
					<title>{arc.name} - {getStatusLabel(arc.status)}</title>
				</rect>

				<!-- Status label on band -->
				<text
					x={padding.left + LABEL_WIDTH + bandWidth / 2}
					y={y + LANE_HEIGHT / 2}
					text-anchor="middle"
					dominant-baseline="middle"
					class="status-label"
				>
					{getStatusLabel(arc.status)}
				</text>
			{/each}
		</svg>
	{/if}
</div>

<style>
	.arc-swimlane {
		width: 100%;
		min-height: 80px;
	}

	.swimlane-svg {
		width: 100%;
		height: auto;
	}

	.arc-label {
		font-size: 10px;
		font-weight: 500;
		fill: var(--color-text-primary);
	}

	.arc-band {
		transition: opacity 0.15s ease;
		cursor: pointer;
	}

	.arc-band:hover {
		opacity: 0.9;
	}

	.status-label {
		font-size: 9px;
		font-weight: 600;
		fill: white;
		pointer-events: none;
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
	}

	.loading-msg,
	.empty-msg {
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-md);
		font-size: var(--font-size-sm);
	}
</style>
