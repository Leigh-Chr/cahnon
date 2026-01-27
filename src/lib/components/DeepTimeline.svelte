<!--
  Deep Timeline visualization.

  SVG grid with time on X-axis and characters (POV) on Y-axis.
  Shows scenes that have time_point or time_start set and on_timeline=true.
  Points for each scene, connected by lines for same character (POV).
-->
<script lang="ts">
	import { SvelteMap } from 'svelte/reactivity';

	import { appState } from '$lib/stores';

	interface TimelineScene {
		id: string;
		title: string;
		pov: string;
		timeLabel: string;
		timeSortKey: string;
	}

	// Collect all timeline-eligible scenes
	let timelineScenes = $derived.by((): TimelineScene[] => {
		const result: TimelineScene[] = [];
		for (const chapter of appState.chapters) {
			const chapterScenes = appState.scenes.get(chapter.id) || [];
			for (const scene of chapterScenes) {
				if (!scene.on_timeline) continue;
				const timeLabel = scene.time_point || scene.time_start;
				if (!timeLabel) continue;
				if (!scene.pov) continue;

				result.push({
					id: scene.id,
					title: scene.title,
					pov: scene.pov,
					timeLabel,
					timeSortKey: timeLabel,
				});
			}
		}
		// Sort by time label alphabetically (simple string sort)
		result.sort((a, b) => a.timeSortKey.localeCompare(b.timeSortKey));
		return result;
	});

	// Unique time points (X-axis)
	let uniqueTimes = $derived([...new Set(timelineScenes.map((s) => s.timeLabel))]);

	// Unique characters/POV (Y-axis)
	let uniqueCharacters = $derived([...new Set(timelineScenes.map((s) => s.pov))]);

	// Grid layout
	const CELL_WIDTH = 80;
	const CELL_HEIGHT = 40;
	const LABEL_WIDTH = 120;
	const HEADER_HEIGHT = 50;
	const padding = { top: 10, right: 20, bottom: 10, left: 10 };

	let svgWidth = $derived(
		padding.left + LABEL_WIDTH + uniqueTimes.length * CELL_WIDTH + padding.right
	);
	let svgHeight = $derived(
		padding.top + HEADER_HEIGHT + uniqueCharacters.length * CELL_HEIGHT + padding.bottom
	);

	// Map time -> X position
	function xForTime(time: string): number {
		const idx = uniqueTimes.indexOf(time);
		return padding.left + LABEL_WIDTH + idx * CELL_WIDTH + CELL_WIDTH / 2;
	}

	// Map character -> Y position
	function yForCharacter(character: string): number {
		const idx = uniqueCharacters.indexOf(character);
		return padding.top + HEADER_HEIGHT + idx * CELL_HEIGHT + CELL_HEIGHT / 2;
	}

	// Group scenes by POV for connecting lines
	let characterLines = $derived.by(
		(): Array<{ pov: string; points: Array<{ x: number; y: number }> }> => {
			const groups = new SvelteMap<string, Array<{ x: number; y: number }>>();
			for (const scene of timelineScenes) {
				const x = xForTime(scene.timeLabel);
				const y = yForCharacter(scene.pov);
				if (!groups.has(scene.pov)) {
					groups.set(scene.pov, []);
				}
				groups.get(scene.pov)!.push({ x, y });
			}
			return [...groups.entries()].map(([pov, pts]) => ({
				pov,
				points: pts.sort((a, b) => a.x - b.x),
			}));
		}
	);

	// Tooltip state
	let hoveredScene = $state<TimelineScene | null>(null);
</script>

<div class="deep-timeline">
	{#if timelineScenes.length === 0}
		<p class="empty-msg">
			No timeline scenes. Set <em>on_timeline</em> and a <em>time_point</em> on scenes to see them here.
		</p>
	{:else}
		<div class="timeline-scroll">
			<svg
				viewBox="0 0 {svgWidth} {svgHeight}"
				width={svgWidth}
				height={svgHeight}
				class="timeline-svg"
				role="img"
				aria-label="Deep timeline grid"
			>
				<!-- Time headers (X-axis) -->
				{#each uniqueTimes as time, ti (time)}
					{@const x = padding.left + LABEL_WIDTH + ti * CELL_WIDTH + CELL_WIDTH / 2}
					<text {x} y={padding.top + HEADER_HEIGHT - 10} text-anchor="middle" class="time-label">
						{time.length > 10 ? time.slice(0, 9) + '..' : time}
					</text>
					<!-- Vertical gridline -->
					<line
						x1={x}
						y1={padding.top + HEADER_HEIGHT}
						x2={x}
						y2={svgHeight - padding.bottom}
						class="gridline"
					/>
				{/each}

				<!-- Character labels (Y-axis) -->
				{#each uniqueCharacters as char, ci (char)}
					{@const y = padding.top + HEADER_HEIGHT + ci * CELL_HEIGHT + CELL_HEIGHT / 2}
					<text
						x={padding.left + LABEL_WIDTH - 8}
						{y}
						text-anchor="end"
						dominant-baseline="middle"
						class="char-label"
					>
						{char.length > 14 ? char.slice(0, 12) + '..' : char}
					</text>
					<!-- Horizontal gridline -->
					<line
						x1={padding.left + LABEL_WIDTH}
						y1={y}
						x2={svgWidth - padding.right}
						y2={y}
						class="gridline"
					/>
				{/each}

				<!-- Connection lines per character -->
				{#each characterLines as group (group.pov)}
					{#if group.points.length > 1}
						<polyline
							points={group.points.map((p) => `${p.x},${p.y}`).join(' ')}
							class="connection-line"
						/>
					{/if}
				{/each}

				<!-- Scene points -->
				{#each timelineScenes as scene (scene.id)}
					{@const cx = xForTime(scene.timeLabel)}
					{@const cy = yForCharacter(scene.pov)}
					<circle
						{cx}
						{cy}
						r={hoveredScene?.id === scene.id ? 6 : 4.5}
						class="scene-point"
						class:hovered={hoveredScene?.id === scene.id}
						onmouseenter={() => (hoveredScene = scene)}
						onmouseleave={() => (hoveredScene = null)}
						role="presentation"
					>
						<title>{scene.title} ({scene.pov}, {scene.timeLabel})</title>
					</circle>
				{/each}
			</svg>
		</div>

		{#if hoveredScene}
			<div class="info-bar">
				<strong>{hoveredScene.title}</strong>
				<span class="info-detail">POV: {hoveredScene.pov}</span>
				<span class="info-detail">Time: {hoveredScene.timeLabel}</span>
			</div>
		{/if}
	{/if}
</div>

<style>
	.deep-timeline {
		width: 100%;
		min-height: 120px;
		position: relative;
	}

	.timeline-scroll {
		overflow-x: auto;
		overflow-y: auto;
		max-height: 500px;
	}

	.timeline-svg {
		display: block;
	}

	.time-label {
		font-size: 9px;
		fill: var(--color-text-secondary);
		font-weight: 500;
	}

	.char-label {
		font-size: 9px;
		fill: var(--color-text-secondary);
	}

	.gridline {
		stroke: var(--color-border-light, var(--color-border));
		stroke-width: 0.5;
		stroke-dasharray: 2 4;
	}

	.connection-line {
		fill: none;
		stroke: var(--color-accent);
		stroke-width: 1.5;
		stroke-opacity: 0.4;
		stroke-linejoin: round;
	}

	.scene-point {
		fill: var(--color-accent);
		stroke: var(--color-bg-primary);
		stroke-width: 2;
		cursor: pointer;
		transition: r 0.15s ease;
	}

	.scene-point.hovered {
		stroke: var(--color-accent);
		stroke-width: 2.5;
	}

	.info-bar {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		padding: var(--spacing-xs) var(--spacing-sm);
		background: var(--color-bg-secondary);
		border-top: 1px solid var(--color-border);
		font-size: var(--font-size-xs);
		color: var(--color-text-primary);
	}

	.info-detail {
		color: var(--color-text-muted);
	}

	.empty-msg {
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-lg);
		font-size: var(--font-size-sm);
	}

	.empty-msg em {
		color: var(--color-text-secondary);
	}
</style>
