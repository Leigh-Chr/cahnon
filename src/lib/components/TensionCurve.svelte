<!--
  Tension Curve visualization.

  SVG polyline showing tension levels across all scenes in manuscript order.
  X-axis: scenes in order (across all chapters).
  Y-axis: tension levels (1=low, 2=medium, 3=high).
  Vertical dashed lines at chapter boundaries.
  Tooltip on hover showing scene title.
-->
<script lang="ts">
	import { appState } from '$lib/stores';

	interface ScenePoint {
		x: number;
		y: number;
		title: string;
		chapterId: string;
		tension: string | null;
	}

	// Ordered list of all scenes across chapters
	let orderedScenes = $derived.by(() => {
		const result: Array<{ title: string; tension: string | null; chapterId: string }> = [];
		for (const chapter of appState.chapters) {
			const chapterScenes = appState.scenes.get(chapter.id) || [];
			for (const scene of chapterScenes) {
				result.push({
					title: scene.title,
					tension: scene.tension,
					chapterId: chapter.id,
				});
			}
		}
		return result;
	});

	// Convert tension string to numeric value
	function tensionToY(tension: string | null): number {
		switch (tension) {
			case 'high':
				return 3;
			case 'medium':
				return 2;
			case 'low':
			default:
				return 1;
		}
	}

	// SVG dimensions
	const padding = { top: 20, right: 20, bottom: 30, left: 40 };
	const viewWidth = 600;
	const viewHeight = 200;
	const chartWidth = viewWidth - padding.left - padding.right;
	const chartHeight = viewHeight - padding.top - padding.bottom;

	// Compute scene points for polyline
	let points = $derived.by((): ScenePoint[] => {
		const total = orderedScenes.length;
		if (total === 0) return [];

		return orderedScenes.map((scene, i) => {
			const x = padding.left + (total === 1 ? chartWidth / 2 : (i / (total - 1)) * chartWidth);
			const tensionVal = tensionToY(scene.tension);
			// Y is inverted in SVG: top=high, bottom=low
			const y = padding.top + chartHeight - ((tensionVal - 1) / 2) * chartHeight;
			return {
				x,
				y,
				title: scene.title,
				chapterId: scene.chapterId,
				tension: scene.tension,
			};
		});
	});

	// Polyline string from points
	let polylineStr = $derived(points.map((p) => `${p.x},${p.y}`).join(' '));

	// Chapter boundary X positions (between last scene of chapterN and first scene of chapterN+1)
	let chapterBoundaries = $derived.by((): number[] => {
		const boundaries: number[] = [];
		const total = orderedScenes.length;
		if (total <= 1) return boundaries;

		let idx = 0;
		for (const chapter of appState.chapters) {
			const chapterScenes = appState.scenes.get(chapter.id) || [];
			idx += chapterScenes.length;
			if (idx < total && idx > 0) {
				// Boundary X is halfway between scene idx-1 and scene idx
				const xBefore = padding.left + ((idx - 1) / (total - 1)) * chartWidth;
				const xAfter = padding.left + (idx / (total - 1)) * chartWidth;
				boundaries.push((xBefore + xAfter) / 2);
			}
		}
		return boundaries;
	});

	// Y-axis labels
	const yLabels = [
		{ label: 'High', value: 3 },
		{ label: 'Mid', value: 2 },
		{ label: 'Low', value: 1 },
	];

	function yForValue(val: number): number {
		return padding.top + chartHeight - ((val - 1) / 2) * chartHeight;
	}

	// Tooltip
	let hoveredIndex = $state<number | null>(null);
</script>

<div class="tension-curve">
	{#if orderedScenes.length === 0}
		<p class="empty-msg">No scenes to display.</p>
	{:else}
		<svg
			viewBox="0 0 {viewWidth} {viewHeight}"
			preserveAspectRatio="xMidYMid meet"
			class="curve-svg"
			role="img"
			aria-label="Tension curve across scenes"
		>
			<!-- Y-axis gridlines and labels -->
			{#each yLabels as yl (yl.value)}
				<line
					x1={padding.left}
					y1={yForValue(yl.value)}
					x2={viewWidth - padding.right}
					y2={yForValue(yl.value)}
					class="gridline"
				/>
				<text
					x={padding.left - 6}
					y={yForValue(yl.value)}
					text-anchor="end"
					dominant-baseline="middle"
					class="axis-label"
				>
					{yl.label}
				</text>
			{/each}

			<!-- Chapter boundary lines -->
			{#each chapterBoundaries as bx, i (i)}
				<line
					x1={bx}
					y1={padding.top}
					x2={bx}
					y2={viewHeight - padding.bottom}
					class="chapter-line"
				/>
			{/each}

			<!-- Tension polyline -->
			{#if points.length > 1}
				<polyline points={polylineStr} class="tension-line" />
			{/if}

			<!-- Data points -->
			{#each points as point, i (i)}
				<circle
					cx={point.x}
					cy={point.y}
					r={hoveredIndex === i ? 5 : 3.5}
					class="data-point"
					class:hovered={hoveredIndex === i}
					onmouseenter={() => (hoveredIndex = i)}
					onmouseleave={() => (hoveredIndex = null)}
					role="presentation"
				>
					<title>{point.title} ({point.tension || 'low'})</title>
				</circle>
			{/each}

			<!-- Tooltip -->
			{#if hoveredIndex !== null && points[hoveredIndex]}
				{@const pt = points[hoveredIndex]}
				<g class="tooltip-group">
					<rect x={pt.x - 50} y={pt.y - 24} width="100" height="18" rx="3" class="tooltip-bg" />
					<text
						x={pt.x}
						y={pt.y - 13}
						text-anchor="middle"
						dominant-baseline="middle"
						class="tooltip-text"
					>
						{pt.title.length > 16 ? pt.title.slice(0, 14) + '...' : pt.title}
					</text>
				</g>
			{/if}
		</svg>
	{/if}
</div>

<style>
	.tension-curve {
		min-height: 200px;
		width: 100%;
	}

	.curve-svg {
		width: 100%;
		height: auto;
		min-height: 200px;
	}

	.gridline {
		stroke: var(--color-border-light, var(--color-border));
		stroke-width: 0.5;
		stroke-dasharray: 2 4;
	}

	.axis-label {
		font-size: 9px;
		fill: var(--color-text-muted);
	}

	.chapter-line {
		stroke: var(--color-border);
		stroke-width: 1;
		stroke-dasharray: 4 3;
	}

	.tension-line {
		fill: none;
		stroke: var(--color-accent);
		stroke-width: 2;
		stroke-linejoin: round;
		stroke-linecap: round;
	}

	.data-point {
		fill: var(--color-accent);
		stroke: var(--color-bg-primary);
		stroke-width: 1.5;
		cursor: pointer;
		transition: r 0.15s ease;
	}

	.data-point.hovered {
		fill: var(--color-accent);
		stroke: var(--color-accent);
		stroke-width: 2;
	}

	.tooltip-bg {
		fill: var(--color-bg-tertiary);
		stroke: var(--color-border);
		stroke-width: 0.5;
	}

	.tooltip-text {
		font-size: 8px;
		fill: var(--color-text-primary);
	}

	.empty-msg {
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-lg);
		font-size: var(--font-size-sm);
	}
</style>
