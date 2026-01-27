<!--
  Dashboard - Aggregate project overview.

  Sections:
  a. Progress Overview: total words, word target, % complete, chapter/scene counts
  b. Issues Summary: count by severity, open vs resolved
  c. Tension Curve: embedded visualization
  d. Manuscript Distribution: embedded visualization
  e. Character Heatmap: shown in revision work mode
-->
<script lang="ts">
	import { type Issue, issueApi } from '$lib/api';
	import { appState } from '$lib/stores';

	import CharacterHeatmap from './CharacterHeatmap.svelte';
	import ManuscriptDistribution from './ManuscriptDistribution.svelte';
	import TensionCurve from './TensionCurve.svelte';

	let issues = $state<Issue[]>([]);
	let issuesLoading = $state(false);

	// Load issues on mount
	$effect(() => {
		if (appState.project) {
			loadIssues();
		}
	});

	async function loadIssues() {
		issuesLoading = true;
		try {
			issues = await issueApi.getAll();
		} catch (e) {
			console.error('Failed to load issues:', e);
			issues = [];
		} finally {
			issuesLoading = false;
		}
	}

	// ---- Progress stats ----
	let totalWords = $derived(appState.wordCounts?.total || 0);
	let wordTarget = $derived(appState.project?.word_target || 0);
	let progressPercent = $derived(
		wordTarget > 0 ? Math.min(100, Math.round((totalWords / wordTarget) * 100)) : 0
	);
	let chapterCount = $derived(appState.chapters.length);
	let sceneCount = $derived.by(() => {
		let count = 0;
		for (const scenes of appState.scenes.values()) {
			count += scenes.length;
		}
		return count;
	});

	// ---- Issue stats ----
	let openIssues = $derived(issues.filter((i) => i.status === 'open'));
	let resolvedIssues = $derived(issues.filter((i) => i.status === 'resolved'));
	let errorCount = $derived(openIssues.filter((i) => i.severity === 'error').length);
	let warningCount = $derived(openIssues.filter((i) => i.severity === 'warning').length);
	let infoCount = $derived(openIssues.filter((i) => i.severity === 'info').length);

	// ---- Status breakdown from word counts ----
	let statusData = $derived(appState.wordCounts?.by_status || []);
	let draftScenes = $derived(statusData.find((s) => s.status === 'draft')?.scene_count || 0);
	let revisionScenes = $derived(statusData.find((s) => s.status === 'revision')?.scene_count || 0);
	let doneScenes = $derived(statusData.find((s) => s.status === 'done')?.scene_count || 0);

	// Show heatmap only in revision mode
	let showHeatmap = $derived(appState.workMode === 'revision');

	function formatNumber(n: number): string {
		return n.toLocaleString();
	}

	function navigateToIssues() {
		appState.navigateTo('issue');
	}
</script>

<div class="dashboard">
	<div class="dashboard-header">
		<h2>{appState.project?.title || 'Project'} Dashboard</h2>
		{#if appState.project?.author}
			<span class="author">by {appState.project.author}</span>
		{/if}
	</div>

	<div class="dashboard-grid">
		<!-- Progress Overview Card -->
		<div class="card progress-card">
			<h3>Progress Overview</h3>
			<div class="stats-grid">
				<div class="stat-item primary">
					<span class="stat-value">{formatNumber(totalWords)}</span>
					<span class="stat-label">Total Words</span>
				</div>
				{#if wordTarget > 0}
					<div class="stat-item">
						<span class="stat-value">{formatNumber(wordTarget)}</span>
						<span class="stat-label">Word Target</span>
					</div>
					<div class="stat-item">
						<span class="stat-value">{progressPercent}%</span>
						<span class="stat-label">Complete</span>
					</div>
				{/if}
				<div class="stat-item">
					<span class="stat-value">{chapterCount}</span>
					<span class="stat-label">Chapters</span>
				</div>
				<div class="stat-item">
					<span class="stat-value">{sceneCount}</span>
					<span class="stat-label">Scenes</span>
				</div>
			</div>

			{#if wordTarget > 0}
				<div class="progress-bar-container">
					<div class="progress-bar">
						<div class="progress-fill" style="width: {progressPercent}%"></div>
					</div>
					<span class="progress-text">{progressPercent}% of target</span>
				</div>
			{/if}

			<!-- Scene status breakdown -->
			{#if sceneCount > 0}
				<div class="status-breakdown">
					<div class="status-item draft">
						<span class="status-count">{draftScenes}</span>
						<span class="status-name">Draft</span>
					</div>
					<div class="status-item revision">
						<span class="status-count">{revisionScenes}</span>
						<span class="status-name">Revision</span>
					</div>
					<div class="status-item done">
						<span class="status-count">{doneScenes}</span>
						<span class="status-name">Done</span>
					</div>
				</div>
			{/if}
		</div>

		<!-- Issues Summary Card -->
		<div class="card issues-card">
			<div class="card-header-row">
				<h3>Issues</h3>
				<button class="link-btn" onclick={navigateToIssues}>View All</button>
			</div>
			{#if issuesLoading}
				<p class="loading-text">Loading...</p>
			{:else if issues.length === 0}
				<p class="no-issues">No issues detected.</p>
			{:else}
				<div class="issues-grid">
					<div class="issue-stat error">
						<span class="issue-count">{errorCount}</span>
						<span class="issue-label">Errors</span>
					</div>
					<div class="issue-stat warning">
						<span class="issue-count">{warningCount}</span>
						<span class="issue-label">Warnings</span>
					</div>
					<div class="issue-stat info">
						<span class="issue-count">{infoCount}</span>
						<span class="issue-label">Info</span>
					</div>
				</div>
				<div class="issues-summary-row">
					<span class="open-count">{openIssues.length} open</span>
					<span class="resolved-count">{resolvedIssues.length} resolved</span>
				</div>
			{/if}
		</div>

		<!-- Tension Curve Card -->
		<div class="card wide-card">
			<h3>Tension Curve</h3>
			<TensionCurve />
		</div>

		<!-- Manuscript Distribution Card -->
		<div class="card wide-card">
			<h3>Manuscript Distribution</h3>
			<ManuscriptDistribution />
		</div>

		<!-- Character Heatmap Card (revision mode only) -->
		{#if showHeatmap}
			<div class="card wide-card">
				<h3>Character Appearances (POV)</h3>
				<CharacterHeatmap />
			</div>
		{/if}
	</div>
</div>

<style>
	.dashboard {
		padding: var(--spacing-lg);
		overflow-y: auto;
		height: 100%;
	}

	.dashboard-header {
		display: flex;
		align-items: baseline;
		gap: var(--spacing-md);
		margin-bottom: var(--spacing-lg);
	}

	.dashboard-header h2 {
		margin: 0;
		font-size: var(--font-size-xl, 1.5rem);
		color: var(--color-text-primary);
	}

	.author {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.dashboard-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--spacing-lg);
	}

	.card {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-lg);
		padding: var(--spacing-lg);
		box-shadow: var(--shadow-sm);
	}

	.card h3 {
		margin: 0 0 var(--spacing-md) 0;
		font-size: var(--font-size-base);
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.wide-card {
		grid-column: 1 / -1;
	}

	/* Progress card */
	.stats-grid {
		display: flex;
		flex-wrap: wrap;
		gap: var(--spacing-lg);
		margin-bottom: var(--spacing-md);
	}

	.stat-item {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.stat-item.primary .stat-value {
		font-size: var(--font-size-xl, 1.5rem);
		color: var(--color-accent);
	}

	.stat-value {
		font-size: var(--font-size-lg);
		font-weight: 700;
		color: var(--color-text-primary);
	}

	.stat-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.progress-bar-container {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
		margin-bottom: var(--spacing-md);
	}

	.progress-bar {
		flex: 1;
		height: 8px;
		background-color: var(--color-bg-tertiary);
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: var(--color-accent);
		border-radius: 4px;
		transition: width 0.3s ease;
	}

	.progress-text {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.status-breakdown {
		display: flex;
		gap: var(--spacing-md);
		padding-top: var(--spacing-sm);
		border-top: 1px solid var(--color-border-light, var(--color-border));
	}

	.status-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		font-size: var(--font-size-sm);
	}

	.status-count {
		font-weight: 600;
	}

	.status-name {
		color: var(--color-text-muted);
	}

	.status-item.draft .status-count {
		color: var(--color-warning);
	}

	.status-item.revision .status-count {
		color: var(--color-info);
	}

	.status-item.done .status-count {
		color: var(--color-success);
	}

	/* Issues card */
	.card-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--spacing-md);
	}

	.card-header-row h3 {
		margin: 0;
	}

	.link-btn {
		background: none;
		border: none;
		font-size: var(--font-size-xs);
		color: var(--color-accent);
		cursor: pointer;
		padding: var(--spacing-xs);
		border-radius: var(--border-radius-sm);
	}

	.link-btn:hover {
		text-decoration: underline;
		background-color: var(--color-bg-hover);
	}

	.loading-text {
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
	}

	.no-issues {
		color: var(--color-success);
		font-size: var(--font-size-sm);
		font-weight: 500;
	}

	.issues-grid {
		display: flex;
		gap: var(--spacing-lg);
		margin-bottom: var(--spacing-sm);
	}

	.issue-stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
	}

	.issue-count {
		font-size: var(--font-size-lg);
		font-weight: 700;
	}

	.issue-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.issue-stat.error .issue-count {
		color: var(--color-error);
	}

	.issue-stat.warning .issue-count {
		color: var(--color-warning);
	}

	.issue-stat.info .issue-count {
		color: var(--color-info);
	}

	.issues-summary-row {
		display: flex;
		gap: var(--spacing-md);
		padding-top: var(--spacing-xs);
		border-top: 1px solid var(--color-border-light, var(--color-border));
		font-size: var(--font-size-xs);
	}

	.open-count {
		color: var(--color-warning);
		font-weight: 500;
	}

	.resolved-count {
		color: var(--color-success);
		font-weight: 500;
	}

	/* Responsive */
	@media (max-width: 700px) {
		.dashboard-grid {
			grid-template-columns: 1fr;
		}

		.wide-card {
			grid-column: 1;
		}
	}
</style>
