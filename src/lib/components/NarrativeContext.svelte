<!--
  World State at Scene N — displays the complete narrative universe state
  at the currently selected scene.

  Features:
  - Character presences with appearance count and gap indicator
  - Character knowledge states (facts known)
  - Open setups (unfired Chekhov's guns)
  - Active arcs with progress
  - Dramatic irony (reader knows, character doesn't)
  - Location history (previous scenes at same location)
-->
<script lang="ts">
	import { SvelteSet } from 'svelte/reactivity';

	import type { WorldState } from '$lib/api';
	import { worldStateApi } from '$lib/api';
	import { appState } from '$lib/stores';

	let worldState = $state<WorldState | null>(null);
	let isLoading = $state(false);
	let expandedSections = new SvelteSet<string>(['characters', 'arcs']);

	let selectedSceneId = $derived(appState.selectedSceneId);

	$effect(() => {
		if (selectedSceneId) {
			loadWorldState(selectedSceneId);
		} else {
			worldState = null;
		}
	});

	async function loadWorldState(sceneId: string) {
		isLoading = true;
		try {
			worldState = await worldStateApi.getAtScene(sceneId);
		} catch (e) {
			console.error('Failed to load world state:', e);
			worldState = null;
		} finally {
			isLoading = false;
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
							<span class="char-name">
								{char.name}
								{#if char.present_here}<span class="here-badge">here</span>{/if}
							</span>
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

		<!-- Character Knowledge -->
		{#if worldState.character_knowledge.length > 0}
			<div class="accordion-section">
				<button class="accordion-header" onclick={() => toggleSection('knowledge')}>
					<span class="accordion-icon">{expandedSections.has('knowledge') ? '▾' : '▸'}</span>
					<span class="accordion-title">Knowledge</span>
				</button>
				{#if expandedSections.has('knowledge')}
					<div class="accordion-body">
						{#each worldState.character_knowledge as ck (ck.bible_entry_id)}
							{#if ck.known_facts.length > 0}
								<div class="knowledge-group">
									<span class="knowledge-name">{ck.name}</span>
									<ul class="fact-list">
										{#each ck.known_facts as fact, i (i)}
											<li>{fact}</li>
										{/each}
									</ul>
								</div>
							{/if}
						{/each}
					</div>
				{/if}
			</div>
		{/if}

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

		<!-- Dramatic Irony -->
		{#if worldState.dramatic_irony.length > 0}
			<div class="accordion-section">
				<button class="accordion-header" onclick={() => toggleSection('irony')}>
					<span class="accordion-icon">{expandedSections.has('irony') ? '▾' : '▸'}</span>
					<span class="accordion-title">Dramatic Irony ({worldState.dramatic_irony.length})</span>
				</button>
				{#if expandedSections.has('irony')}
					<div class="accordion-body">
						{#each worldState.dramatic_irony as item, i (i)}
							<div class="irony-item">
								<span class="irony-fact">{item.fact_content}</span>
								<span class="irony-meta">
									{item.character_name} doesn't know
									<span class="irony-source">(revealed in "{item.revealed_in_scene_title}")</span>
								</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}

		<!-- Location History -->
		{#if worldState.location_history.length > 0}
			<div class="accordion-section">
				<button class="accordion-header" onclick={() => toggleSection('location')}>
					<span class="accordion-icon">{expandedSections.has('location') ? '▾' : '▸'}</span>
					<span class="accordion-title"
						>Location History ({worldState.location_history.length})</span
					>
				</button>
				{#if expandedSections.has('location')}
					<div class="accordion-body">
						{#each worldState.location_history as loc (loc.scene_id)}
							<div class="location-item">
								<span class="loc-title">{loc.scene_title}</span>
								<span class="loc-chapter">{loc.chapter_title}</span>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/if}
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

	/* Knowledge */
	.knowledge-group {
		padding: var(--spacing-xs) 0;
	}

	.knowledge-name {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.fact-list {
		margin: 2px 0 0;
		padding-left: var(--spacing-md);
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
	}

	.fact-list li {
		margin-bottom: 1px;
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

	/* Dramatic irony */
	.irony-item {
		display: flex;
		flex-direction: column;
		padding: var(--spacing-xs);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		border-left: 3px solid var(--color-info);
	}

	.irony-fact {
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		font-style: italic;
	}

	.irony-meta {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.irony-source {
		font-style: italic;
	}

	/* Location history */
	.location-item {
		display: flex;
		justify-content: space-between;
		padding: 3px var(--spacing-xs);
		font-size: var(--font-size-sm);
	}

	.loc-title {
		color: var(--color-text-primary);
	}

	.loc-chapter {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}
</style>
