<!--
  Main application toolbar.

  Features:
  - View mode tabs (Editor, Corkboard, Timeline, Bible)
  - Outline toggle
  - Context panel toggle
  - Quick Open trigger
  - Dark mode toggle
  - Export dialog trigger
  - Settings menu (import, review grid, etc.)
  - Project title display
-->
<script lang="ts">
	import { appState } from '$lib/stores';
	import { formatShortcut } from '$lib/utils';

	import { Icon } from './ui';

	let showMoreMenu = $state(false);

	function closeMoreMenu() {
		showMoreMenu = false;
	}

	function handleMoreMenuAction(action: (() => void) | null | undefined) {
		action?.();
		closeMoreMenu();
	}

	// BB4: View modes with descriptive tooltips
	const viewModes = [
		{ id: 'editor', label: 'Editor', shortcut: '1', tooltip: 'Write and edit your manuscript' },
		{
			id: 'corkboard',
			label: 'Corkboard',
			shortcut: '2',
			tooltip: 'Visual overview of scenes as cards',
		},
		{
			id: 'timeline',
			label: 'Timeline',
			shortcut: '3',
			tooltip: 'Chronological view of story events',
		},
		{
			id: 'bible',
			label: 'Codex',
			shortcut: '4',
			tooltip: 'Characters, locations & world-building',
		},
		{
			id: 'issues',
			label: 'Continuity',
			shortcut: '5',
			tooltip: 'Track plot holes & inconsistencies',
		},
		{
			id: 'dashboard',
			label: 'Dashboard',
			shortcut: '6',
			tooltip: 'Project overview & statistics',
		},
	] as const;

	interface Props {
		onOpenReviewGrid?: (() => void) | null;
		onOpenImportDialog?: (() => void) | null;
		onOpenSettings?: (() => void) | null;
		onOpenSnapshots?: (() => void) | null;
		onOpenKeyboardShortcuts?: (() => void) | null;
	}

	let {
		onOpenReviewGrid = null,
		onOpenImportDialog = null,
		onOpenSettings = null,
		onOpenSnapshots = null,
		onOpenKeyboardShortcuts = null,
	}: Props = $props();
</script>

<header class="toolbar">
	<div class="toolbar-left">
		<button
			class="toolbar-btn icon-btn"
			disabled={!appState.canNavigateBack}
			onclick={() => appState.navigateBack()}
			title="Navigate Back (Alt+Left)"
			aria-label="Navigate Back"
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<polyline points="15 18 9 12 15 6" />
			</svg>
		</button>
		<button
			class="toolbar-btn icon-btn"
			disabled={!appState.canNavigateForward}
			onclick={() => appState.navigateForward()}
			title="Navigate Forward (Alt+Right)"
			aria-label="Navigate Forward"
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<polyline points="9 18 15 12 9 6" />
			</svg>
		</button>

		<div class="separator"></div>

		{#if appState.viewMode !== 'dashboard' && appState.viewMode !== 'issues'}
			<button
				class="toolbar-btn icon-btn"
				class:active={appState.showOutline}
				onclick={() => appState.toggleOutline()}
				title="Toggle Outline ({formatShortcut('\\')})"
				aria-label="Toggle Outline"
			>
				<svg
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<rect x="3" y="3" width="18" height="18" rx="2" />
					<line x1="9" y1="3" x2="9" y2="21" />
				</svg>
			</button>
		{/if}

		<!-- AG1: Quick Open button (visible search trigger) -->
		<button
			class="toolbar-btn icon-btn"
			onclick={() => appState.toggleQuickOpen()}
			title="Quick Open ({formatShortcut('K')})"
			aria-label="Quick Open"
		>
			<Icon name="search" size={18} />
		</button>

		<div class="separator"></div>

		<!-- AW1, BB4: View switcher with visible shortcut hints and descriptive tooltips -->
		<div class="view-switcher">
			{#each viewModes as mode (mode.id)}
				<button
					class="view-btn"
					class:active={appState.viewMode === mode.id}
					onclick={() => appState.setViewMode(mode.id)}
					title="{mode.tooltip} ({formatShortcut(mode.shortcut)})"
				>
					<span>{mode.label}</span>
					<span class="shortcut-hint">{formatShortcut(mode.shortcut)}</span>
				</button>
			{/each}
		</div>
	</div>

	<div class="toolbar-center">
		{#if appState.project}
			<nav class="breadcrumb" aria-label="Navigation breadcrumb">
				<button
					class="breadcrumb-item"
					onclick={() => appState.setViewMode('dashboard')}
					title={appState.project.title}
				>
					{appState.project.title}
				</button>
				{#if appState.isDemo}
					<span class="demo-badge">Demo</span>
				{/if}
				{#if appState.selectedChapter}
					<span class="breadcrumb-sep" aria-hidden="true"
						><svg
							width="12"
							height="12"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"><polyline points="9 18 15 12 9 6" /></svg
						></span
					>
					<button
						class="breadcrumb-item"
						onclick={() => {
							if (appState.selectedChapterId) {
								const chapterScenes = appState.scenes.get(appState.selectedChapterId);
								if (chapterScenes?.length) {
									appState.selectScene(chapterScenes[0].id, appState.selectedChapterId);
								}
								appState.setViewMode('editor');
							}
						}}
						title={appState.selectedChapter.title}
					>
						<span class="truncate">{appState.selectedChapter.title}</span>
					</button>
				{/if}
				{#if appState.selectedScene}
					<span class="breadcrumb-sep" aria-hidden="true"
						><svg
							width="12"
							height="12"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"><polyline points="9 18 15 12 9 6" /></svg
						></span
					>
					<span class="breadcrumb-current truncate" title={appState.selectedScene.title}>
						{appState.selectedScene.title}
					</span>
				{/if}
				<!-- AF1, BA5: More visible unsaved badge with pending chars count -->
				{#if appState.hasUnsavedChanges}
					<span class="unsaved-badge" title="You have unsaved changes">
						<span class="unsaved-dot-pulse"></span>
						{#if appState.pendingCharsCount > 0}
							<span class="chars-pending">{appState.pendingCharsCount} chars</span>
						{:else}
							Unsaved
						{/if}
					</span>
				{/if}
				{#if appState.viewMode === 'bible' && appState.canNavigateBack}
					<button
						class="breadcrumb-back-link"
						onclick={() => appState.navigateBack()}
						title="Return to previous view"
					>
						&#8592; Back
					</button>
				{/if}
			</nav>
		{/if}
	</div>

	<div class="toolbar-right">
		<button
			class="toolbar-btn icon-btn"
			onclick={() => onOpenReviewGrid?.()}
			title="Review Grid ({formatShortcut('G')})"
			aria-label="Review Grid"
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<rect x="3" y="3" width="7" height="7" />
				<rect x="14" y="3" width="7" height="7" />
				<rect x="14" y="14" width="7" height="7" />
				<rect x="3" y="14" width="7" height="7" />
			</svg>
		</button>

		<div class="separator"></div>

		<div class="more-menu-container">
			<button
				class="toolbar-btn icon-btn"
				class:active={showMoreMenu}
				onclick={() => (showMoreMenu = !showMoreMenu)}
				title="More actions"
				aria-label="More actions"
				aria-expanded={showMoreMenu}
			>
				<svg
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<circle cx="12" cy="5" r="1" />
					<circle cx="12" cy="12" r="1" />
					<circle cx="12" cy="19" r="1" />
				</svg>
			</button>

			{#if showMoreMenu}
				<div
					class="more-menu-backdrop"
					onclick={closeMoreMenu}
					onkeydown={(e) => {
						if (e.key === 'Escape') closeMoreMenu();
					}}
					role="presentation"
					tabindex="-1"
				></div>
				<!-- AW2: Organized menu with separators -->
				<div class="more-menu" role="menu">
					<!-- File operations -->
					<button
						class="more-menu-item"
						onclick={() => handleMoreMenuAction(onOpenImportDialog)}
						role="menuitem"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
							<polyline points="7 10 12 15 17 10" />
							<line x1="12" y1="15" x2="12" y2="3" />
						</svg>
						<span>Import</span>
						<kbd>{formatShortcut('I', true, true)}</kbd>
					</button>
					<button
						class="more-menu-item"
						onclick={() => handleMoreMenuAction(() => appState.openExportDialog())}
						role="menuitem"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
							<polyline points="17 8 12 3 7 8" />
							<line x1="12" y1="3" x2="12" y2="15" />
						</svg>
						<span>Export</span>
						<kbd>{formatShortcut('E')}</kbd>
					</button>

					<div class="more-menu-separator" role="separator"></div>

					<!-- Project management -->
					<button
						class="more-menu-item"
						onclick={() => handleMoreMenuAction(onOpenSnapshots)}
						role="menuitem"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
							<circle cx="8.5" cy="8.5" r="1.5" />
							<polyline points="21 15 16 10 5 21" />
						</svg>
						<span>Snapshots</span>
						<kbd>{formatShortcut('S', true, true)}</kbd>
					</button>
					<button
						class="more-menu-item"
						onclick={() => handleMoreMenuAction(() => appState.openTrashView())}
						role="menuitem"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<polyline points="3 6 5 6 21 6" />
							<path
								d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
							/>
						</svg>
						<span>Trash</span>
					</button>
				</div>
			{/if}
		</div>

		<!-- AG2: Help/Keyboard shortcuts button -->
		<button
			class="toolbar-btn icon-btn"
			onclick={() => onOpenKeyboardShortcuts?.()}
			title="Keyboard Shortcuts ({formatShortcut('/')})"
			aria-label="Keyboard Shortcuts"
		>
			<Icon name="help-circle" size={18} />
		</button>

		<button
			class="toolbar-btn icon-btn"
			onclick={() => onOpenSettings?.()}
			title="Settings"
			aria-label="Settings"
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<circle cx="12" cy="12" r="3" />
				<path
					d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"
				/>
			</svg>
		</button>

		<div class="separator"></div>

		{#if appState.viewMode !== 'issues' && appState.viewMode !== 'dashboard'}
			<button
				class="toolbar-btn icon-btn"
				class:active={appState.showContextPanel}
				onclick={() => appState.toggleContextPanel()}
				title="Toggle Context Panel ({formatShortcut('\\', true, true)})"
				aria-label="Toggle Context Panel"
			>
				<svg
					width="18"
					height="18"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<rect x="3" y="3" width="18" height="18" rx="2" />
					<line x1="15" y1="3" x2="15" y2="21" />
				</svg>
			</button>
		{/if}
	</div>
</header>

<style>
	.toolbar {
		height: var(--toolbar-height);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border);
		gap: var(--spacing-md);
		-webkit-app-region: drag;
	}

	.toolbar-left,
	.toolbar-right {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		-webkit-app-region: no-drag;
	}

	.toolbar-center {
		flex: 1;
		text-align: center;
	}

	.breadcrumb {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
		max-width: 100%;
		overflow: hidden;
		-webkit-app-region: no-drag;
	}

	.breadcrumb-item {
		color: var(--color-text-muted);
		font-weight: 500;
		max-width: 160px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		border-radius: var(--border-radius-sm);
		padding: 1px var(--spacing-xs);
		transition: all var(--transition-fast);
	}

	.breadcrumb-item:hover {
		color: var(--color-text-primary);
		background-color: var(--color-bg-hover);
	}

	.breadcrumb-sep {
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.breadcrumb-current {
		font-weight: 600;
		color: var(--color-text-primary);
		max-width: 180px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.breadcrumb-back-link {
		color: var(--color-accent);
		font-size: var(--font-size-sm);
		display: inline-flex;
		align-items: center;
		padding: 2px var(--spacing-xs);
		border-radius: var(--border-radius-sm);
		margin-left: var(--spacing-sm);
	}

	.breadcrumb-back-link:hover {
		background-color: var(--color-bg-hover);
	}

	/* AF1: More visible unsaved badge in breadcrumb */
	.unsaved-badge {
		display: inline-flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: 2px var(--spacing-sm);
		font-size: var(--font-size-xs);
		font-weight: 600;
		color: var(--color-warning);
		background-color: color-mix(in srgb, var(--color-warning) 15%, transparent);
		border-radius: var(--border-radius-sm);
		margin-left: var(--spacing-xs);
	}

	.unsaved-dot-pulse {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background-color: var(--color-warning);
		animation: unsaved-pulse 1.5s ease-in-out infinite;
	}

	@keyframes unsaved-pulse {
		0%,
		100% {
			transform: scale(1);
			opacity: 0.7;
		}
		50% {
			transform: scale(1.3);
			opacity: 1;
		}
	}

	.demo-badge {
		display: inline-block;
		padding: 1px 6px;
		font-size: var(--font-size-xs);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		background-color: var(--color-accent-light);
		color: var(--color-accent);
		border-radius: var(--border-radius-sm);
		vertical-align: middle;
		margin-left: var(--spacing-xs);
	}

	.toolbar-btn {
		padding: var(--spacing-xs) var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		transition: background-color var(--transition-fast);
	}

	.toolbar-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.toolbar-btn.active {
		background-color: var(--color-bg-tertiary);
	}

	.icon-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		padding: 0;
	}

	.icon-btn svg {
		color: var(--color-text-secondary);
	}

	.icon-btn.active svg {
		color: var(--color-accent);
	}

	.separator {
		width: 1px;
		height: 20px;
		background-color: var(--color-border);
		margin: 0 var(--spacing-xs);
	}

	.view-switcher {
		display: flex;
		gap: 2px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-md);
		padding: 2px;
	}

	.view-btn {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1px;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		border-radius: var(--border-radius-sm);
		color: var(--color-text-secondary);
		transition: all var(--transition-fast);
	}

	.view-btn:hover {
		color: var(--color-text-primary);
	}

	.view-btn.active {
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
		box-shadow: var(--shadow-sm);
	}

	/* AW1: Visible keyboard shortcut hints */
	.view-btn .shortcut-hint {
		font-size: 9px;
		font-family: var(--font-family-mono);
		color: var(--color-text-muted);
		opacity: 0.7;
		line-height: 1;
	}

	.view-btn:hover .shortcut-hint,
	.view-btn.active .shortcut-hint {
		opacity: 1;
	}

	/* More menu */
	.more-menu-container {
		position: relative;
	}

	.more-menu-backdrop {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 99;
	}

	.more-menu {
		position: absolute;
		top: 100%;
		right: 0;
		margin-top: var(--spacing-xs);
		min-width: 180px;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		box-shadow: var(--shadow-lg);
		padding: var(--spacing-xs);
		z-index: 100;
	}

	.more-menu-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		width: 100%;
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-sm);
		text-align: left;
		border-radius: var(--border-radius-sm);
		color: var(--color-text-primary);
		transition: background-color var(--transition-fast);
	}

	.more-menu-item:hover {
		background-color: var(--color-bg-hover);
	}

	.more-menu-item svg {
		flex-shrink: 0;
		color: var(--color-text-muted);
	}

	.more-menu-item span {
		flex: 1;
	}

	.more-menu-item kbd {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		font-family: inherit;
	}

	/* AW2: Menu separator for grouping */
	.more-menu-separator {
		height: 1px;
		background-color: var(--color-border);
		margin: var(--spacing-xs) 0;
	}

	/* Responsive toolbar adjustments */
	@media (max-width: 1000px) {
		.breadcrumb-item {
			max-width: 120px;
		}
		.breadcrumb-current {
			max-width: 140px;
		}
		/* Hide shortcut hints to save space */
		.view-btn .shortcut-hint {
			display: none;
		}
		.view-btn {
			padding: var(--spacing-xs) var(--spacing-sm);
		}
	}

	@media (max-width: 900px) {
		.toolbar {
			gap: var(--spacing-sm);
			padding: 0 var(--spacing-sm);
		}
		.breadcrumb-item {
			max-width: 100px;
		}
		.breadcrumb-current {
			max-width: 120px;
		}
		.view-btn {
			padding: var(--spacing-xs);
			font-size: var(--font-size-xs);
		}
		.separator {
			margin: 0 2px;
		}
		/* Hide pending chars count on small screens */
		.chars-pending {
			display: none;
		}
	}

	@media (max-width: 800px) {
		/* Hide view labels, show only abbreviations */
		.view-btn span:first-child {
			max-width: 40px;
			overflow: hidden;
			text-overflow: ellipsis;
		}
		.breadcrumb-item {
			max-width: 80px;
		}
		.breadcrumb-current {
			max-width: 100px;
		}
		/* Stack unsaved badge below breadcrumb if needed */
		.unsaved-badge {
			font-size: 10px;
			padding: 1px var(--spacing-xs);
		}
	}
</style>
