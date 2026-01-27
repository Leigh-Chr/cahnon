<!--
  Individual menu item for ContextMenu.
  Supports icon, label, keyboard shortcut hint, disabled state, and danger variant.
-->
<script lang="ts">
	let {
		label,
		shortcut,
		disabled = false,
		danger = false,
		onclick,
	}: {
		label: string;
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
	role="menuitem"
	aria-disabled={disabled}
	onclick={handleClick}
>
	<span class="label">{label}</span>
	{#if shortcut}
		<span class="shortcut">{shortcut}</span>
	{/if}
</button>

<style>
	.context-menu-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-md);
		text-align: left;
		color: var(--text-primary);
		background: none;
		border: none;
		font: inherit;
		font-size: var(--font-size-sm);
		gap: var(--spacing-lg);
	}

	.context-menu-item:hover:not(.disabled) {
		background-color: var(--surface-hover);
	}

	.context-menu-item.danger {
		color: var(--danger-default);
	}

	.context-menu-item.danger:hover:not(.disabled) {
		background-color: var(--danger-subtle);
	}

	.context-menu-item.disabled {
		color: var(--text-disabled);
	}

	.shortcut {
		color: var(--text-muted);
		font-size: var(--font-size-xs);
	}
</style>
