<!--
  Main application toolbar.

  Features:
  - View mode tabs (Editor, Corkboard, Timeline, Bible)
  - Work mode toggle (Writing/Revision)
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
	import { revisionPasses, type RevisionPassId } from '$lib/utils/revision-passes';

	const viewModes = [
		{ id: 'editor', label: 'Editor', shortcut: '1' },
		{ id: 'corkboard', label: 'Corkboard', shortcut: '2' },
		{ id: 'timeline', label: 'Timeline', shortcut: '3' },
		{ id: 'bible', label: 'Bible', shortcut: '4' },
		{ id: 'issues', label: 'Issues', shortcut: '5' },
		{ id: 'dashboard', label: 'Dashboard', shortcut: '6' },
	] as const;

	interface Props {
		onOpenReviewGrid?: (() => void) | null;
		onOpenImportDialog?: (() => void) | null;
		onOpenSettings?: (() => void) | null;
		onOpenSnapshots?: (() => void) | null;
	}

	let {
		onOpenReviewGrid = null,
		onOpenImportDialog = null,
		onOpenSettings = null,
		onOpenSnapshots = null,
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

		<div class="separator"></div>

		<div class="view-switcher">
			{#each viewModes as mode (mode.id)}
				<button
					class="view-btn"
					class:active={appState.viewMode === mode.id}
					onclick={() => appState.setViewMode(mode.id)}
					title="{mode.label} ({formatShortcut(mode.shortcut)})"
				>
					{mode.label}
				</button>
			{/each}
		</div>
	</div>

	<div class="toolbar-center">
		{#if appState.project}
			<span class="project-title">
				{appState.project.title}
				{#if appState.isDemo}
					<span class="demo-badge">Demo</span>
				{/if}
				{#if appState.hasUnsavedChanges}
					<span class="unsaved-indicator" title="Unsaved changes">•</span>
				{/if}
			</span>
		{/if}
	</div>

	<div class="toolbar-right">
		<button
			class="mode-toggle"
			class:revision={appState.workMode === 'revision'}
			onclick={() => appState.toggleWorkMode()}
			title="Toggle Work Mode ({formatShortcut('D')})"
		>
			{appState.workMode === 'writing' ? 'Writing' : 'Revision'}
		</button>

		{#if appState.workMode === 'revision'}
			<div class="revision-passes">
				<button
					class="pass-btn"
					class:active={appState.revisionPass === null}
					onclick={() => appState.setRevisionPass(null)}
					title="Show all sections">All</button
				>
				{#each revisionPasses as pass (pass.id)}
					<button
						class="pass-btn"
						class:active={appState.revisionPass === pass.id}
						onclick={() => appState.setRevisionPass(pass.id as RevisionPassId)}
						title={pass.description}>{pass.label}</button
					>
				{/each}
			</div>
		{/if}

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

		<button
			class="toolbar-btn icon-btn"
			onclick={() => onOpenImportDialog?.()}
			title="Import Content ({formatShortcut('I')})"
			aria-label="Import Content"
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
				<polyline points="7 10 12 15 17 10" />
				<line x1="12" y1="15" x2="12" y2="3" />
			</svg>
		</button>

		<button
			class="toolbar-btn icon-btn"
			onclick={() => appState.openExportDialog()}
			title="Export Project ({formatShortcut('E')})"
			aria-label="Export Project"
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
				<polyline points="17 8 12 3 7 8" />
				<line x1="12" y1="3" x2="12" y2="15" />
			</svg>
		</button>

		<button
			class="toolbar-btn icon-btn"
			onclick={() => onOpenSnapshots?.()}
			title="Snapshots"
			aria-label="Snapshots"
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
				<circle cx="8.5" cy="8.5" r="1.5" />
				<polyline points="21 15 16 10 5 21" />
			</svg>
		</button>

		<button
			class="toolbar-btn icon-btn"
			onclick={() => appState.openTrashView()}
			title="Trash"
			aria-label="Trash"
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<polyline points="3 6 5 6 21 6" />
				<path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
			</svg>
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

	.project-title {
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-secondary);
	}

	.unsaved-indicator {
		color: var(--color-warning);
		font-size: var(--font-size-lg);
		vertical-align: middle;
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

	.mode-toggle {
		padding: var(--spacing-xs) var(--spacing-md);
		font-size: var(--font-size-sm);
		font-weight: 500;
		border-radius: var(--border-radius-md);
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-secondary);
		transition: all var(--transition-fast);
	}

	.mode-toggle:hover {
		background-color: var(--color-bg-hover);
	}

	.mode-toggle.revision {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}

	.revision-passes {
		display: flex;
		gap: 1px;
		background-color: var(--color-border-light);
		border-radius: var(--border-radius-sm);
		overflow: hidden;
		margin-left: var(--spacing-xs);
	}

	.pass-btn {
		padding: var(--spacing-xs) var(--spacing-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
		background-color: var(--color-bg-secondary);
		color: var(--color-text-muted);
		transition: all var(--transition-fast);
	}

	.pass-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.pass-btn.active {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}
</style>
