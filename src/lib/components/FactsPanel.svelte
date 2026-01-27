<!--
  Facts & Revelations Panel

  Displays facts associated with the current scene and which characters
  know each fact. Supports creating new facts, linking characters to facts,
  and deleting facts. Designed for use inside the ContextPanel sidebar.

  Features:
  - List facts revealed in or associated with the current scene
  - Show category and status badges per fact
  - Display characters who know each fact
  - Create new facts with category selection
  - Link characters to facts via search dropdown
  - Delete facts
-->
<script lang="ts">
	import { SvelteMap } from 'svelte/reactivity';

	import { type Fact, factApi, type FactCharacter } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showError } from '$lib/toast';

	import { Button, Icon } from './ui';

	interface Props {
		sceneId: string;
	}

	let { sceneId }: Props = $props();

	// Facts state
	let facts = $state<Fact[]>([]);
	let factCharacters: SvelteMap<string, FactCharacter[]> = new SvelteMap();
	let isLoading = $state(false);

	// New fact form state
	let showNewForm = $state(false);
	let newContent = $state('');
	let newCategory = $state('plot');
	let newStatus = $state('secret');

	// Character linking state
	let linkingFactId = $state<string | null>(null);
	let characterSearchQuery = $state('');

	const categories = [
		{ value: 'plot', label: 'Plot' },
		{ value: 'worldbuilding', label: 'Worldbuilding' },
		{ value: 'character', label: 'Character' },
		{ value: 'mystery', label: 'Mystery' },
	];

	const statuses = [
		{ value: 'secret', label: 'Secret' },
		{ value: 'revealed', label: 'Revealed' },
		{ value: 'common_knowledge', label: 'Common Knowledge' },
	];

	// Characters available for linking (from bible entries)
	let availableCharacters = $derived(
		appState.bibleEntries.filter((e) => e.entry_type === 'character')
	);

	let filteredCharacters = $derived(
		characterSearchQuery
			? availableCharacters.filter(
					(c) =>
						c.name.toLowerCase().includes(characterSearchQuery.toLowerCase()) ||
						(c.aliases && c.aliases.toLowerCase().includes(characterSearchQuery.toLowerCase()))
				)
			: availableCharacters
	);

	$effect(() => {
		if (sceneId) {
			loadFacts();
		}
	});

	async function loadFacts() {
		isLoading = true;
		try {
			facts = await factApi.getForScene(sceneId);
			// Load characters for each fact
			factCharacters.clear();
			await Promise.all(
				facts.map(async (fact) => {
					try {
						const chars = await factApi.getCharacters(fact.id);
						factCharacters.set(fact.id, chars);
					} catch {
						factCharacters.set(fact.id, []);
					}
				})
			);
		} catch (e) {
			console.error('Failed to load facts:', e);
			showError('Failed to load facts');
			facts = [];
		} finally {
			isLoading = false;
		}
	}

	async function createFact() {
		if (!newContent.trim()) return;
		try {
			await factApi.create({
				content: newContent.trim(),
				category: newCategory,
				status: newStatus,
				revealed_in_scene_id: sceneId,
			});
			newContent = '';
			newCategory = 'plot';
			newStatus = 'secret';
			showNewForm = false;
			await loadFacts();
		} catch (e) {
			console.error('Failed to create fact:', e);
			showError('Failed to create fact');
		}
	}

	async function deleteFact(factId: string) {
		try {
			await factApi.delete(factId);
			await loadFacts();
		} catch (e) {
			console.error('Failed to delete fact:', e);
			showError('Failed to delete fact');
		}
	}

	async function linkCharacter(factId: string, bibleEntryId: string) {
		try {
			await factApi.linkCharacter(factId, bibleEntryId, sceneId);
			// Reload characters for this fact
			const chars = await factApi.getCharacters(factId);
			factCharacters.set(factId, chars);
			linkingFactId = null;
			characterSearchQuery = '';
		} catch (e) {
			console.error('Failed to link character:', e);
			showError('Failed to link character');
		}
	}

	async function unlinkCharacter(factId: string, bibleEntryId: string) {
		try {
			await factApi.unlinkCharacter(factId, bibleEntryId);
			const chars = await factApi.getCharacters(factId);
			factCharacters.set(factId, chars);
		} catch (e) {
			console.error('Failed to unlink character:', e);
			showError('Failed to unlink character');
		}
	}

	function getCharacterName(bibleEntryId: string): string {
		const entry = appState.bibleEntries.find((e) => e.id === bibleEntryId);
		return entry?.name || 'Unknown';
	}

	function getCategoryColor(category: string): string {
		switch (category) {
			case 'plot':
				return 'var(--color-accent)';
			case 'worldbuilding':
				return 'var(--color-success, #22c55e)';
			case 'character':
				return 'var(--color-warning, #f59e0b)';
			case 'mystery':
				return 'var(--color-error, #ef4444)';
			default:
				return 'var(--color-text-muted)';
		}
	}
</script>

<div class="facts-panel">
	{#if isLoading}
		<p class="empty-message">Loading facts...</p>
	{:else}
		<!-- Facts list -->
		<div class="facts-list">
			{#each facts as fact (fact.id)}
				{@const chars = factCharacters.get(fact.id) || []}
				<div class="fact-item">
					<div class="fact-header">
						<div class="fact-badges">
							<span
								class="badge category-badge"
								style="--badge-color: {getCategoryColor(fact.category)}"
							>
								{fact.category}
							</span>
							<span class="badge status-badge" data-status={fact.status}>
								{fact.status.replace('_', ' ')}
							</span>
						</div>
						<div class="fact-actions">
							<Button
								variant="icon"
								size="sm"
								onclick={() => {
									linkingFactId = linkingFactId === fact.id ? null : fact.id;
									characterSearchQuery = '';
								}}
								title="Link character"
							>
								<Icon name="user" size={12} />
							</Button>
							<Button
								variant="icon"
								size="sm"
								onclick={() => deleteFact(fact.id)}
								title="Delete fact"
							>
								<Icon name="trash" size={12} />
							</Button>
						</div>
					</div>

					<p class="fact-content">{fact.content}</p>

					<!-- Characters who know this fact -->
					{#if chars.length > 0}
						<div class="fact-characters">
							{#each chars as fc (fc.id)}
								<span class="character-tag">
									{getCharacterName(fc.bible_entry_id)}
									<button
										class="character-remove"
										onclick={() => unlinkCharacter(fact.id, fc.bible_entry_id)}
										title="Remove character"
									>
										&times;
									</button>
								</span>
							{/each}
						</div>
					{/if}

					<!-- Character linking dropdown -->
					{#if linkingFactId === fact.id}
						<div class="character-search">
							<input
								type="text"
								placeholder="Search characters..."
								bind:value={characterSearchQuery}
							/>
							{#if filteredCharacters.length > 0}
								<div class="search-results">
									{#each filteredCharacters.slice(0, 8) as character (character.id)}
										{@const alreadyLinked = chars.some((c) => c.bible_entry_id === character.id)}
										<button
											class="search-result"
											class:disabled={alreadyLinked}
											disabled={alreadyLinked}
											onclick={() => linkCharacter(fact.id, character.id)}
										>
											<span class="result-name">{character.name}</span>
											{#if alreadyLinked}
												<span class="result-linked">linked</span>
											{/if}
										</button>
									{/each}
								</div>
							{:else}
								<p class="empty-message">No characters found.</p>
							{/if}
						</div>
					{/if}
				</div>
			{:else}
				<p class="empty-message">No facts linked to this scene.</p>
			{/each}
		</div>

		<!-- New fact button / form -->
		{#if showNewForm}
			<div class="new-fact-form">
				<textarea
					bind:value={newContent}
					placeholder="Describe the fact or revelation..."
					rows="3"
					class="fact-textarea"
				></textarea>
				<div class="form-row">
					<select bind:value={newCategory} class="fact-select">
						{#each categories as cat (cat.value)}
							<option value={cat.value}>{cat.label}</option>
						{/each}
					</select>
					<select bind:value={newStatus} class="fact-select">
						{#each statuses as st (st.value)}
							<option value={st.value}>{st.label}</option>
						{/each}
					</select>
				</div>
				<div class="form-actions">
					<Button
						variant="ghost"
						size="sm"
						onclick={() => {
							showNewForm = false;
							newContent = '';
						}}>Cancel</Button
					>
					<Button variant="primary" size="sm" onclick={createFact} disabled={!newContent.trim()}>
						Add Fact
					</Button>
				</div>
			</div>
		{:else}
			<button class="add-fact-btn" onclick={() => (showNewForm = true)}>
				<Icon name="plus" size={14} />
				Add fact
			</button>
		{/if}
	{/if}
</div>

<style>
	.facts-panel {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.facts-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.fact-item {
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		border-left: 3px solid var(--color-border);
	}

	.fact-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-xs);
	}

	.fact-badges {
		display: flex;
		gap: var(--spacing-xs);
	}

	.badge {
		display: inline-flex;
		align-items: center;
		padding: 1px 6px;
		border-radius: var(--border-radius-sm);
		font-size: 10px;
		font-weight: 500;
		text-transform: capitalize;
		line-height: 1.4;
	}

	.category-badge {
		background-color: color-mix(in srgb, var(--badge-color) 15%, transparent);
		color: var(--badge-color);
	}

	.status-badge {
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-muted);
	}

	.status-badge[data-status='secret'] {
		background-color: color-mix(in srgb, var(--color-error, #ef4444) 15%, transparent);
		color: var(--color-error, #ef4444);
	}

	.status-badge[data-status='revealed'] {
		background-color: color-mix(in srgb, var(--color-accent) 15%, transparent);
		color: var(--color-accent);
	}

	.status-badge[data-status='common_knowledge'] {
		background-color: color-mix(in srgb, var(--color-success, #22c55e) 15%, transparent);
		color: var(--color-success, #22c55e);
	}

	.fact-actions {
		display: flex;
		gap: 2px;
		opacity: 0;
		transition: opacity var(--transition-fast);
	}

	.fact-item:hover .fact-actions {
		opacity: 1;
	}

	.fact-content {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		line-height: var(--line-height-normal);
		margin: 0;
	}

	/* Characters who know a fact */
	.fact-characters {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
		margin-top: var(--spacing-xs);
	}

	.character-tag {
		display: inline-flex;
		align-items: center;
		gap: 2px;
		padding: 1px 6px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-primary);
	}

	.character-remove {
		width: 14px;
		height: 14px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		color: var(--color-text-muted);
		border-radius: 2px;
		opacity: 0;
		transition: all var(--transition-fast);
	}

	.character-tag:hover .character-remove {
		opacity: 1;
	}

	.character-remove:hover {
		background-color: var(--color-error);
		color: var(--text-on-accent);
	}

	/* Character search */
	.character-search {
		margin-top: var(--spacing-xs);
	}

	.character-search input {
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-primary);
	}

	.character-search input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.search-results {
		margin-top: var(--spacing-xs);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		max-height: 160px;
		overflow-y: auto;
	}

	.search-result {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		text-align: left;
		font-size: var(--font-size-sm);
		transition: background-color var(--transition-fast);
	}

	.search-result:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
	}

	.search-result.disabled {
		opacity: 0.5;
		cursor: default;
	}

	.result-name {
		flex: 1;
	}

	.result-linked {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		font-style: italic;
	}

	/* New fact form */
	.new-fact-form {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.fact-textarea {
		width: 100%;
		padding: var(--spacing-sm);
		font-size: var(--font-size-sm);
		font-family: inherit;
		line-height: var(--line-height-normal);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		resize: vertical;
		background-color: var(--color-bg-secondary);
	}

	.fact-textarea:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.form-row {
		display: flex;
		gap: var(--spacing-sm);
	}

	.fact-select {
		flex: 1;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
	}

	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
	}

	.add-fact-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-accent);
		border: 1px dashed var(--color-border);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
		width: 100%;
		justify-content: center;
	}

	.add-fact-btn:hover {
		background-color: var(--color-bg-hover);
		border-color: var(--color-accent);
	}

	.empty-message {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
	}
</style>
