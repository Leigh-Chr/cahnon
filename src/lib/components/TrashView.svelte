<script lang="ts">
	import { onMount } from 'svelte';
	import { trashApi, type Scene, type Chapter } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showSuccess, showError } from '$lib/toast';

	let deletedScenes = $state<Scene[]>([]);
	let deletedChapters = $state<Chapter[]>([]);
	let isLoading = $state(true);
	let activeTab = $state<'scenes' | 'chapters'>('scenes');

	// Use onMount for one-time initialization
	onMount(() => {
		loadTrash();
	});

	async function loadTrash() {
		isLoading = true;
		try {
			[deletedScenes, deletedChapters] = await Promise.all([
				trashApi.getDeletedScenes(),
				trashApi.getDeletedChapters(),
			]);
		} catch (e) {
			console.error('Failed to load trash:', e);
		}
		isLoading = false;
	}

	async function restoreScene(scene: Scene) {
		try {
			await trashApi.restoreScene(scene.id);
			deletedScenes = deletedScenes.filter((s) => s.id !== scene.id);
			await appState.loadChapters();
			showSuccess(`Scene "${scene.title}" restored`);
		} catch (e) {
			console.error('Failed to restore scene:', e);
			showError('Failed to restore scene');
		}
	}

	async function restoreChapter(chapter: Chapter) {
		try {
			await trashApi.restoreChapter(chapter.id);
			deletedChapters = deletedChapters.filter((c) => c.id !== chapter.id);
			await appState.loadChapters();
			showSuccess(`Chapter "${chapter.title}" restored`);
		} catch (e) {
			console.error('Failed to restore chapter:', e);
			showError('Failed to restore chapter');
		}
	}

	function getChapterTitle(chapterId: string): string {
		const chapter = appState.chapters.find((c) => c.id === chapterId);
		return chapter?.title || 'Unknown Chapter';
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleString();
	}
</script>

<div class="trash-view">
	<div class="trash-header">
		<h2>Trash</h2>
		<div class="tab-toggle">
			<button class:active={activeTab === 'scenes'} onclick={() => (activeTab = 'scenes')}>
				Scenes ({deletedScenes.length})
			</button>
			<button class:active={activeTab === 'chapters'} onclick={() => (activeTab = 'chapters')}>
				Chapters ({deletedChapters.length})
			</button>
		</div>
	</div>

	{#if isLoading}
		<div class="loading">Loading trash...</div>
	{:else if activeTab === 'scenes'}
		{#if deletedScenes.length === 0}
			<div class="empty-state">
				<svg
					width="48"
					height="48"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
				>
					<polyline points="3 6 5 6 21 6" />
					<path
						d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
					/>
				</svg>
				<h3>Trash is empty</h3>
				<p>Deleted scenes will appear here.</p>
			</div>
		{:else}
			<div class="trash-list">
				{#each deletedScenes as scene (scene.id)}
					<div class="trash-item">
						<div class="item-icon">
							<svg
								width="20"
								height="20"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
								<polyline points="14 2 14 8 20 8" />
							</svg>
						</div>
						<div class="item-info">
							<h4>{scene.title}</h4>
							<div class="item-meta">
								<span class="chapter">{getChapterTitle(scene.chapter_id)}</span>
								<span class="date">Deleted {formatDate(scene.updated_at)}</span>
							</div>
							{#if scene.summary}
								<p class="item-summary">{scene.summary}</p>
							{/if}
						</div>
						<button class="restore-btn" onclick={() => restoreScene(scene)}>
							<svg
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<polyline points="1 4 1 10 7 10" />
								<path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
							</svg>
							Restore
						</button>
					</div>
				{/each}
			</div>
		{/if}
	{:else if deletedChapters.length === 0}
		<div class="empty-state">
			<svg
				width="48"
				height="48"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<polyline points="3 6 5 6 21 6" />
				<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
			</svg>
			<h3>No deleted chapters</h3>
			<p>Deleted chapters will appear here.</p>
		</div>
	{:else}
		<div class="trash-list">
			{#each deletedChapters as chapter (chapter.id)}
				<div class="trash-item">
					<div class="item-icon">
						<svg
							width="20"
							height="20"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
							<path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
						</svg>
					</div>
					<div class="item-info">
						<h4>{chapter.title}</h4>
						<div class="item-meta">
							<span class="date">Deleted {formatDate(chapter.updated_at)}</span>
						</div>
						{#if chapter.summary}
							<p class="item-summary">{chapter.summary}</p>
						{/if}
					</div>
					<button class="restore-btn" onclick={() => restoreChapter(chapter)}>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<polyline points="1 4 1 10 7 10" />
							<path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
						</svg>
						Restore
					</button>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.trash-view {
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-primary);
	}

	.trash-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.trash-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.tab-toggle {
		display: flex;
		gap: 2px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-md);
		padding: 2px;
	}

	.tab-toggle button {
		padding: var(--spacing-xs) var(--spacing-md);
		font-size: var(--font-size-sm);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		transition: all var(--transition-fast);
	}

	.tab-toggle button.active {
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
	}

	.loading,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-xl);
	}

	.empty-state svg {
		opacity: 0.5;
		margin-bottom: var(--spacing-md);
	}

	.empty-state h3 {
		font-size: var(--font-size-lg);
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-sm);
	}

	.trash-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.trash-item {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-md);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
	}

	.item-icon {
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.item-info {
		flex: 1;
		min-width: 0;
	}

	.item-info h4 {
		font-size: var(--font-size-base);
		font-weight: 500;
		margin-bottom: var(--spacing-xs);
	}

	.item-meta {
		display: flex;
		gap: var(--spacing-md);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.item-summary {
		margin-top: var(--spacing-xs);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.restore-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-md);
		color: var(--color-accent);
		border: 1px solid var(--color-accent);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		font-weight: 500;
		flex-shrink: 0;
	}

	.restore-btn:hover {
		background-color: var(--color-accent);
		color: white;
	}
</style>
