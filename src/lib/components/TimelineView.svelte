<!--
  Timeline view for visualizing story chronology.

  Features:
  - Display scenes and events in chronological order
  - Toggle between chronological and narrative order
  - Link scenes to timeline events
  - Automatic conflict detection (character in two places at once)
  - Visual timeline with scene cards
  - Event creation and management
  - Click to navigate to scene
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import {
		eventApi,
		timelineApi,
		type Scene,
		type TimelineEvent,
		type TimelineConflict,
	} from '$lib/api';
	import { appState } from '$lib/stores';
	import { countWords, formatWordCount } from '$lib/utils';
	import { Icon, Button, EmptyState, LoadingState } from './ui';

	let timelineScenes = $state<Scene[]>([]);
	let _events = $state<TimelineEvent[]>([]);
	let conflicts = $state<TimelineConflict[]>([]);
	let viewMode = $state<'chronological' | 'narrative'>('chronological');
	let showConflicts = $state(false);
	let isLoading = $state(true);
	let isCheckingConflicts = $state(false);

	let allScenes = $derived(Array.from(appState.scenes.values()).flat());
	let scenesWithTime = $derived(
		allScenes.filter((s) => s.on_timeline && (s.time_point || s.time_start))
	);

	// Use onMount for one-time initialization instead of $effect
	onMount(() => {
		loadData();
	});

	async function loadData() {
		isLoading = true;
		try {
			timelineScenes = await eventApi.getTimelineScenes();
			_events = await eventApi.getAll();
		} catch (e) {
			console.error('Failed to load timeline data:', e);
		}
		isLoading = false;
	}

	async function checkConflicts() {
		isCheckingConflicts = true;
		try {
			conflicts = await timelineApi.detectConflicts();
			showConflicts = true;
		} catch (e) {
			console.error('Failed to detect conflicts:', e);
		}
		isCheckingConflicts = false;
	}

	function getConflictTypeIcon(type: string): string {
		const icons: Record<string, string> = {
			character_in_two_places:
				'M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8M23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75',
			overlapping_times:
				'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 6v6l4 2',
			impossible_sequence: 'M3 6h18M3 12h18M3 18h18',
		};
		return icons[type] || icons['overlapping_times'];
	}

	function getSceneTitle(sceneId: string): string {
		const scene =
			timelineScenes.find((s) => s.id === sceneId) || allScenes.find((s) => s.id === sceneId);
		return scene?.title || 'Unknown scene';
	}

	function selectScene(scene: Scene) {
		appState.selectedSceneId = scene.id;
		appState.setViewMode('editor');
	}

	function getChapterTitle(chapterId: string): string {
		const chapter = appState.chapters.find((c) => c.id === chapterId);
		return chapter?.title || 'Unknown';
	}

	function getTimeDisplay(scene: Scene): string {
		if (scene.time_point) return scene.time_point;
		if (scene.time_start && scene.time_end) return `${scene.time_start} - ${scene.time_end}`;
		if (scene.time_start) return `From ${scene.time_start}`;
		return 'No time set';
	}

	function getStatusColor(status: string): string {
		const colors: Record<string, string> = {
			planned: 'var(--status-planned)',
			'to write': 'var(--status-to-write)',
			draft: 'var(--status-draft)',
			'in revision': 'var(--status-in-revision)',
			done: 'var(--status-done)',
			'to cut': 'var(--status-to-cut)',
		};
		return colors[status] || 'var(--color-text-muted)';
	}
</script>

<div class="timeline-view">
	<div class="timeline-header">
		<h2>Timeline</h2>
		<div class="header-actions">
			<Button
				variant="secondary"
				class={conflicts.length > 0 ? 'has-conflicts' : ''}
				onclick={checkConflicts}
				disabled={isCheckingConflicts}
			>
				{#if isCheckingConflicts}
					Checking...
				{:else if conflicts.length > 0}
					<Icon name="alert" size={16} />
					{conflicts.length} Conflict{conflicts.length !== 1 ? 's' : ''}
				{:else}
					Check Conflicts
				{/if}
			</Button>
			<div class="view-toggle">
				<button
					class:active={viewMode === 'chronological'}
					onclick={() => (viewMode = 'chronological')}
				>
					Chronological
				</button>
				<button class:active={viewMode === 'narrative'} onclick={() => (viewMode = 'narrative')}>
					Narrative
				</button>
			</div>
		</div>
	</div>

	{#if showConflicts && conflicts.length > 0}
		<div class="conflicts-panel">
			<div class="conflicts-header">
				<h3>Timeline Conflicts</h3>
				<Button variant="icon" onclick={() => (showConflicts = false)} title="Close">
					<Icon name="close" size={16} />
				</Button>
			</div>
			<div class="conflicts-list">
				{#each conflicts as conflict, index (index)}
					<div class="conflict-item">
						<div class="conflict-icon">
							<svg
								width="20"
								height="20"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2"
							>
								<path d={getConflictTypeIcon(conflict.conflict_type)} />
							</svg>
						</div>
						<div class="conflict-info">
							<div class="conflict-type">{conflict.conflict_type.replace(/_/g, ' ')}</div>
							<div class="conflict-description">{conflict.description}</div>
							{#if conflict.character_name}
								<div class="conflict-character">Character: {conflict.character_name}</div>
							{/if}
							{#if conflict.time_point}
								<div class="conflict-time">Time: {conflict.time_point}</div>
							{/if}
							<div class="conflict-scenes">
								Scenes: {conflict.scene_ids.map((id) => getSceneTitle(id)).join(', ')}
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	{#if isLoading}
		<LoadingState message="Loading timeline..." />
	{:else if scenesWithTime.length === 0}
		<EmptyState
			icon="clock"
			title="No scenes on timeline"
			description="Add time information to your scenes to see them here."
		>
			<p class="hint">Set the time_point or time_start field in the scene editor.</p>
		</EmptyState>
	{:else}
		<div class="timeline-content">
			{#if viewMode === 'chronological'}
				<div class="timeline-track">
					{#each timelineScenes as scene (scene.id)}
						<button
							class="timeline-item"
							class:selected={appState.selectedSceneId === scene.id}
							onclick={() => selectScene(scene)}
							style="--status-color: {getStatusColor(scene.status)}"
						>
							<div class="item-time">{getTimeDisplay(scene)}</div>
							<div class="item-marker"></div>
							<div class="item-content">
								<div class="item-title">{scene.title}</div>
								<div class="item-meta">
									<span class="chapter">{getChapterTitle(scene.chapter_id)}</span>
									{#if scene.pov}
										<span class="pov">POV: {scene.pov}</span>
									{/if}
								</div>
								{#if scene.summary}
									<div class="item-summary">{scene.summary}</div>
								{/if}
								<div class="item-stats">
									<span class="word-count">{formatWordCount(countWords(scene.text))} words</span>
									<span class="status" style="color: {getStatusColor(scene.status)}"
										>{scene.status}</span
									>
								</div>
							</div>
						</button>
					{/each}
				</div>
			{:else}
				<div class="dual-track">
					<div class="track-column">
						<h4>Chronological Order</h4>
						{#each timelineScenes as scene (scene.id)}
							<button
								class="track-item"
								class:selected={appState.selectedSceneId === scene.id}
								onclick={() => selectScene(scene)}
							>
								<span class="time">{getTimeDisplay(scene)}</span>
								<span class="title">{scene.title}</span>
							</button>
						{/each}
					</div>
					<div class="track-column">
						<h4>Narrative Order</h4>
						{#each allScenes.filter((s) => s.on_timeline) as scene (scene.id)}
							<button
								class="track-item"
								class:selected={appState.selectedSceneId === scene.id}
								onclick={() => selectScene(scene)}
							>
								<span class="position">#{scene.position + 1}</span>
								<span class="title">{scene.title}</span>
							</button>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.timeline-view {
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-primary);
	}

	.timeline-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.timeline-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.view-toggle {
		display: flex;
		gap: 2px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-md);
		padding: 2px;
	}

	.view-toggle button {
		padding: var(--spacing-xs) var(--spacing-md);
		font-size: var(--font-size-sm);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		transition: all var(--transition-fast);
	}

	.view-toggle button.active {
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
	}

	.hint {
		font-size: var(--font-size-sm);
		margin-top: var(--spacing-md);
		color: var(--color-text-muted);
	}

	.timeline-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
	}

	.timeline-track {
		position: relative;
		padding-left: 120px;
	}

	.timeline-track::before {
		content: '';
		position: absolute;
		left: 115px;
		top: 0;
		bottom: 0;
		width: 2px;
		background-color: var(--color-border);
	}

	.timeline-item {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-md);
		margin-bottom: var(--spacing-lg);
		text-align: left;
		width: 100%;
		position: relative;
	}

	.timeline-item:hover .item-content {
		background-color: var(--color-bg-hover);
	}

	.timeline-item.selected .item-content {
		background-color: var(--color-accent-light);
		border-color: var(--color-accent);
	}

	.item-time {
		position: absolute;
		left: -120px;
		width: 100px;
		text-align: right;
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		padding-top: var(--spacing-sm);
	}

	.item-marker {
		position: absolute;
		left: -9px;
		width: 16px;
		height: 16px;
		border-radius: 50%;
		background-color: var(--status-color);
		border: 3px solid var(--color-bg-primary);
		z-index: 1;
	}

	.item-content {
		flex: 1;
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
		transition: all var(--transition-fast);
	}

	.item-title {
		font-weight: 500;
		color: var(--color-text-primary);
		margin-bottom: var(--spacing-xs);
	}

	.item-meta {
		display: flex;
		gap: var(--spacing-md);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-sm);
	}

	.item-summary {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-sm);
	}

	.item-stats {
		display: flex;
		gap: var(--spacing-md);
		font-size: var(--font-size-xs);
	}

	.word-count {
		color: var(--color-text-muted);
	}

	.status {
		font-weight: 500;
	}

	.dual-track {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--spacing-lg);
	}

	.track-column {
		background-color: var(--color-bg-secondary);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
	}

	.track-column h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-md);
		padding-bottom: var(--spacing-sm);
		border-bottom: 1px solid var(--color-border-light);
	}

	.track-item {
		display: flex;
		gap: var(--spacing-sm);
		width: 100%;
		padding: var(--spacing-sm);
		text-align: left;
		border-radius: var(--border-radius-sm);
		transition: background-color var(--transition-fast);
	}

	.track-item:hover {
		background-color: var(--color-bg-hover);
	}

	.track-item.selected {
		background-color: var(--color-accent-light);
	}

	.track-item .time,
	.track-item .position {
		flex-shrink: 0;
		width: 80px;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.track-item .title {
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.header-actions {
		display: flex;
		gap: var(--spacing-md);
		align-items: center;
	}

	.header-actions :global(.has-conflicts) {
		background-color: var(--color-bg-tertiary);
		border-color: var(--color-text-muted);
		color: var(--color-text-primary);
	}

	.conflicts-panel {
		background-color: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border);
		max-height: 300px;
		overflow-y: auto;
	}

	.conflicts-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-sm) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
		position: sticky;
		top: 0;
		background-color: var(--color-bg-secondary);
	}

	.conflicts-header h3 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.conflicts-list {
		padding: var(--spacing-md) var(--spacing-lg);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.conflict-item {
		display: flex;
		gap: var(--spacing-md);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-left: 3px solid var(--color-text-primary);
		border-radius: var(--border-radius-sm);
	}

	.conflict-icon {
		color: var(--color-text-primary);
		flex-shrink: 0;
	}

	.conflict-info {
		flex: 1;
	}

	.conflict-type {
		font-size: var(--font-size-sm);
		font-weight: 500;
		text-transform: capitalize;
		color: var(--color-text-primary);
	}

	.conflict-description {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		margin-top: var(--spacing-xs);
	}

	.conflict-character,
	.conflict-time,
	.conflict-scenes {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-top: var(--spacing-xs);
	}
</style>
