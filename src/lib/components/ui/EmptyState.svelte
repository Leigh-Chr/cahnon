<script lang="ts">
	/**
	 * EmptyState Component
	 *
	 * Consistent empty state display with icon, title, description, and optional CTA.
	 * Replaces the repeated .empty-state pattern found across 11+ components.
	 */

	import type { Snippet } from 'svelte';
	import Icon from './Icon.svelte';
	import Button from './Button.svelte';

	// Manually define the icon names to avoid complex type inference
	type IconName =
		| 'plus'
		| 'edit'
		| 'delete'
		| 'close'
		| 'check'
		| 'chevron-down'
		| 'chevron-right'
		| 'chevron-left'
		| 'search'
		| 'settings'
		| 'save'
		| 'folder'
		| 'file'
		| 'book'
		| 'alert'
		| 'info'
		| 'warning'
		| 'error'
		| 'drag'
		| 'link'
		| 'unlink'
		| 'eye'
		| 'eye-off'
		| 'copy'
		| 'trash'
		| 'archive'
		| 'restore'
		| 'download'
		| 'upload'
		| 'external'
		| 'menu'
		| 'more'
		| 'filter'
		| 'sort'
		| 'calendar'
		| 'clock'
		| 'user'
		| 'users'
		| 'tag'
		| 'bookmark'
		| 'star'
		| 'heart'
		| 'flag'
		| 'pin'
		| 'lock'
		| 'unlock'
		| 'refresh'
		| 'undo'
		| 'redo'
		| 'image'
		| 'scissors';

	interface Props {
		icon?: IconName;
		title: string;
		description?: string;
		actionLabel?: string;
		onaction?: () => void;
		children?: Snippet;
	}

	let { icon, title, description, actionLabel, onaction, children }: Props = $props();
</script>

<div class="empty-state">
	{#if icon}
		<div class="empty-icon">
			<Icon name={icon} size={48} strokeWidth={1.5} />
		</div>
	{/if}
	<h3 class="empty-title">{title}</h3>
	{#if description}
		<p class="empty-description">{description}</p>
	{/if}
	{#if children}
		<div class="empty-content">
			{@render children()}
		</div>
	{/if}
	{#if actionLabel && onaction}
		<Button variant="primary" size="lg" onclick={onaction}>
			{actionLabel}
		</Button>
	{/if}
</div>

<style>
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-xl);
		min-height: 200px;
	}

	.empty-icon {
		opacity: 0.5;
		margin-bottom: var(--spacing-md);
	}

	.empty-title {
		font-size: var(--font-size-lg);
		font-weight: 600;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-sm);
	}

	.empty-description {
		font-size: var(--font-size-sm);
		max-width: 300px;
		line-height: var(--line-height-relaxed);
		margin-bottom: var(--spacing-lg);
	}

	.empty-content {
		margin-bottom: var(--spacing-lg);
	}

	.empty-state :global(.btn) {
		margin-top: var(--spacing-md);
	}
</style>
