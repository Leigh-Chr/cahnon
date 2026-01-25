<script lang="ts">
	import { appState } from '$lib/stores';
	import { sceneStatuses, statusColors, countWords, formatWordCount } from '$lib/utils';
	import type { Scene } from '$lib/api';

	interface Props {
		isOpen?: boolean;
	}

	let { isOpen = $bindable(false) }: Props = $props();

	type SortKey = 'order' | 'status' | 'words' | 'title' | 'conflict' | 'change' | 'tension';
	type SortOrder = 'asc' | 'desc';

	let sortKey = $state<SortKey>('order');
	let sortOrder = $state<SortOrder>('asc');
	let filterStatus = $state('all');
	let searchQuery = $state('');

	const tensionLevels = [
		{ value: 'low', label: 'Low' },
		{ value: 'medium', label: 'Medium' },
		{ value: 'high', label: 'High' },
	];

	let allScenes = $derived(getAllScenes());

	function getAllScenes(): Array<
		Scene & { chapterTitle: string; chapterIndex: number; sceneIndex: number }
	> {
		const result: Array<
			Scene & { chapterTitle: string; chapterIndex: number; sceneIndex: number }
		> = [];
		appState.chapters.forEach((chapter, chapterIndex) => {
			const chapterScenes = appState.scenes.get(chapter.id) || [];
			chapterScenes.forEach((scene, sceneIndex) => {
				result.push({
					...scene,
					chapterTitle: chapter.title,
					chapterIndex,
					sceneIndex,
				});
			});
		});
		return result;
	}

	let filteredScenes = $derived(
		allScenes
			.filter((scene) => {
				if (filterStatus !== 'all' && scene.status !== filterStatus) return false;
				if (searchQuery) {
					const query = searchQuery.toLowerCase();
					return (
						scene.title.toLowerCase().includes(query) ||
						scene.chapterTitle.toLowerCase().includes(query) ||
						(scene.summary && scene.summary.toLowerCase().includes(query))
					);
				}
				return true;
			})
			.sort((a, b) => {
				let cmp = 0;
				switch (sortKey) {
					case 'order':
						cmp = a.chapterIndex - b.chapterIndex || a.sceneIndex - b.sceneIndex;
						break;
					case 'status':
						cmp = a.status.localeCompare(b.status);
						break;
					case 'words':
						cmp = countWords(a.text) - countWords(b.text);
						break;
					case 'title':
						cmp = a.title.localeCompare(b.title);
						break;
					case 'conflict':
						cmp = (a.has_conflict ? 1 : 0) - (b.has_conflict ? 1 : 0);
						break;
					case 'change':
						cmp = (a.has_change ? 1 : 0) - (b.has_change ? 1 : 0);
						break;
					case 'tension':
						const tensionOrder = { low: 1, medium: 2, high: 3 };
						cmp =
							(tensionOrder[a.tension as keyof typeof tensionOrder] || 0) -
							(tensionOrder[b.tension as keyof typeof tensionOrder] || 0);
						break;
				}
				return sortOrder === 'asc' ? cmp : -cmp;
			})
	);

	function toggleSort(key: SortKey) {
		if (sortKey === key) {
			sortOrder = sortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			sortKey = key;
			sortOrder = 'asc';
		}
	}

	function selectScene(sceneId: string) {
		// Find the chapter containing this scene
		for (const [chapterId, chapterScenes] of appState.scenes.entries()) {
			if (chapterScenes.some((s) => s.id === sceneId)) {
				appState.selectScene(sceneId, chapterId);
				isOpen = false;
				break;
			}
		}
	}

	async function updateSceneStatus(scene: Scene, status: string) {
		await appState.updateScene(scene.id, { status });
	}

	async function updateSceneConflict(scene: Scene, hasConflict: boolean | null) {
		await appState.updateScene(scene.id, { has_conflict: hasConflict });
	}

	async function updateSceneChange(scene: Scene, hasChange: boolean | null) {
		await appState.updateScene(scene.id, { has_change: hasChange });
	}

	async function updateSceneTension(scene: Scene, tension: string | null) {
		await appState.updateScene(scene.id, { tension });
	}

	async function updateScenePovGoal(scene: Scene, povGoal: string) {
		await appState.updateScene(scene.id, { pov_goal: povGoal || null });
	}

	async function updateSceneRevisionNotes(scene: Scene, notes: string) {
		await appState.updateScene(scene.id, { revision_notes: notes || null });
	}

	async function updateSceneSetupFor(scene: Scene, targetSceneId: string | null) {
		await appState.updateScene(scene.id, { setup_for_scene_id: targetSceneId });
	}

	async function updateScenePayoffOf(scene: Scene, targetSceneId: string | null) {
		await appState.updateScene(scene.id, { payoff_of_scene_id: targetSceneId });
	}

	function _getSceneTitleById(sceneId: string | null): string {
		if (!sceneId) return '—';
		for (const [_, chapterScenes] of appState.scenes.entries()) {
			const found = chapterScenes.find((s) => s.id === sceneId);
			if (found) return found.title;
		}
		return '—';
	}

	function getTotalStats() {
		const total = allScenes.length;
		const words = allScenes.reduce((sum, s) => sum + countWords(s.text), 0);
		const byStatus = sceneStatuses.map((s) => ({
			...s,
			count: allScenes.filter((scene) => scene.status === s.value).length,
		}));
		return { total, words, byStatus };
	}

	let stats = $derived(getTotalStats());
</script>

{#if isOpen}
	<div
		class="review-grid-overlay"
		onclick={(e) => {
			if (e.target === e.currentTarget) isOpen = false;
		}}
		role="presentation"
	>
		<div class="review-grid-container">
			<div class="review-grid-header">
				<h2>Review Grid</h2>
				<button class="close-btn" onclick={() => (isOpen = false)} aria-label="Close">
					<svg
						width="20"
						height="20"
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

			<div class="stats-bar">
				<div class="stat-item">
					<span class="stat-value">{stats.total}</span>
					<span class="stat-label">Scenes</span>
				</div>
				<div class="stat-item">
					<span class="stat-value">{formatWordCount(stats.words)}</span>
					<span class="stat-label">Words</span>
				</div>
				<div class="status-breakdown">
					{#each stats.byStatus as status (status.value)}
						<div class="status-stat" style="--status-color: {statusColors[status.value]}">
							<span class="status-count">{status.count}</span>
							<span class="status-name">{status.label}</span>
						</div>
					{/each}
				</div>
			</div>

			<div class="filters-bar">
				<input
					type="text"
					placeholder="Search scenes..."
					bind:value={searchQuery}
					class="search-input"
				/>
				<select bind:value={filterStatus} class="filter-select">
					<option value="all">All statuses</option>
					{#each sceneStatuses as status (status.value)}
						<option value={status.value}>{status.label}</option>
					{/each}
				</select>
			</div>

			<div class="grid-wrapper">
				<table class="review-table">
					<thead>
						<tr>
							<th
								class="sortable"
								class:active={sortKey === 'order'}
								onclick={() => toggleSort('order')}
							>
								#
								{#if sortKey === 'order'}
									<span class="sort-arrow">{sortOrder === 'asc' ? '↑' : '↓'}</span>
								{/if}
							</th>
							<th>Chapter</th>
							<th
								class="sortable"
								class:active={sortKey === 'title'}
								onclick={() => toggleSort('title')}
							>
								Scene
								{#if sortKey === 'title'}
									<span class="sort-arrow">{sortOrder === 'asc' ? '↑' : '↓'}</span>
								{/if}
							</th>
							<th
								class="sortable"
								class:active={sortKey === 'status'}
								onclick={() => toggleSort('status')}
							>
								Status
								{#if sortKey === 'status'}
									<span class="sort-arrow">{sortOrder === 'asc' ? '↑' : '↓'}</span>
								{/if}
							</th>
							<th
								class="sortable"
								class:active={sortKey === 'words'}
								onclick={() => toggleSort('words')}
							>
								Words
								{#if sortKey === 'words'}
									<span class="sort-arrow">{sortOrder === 'asc' ? '↑' : '↓'}</span>
								{/if}
							</th>
							<th>POV Goal</th>
							<th
								class="sortable"
								class:active={sortKey === 'conflict'}
								onclick={() => toggleSort('conflict')}
							>
								Conflict?
								{#if sortKey === 'conflict'}
									<span class="sort-arrow">{sortOrder === 'asc' ? '↑' : '↓'}</span>
								{/if}
							</th>
							<th
								class="sortable"
								class:active={sortKey === 'change'}
								onclick={() => toggleSort('change')}
							>
								Change?
								{#if sortKey === 'change'}
									<span class="sort-arrow">{sortOrder === 'asc' ? '↑' : '↓'}</span>
								{/if}
							</th>
							<th
								class="sortable"
								class:active={sortKey === 'tension'}
								onclick={() => toggleSort('tension')}
							>
								Tension
								{#if sortKey === 'tension'}
									<span class="sort-arrow">{sortOrder === 'asc' ? '↑' : '↓'}</span>
								{/if}
							</th>
							<th>Setup For</th>
							<th>Payoff Of</th>
							<th>Notes</th>
						</tr>
					</thead>
					<tbody>
						{#each filteredScenes as scene, index (scene.id)}
							<tr
								class:selected={appState.selectedSceneId === scene.id}
								onclick={() => selectScene(scene.id)}
							>
								<td class="index-cell">{index + 1}</td>
								<td class="chapter-cell">{scene.chapterTitle}</td>
								<td class="title-cell">{scene.title}</td>
								<td class="status-cell">
									<select
										value={scene.status}
										onclick={(e) => e.stopPropagation()}
										onchange={(e) => updateSceneStatus(scene, e.currentTarget.value)}
										class="status-select"
										style="color: {statusColors[scene.status]}"
									>
										{#each sceneStatuses as status (status.value)}
											<option value={status.value}>{status.label}</option>
										{/each}
									</select>
								</td>
								<td class="words-cell">{formatWordCount(countWords(scene.text))}</td>
								<td class="pov-goal-cell">
									<input
										type="text"
										value={scene.pov_goal || ''}
										placeholder="POV goal..."
										onclick={(e) => e.stopPropagation()}
										onchange={(e) => updateScenePovGoal(scene, e.currentTarget.value)}
										class="inline-input"
									/>
								</td>
								<td class="checkbox-cell">
									<input
										type="checkbox"
										checked={scene.has_conflict === true}
										indeterminate={scene.has_conflict === null}
										onclick={(e) => e.stopPropagation()}
										onchange={(e) => updateSceneConflict(scene, e.currentTarget.checked)}
										class="checkbox-input"
									/>
								</td>
								<td class="checkbox-cell">
									<input
										type="checkbox"
										checked={scene.has_change === true}
										indeterminate={scene.has_change === null}
										onclick={(e) => e.stopPropagation()}
										onchange={(e) => updateSceneChange(scene, e.currentTarget.checked)}
										class="checkbox-input"
									/>
								</td>
								<td class="tension-cell">
									<select
										value={scene.tension || ''}
										onclick={(e) => e.stopPropagation()}
										onchange={(e) => updateSceneTension(scene, e.currentTarget.value || null)}
										class="tension-select"
									>
										<option value="">—</option>
										{#each tensionLevels as level (level.value)}
											<option value={level.value}>{level.label}</option>
										{/each}
									</select>
								</td>
								<td class="link-cell">
									<select
										value={scene.setup_for_scene_id || ''}
										onclick={(e) => e.stopPropagation()}
										onchange={(e) => updateSceneSetupFor(scene, e.currentTarget.value || null)}
										class="link-select"
									>
										<option value="">—</option>
										{#each allScenes.filter((s) => s.id !== scene.id) as otherScene (otherScene.id)}
											<option value={otherScene.id}>{otherScene.title}</option>
										{/each}
									</select>
								</td>
								<td class="link-cell">
									<select
										value={scene.payoff_of_scene_id || ''}
										onclick={(e) => e.stopPropagation()}
										onchange={(e) => updateScenePayoffOf(scene, e.currentTarget.value || null)}
										class="link-select"
									>
										<option value="">—</option>
										{#each allScenes.filter((s) => s.id !== scene.id) as otherScene (otherScene.id)}
											<option value={otherScene.id}>{otherScene.title}</option>
										{/each}
									</select>
								</td>
								<td class="notes-cell">
									<input
										type="text"
										value={scene.revision_notes || ''}
										placeholder="Notes..."
										onclick={(e) => e.stopPropagation()}
										onchange={(e) => updateSceneRevisionNotes(scene, e.currentTarget.value)}
										class="inline-input"
									/>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<div class="grid-footer">
				<span>{filteredScenes.length} of {allScenes.length} scenes</span>
			</div>
		</div>
	</div>
{/if}

<style>
	.review-grid-overlay {
		position: fixed;
		inset: 0;
		background-color: var(--overlay-backdrop);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
		padding: var(--spacing-xl);
	}

	.review-grid-container {
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		width: 100%;
		max-width: 1200px;
		max-height: 90vh;
		display: flex;
		flex-direction: column;
	}

	.review-grid-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border-light);
	}

	.review-grid-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		transition: all var(--transition-fast);
	}

	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.stats-bar {
		display: flex;
		align-items: center;
		gap: var(--spacing-lg);
		padding: var(--spacing-md) var(--spacing-lg);
		background-color: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border-light);
	}

	.stat-item {
		display: flex;
		flex-direction: column;
	}

	.stat-value {
		font-size: var(--font-size-xl);
		font-weight: 600;
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.status-breakdown {
		display: flex;
		gap: var(--spacing-md);
		margin-left: auto;
	}

	.status-stat {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		border-left: 3px solid var(--status-color);
	}

	.status-count {
		font-weight: 600;
		font-size: var(--font-size-sm);
	}

	.status-name {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.filters-bar {
		display: flex;
		gap: var(--spacing-md);
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border-light);
	}

	.search-input {
		flex: 1;
		padding: var(--spacing-sm) var(--spacing-md);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.filter-select {
		padding: var(--spacing-sm) var(--spacing-md);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		background-color: var(--color-bg-primary);
	}

	.grid-wrapper {
		flex: 1;
		overflow: auto;
		min-height: 300px;
	}

	.review-table {
		width: 100%;
		border-collapse: collapse;
	}

	.review-table th,
	.review-table td {
		padding: var(--spacing-sm) var(--spacing-md);
		text-align: left;
		border-bottom: 1px solid var(--color-border-light);
	}

	.review-table th {
		position: sticky;
		top: 0;
		background-color: var(--color-bg-secondary);
		font-size: var(--font-size-xs);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
	}

	.review-table th.sortable {
		cursor: pointer;
		user-select: none;
	}

	.review-table th.sortable:hover {
		color: var(--color-text-primary);
	}

	.review-table th.active {
		color: var(--color-accent);
	}

	.sort-arrow {
		margin-left: var(--spacing-xs);
	}

	.review-table tbody tr {
		cursor: pointer;
		transition: background-color var(--transition-fast);
	}

	.review-table tbody tr:hover {
		background-color: var(--color-bg-hover);
	}

	.review-table tbody tr.selected {
		background-color: var(--color-accent-light);
	}

	.index-cell {
		color: var(--color-text-muted);
		font-size: var(--font-size-xs);
		width: 40px;
	}

	.chapter-cell {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		max-width: 150px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.title-cell {
		font-size: var(--font-size-sm);
		font-weight: 500;
		max-width: 200px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.status-cell {
		width: 120px;
	}

	.status-select {
		font-size: var(--font-size-xs);
		padding: var(--spacing-xs) var(--spacing-sm);
		background: transparent;
		border: 1px solid var(--color-border-light);
		border-radius: var(--border-radius-sm);
		font-weight: 500;
		cursor: pointer;
	}

	.words-cell {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		width: 80px;
		text-align: right;
	}

	.pov-goal-cell,
	.notes-cell {
		min-width: 120px;
	}

	.inline-input {
		width: 100%;
		padding: var(--spacing-xs);
		font-size: var(--font-size-xs);
		background: transparent;
		border: 1px solid transparent;
		border-radius: var(--border-radius-sm);
	}

	.inline-input:hover,
	.inline-input:focus {
		border-color: var(--color-border);
		background-color: var(--color-bg-primary);
	}

	.checkbox-cell {
		width: 60px;
		text-align: center;
	}

	.checkbox-input {
		width: 16px;
		height: 16px;
		cursor: pointer;
	}

	.tension-cell {
		width: 90px;
	}

	.tension-select {
		font-size: var(--font-size-xs);
		padding: var(--spacing-xs);
		background: transparent;
		border: 1px solid var(--color-border-light);
		border-radius: var(--border-radius-sm);
		cursor: pointer;
		width: 100%;
	}

	.link-cell {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		max-width: 120px;
		min-width: 100px;
	}

	.link-select {
		font-size: var(--font-size-xs);
		padding: var(--spacing-xs);
		background: transparent;
		border: 1px solid transparent;
		border-radius: var(--border-radius-sm);
		cursor: pointer;
		width: 100%;
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.link-select:hover,
	.link-select:focus {
		border-color: var(--color-border);
		background-color: var(--color-bg-primary);
	}

	.grid-footer {
		padding: var(--spacing-sm) var(--spacing-lg);
		border-top: 1px solid var(--color-border-light);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-align: right;
	}
</style>
