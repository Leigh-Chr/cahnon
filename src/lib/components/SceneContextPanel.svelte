<!--
  "Previously On..." panel — shows immediate narrative context when
  selecting a scene: previous scenes, present characters, nearby issues.
-->
<script lang="ts">
	import type { SceneContext } from '$lib/api';
	import { sceneContextApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import { bibleEntryTypes, formatWordCount } from '$lib/utils';

	let context = $state<SceneContext | null>(null);
	let isLoading = $state(false);

	let selectedSceneId = $derived(appState.selectedSceneId);

	$effect(() => {
		if (selectedSceneId) {
			loadContext(selectedSceneId);
		} else {
			context = null;
		}
	});

	async function loadContext(sceneId: string) {
		isLoading = true;
		try {
			context = await sceneContextApi.get(sceneId);
		} catch (e) {
			console.error('Failed to load scene context:', e);
			context = null;
		} finally {
			isLoading = false;
		}
	}

	function getTypeIcon(type: string): string {
		return bibleEntryTypes.find((t) => t.value === type)?.icon || '?';
	}
</script>

<div class="scene-context">
	{#if isLoading}
		<p class="loading">Loading context...</p>
	{:else if context}
		<!-- Previous Scenes -->
		{#if context.previous_scenes.length > 0}
			<div class="context-section">
				<h4>Previously</h4>
				<div class="previous-scenes">
					{#each context.previous_scenes as scene (scene.scene_id)}
						<button
							class="prev-scene"
							onclick={() => {
								appState.selectedSceneId = scene.scene_id;
							}}
						>
							<span class="prev-title">{scene.title}</span>
							<span class="prev-meta">
								{#if scene.pov}<span class="prev-pov">{scene.pov}</span>{/if}
								<span class="prev-words">{formatWordCount(scene.word_count)}</span>
							</span>
							{#if scene.summary}
								<span class="prev-summary">{scene.summary}</span>
							{/if}
						</button>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Present Characters -->
		{#if context.present_characters.length > 0}
			<div class="context-section">
				<h4>Present</h4>
				<div class="present-chars">
					{#each context.present_characters as char (char.bible_entry_id)}
						<div class="char-chip">
							<span class="char-icon">{getTypeIcon(char.entry_type)}</span>
							<span class="char-name">{char.name}</span>
							{#if char.short_description}
								<span class="char-desc">{char.short_description}</span>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Nearby Issues -->
		{#if context.nearby_issues.length > 0}
			<div class="context-section">
				<h4>Nearby Issues</h4>
				<div class="issues">
					{#each context.nearby_issues as issue (issue.issue_id)}
						<div class="issue-chip" data-severity={issue.severity}>
							<span class="issue-dot"></span>
							<span class="issue-text">{issue.title}</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- TODOs -->
		{#if context.todos.length > 0}
			<div class="context-section">
				<h4>TODOs</h4>
				<ul class="todo-list">
					{#each context.todos as todo, i (i)}
						<li>{todo}</li>
					{/each}
				</ul>
			</div>
		{/if}

		<!-- Last Session -->
		{#if context.last_session}
			<div class="context-section">
				<h4>Last Session</h4>
				<p class="session-info">
					{context.last_session.date}:
					{context.last_session.words_written > 0 ? '+' : ''}{context.last_session.words_written} words
					({context.last_session.duration_minutes}min)
				</p>
			</div>
		{/if}
	{/if}
</div>

<style>
	.scene-context {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.loading {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
	}

	.context-section {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	h4 {
		font-size: var(--font-size-xs);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
		margin: 0;
	}

	/* Previous scenes */
	.previous-scenes {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.prev-scene {
		display: flex;
		flex-direction: column;
		gap: 1px;
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		text-align: left;
		transition: background-color var(--transition-fast);
	}

	.prev-scene:hover {
		background-color: var(--color-bg-hover);
	}

	.prev-title {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.prev-meta {
		display: flex;
		gap: var(--spacing-sm);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.prev-pov {
		color: var(--color-accent);
	}

	.prev-summary {
		font-size: var(--font-size-xs);
		color: var(--color-text-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Present characters */
	.present-chars {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-xs);
	}

	.char-chip {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
	}

	.char-icon {
		font-size: var(--font-size-xs);
	}

	.char-name {
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.char-desc {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	/* Issues */
	.issues {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.issue-chip {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: 2px var(--spacing-sm);
		font-size: var(--font-size-sm);
	}

	.issue-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background-color: var(--color-warning);
		flex-shrink: 0;
	}

	.issue-chip[data-severity='error'] .issue-dot {
		background-color: var(--color-error);
	}

	.issue-chip[data-severity='info'] .issue-dot {
		background-color: var(--color-info);
	}

	.issue-text {
		color: var(--color-text-primary);
	}

	/* TODOs */
	.todo-list {
		margin: 0;
		padding-left: var(--spacing-md);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.todo-list li {
		margin-bottom: 1px;
	}

	/* Session */
	.session-info {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		margin: 0;
	}
</style>
