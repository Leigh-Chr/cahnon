<!--
  Individual menu item for ContextMenu.
  Supports icon, label, keyboard shortcut hint, disabled state, and danger variant.

  AL3: Added icon support and improved shortcut display styling
-->
<script lang="ts">
	import Icon from './Icon.svelte';

	type IconName =
		| 'plus'
		| 'edit'
		| 'delete'
		| 'close'
		| 'copy'
		| 'trash'
		| 'restore'
		| 'archive'
		| 'eye'
		| 'eye-off'
		| 'link'
		| 'unlink'
		| 'pin'
		| 'star'
		| 'flag'
		| 'bookmark'
		| 'undo'
		| 'redo'
		| 'download'
		| 'upload'
		| 'refresh'
		| 'search'
		| 'settings'
		| 'user'
		| 'scissors';

	let {
		label,
		icon,
		shortcut,
		disabled = false,
		danger = false,
		onclick,
	}: {
		label: string;
		icon?: IconName;
		shortcut?: string;
		disabled?: boolean;
		danger?: boolean;
		onclick: () => void;
	} = $props();

	function handleClick() {
		if (!disabled) {
			onclick();
		}
	}
</script>

<button
	class="context-menu-item"
	class:danger
	class:disabled
	class:has-icon={icon}
	role="menuitem"
	aria-disabled={disabled}
	onclick={handleClick}
>
	{#if icon}
		<span class="item-icon">
			<Icon name={icon} size={14} />
		</span>
	{/if}
	<span class="label">{label}</span>
	{#if shortcut}
		<kbd class="shortcut">{shortcut}</kbd>
	{/if}
</button>

<style>
	.context-menu-item {
		display: flex;
		align-items: center;
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-md);
		text-align: left;
		color: var(--text-primary);
		background: none;
		border: none;
		border-radius: var(--border-radius-sm);
		font: inherit;
		font-size: var(--font-size-sm);
		gap: var(--spacing-sm);
		transition: background-color var(--transition-fast);
	}

	.context-menu-item:hover:not(.disabled),
	.context-menu-item:focus:not(.disabled) {
		background-color: var(--surface-hover);
		outline: none;
	}

	.context-menu-item.danger {
		color: var(--danger-default);
	}

	.context-menu-item.danger:hover:not(.disabled),
	.context-menu-item.danger:focus:not(.disabled) {
		background-color: var(--danger-subtle);
	}

	.context-menu-item.disabled {
		color: var(--text-disabled);
		cursor: not-allowed;
	}

	/* AL3: Icon support */
	.item-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		color: var(--text-muted);
	}

	.context-menu-item.danger .item-icon {
		color: var(--danger-default);
	}

	.label {
		flex: 1;
	}

	/* AL3: Keyboard shortcut hint styling */
	.shortcut {
		margin-left: auto;
		color: var(--text-muted);
		font-size: var(--font-size-xs);
		font-family: var(--font-family-mono);
		background-color: var(--surface-tertiary);
		padding: 1px 4px;
		border-radius: var(--border-radius-sm);
	}
</style>
