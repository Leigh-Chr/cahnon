<!--
  Character Thread — shows all scenes where a character appears,
  in manuscript order, with gap indicators and navigation.
-->
<script lang="ts">
	import type { CharacterThread } from '$lib/api';
	import { worldStateApi } from '$lib/api';
	import { appState } from '$lib/stores';

	let thread = $state<CharacterThread | null>(null);
	let isLoading = $state(false);

	let bibleEntryId = $derived(appState.characterThreadId);

	$effect(() => {
		if (bibleEntryId) {
			loadThread(bibleEntryId);
		} else {
			thread = null;
		}
	});

	async function loadThread(id: string) {
		isLoading = true;
		try {
			thread = await worldStateApi.getCharacterThread(id);
		} catch (e) {
			console.error('Failed to load character thread:', e);
			thread = null;
		} finally {
			isLoading = false;
		}
	}

	function navigateToScene(sceneId: string, chapterId: string) {
		appState.selectScene(sceneId, chapterId);
		appState.setViewMode('editor');
	}

	function close() {
		appState.closeCharacterThread();
	}

	function tensionLabel(tension: string | null): string {
		switch (tension) {
			case 'high':
				return 'High';
			case 'medium':
				return 'Mid';
			case 'low':
				return 'Low';
			default:
				return '';
		}
	}
</script>

<div class="character-thread">
	{#if isLoading}
		<p class="loading">Loading character thread...</p>
	{:else if thread}
		<div class="thread-header">
			<button class="back-btn" onclick={close} title="Back to world state">
				<svg
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<polyline points="15 18 9 12 15 6" />
				</svg>
			</button>
			<h3 class="thread-title">{thread.character_name}</h3>
			<span class="thread-count">{thread.scenes.length} scenes</span>
		</div>

		<div class="thread-timeline">
			{#each thread.scenes as scene, i (scene.scene_id)}
				{#if scene.gap_from_previous > 1}
					<div class="gap-indicator" class:large-gap={scene.gap_from_previous >= 10}>
						<span class="gap-line"></span>
						<span class="gap-label">{scene.gap_from_previous} scenes gap</span>
						<span class="gap-line"></span>
					</div>
				{/if}
				<button
					class="thread-scene"
					class:current={appState.selectedSceneId === scene.scene_id}
					onclick={() => navigateToScene(scene.scene_id, scene.chapter_id)}
				>
					<div class="scene-main">
						<span class="scene-index">{i + 1}</span>
						<div class="scene-info">
							<span class="scene-name">{scene.scene_title}</span>
							<span class="scene-chapter">{scene.chapter_title}</span>
						</div>
					</div>
					<div class="scene-meta">
						{#if scene.pov}
							<span class="meta-tag" class:is-pov={scene.pov === thread.character_name}>
								{scene.pov === thread.character_name ? 'POV' : scene.pov}
							</span>
						{/if}
						{#if scene.tension}
							<span class="meta-tension">{tensionLabel(scene.tension)}</span>
						{/if}
					</div>
					{#if scene.summary}
						<p class="scene-summary">{scene.summary}</p>
					{/if}
					{#if scene.other_characters.length > 0}
						<div class="scene-others">
							with {scene.other_characters.slice(0, 3).join(', ')}{scene.other_characters.length > 3
								? ` +${scene.other_characters.length - 3}`
								: ''}
						</div>
					{/if}
				</button>
			{:else}
				<p class="empty">No scenes found for this character.</p>
			{/each}
		</div>
	{:else}
		<p class="empty">Select a character to view their thread.</p>
	{/if}
</div>

<style>
	.character-thread {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow-y: auto;
	}

	.loading {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
		padding: var(--spacing-sm);
	}

	.thread-header {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		padding-bottom: var(--spacing-sm);
		border-bottom: 1px solid var(--color-border-light);
		margin-bottom: var(--spacing-sm);
	}

	.back-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		flex-shrink: 0;
	}

	.back-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.thread-title {
		margin: 0;
		font-size: var(--font-size-base);
		font-weight: 600;
		color: var(--color-text-primary);
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.thread-count {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.thread-timeline {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.gap-indicator {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) 0;
	}

	.gap-indicator.large-gap .gap-label {
		color: var(--color-warning);
	}

	.gap-line {
		flex: 1;
		height: 1px;
		background-color: var(--color-border-light);
	}

	.gap-label {
		font-size: 10px;
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.thread-scene {
		display: flex;
		flex-direction: column;
		gap: 3px;
		padding: var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		border-left: 3px solid transparent;
		text-align: left;
		width: 100%;
		transition: background-color var(--transition-fast);
	}

	.thread-scene:hover {
		background-color: var(--color-bg-hover);
	}

	.thread-scene.current {
		background-color: var(--color-accent-light);
		border-left-color: var(--color-accent);
	}

	.scene-main {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.scene-index {
		font-size: 10px;
		font-weight: 600;
		color: var(--color-text-muted);
		width: 20px;
		text-align: center;
		flex-shrink: 0;
	}

	.scene-info {
		display: flex;
		flex-direction: column;
		min-width: 0;
		flex: 1;
	}

	.scene-name {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.scene-chapter {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.scene-meta {
		display: flex;
		gap: var(--spacing-xs);
		padding-left: 28px;
	}

	.meta-tag {
		font-size: 10px;
		padding: 1px 4px;
		border-radius: 3px;
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-secondary);
	}

	.meta-tag.is-pov {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
		font-weight: 500;
	}

	.meta-tension {
		font-size: 10px;
		color: var(--color-text-muted);
	}

	.scene-summary {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		padding-left: 28px;
		line-height: 1.4;
		overflow: hidden;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}

	.scene-others {
		font-size: 10px;
		color: var(--color-text-muted);
		padding-left: 28px;
		font-style: italic;
	}

	.empty {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
		padding: var(--spacing-sm);
	}
</style>
