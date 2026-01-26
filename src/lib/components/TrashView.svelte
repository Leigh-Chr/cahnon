<script lang="ts">
	/**
	 * TrashView Component
	 *
	 * Displays deleted scenes and chapters with restore functionality.
	 * Supports tabbed view between scenes and chapters.
	 */

	import { onMount } from 'svelte';
	import { trashApi, type Scene, type Chapter } from '$lib/api';
	import { appState } from '$lib/stores';
	import { showSuccess, showError } from '$lib/toast';
	import { Icon, Button, EmptyState, LoadingState } from './ui';

	let deletedScenes = $state<Scene[]>([]);
	let deletedChapters = $state<Chapter[]>([]);
	let isLoading = $state(true);
	let activeTab = $state<'scenes' | 'chapters'>('scenes');

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
			showError('Failed to load trash');
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
		<LoadingState message="Loading trash..." />
	{:else if activeTab === 'scenes'}
		{#if deletedScenes.length === 0}
			<EmptyState
				icon="trash"
				title="Trash is empty"
				description="Deleted scenes will appear here."
			/>
		{:else}
			<div class="trash-list">
				{#each deletedScenes as scene (scene.id)}
					<div class="trash-item">
						<div class="item-icon">
							<Icon name="file" size={20} />
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
						<Button variant="secondary" onclick={() => restoreScene(scene)}>
							<Icon name="restore" size={16} />
							Restore
						</Button>
					</div>
				{/each}
			</div>
		{/if}
	{:else if deletedChapters.length === 0}
		<EmptyState
			icon="trash"
			title="No deleted chapters"
			description="Deleted chapters will appear here."
		/>
	{:else}
		<div class="trash-list">
			{#each deletedChapters as chapter (chapter.id)}
				<div class="trash-item">
					<div class="item-icon">
						<Icon name="book" size={20} />
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
					<Button variant="secondary" onclick={() => restoreChapter(chapter)}>
						<Icon name="restore" size={16} />
						Restore
					</Button>
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
</style>
