<!--
  Keyboard shortcuts cheatsheet dialog.
  Opened via Cmd+/ or from the toolbar.
  Read-only modal with categorized shortcut grid.
-->
<script lang="ts">
	import { appState } from '$lib/stores';
	import {
		defaultKeyboardShortcuts,
		type KeyboardShortcuts,
		shortcutLabels,
	} from '$lib/stores/types';
	import { formatShortcut } from '$lib/utils';
	import { trapFocus } from '$lib/utils/focus-trap';

	interface Props {
		isOpen: boolean;
		onclose: () => void;
	}

	let { isOpen = $bindable(), onclose }: Props = $props();

	// CF2: Search shortcuts
	let searchQuery = $state('');

	// Filter shortcuts based on search query
	let filteredCategories = $derived.by(() => {
		if (!searchQuery.trim()) return categories;

		const q = searchQuery.toLowerCase();
		return categories
			.map((category) => ({
				...category,
				actions: category.actions.filter((action) => {
					const label = shortcutLabels[action].toLowerCase();
					const binding = formatBinding(action).toLowerCase();
					return label.includes(q) || binding.includes(q);
				}),
			}))
			.filter((category) => category.actions.length > 0);
	});

	// Clear search when dialog closes
	$effect(() => {
		if (!isOpen) {
			searchQuery = '';
		}
	});

	type ShortcutCategory = {
		label: string;
		actions: (keyof KeyboardShortcuts)[];
	};

	const categories: ShortcutCategory[] = [
		{
			label: 'Navigation',
			actions: ['quickOpen', 'toggleOutline', 'toggleContextPanel', 'nextScene', 'prevScene'],
		},
		{
			label: 'Views',
			actions: [
				'viewEditor',
				'viewCorkboard',
				'viewTimeline',
				'viewBible',
				'viewIssues',
				'viewDashboard',
			],
		},
		{
			label: 'Editing',
			actions: ['save', 'find', 'findReplace', 'addAnnotation'],
		},
		{
			label: 'Tools',
			actions: [
				'export',
				'importDialog',
				'reviewGrid',
				'arcsManager',
				'eventsManager',
				'templatesManager',
			],
		},
		{
			label: 'Modes',
			actions: ['toggleWorkMode', 'focusMode', 'fullscreen', 'showShortcuts'],
		},
	];

	function formatBinding(action: keyof KeyboardShortcuts): string {
		const binding = appState.keyboardShortcuts?.[action] ?? defaultKeyboardShortcuts[action];
		if (!binding) return '';
		return formatShortcut(binding.key, binding.mod, binding.shift);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			onclose();
		}
	}
</script>

{#if isOpen}
	<div
		class="shortcuts-overlay"
		onclick={onclose}
		onkeydown={handleKeydown}
		role="presentation"
		tabindex="-1"
	>
		<!-- AE1: Focus trap -->
		<div
			class="shortcuts-dialog modal-enter"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			aria-label="Keyboard Shortcuts"
			tabindex="-1"
			use:trapFocus={{ onEscape: onclose }}
		>
			<div class="dialog-header">
				<h2>Keyboard Shortcuts</h2>
				<button class="close-btn" onclick={onclose} aria-label="Close">
					<svg
						width="18"
						height="18"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
					>
						<line x1="18" y1="6" x2="6" y2="18" />
						<line x1="6" y1="6" x2="18" y2="18" />
					</svg>
				</button>
			</div>

			<!-- CF2: Search shortcuts -->
			<div class="search-container">
				<input
					type="search"
					class="shortcut-search"
					placeholder="Search shortcuts..."
					bind:value={searchQuery}
					aria-label="Search shortcuts"
				/>
			</div>

			<div class="shortcuts-grid">
				{#each filteredCategories as category (category.label)}
					<div class="category">
						<h3>{category.label}</h3>
						<div class="shortcuts-list">
							{#each category.actions as action (action)}
								<div class="shortcut-row">
									<span class="shortcut-label">{shortcutLabels[action]}</span>
									<kbd class="shortcut-key">{formatBinding(action)}</kbd>
								</div>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>
{/if}

<style>
	.shortcuts-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: var(--overlay-backdrop);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1100;
	}

	.shortcuts-dialog {
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-lg);
		width: 600px;
		max-width: 90vw;
		max-height: 80vh;
		overflow-y: auto;
		padding: var(--spacing-lg);
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-lg);
	}

	.dialog-header h2 {
		margin: 0;
		font-size: var(--font-size-lg);
		color: var(--color-text-primary);
	}

	.close-btn {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.close-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	/* CF2: Search shortcuts */
	.search-container {
		margin-bottom: var(--spacing-md);
	}

	.shortcut-search {
		width: 100%;
		padding: var(--spacing-sm) var(--spacing-md);
		font-size: var(--font-size-sm);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		color: var(--color-text-primary);
		outline: none;
	}

	.shortcut-search:focus {
		border-color: var(--color-accent);
		box-shadow: 0 0 0 2px color-mix(in srgb, var(--color-accent) 20%, transparent);
	}

	.shortcut-search::placeholder {
		color: var(--color-text-muted);
	}

	.shortcuts-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--spacing-lg);
	}

	.category h3 {
		margin: 0 0 var(--spacing-sm) 0;
		font-size: var(--font-size-sm);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
	}

	.shortcuts-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.shortcut-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-xs) 0;
	}

	.shortcut-label {
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.shortcut-key {
		font-size: var(--font-size-xs);
		font-family: inherit;
		padding: 2px var(--spacing-sm);
		background-color: var(--color-bg-tertiary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		white-space: nowrap;
	}

	@media (max-width: 900px) {
		.shortcuts-dialog {
			width: 500px;
			padding: var(--spacing-md);
		}
	}

	@media (max-width: 600px) {
		.shortcuts-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
