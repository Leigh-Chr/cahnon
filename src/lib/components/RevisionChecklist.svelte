<script lang="ts">
	/**
	 * RevisionChecklist Component
	 *
	 * Checklist for scene revision with predefined writing quality checks.
	 * Tracks completion with progress bar.
	 */

	import { Icon } from './ui';

	interface Props {
		checklist?: Record<string, boolean>;
		onchange?: (data: { checklist: Record<string, boolean> }) => void;
	}

	let { checklist = {}, onchange }: Props = $props();

	interface ChecklistItem {
		id: string;
		label: string;
		description: string;
	}

	const checklistItems: ChecklistItem[] = [
		{
			id: 'has_conflict',
			label: 'Scene has conflict',
			description: 'Something is at stake, someone wants something',
		},
		{
			id: 'has_change',
			label: 'Something changes',
			description: 'Character, situation, or understanding shifts',
		},
		{
			id: 'pov_consistent',
			label: 'POV consistent',
			description: 'Single perspective maintained throughout',
		},
		{
			id: 'enters_late',
			label: 'Enters late, exits early',
			description: 'Scene starts in action, ends before resolution',
		},
		{
			id: 'no_info_dumps',
			label: 'No info dumps',
			description: 'Information woven naturally into action',
		},
		{
			id: 'sensory_details',
			label: 'Sensory details present',
			description: 'Reader can see, hear, feel the scene',
		},
		{
			id: 'character_voice',
			label: 'Distinct character voice',
			description: 'POV character has unique perspective/language',
		},
		{
			id: 'advances_plot',
			label: 'Advances plot or character',
			description: 'Scene has clear purpose in the story',
		},
	];

	function toggle(itemId: string) {
		checklist = {
			...checklist,
			[itemId]: !checklist[itemId],
		};
		onchange?.({ checklist });
	}

	let completedCount = $derived(Object.values(checklist).filter(Boolean).length);
	let totalCount = $derived(checklistItems.length);
	let completionPercent = $derived(Math.round((completedCount / totalCount) * 100));
</script>

<div class="revision-checklist">
	<div class="checklist-header">
		<h4>Revision Checklist</h4>
		<div class="progress-indicator">
			<div class="progress-bar">
				<div class="progress-fill" style="width: {completionPercent}%"></div>
			</div>
			<span class="progress-text">{completedCount}/{totalCount}</span>
		</div>
	</div>

	<div class="checklist-items">
		{#each checklistItems as item (item.id)}
			<label class="checklist-item" class:checked={checklist[item.id]}>
				<input
					type="checkbox"
					checked={checklist[item.id] || false}
					onchange={() => toggle(item.id)}
				/>
				<div class="item-content">
					<span class="item-label">{item.label}</span>
					<span class="item-description">{item.description}</span>
				</div>
				<div class="checkmark">
					{#if checklist[item.id]}
						<Icon name="check" size={16} />
					{/if}
				</div>
			</label>
		{/each}
	</div>

	{#if completedCount === totalCount}
		<div class="completion-message">
			<Icon name="check" size={20} />
			<span>All checks complete!</span>
		</div>
	{/if}
</div>

<style>
	.revision-checklist {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		overflow: hidden;
	}

	.checklist-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--color-bg-tertiary);
		border-bottom: 1px solid var(--color-border-light);
	}

	.checklist-header h4 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
	}

	.progress-indicator {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.progress-bar {
		width: 60px;
		height: 4px;
		background-color: var(--color-bg-primary);
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: var(--color-success);
		transition: width var(--transition-fast);
	}

	.progress-text {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.checklist-items {
		padding: var(--spacing-xs) 0;
	}

	.checklist-item {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
		padding: var(--spacing-sm) var(--spacing-md);
		cursor: pointer;
		transition: background-color var(--transition-fast);
	}

	.checklist-item:hover {
		background-color: var(--color-bg-hover);
	}

	.checklist-item.checked {
		opacity: 0.7;
	}

	.checklist-item.checked .item-label {
		text-decoration: line-through;
	}

	.checklist-item input[type='checkbox'] {
		display: none;
	}

	.item-content {
		flex: 1;
		min-width: 0;
	}

	.item-label {
		display: block;
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.item-description {
		display: block;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-top: 2px;
	}

	.checkmark {
		width: 20px;
		height: 20px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 2px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		flex-shrink: 0;
		transition: all var(--transition-fast);
	}

	.checklist-item.checked .checkmark {
		background-color: var(--color-success);
		border-color: var(--color-success);
		color: var(--text-on-accent);
	}

	.completion-message {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: var(--spacing-sm);
		padding: var(--spacing-md);
		background-color: var(--color-bg-tertiary);
		color: var(--color-success);
		font-size: var(--font-size-sm);
		font-weight: 500;
		border-top: 1px solid var(--color-border-light);
	}
</style>
