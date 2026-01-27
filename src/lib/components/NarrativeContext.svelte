<!--
  World State at Scene N — displays the complete narrative universe state
  at the currently selected scene.

  Features:
  - Character presences with appearance count and gap indicator
  - Open setups (unfired Chekhov's guns)
  - Active arcs with progress
-->
<script lang="ts">
	import { SvelteSet } from 'svelte/reactivity';

	import type { WorldState } from '$lib/api';
	import { worldStateApi } from '$lib/api';
	import { appState } from '$lib/stores';

	let worldState = $state<WorldState | null>(null);
	let isLoading = $state(false);
	let expandedSections = new SvelteSet<string>(['characters', 'arcs']);
	let loadGeneration = 0;

	let selectedSceneId = $derived(appState.selectedSceneId);

	$effect(() => {
		if (selectedSceneId) {
			loadWorldState(selectedSceneId);
		} else {
			worldState = null;
		}
	});

	async function loadWorldState(sceneId: string) {
		const gen = ++loadGeneration;
		isLoading = true;
		try {
			const result = await worldStateApi.getAtScene(sceneId);
			if (gen !== loadGeneration) return;
			worldState = result;
		} catch (e) {
			if (gen !== loadGeneration) return;
			console.error('Failed to load world state:', e);
			worldState = null;
		} finally {
			if (gen === loadGeneration) {
				isLoading = false;
			}
		}
	}

	function toggleSection(section: string) {
		if (expandedSections.has(section)) {
			expandedSections.delete(section);
		} else {
			expandedSections.add(section);
		}
	}
</script>

<div class="narrative-context">
	{#if isLoading}
		<p class="loading">Loading world state...</p>
	{:else if worldState}
		<!-- Character Presences -->
		<div class="accordion-section">
			<button class="accordion-header" onclick={() => toggleSection('characters')}>
				<span class="accordion-icon">{expandedSections.has('characters') ? '▾' : '▸'}</span>
				<span class="accordion-title">Characters ({worldState.character_presences.length})</span>
			</button>
			{#if expandedSections.has('characters')}
				<div class="accordion-body">
					{#each worldState.character_presences as char (char.bible_entry_id)}
						<div class="character-row" class:present-here={char.present_here}>
							<button
								class="char-name"
								onclick={() => appState.showCharacterThread(char.bible_entry_id)}
							>
								{char.name}
								{#if char.present_here}<span class="here-badge">here</span>{/if}
							</button>
							<span class="char-meta">
								{char.appearance_count} scene{char.appearance_count !== 1 ? 's' : ''}
								{#if char.gap_scenes > 0 && !char.present_here}
									<span class="gap-indicator" title="Not seen for {char.gap_scenes} scenes">
										({char.gap_scenes} ago)
									</span>
								{/if}
							</span>
						</div>
					{:else}
						<p class="empty">No characters yet</p>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Open Setups -->
		{#if worldState.open_setups.length > 0}
			<div class="accordion-section">
				<button class="accordion-header" onclick={() => toggleSection('setups')}>
					<span class="accordion-icon">{expandedSections.has('setups') ? '▾' : '▸'}</span>
					<span class="accordion-title">Open Setups ({worldState.open_setups.length})</span>
				</button>
				{#if expandedSections.has('setups')}
					<div class="accordion-body">
						{#each worldState.open_setups as setup (setup.scene_id)}
							<div class="setup-item">
								<span class="setup-scene">{setup.scene_title}</span>
								<span class="setup-hint">Setup pending payoff</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}

		<!-- Active Arcs -->
		<div class="accordion-section">
			<button class="accordion-header" onclick={() => toggleSection('arcs')}>
				<span class="accordion-icon">{expandedSections.has('arcs') ? '▾' : '▸'}</span>
				<span class="accordion-title">Active Arcs ({worldState.active_arcs.length})</span>
			</button>
			{#if expandedSections.has('arcs')}
				<div class="accordion-body">
					{#each worldState.active_arcs as arc (arc.arc_id)}
						<div class="arc-row">
							<span class="arc-dot" style="background-color: {arc.color || 'var(--color-accent)'}"
							></span>
							<span class="arc-name">{arc.arc_name}</span>
							<span class="arc-progress">
								{arc.scenes_before}/{arc.scenes_total}
							</span>
						</div>
					{:else}
						<p class="empty">No arcs active yet</p>
					{/each}
				</div>
			{/if}
		</div>
	{:else}
		<p class="empty">Select a scene to view world state</p>
	{/if}
</div>

<style>
	.narrative-context {
		display: flex;
		flex-direction: column;
		gap: 0;
	}

	.loading {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
		padding: var(--spacing-sm);
	}

	.accordion-section {
		border-bottom: 1px solid var(--color-border-light);
	}

	.accordion-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		width: 100%;
		padding: var(--spacing-sm);
		font-size: var(--font-size-xs);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
		text-align: left;
		transition: background-color var(--transition-fast);
	}

	.accordion-header:hover {
		background-color: var(--color-bg-hover);
	}

	.accordion-icon {
		font-size: var(--font-size-xs);
		width: 12px;
		flex-shrink: 0;
	}

	.accordion-title {
		flex: 1;
	}

	.accordion-body {
		padding: 0 var(--spacing-sm) var(--spacing-sm);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.empty {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
		padding: var(--spacing-sm);
	}

	/* Character rows */
	.character-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 3px var(--spacing-xs);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.character-row.present-here {
		background-color: var(--color-bg-primary);
	}

	.char-name {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		color: var(--color-text-primary);
		font-weight: 500;
		background: none;
		border: none;
		padding: 0;
		font-size: inherit;
		text-align: left;
	}

	.char-name:hover {
		color: var(--color-accent);
	}

	.here-badge {
		font-size: 10px;
		padding: 1px 4px;
		border-radius: 3px;
		background-color: var(--color-accent-light);
		color: var(--color-accent);
		font-weight: 500;
	}

	.char-meta {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.gap-indicator {
		color: var(--color-warning);
	}

	/* Setup items */
	.setup-item {
		display: flex;
		flex-direction: column;
		padding: var(--spacing-xs);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		border-left: 3px solid var(--color-warning);
	}

	.setup-scene {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.setup-hint {
		font-size: var(--font-size-xs);
		color: var(--color-warning);
	}

	/* Arc rows */
	.arc-row {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding: 3px var(--spacing-xs);
		font-size: var(--font-size-sm);
	}

	.arc-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.arc-name {
		flex: 1;
		color: var(--color-text-primary);
	}

	.arc-progress {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}
</style>
