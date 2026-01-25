<!--
  Quick Open dialog (Cmd+K).

  Features:
  - Fuzzy search across scenes, chapters, and bible entries
  - Keyboard navigation (arrow keys, enter to select)
  - Type indicators (scene, chapter, character, location, etc.)
  - Instant local search + async full-text search
  - Navigate to selected item on enter
  - Auto-focus on open
-->
<script lang="ts">
	import { tick } from 'svelte';
	import { appState } from '$lib/stores';
	import { searchApi, type SearchResult } from '$lib/api';
	import { bibleEntryTypes, debounce } from '$lib/utils';

	let inputElement = $state<HTMLInputElement | null>(null);
	let query = $state('');
	let results = $state<SearchResult[]>([]);
	let localResults = $state<
		Array<{ type: string; id: string; title: string; subtitle?: string; chapterId?: string }>
	>([]);
	let selectedIndex = $state(0);
	let isSearching = $state(false);

	let allResults = $derived([
		...localResults,
		...results.map((r) => ({
			type: r.result_type,
			id: r.id,
			title: r.title,
			subtitle: r.snippet || r.parent_title || undefined,
			chapterId: r.parent_id || undefined,
		})),
	]);

	const searchRemote = debounce(async (q: string) => {
		if (q.length < 2) {
			results = [];
			return;
		}
		isSearching = true;
		try {
			results = await searchApi.global(q);
		} catch (_e) {
			results = [];
		}
		isSearching = false;
	}, 300);

	function searchLocal(q: string) {
		const lower = q.toLowerCase();
		const found: typeof localResults = [];

		// Search chapters
		for (const chapter of appState.chapters) {
			if (chapter.title.toLowerCase().includes(lower)) {
				found.push({
					type: 'chapter',
					id: chapter.id,
					title: chapter.title,
					subtitle: `Chapter • ${chapter.status}`,
				});
			}
		}

		// Search scenes
		for (const [chapterId, sceneList] of appState.scenes.entries()) {
			const chapter = appState.chapters.find((c) => c.id === chapterId);
			for (const scene of sceneList) {
				if (scene.title.toLowerCase().includes(lower)) {
					found.push({
						type: 'scene',
						id: scene.id,
						title: scene.title,
						subtitle: chapter ? `Scene in ${chapter.title}` : 'Scene',
						chapterId,
					});
				}
			}
		}

		// Search bible entries
		for (const entry of appState.bibleEntries) {
			if (
				entry.name.toLowerCase().includes(lower) ||
				(entry.aliases && entry.aliases.toLowerCase().includes(lower))
			) {
				const typeInfo = bibleEntryTypes.find((t) => t.value === entry.entry_type);
				found.push({
					type: 'bible',
					id: entry.id,
					title: entry.name,
					subtitle: typeInfo ? `${typeInfo.icon} ${typeInfo.label}` : entry.entry_type,
				});
			}
		}

		localResults = found.slice(0, 10);
	}

	function handleInput() {
		selectedIndex = 0;
		if (query.trim()) {
			searchLocal(query);
			searchRemote(query);
		} else {
			localResults = [];
			results = [];
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		switch (event.key) {
			case 'ArrowDown':
				event.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, allResults.length - 1);
				break;
			case 'ArrowUp':
				event.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Enter':
				event.preventDefault();
				if (allResults[selectedIndex]) {
					selectResult(allResults[selectedIndex]);
				}
				break;
			case 'Escape':
				event.preventDefault();
				close();
				break;
		}
	}

	function selectResult(result: (typeof allResults)[0]) {
		if (result.type === 'scene') {
			appState.selectScene(result.id, result.chapterId);
			appState.setViewMode('editor');
		} else if (result.type === 'chapter') {
			appState.selectScene('', result.id);
			appState.setViewMode('editor');
		} else if (result.type === 'bible' || result.type === 'bible_entry') {
			appState.setViewMode('bible');
			// TODO: select the bible entry
		}
		close();
	}

	function close() {
		appState.isQuickOpenVisible = false;
		query = '';
		localResults = [];
		results = [];
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			close();
		}
	}

	// Focus input when dialog opens
	$effect(() => {
		if (inputElement) {
			tick().then(() => {
				inputElement?.focus();
			});
		}
	});
</script>

<div
	class="quick-open-overlay"
	role="dialog"
	aria-modal="true"
	tabindex="-1"
	onclick={handleBackdropClick}
	onkeydown={handleKeydown}
>
	<div class="quick-open">
		<div class="search-input-wrapper">
			<svg
				class="search-icon"
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<circle cx="11" cy="11" r="8" />
				<line x1="21" y1="21" x2="16.65" y2="16.65" />
			</svg>
			<input
				bind:this={inputElement}
				type="text"
				placeholder="Search scenes, chapters, characters..."
				bind:value={query}
				oninput={handleInput}
				onkeydown={handleKeydown}
			/>
			{#if isSearching}
				<div class="loading-indicator"></div>
			{/if}
		</div>

		{#if allResults.length > 0}
			<div class="results-list">
				{#each allResults as result, i (result.id)}
					<button
						class="result-item"
						class:selected={i === selectedIndex}
						onclick={() => selectResult(result)}
						onmouseenter={() => (selectedIndex = i)}
					>
						<div class="result-icon">
							{#if result.type === 'chapter'}
								<svg
									width="16"
									height="16"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
									<path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
								</svg>
							{:else if result.type === 'scene'}
								<svg
									width="16"
									height="16"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
								</svg>
							{:else}
								<svg
									width="16"
									height="16"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<circle cx="12" cy="12" r="10" />
									<line x1="12" y1="16" x2="12" y2="12" />
									<line x1="12" y1="8" x2="12.01" y2="8" />
								</svg>
							{/if}
						</div>
						<div class="result-content">
							<span class="result-title">{result.title}</span>
							{#if result.subtitle}
								<span class="result-subtitle">{result.subtitle}</span>
							{/if}
						</div>
					</button>
				{/each}
			</div>
		{:else if query.length > 0 && !isSearching}
			<div class="no-results">
				<p>No results found</p>
			</div>
		{/if}

		<div class="quick-open-footer">
			<span><kbd>↑↓</kbd> to navigate</span>
			<span><kbd>↵</kbd> to select</span>
			<span><kbd>esc</kbd> to close</span>
		</div>
	</div>
</div>

<style>
	.quick-open-overlay {
		position: fixed;
		inset: 0;
		background-color: rgba(0, 0, 0, 0.3);
		display: flex;
		justify-content: center;
		padding-top: 100px;
		z-index: 1000;
	}

	.quick-open {
		width: 100%;
		max-width: 600px;
		max-height: 400px;
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.search-input-wrapper {
		display: flex;
		align-items: center;
		padding: var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
		gap: var(--spacing-sm);
	}

	.search-icon {
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.search-input-wrapper input {
		flex: 1;
		border: none;
		background: none;
		font-size: var(--font-size-md);
		padding: 0;
	}

	.search-input-wrapper input:focus {
		outline: none;
	}

	.loading-indicator {
		width: 16px;
		height: 16px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-accent);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.results-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-xs) 0;
	}

	.result-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		width: 100%;
		padding: var(--spacing-sm) var(--spacing-md);
		text-align: left;
		transition: background-color var(--transition-fast);
	}

	.result-item:hover,
	.result-item.selected {
		background-color: var(--color-bg-hover);
	}

	.result-item.selected {
		background-color: var(--color-accent-light);
	}

	.result-icon {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		flex-shrink: 0;
	}

	.result-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.result-title {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.result-subtitle {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.no-results {
		padding: var(--spacing-lg);
		text-align: center;
		color: var(--color-text-muted);
	}

	.quick-open-footer {
		display: flex;
		justify-content: center;
		gap: var(--spacing-lg);
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-top: 1px solid var(--color-border);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	kbd {
		display: inline-block;
		padding: 2px 6px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-family: inherit;
		font-size: inherit;
	}
</style>
