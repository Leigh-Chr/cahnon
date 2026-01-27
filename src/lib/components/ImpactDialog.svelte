<!--
  Impact Awareness dialog — shows what would be affected before deleting
  a scene, chapter, or bible entry.
-->
<script lang="ts">
	import type { ImpactPreview } from '$lib/api';
	import { impactApi } from '$lib/api';

	import { Button } from './ui';

	interface Props {
		entityType: 'scene' | 'chapter' | 'bible_entry';
		entityId: string;
		entityName: string;
		onconfirm: () => void;
		oncancel: () => void;
	}

	let { entityType, entityId, entityName, onconfirm, oncancel }: Props = $props();

	let impact = $state<ImpactPreview | null>(null);
	let isLoading = $state(true);

	$effect(() => {
		loadImpact();
	});

	async function loadImpact() {
		isLoading = true;
		try {
			if (entityType === 'scene') {
				impact = await impactApi.previewDeleteScene(entityId);
			} else if (entityType === 'chapter') {
				impact = await impactApi.previewDeleteChapter(entityId);
			} else {
				impact = await impactApi.previewDeleteBibleEntry(entityId);
			}
		} catch (e) {
			console.error('Failed to load impact preview:', e);
			impact = { items: [], total_count: 0 };
		} finally {
			isLoading = false;
		}
	}

	function getImpactIcon(type: string): string {
		switch (type) {
			case 'arc_loses_scene':
			case 'arcs_affected':
				return '🎭';
			case 'orphan_event':
			case 'orphan_events':
				return '📅';
			case 'broken_setup_payoff':
				return '🔗';
			case 'associations_lost':
			case 'scenes_lose_association':
				return '🏷️';
			case 'issues_unlinked':
				return '⚠️';
			case 'relationships_broken':
				return '🤝';
			case 'arc_character_lost':
				return '👤';
			case 'scenes_deleted':
				return '📄';
			default:
				return '•';
		}
	}
</script>

<div class="impact-overlay" onclick={oncancel} role="presentation">
	<div class="impact-dialog" onclick={(e) => e.stopPropagation()} role="presentation">
		<h3>Delete {entityType === 'bible_entry' ? 'entry' : entityType}?</h3>
		<p class="entity-name">"{entityName}"</p>

		{#if isLoading}
			<p class="loading">Checking impact...</p>
		{:else if impact && impact.items.length > 0}
			<div class="impact-warning">
				<p class="warning-text">This will affect:</p>
				<ul class="impact-list">
					{#each impact.items as item, i (i)}
						<li class="impact-item">
							<span class="impact-icon">{getImpactIcon(item.impact_type)}</span>
							<span class="impact-desc">{item.description}</span>
						</li>
					{/each}
				</ul>
			</div>
		{:else}
			<p class="no-impact">No other entities will be affected.</p>
		{/if}

		<div class="dialog-actions">
			<Button variant="ghost" onclick={oncancel}>Cancel</Button>
			<Button variant="danger" onclick={onconfirm}>Delete</Button>
		</div>
	</div>
</div>

<style>
	.impact-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.impact-dialog {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-lg);
		max-width: 480px;
		width: 90%;
		max-height: 80vh;
		overflow-y: auto;
	}

	h3 {
		margin: 0 0 var(--spacing-xs);
		font-size: var(--font-size-lg);
		color: var(--color-text-primary);
	}

	.entity-name {
		font-size: var(--font-size-md);
		color: var(--color-text-secondary);
		font-style: italic;
		margin: 0 0 var(--spacing-md);
	}

	.loading {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		font-style: italic;
	}

	.impact-warning {
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-warning);
		border-radius: var(--border-radius-sm);
		padding: var(--spacing-md);
		margin-bottom: var(--spacing-md);
	}

	.warning-text {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-warning);
		margin: 0 0 var(--spacing-sm);
	}

	.impact-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.impact-item {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.impact-icon {
		flex-shrink: 0;
		font-size: var(--font-size-md);
	}

	.impact-desc {
		flex: 1;
	}

	.no-impact {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
		margin: 0 0 var(--spacing-md);
	}

	.dialog-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
		margin-top: var(--spacing-md);
	}
</style>
