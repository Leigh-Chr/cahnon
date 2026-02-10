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

	import {
		type Arc,
		arcApi,
		eventApi,
		searchApi,
		type SearchResult,
		type TimelineEvent,
	} from '$lib/api';
	import { appState } from '$lib/stores';
	import { bibleEntryTypes, countWords, debounce } from '$lib/utils';

	// Pre-load arcs and events for local search
	let cachedArcs = $state<Arc[]>([]);
	let cachedEvents = $state<TimelineEvent[]>([]);

	$effect(() => {
		if (appState.isQuickOpenVisible) {
			arcApi
				.getAll()
				.then((a) => {
					cachedArcs = a;
				})
				.catch(() => {
					cachedArcs = [];
				});
			eventApi
				.getAll()
				.then((e) => {
					cachedEvents = e;
				})
				.catch(() => {
					cachedEvents = [];
				});
		}
	});

	let inputElement = $state<HTMLInputElement | null>(null);
	let query = $state('');

	// AK4: Debounced query to prevent reordering during typing
	let debouncedQuery = $state('');
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	$effect(() => {
		// Clear previous timer
		if (debounceTimer) clearTimeout(debounceTimer);

		// Set new timer - short delay for local results stability
		debounceTimer = setTimeout(() => {
			debouncedQuery = query;
		}, 100);

		return () => {
			if (debounceTimer) clearTimeout(debounceTimer);
		};
	});

	// Y6: Save focus to restore on close
	let previouslyFocused: Element | null = null;
	let results = $state<SearchResult[]>([]);
	let localResults = $state<
		Array<{ type: string; id: string; title: string; subtitle?: string; chapterId?: string }>
	>([]);
	let selectedIndex = $state(0);
	let isSearching = $state(false);

	// BC4: Recent items (stored in localStorage)
	const RECENT_KEY = 'cahnon-quickopen-recent';
	const MAX_RECENT = 5;

	type RecentItem = {
		type: string;
		id: string;
		title: string;
		subtitle?: string;
		chapterId?: string;
	};

	function loadRecentItems(): RecentItem[] {
		try {
			return JSON.parse(localStorage.getItem(RECENT_KEY) || '[]');
		} catch {
			return [];
		}
	}

	let recentItems = $state<RecentItem[]>(loadRecentItems());

	// CE5: Saved searches
	const SAVED_SEARCHES_KEY = 'cahnon-saved-searches';
	const MAX_SAVED = 10;

	function loadSavedSearches(): string[] {
		try {
			return JSON.parse(localStorage.getItem(SAVED_SEARCHES_KEY) || '[]');
		} catch {
			return [];
		}
	}

	let savedSearches = $state<string[]>(loadSavedSearches());

	function saveCurrentSearch() {
		if (!query.trim()) return;
		const updated = [query.trim(), ...savedSearches.filter((s) => s !== query.trim())].slice(
			0,
			MAX_SAVED
		);
		savedSearches = updated;
		try {
			localStorage.setItem(SAVED_SEARCHES_KEY, JSON.stringify(updated));
		} catch {
			// localStorage unavailable
		}
	}

	function removeSavedSearch(search: string) {
		savedSearches = savedSearches.filter((s) => s !== search);
		try {
			localStorage.setItem(SAVED_SEARCHES_KEY, JSON.stringify(savedSearches));
		} catch {
			// localStorage unavailable
		}
	}

	function applySavedSearch(search: string) {
		query = search;
		handleInput();
	}

	// CE4: Search operators parser
	interface ParsedQuery {
		text: string;
		filters: {
			status?: string;
			pov?: string;
			type?: string;
		};
	}

	function parseQueryOperators(q: string): ParsedQuery {
		const filters: ParsedQuery['filters'] = {};
		const operatorRegex = /(\w+):(\S+)/g;
		let match;
		let text = q;

		while ((match = operatorRegex.exec(q)) !== null) {
			const [full, key, value] = match;
			if (key === 'status') filters.status = value.toLowerCase();
			else if (key === 'pov') filters.pov = value.toLowerCase();
			else if (key === 'type') filters.type = value.toLowerCase();
			text = text.replace(full, '');
		}

		return { text: text.trim(), filters };
	}

	function addToRecent(result: RecentItem) {
		const updated = [result, ...recentItems.filter((r) => r.id !== result.id)].slice(0, MAX_RECENT);
		recentItems = updated;
		try {
			localStorage.setItem(RECENT_KEY, JSON.stringify(updated));
		} catch {
			// localStorage unavailable
		}
	}

	type FilterType = 'all' | 'chapter' | 'scene' | 'bible' | 'arc' | 'event';
	let activeFilter = $state<FilterType>('all');
	const filterOptions: Array<{ value: FilterType; label: string }> = [
		{ value: 'all', label: 'All' },
		{ value: 'chapter', label: 'Chapters' },
		{ value: 'scene', label: 'Scenes' },
		{ value: 'bible', label: 'Bible' },
		{ value: 'arc', label: 'Arcs' },
		{ value: 'event', label: 'Events' },
	];

	// AK4: Use debouncedQuery for stable sorting
	let allResults = $derived.by(() => {
		let combined = [
			...localResults,
			...results.map((r) => ({
				type: r.result_type,
				id: r.id,
				title: r.title,
				subtitle: r.snippet || r.parent_title || undefined,
				chapterId: r.parent_id || undefined,
			})),
		];
		if (activeFilter !== 'all') {
			combined = combined.filter((r) => {
				if (activeFilter === 'bible') return r.type === 'bible' || r.type === 'bible_entry';
				return r.type === activeFilter;
			});
		}
		// Use debouncedQuery for sorting to prevent reordering during typing
		if (!debouncedQuery.trim()) return combined;
		return combined.sort(
			(a, b) => relevanceScore(b.title, debouncedQuery) - relevanceScore(a.title, debouncedQuery)
		);
	});

	// AQ1: Group results by type with headers when showing all
	const typeLabels: Record<string, string> = {
		chapter: 'Chapters',
		scene: 'Scenes',
		bible: 'Bible',
		bible_entry: 'Bible',
		arc: 'Arcs',
		event: 'Events',
	};

	let groupedResults = $derived.by(() => {
		if (activeFilter !== 'all' || allResults.length === 0) return null;

		const groups: { type: string; label: string; items: typeof allResults }[] = [];
		const typeOrder = ['chapter', 'scene', 'bible', 'arc', 'event'];

		for (const type of typeOrder) {
			const items = allResults.filter((r) =>
				type === 'bible' ? r.type === 'bible' || r.type === 'bible_entry' : r.type === type
			);
			if (items.length > 0) {
				groups.push({
					type,
					label: typeLabels[type] || type,
					items,
				});
			}
		}
		return groups;
	});

	// Flat list index calculation for keyboard navigation with groups
	let flatResultsList = $derived.by(() => {
		if (groupedResults) {
			return groupedResults.flatMap((g) => g.items);
		}
		return allResults;
	});

	// UC1: Preview data for highlighted item
	let previewData = $derived.by(() => {
		const list = !query && recentItems.length > 0 ? recentItems : flatResultsList;
		const item = list[selectedIndex];
		if (!item) return null;

		if (item.type === 'scene') {
			// Find full scene data
			for (const [chapterId, scenes] of appState.scenes) {
				const scene = scenes.find((s) => s.id === item.id);
				if (scene) {
					const chapter = appState.chapters.find((c) => c.id === chapterId);
					return {
						type: 'scene' as const,
						title: scene.title,
						chapter: chapter?.title || 'Unknown Chapter',
						summary: scene.summary || null,
						text: scene.text || '',
						wordCount: countWords(scene.text || ''),
						status: scene.status,
						pov: scene.pov,
					};
				}
			}
		} else if (item.type === 'bible' || item.type === 'bible_entry') {
			const entry = appState.bibleEntries.find((e) => e.id === item.id);
			if (entry) {
				return {
					type: 'bible' as const,
					title: entry.name,
					entryType: entry.entry_type,
					description: entry.full_description || entry.summary || null,
					aliases: entry.aliases || null,
				};
			}
		} else if (item.type === 'chapter') {
			const chapter = appState.chapters.find((c) => c.id === item.id);
			if (chapter) {
				const scenes = appState.scenes.get(item.id) || [];
				const totalWords = scenes.reduce((sum, s) => sum + countWords(s.text || ''), 0);
				return {
					type: 'chapter' as const,
					title: chapter.title,
					sceneCount: scenes.length,
					wordCount: totalWords,
					status: chapter.status,
				};
			}
		}
		return null;
	});

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

	function relevanceScore(title: string, q: string): number {
		const lower = title.toLowerCase();
		const query = q.toLowerCase();
		if (lower === query) return 100;
		if (lower.startsWith(query)) return 80;
		const wordStart = lower.split(/\s+/).some((w) => w.startsWith(query));
		if (wordStart) return 60;
		return 40;
	}

	function searchLocal(q: string) {
		// CE4: Parse search operators
		const { text, filters } = parseQueryOperators(q);
		const lower = text.toLowerCase();
		const found: typeof localResults = [];

		// CE4: Filter by type if specified
		const searchChapters = !filters.type || filters.type === 'chapter';
		const searchScenes = !filters.type || filters.type === 'scene';
		const searchBible = !filters.type || filters.type === 'bible';
		const searchArcs = !filters.type || filters.type === 'arc';
		const searchEvents = !filters.type || filters.type === 'event';

		// Search chapters
		if (searchChapters) {
			for (const chapter of appState.chapters) {
				if (!lower || chapter.title.toLowerCase().includes(lower)) {
					// CE4: Filter by status
					if (filters.status && chapter.status?.toLowerCase() !== filters.status) continue;
					found.push({
						type: 'chapter',
						id: chapter.id,
						title: chapter.title,
						subtitle: `Chapter • ${chapter.status}`,
					});
				}
			}
		}

		// Search scenes
		if (searchScenes) {
			for (const [chapterId, sceneList] of appState.scenes.entries()) {
				const chapter = appState.chapters.find((c) => c.id === chapterId);
				for (const scene of sceneList) {
					if (!lower || scene.title.toLowerCase().includes(lower)) {
						// CE4: Filter by status
						if (filters.status && scene.status?.toLowerCase() !== filters.status) continue;
						// CE4: Filter by POV
						if (filters.pov && (!scene.pov || !scene.pov.toLowerCase().includes(filters.pov)))
							continue;
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
		}

		// Search bible entries
		if (searchBible) {
			for (const entry of appState.bibleEntries) {
				if (
					!lower ||
					entry.name.toLowerCase().includes(lower) ||
					(entry.aliases && entry.aliases.toLowerCase().includes(lower))
				) {
					// CE4: Filter by status
					if (filters.status && entry.status?.toLowerCase() !== filters.status) continue;
					// CE4: Filter by type (bible entry type)
					if (filters.type && filters.type !== 'bible' && entry.entry_type !== filters.type)
						continue;
					const typeInfo = bibleEntryTypes.find((t) => t.value === entry.entry_type);
					found.push({
						type: 'bible',
						id: entry.id,
						title: entry.name,
						subtitle: typeInfo ? `${typeInfo.icon} ${typeInfo.label}` : entry.entry_type,
					});
				}
			}
		}

		// Search arcs
		if (searchArcs) {
			for (const arc of cachedArcs) {
				if (!lower || arc.name.toLowerCase().includes(lower)) {
					// CE4: Filter by status
					if (filters.status && arc.status?.toLowerCase() !== filters.status) continue;
					found.push({
						type: 'arc',
						id: arc.id,
						title: arc.name,
						subtitle: 'Arc',
					});
				}
			}
		}

		// Search events
		if (searchEvents) {
			for (const event of cachedEvents) {
				if (!lower || event.title.toLowerCase().includes(lower)) {
					found.push({
						type: 'event',
						id: event.id,
						title: event.title,
						subtitle: 'Event',
					});
				}
			}
		}

		localResults = found.slice(0, 15);
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
				{
					// BC4: Use recent items if no query
					const list = !query && recentItems.length > 0 ? recentItems : flatResultsList;
					selectedIndex = Math.min(selectedIndex + 1, list.length - 1);
				}
				break;
			case 'ArrowUp':
				event.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Enter':
				event.preventDefault();
				{
					// BC4: Use recent items if no query
					const list = !query && recentItems.length > 0 ? recentItems : flatResultsList;
					if (list[selectedIndex]) {
						selectResult(list[selectedIndex]);
					}
				}
				break;
			case 'Escape':
				event.preventDefault();
				close();
				break;
		}
	}

	function selectResult(result: (typeof allResults)[0]) {
		// BC4: Add to recent items
		addToRecent(result);

		if (result.type === 'scene') {
			appState.selectScene(result.id, result.chapterId);
			appState.setViewMode('editor');
		} else if (result.type === 'chapter') {
			appState.selectScene('', result.id);
			appState.setViewMode('editor');
		} else if (result.type === 'bible' || result.type === 'bible_entry') {
			appState.setViewMode('bible');
			appState.selectedBibleEntryId = result.id;
		} else if (result.type === 'arc') {
			appState.navigateToArc(result.id);
		} else if (result.type === 'event') {
			appState.navigateToEvent(result.id);
		}
		close();
	}

	function close() {
		appState.isQuickOpenVisible = false;
		query = '';
		localResults = [];
		results = [];
		activeFilter = 'all';
		searchRemote.cancel();

		// Y6, AV7: Restore focus on close with requestAnimationFrame for reliability
		if (previouslyFocused instanceof HTMLElement) {
			requestAnimationFrame(() => {
				if (previouslyFocused instanceof HTMLElement) {
					previouslyFocused.focus();
				}
			});
		}
	}

	function handleBackdropClick(event: MouseEvent) {
		if (event.target === event.currentTarget) {
			close();
		}
	}

	// Focus input when dialog opens, save previous focus
	$effect(() => {
		if (appState.isQuickOpenVisible) {
			// Y6: Save current focus before opening
			previouslyFocused = document.activeElement;
		}
	});

	$effect(() => {
		if (inputElement) {
			tick().then(() => {
				inputElement?.focus();
			});
		}
	});
</script>

{#snippet resultIcon(type: string)}
	<div class="result-icon">
		{#if type === 'chapter'}
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
		{:else if type === 'scene'}
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
		{:else if type === 'arc'}
			<svg
				width="16"
				height="16"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path d="M22 12h-4l-3 9L9 3l-3 9H2" />
			</svg>
		{:else if type === 'event'}
			<svg
				width="16"
				height="16"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<circle cx="12" cy="12" r="10" />
				<polyline points="12 6 12 12 16 14" />
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
{/snippet}

<div
	class="quick-open-overlay"
	role="dialog"
	aria-modal="true"
	tabindex="-1"
	onclick={handleBackdropClick}
	onkeydown={handleKeydown}
>
	<div class="quick-open-container" class:has-preview={previewData}>
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
					placeholder="Search... (try status:draft or pov:Alice)"
					bind:value={query}
					oninput={handleInput}
					onkeydown={handleKeydown}
				/>
				{#if isSearching}
					<div class="loading-indicator"></div>
				{/if}
				<!-- CE5: Save search button -->
				{#if query.trim()}
					<button class="save-search-btn" onclick={saveCurrentSearch} title="Save this search">
						<svg
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<polygon
								points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
							/>
						</svg>
					</button>
				{/if}
			</div>

			<div class="filter-chips">
				{#each filterOptions as opt (opt.value)}
					<button
						class="filter-chip"
						class:active={activeFilter === opt.value}
						onclick={() => {
							activeFilter = opt.value;
							selectedIndex = 0;
						}}
					>
						{opt.label}
					</button>
				{/each}
			</div>

			<!-- CE5: Saved searches -->
			{#if !query && savedSearches.length > 0}
				<div class="saved-searches">
					<div class="saved-searches-header">Saved Searches</div>
					<div class="saved-searches-list">
						{#each savedSearches as search (search)}
							<div class="saved-search-item">
								<button class="saved-search-text-btn" onclick={() => applySavedSearch(search)}>
									{search}
								</button>
								<button
									class="remove-saved-btn"
									onclick={() => removeSavedSearch(search)}
									title="Remove"
								>
									<svg
										width="12"
										height="12"
										viewBox="0 0 24 24"
										fill="none"
										stroke="currentColor"
										stroke-width="2"
									>
										<line x1="18" y1="6" x2="6" y2="18" />
										<line x1="6" y1="6" x2="18" y2="18" />
									</svg>
								</button>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<!-- BC4: Show recent items when no query -->
			{#if !query && recentItems.length > 0}
				<div class="results-list">
					<div class="result-group-header">Recent</div>
					{#each recentItems as result, i (result.id)}
						<button
							class="result-item"
							class:selected={i === selectedIndex}
							onclick={() => selectResult(result)}
							onmouseenter={() => (selectedIndex = i)}
						>
							{@render resultIcon(result.type)}
							<div class="result-content">
								<span class="result-title">{result.title}</span>
								{#if result.subtitle}
									<span class="result-subtitle">{result.subtitle}</span>
								{/if}
							</div>
						</button>
					{/each}
				</div>
			{:else if flatResultsList.length > 0}
				<div class="results-list">
					<!-- AQ1: Show grouped results with headers when filter is 'all' -->
					{#if groupedResults}
						{@const flatIndex = { value: 0 }}
						{#each groupedResults as group (group.type)}
							<div class="result-group-header">{group.label}</div>
							{#each group.items as result (result.id)}
								{@const currentIndex = flatIndex.value++}
								<button
									class="result-item"
									class:selected={currentIndex === selectedIndex}
									onclick={() => selectResult(result)}
									onmouseenter={() => (selectedIndex = currentIndex)}
								>
									{@render resultIcon(result.type)}
									<div class="result-content">
										<span class="result-title">{result.title}</span>
										{#if result.subtitle}
											<span class="result-subtitle">{result.subtitle}</span>
										{/if}
									</div>
								</button>
							{/each}
						{/each}
					{:else}
						{#each allResults as result, i (result.id)}
							<button
								class="result-item"
								class:selected={i === selectedIndex}
								onclick={() => selectResult(result)}
								onmouseenter={() => (selectedIndex = i)}
							>
								{@render resultIcon(result.type)}
								<div class="result-content">
									<span class="result-title">{result.title}</span>
									{#if result.subtitle}
										<span class="result-subtitle">{result.subtitle}</span>
									{/if}
								</div>
							</button>
						{/each}
					{/if}
				</div>
			{:else if query.length > 0 && !isSearching}
				<div class="no-results">
					<p>No results found</p>
				</div>
			{/if}

			<!-- X4: Screen reader announcement for results -->
			<div class="sr-only" aria-live="polite">
				{#if isSearching}
					Searching...
				{:else if query.length > 0}
					{allResults.length} result{allResults.length !== 1 ? 's' : ''} found
				{/if}
			</div>

			<div class="quick-open-footer">
				<span><kbd>↑↓</kbd> to navigate</span>
				<span><kbd>↵</kbd> to select</span>
				<span><kbd>esc</kbd> to close</span>
			</div>
		</div>

		<!-- UC1: Preview Panel -->
		{#if previewData}
			<div class="preview-panel">
				{#if previewData.type === 'scene'}
					<div class="preview-header">
						<span class="preview-type">Scene</span>
						<span class="preview-chapter">{previewData.chapter}</span>
					</div>
					<h4 class="preview-title">{previewData.title}</h4>
					<div class="preview-meta">
						<span>{previewData.wordCount.toLocaleString()} words</span>
						{#if previewData.status}
							<span class="preview-status">{previewData.status}</span>
						{/if}
						{#if previewData.pov}
							<span>POV: {previewData.pov}</span>
						{/if}
					</div>
					{#if previewData.summary}
						<p class="preview-summary">{previewData.summary}</p>
					{/if}
					{#if previewData.text}
						<p class="preview-text">
							{previewData.text.slice(0, 300)}{previewData.text.length > 300 ? '...' : ''}
						</p>
					{/if}
				{:else if previewData.type === 'bible'}
					<div class="preview-header">
						<span class="preview-type">{previewData.entryType}</span>
					</div>
					<h4 class="preview-title">{previewData.title}</h4>
					{#if previewData.aliases}
						<p class="preview-aliases">Also: {previewData.aliases}</p>
					{/if}
					{#if previewData.description}
						<p class="preview-text">
							{previewData.description.slice(0, 400)}{previewData.description.length > 400
								? '...'
								: ''}
						</p>
					{/if}
				{:else if previewData.type === 'chapter'}
					<div class="preview-header">
						<span class="preview-type">Chapter</span>
					</div>
					<h4 class="preview-title">{previewData.title}</h4>
					<div class="preview-meta">
						<span>{previewData.sceneCount} scene{previewData.sceneCount !== 1 ? 's' : ''}</span>
						<span>{previewData.wordCount.toLocaleString()} words</span>
						{#if previewData.status}
							<span class="preview-status">{previewData.status}</span>
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	</div>
</div>

<style>
	.quick-open-overlay {
		position: fixed;
		inset: 0;
		background-color: var(--overlay-backdrop);
		display: flex;
		justify-content: center;
		padding-top: 100px;
		z-index: 1000;
	}

	/* UC1: Container for QuickOpen + Preview */
	.quick-open-container {
		display: flex;
		gap: var(--spacing-md);
		max-width: 900px;
		width: 100%;
	}

	.quick-open-container.has-preview {
		max-width: 900px;
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

	/* UC1: Preview Panel */
	.preview-panel {
		width: 280px;
		max-height: 400px;
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-md);
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.preview-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		font-size: var(--font-size-xs);
	}

	.preview-type {
		text-transform: uppercase;
		font-weight: 600;
		color: var(--color-accent);
		letter-spacing: 0.5px;
	}

	.preview-chapter {
		color: var(--color-text-muted);
	}

	.preview-title {
		margin: 0;
		font-size: var(--font-size-md);
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.preview-meta {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.preview-status {
		text-transform: capitalize;
	}

	.preview-summary {
		margin: 0;
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		font-style: italic;
		padding: var(--spacing-xs) 0;
		border-top: 1px solid var(--color-border-light, var(--color-border));
	}

	.preview-text {
		margin: 0;
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		line-height: var(--line-height-relaxed);
	}

	.preview-aliases {
		margin: 0;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		font-style: italic;
	}

	/* Hide preview on narrow screens */
	@media (max-width: 800px) {
		.preview-panel {
			display: none;
		}
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

	/* CE5: Save search button */
	.save-search-btn {
		display: flex;
		align-items: center;
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
	}

	.save-search-btn:hover {
		color: var(--color-accent);
		background-color: var(--color-bg-hover);
	}

	/* CE5: Saved searches section */
	.saved-searches {
		border-bottom: 1px solid var(--color-border);
		padding: var(--spacing-xs) var(--spacing-md);
	}

	.saved-searches-header {
		font-size: var(--font-size-xs);
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: var(--spacing-xs);
	}

	.saved-searches-list {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
	}

	.saved-search-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-xs);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
	}

	.saved-search-item:hover {
		border-color: var(--color-accent);
	}

	.saved-search-text-btn {
		padding: 2px var(--spacing-sm);
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--color-text-secondary);
		cursor: pointer;
	}

	.saved-search-text-btn:hover {
		color: var(--color-accent);
	}

	.remove-saved-btn {
		display: flex;
		align-items: center;
		padding: 2px;
		color: var(--color-text-muted);
		border-radius: 2px;
	}

	.remove-saved-btn:hover {
		color: var(--color-error);
		background-color: var(--color-error-light, rgba(239, 68, 68, 0.1));
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.filter-chips {
		display: flex;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-md);
		border-bottom: 1px solid var(--color-border);
	}

	.filter-chip {
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-sm);
		border-radius: 999px;
		color: var(--color-text-muted);
		background: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		cursor: pointer;
		transition: all var(--duration-fast);
	}

	.filter-chip:hover {
		color: var(--color-text-primary);
		border-color: var(--color-text-muted);
	}

	.filter-chip.active {
		color: var(--color-accent);
		background: var(--color-accent-light);
		border-color: var(--color-accent);
	}

	.results-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-xs) 0;
	}

	/* AQ1: Group headers for organized results */
	.result-group-header {
		font-size: var(--font-size-xs);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
		padding: var(--spacing-sm) var(--spacing-md) var(--spacing-xs);
		margin-top: var(--spacing-xs);
	}

	.result-group-header:first-child {
		margin-top: 0;
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
