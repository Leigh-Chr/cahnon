<!--
  Character Heatmap visualization.

  SVG grid showing character appearances per scene.
  Y-axis: characters (bible entries with type='character').
  X-axis: scenes in manuscript order.
  Cells are filled if scene.pov matches the character name.
  Max 50 scenes shown, max 20 characters. Scrollable container.
-->
<script lang="ts">
	import { appState } from '$lib/stores';

	const MAX_SCENES = 50;
	const MAX_CHARACTERS = 20;
	const CELL_SIZE = 16;
	const LABEL_WIDTH = 120;
	const HEADER_HEIGHT = 40;

	// Characters from bible entries
	let characters = $derived(
		appState.bibleEntries.filter((e) => e.entry_type === 'character').slice(0, MAX_CHARACTERS)
	);

	// All scenes in manuscript order
	let orderedScenes = $derived.by(() => {
		const result: Array<{
			id: string;
			title: string;
			pov: string | null;
			index: number;
		}> = [];
		let idx = 0;
		for (const chapter of appState.chapters) {
			const chapterScenes = appState.scenes.get(chapter.id) || [];
			for (const scene of chapterScenes) {
				result.push({
					id: scene.id,
					title: scene.title,
					pov: scene.pov,
					index: idx,
				});
				idx++;
			}
		}
		return result.slice(0, MAX_SCENES);
	});

	// Determine if a character appears in a scene (by POV match)
	function isPresent(characterName: string, scene: { pov: string | null }): boolean {
		if (!scene.pov) return false;
		return scene.pov.toLowerCase() === characterName.toLowerCase();
	}

	// SVG dimensions
	let svgWidth = $derived(LABEL_WIDTH + orderedScenes.length * CELL_SIZE + 10);
	let svgHeight = $derived(HEADER_HEIGHT + characters.length * CELL_SIZE + 10);

	// Tooltip
	let hoveredCell = $state<{ charName: string; sceneName: string } | null>(null);
	let tooltipPos = $state({ x: 0, y: 0 });
</script>

<div class="character-heatmap">
	{#if characters.length === 0 || orderedScenes.length === 0}
		<p class="empty-msg">
			{characters.length === 0 ? 'No characters in bible.' : 'No scenes to display.'}
		</p>
	{:else}
		<div class="heatmap-scroll">
			<svg
				viewBox="0 0 {svgWidth} {svgHeight}"
				width={svgWidth}
				height={svgHeight}
				class="heatmap-svg"
				role="img"
				aria-label="Character appearance heatmap"
			>
				<!-- Scene number headers -->
				{#each orderedScenes as scene, si (scene.id)}
					<text
						x={LABEL_WIDTH + si * CELL_SIZE + CELL_SIZE / 2}
						y={HEADER_HEIGHT - 6}
						text-anchor="middle"
						class="header-label"
						transform="rotate(-45 {LABEL_WIDTH + si * CELL_SIZE + CELL_SIZE / 2} {HEADER_HEIGHT -
							6})"
					>
						{si + 1}
					</text>
				{/each}

				<!-- Character rows -->
				{#each characters as char, ci (char.id)}
					<!-- Character name label -->
					<text
						x={LABEL_WIDTH - 6}
						y={HEADER_HEIGHT + ci * CELL_SIZE + CELL_SIZE / 2}
						text-anchor="end"
						dominant-baseline="middle"
						class="char-label"
					>
						{char.name.length > 14 ? char.name.slice(0, 12) + '..' : char.name}
					</text>

					<!-- Grid cells -->
					{#each orderedScenes as scene, si (scene.id)}
						{@const present = isPresent(char.name, scene)}
						<rect
							x={LABEL_WIDTH + si * CELL_SIZE}
							y={HEADER_HEIGHT + ci * CELL_SIZE}
							width={CELL_SIZE - 1}
							height={CELL_SIZE - 1}
							rx="2"
							class="cell"
							class:filled={present}
							onmouseenter={(e) => {
								hoveredCell = { charName: char.name, sceneName: scene.title };
								const rect = (e.target as SVGElement).getBoundingClientRect();
								tooltipPos = { x: rect.x + rect.width / 2, y: rect.y };
							}}
							onmouseleave={() => (hoveredCell = null)}
							role="presentation"
						>
							<title>{char.name} - {scene.title}{present ? ' (POV)' : ''}</title>
						</rect>
					{/each}
				{/each}
			</svg>
		</div>

		{#if hoveredCell}
			<div class="tooltip" style="left: {tooltipPos.x}px; top: {tooltipPos.y - 4}px;">
				<strong>{hoveredCell.charName}</strong> - {hoveredCell.sceneName}
			</div>
		{/if}
	{/if}
</div>

<style>
	.character-heatmap {
		position: relative;
		min-height: 120px;
		width: 100%;
	}

	.heatmap-scroll {
		overflow-x: auto;
		overflow-y: auto;
		max-height: 400px;
	}

	.heatmap-svg {
		display: block;
	}

	.header-label {
		font-size: 8px;
		fill: var(--color-text-muted);
	}

	.char-label {
		font-size: 9px;
		fill: var(--color-text-secondary);
	}

	.cell {
		fill: var(--color-bg-tertiary);
		cursor: pointer;
		transition: opacity 0.1s ease;
	}

	.cell:hover {
		opacity: 0.8;
		stroke: var(--color-text-muted);
		stroke-width: 1;
	}

	.cell.filled {
		fill: var(--color-accent);
	}

	.tooltip {
		position: fixed;
		transform: translate(-50%, -100%);
		padding: 4px 8px;
		background: var(--color-bg-tertiary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-primary);
		white-space: nowrap;
		pointer-events: none;
		z-index: 100;
		box-shadow: var(--shadow-sm);
	}

	.empty-msg {
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-lg);
		font-size: var(--font-size-sm);
	}
</style>
