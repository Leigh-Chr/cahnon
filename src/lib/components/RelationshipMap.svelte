<!--
  Radial graph visualization of bible entry relationships.
  Displays the selected entry at center with related entries in a circle.
  Pure SVG, no external dependencies.
-->
<script lang="ts">
	import type { BibleRelationshipWithEntry } from '$lib/api';
	import { relationshipApi } from '$lib/api';
	import { appState } from '$lib/stores';

	interface Props {
		entryId: string;
		entryName: string;
	}

	let { entryId, entryName }: Props = $props();

	let relationships = $state<BibleRelationshipWithEntry[]>([]);
	let loading = $state(true);

	$effect(() => {
		loadRelationships(entryId);
	});

	async function loadRelationships(id: string) {
		loading = true;
		try {
			relationships = await relationshipApi.getByEntry(id);
		} catch {
			relationships = [];
		}
		loading = false;
	}

	const WIDTH = 400;
	const HEIGHT = 360;
	const CX = WIDTH / 2;
	const CY = HEIGHT / 2;
	const RADIUS = 130;
	const NODE_RADIUS = 6;

	const typeColors: Record<string, string> = {
		parent: 'var(--color-accent)',
		child: 'var(--color-accent)',
		spouse: 'var(--color-error)',
		ally: 'var(--color-success)',
		rival: 'var(--color-warning)',
		sibling: 'var(--color-info)',
		member: 'var(--color-text-secondary)',
	};

	function getColor(type: string): string {
		return typeColors[type] || 'var(--color-text-muted)';
	}

	function getNodePosition(index: number, total: number) {
		const angle = (2 * Math.PI * index) / total - Math.PI / 2;
		return {
			x: CX + RADIUS * Math.cos(angle),
			y: CY + RADIUS * Math.sin(angle),
		};
	}

	function navigateToEntry(id: string) {
		appState.selectedBibleEntryId = id;
	}
</script>

<div class="relationship-map">
	{#if loading}
		<p class="loading-text">Loading relationships...</p>
	{:else if relationships.length === 0}
		<p class="empty-text">No relationships defined for this entry.</p>
	{:else}
		<svg viewBox="0 0 {WIDTH} {HEIGHT}" class="graph">
			<!-- Lines from center to each related entry -->
			{#each relationships as rel, i (rel.id)}
				{@const pos = getNodePosition(i, relationships.length)}
				<line
					x1={CX}
					y1={CY}
					x2={pos.x}
					y2={pos.y}
					stroke={getColor(rel.relationship_type)}
					stroke-width="1.5"
					opacity="0.6"
				/>
			{/each}

			<!-- Center node -->
			<circle cx={CX} cy={CY} r={NODE_RADIUS + 2} fill="var(--color-accent)" />
			<text x={CX} y={CY + NODE_RADIUS + 16} text-anchor="middle" class="node-label center-label"
				>{entryName}</text
			>

			<!-- Related entry nodes -->
			{#each relationships as rel, i (rel.id)}
				{@const pos = getNodePosition(i, relationships.length)}
				{@const angle = (2 * Math.PI * i) / relationships.length - Math.PI / 2}
				{@const isRight = Math.cos(angle) >= 0}
				<g
					class="node-group"
					onclick={() => navigateToEntry(rel.related_entry_id)}
					role="button"
					tabindex="0"
					onkeydown={(e) => e.key === 'Enter' && navigateToEntry(rel.related_entry_id)}
				>
					<circle cx={pos.x} cy={pos.y} r={NODE_RADIUS} fill={getColor(rel.relationship_type)} />
					<text
						x={pos.x + (isRight ? NODE_RADIUS + 6 : -(NODE_RADIUS + 6))}
						y={pos.y + 4}
						text-anchor={isRight ? 'start' : 'end'}
						class="node-label">{rel.related_entry_name}</text
					>
					<text
						x={pos.x + (isRight ? NODE_RADIUS + 6 : -(NODE_RADIUS + 6))}
						y={pos.y + 16}
						text-anchor={isRight ? 'start' : 'end'}
						class="type-label">{rel.relationship_type}</text
					>
				</g>
			{/each}
		</svg>
	{/if}
</div>

<style>
	.relationship-map {
		width: 100%;
		min-height: 200px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.graph {
		width: 100%;
		max-height: 360px;
	}

	.loading-text,
	.empty-text {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		text-align: center;
	}

	.node-label {
		font-size: 11px;
		fill: var(--color-text-primary);
	}

	.center-label {
		font-weight: 600;
		font-size: 12px;
	}

	.type-label {
		font-size: 9px;
		fill: var(--color-text-muted);
		text-transform: capitalize;
	}

	.node-group {
		cursor: pointer;
	}

	.node-group:hover circle {
		r: 8;
		filter: drop-shadow(0 0 3px oklch(50% 0.1 250 / 40%));
	}

	.node-group:hover .node-label {
		font-weight: 600;
	}

	.node-group:focus {
		outline: none;
	}

	.node-group:focus circle {
		r: 8;
		stroke: var(--color-accent);
		stroke-width: 2;
	}
</style>
