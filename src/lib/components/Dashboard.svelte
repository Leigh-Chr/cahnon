<!--
  Dashboard - Aggregate project overview.

  Sections:
  a. Progress Overview: total words, word target, % complete, chapter/scene counts
  b. Issues Summary: count by severity, open vs resolved
  c. Tension Curve: embedded visualization
  d. Manuscript Distribution: embedded visualization
-->
<script lang="ts">
	import { untrack } from 'svelte';

	import { type Issue, issueApi } from '$lib/api';
	import { appState } from '$lib/stores';
	import {
		type ChecklistState,
		firstProjectSteps,
		getChecklistState,
		isChecklistComplete,
		markChecklistStep,
	} from '$lib/stores/onboarding';

	import ManuscriptDistribution from './ManuscriptDistribution.svelte';
	import TensionCurve from './TensionCurve.svelte';
	import { EmptyState, Icon, LoadingState } from './ui';

	let issues = $state<Issue[]>([]);
	let issuesLoading = $state(false);

	// UA1: First project checklist state
	let checklistState = $state<ChecklistState>(getChecklistState());
	let showChecklist = $state(!isChecklistComplete());

	// Refresh checklist state when data changes
	$effect(() => {
		// Track these dependencies explicitly
		const chapters = chapterCount;
		const scenes = sceneCount;
		const hasCharacter = appState.bibleEntries.some((e) => e.entry_type === 'character');

		// Read current state without tracking to avoid infinite loop
		const currentState = untrack(() => checklistState);
		const newState = { ...currentState };
		let changed = false;

		if (chapters > 0 && !newState['first-chapter']) {
			newState['first-chapter'] = true;
			markChecklistStep('first-chapter');
			changed = true;
		}
		if (scenes > 0 && !newState['first-scene']) {
			newState['first-scene'] = true;
			markChecklistStep('first-scene');
			changed = true;
		}
		if (hasCharacter && !newState['first-character']) {
			newState['first-character'] = true;
			markChecklistStep('first-character');
			changed = true;
		}

		if (changed) {
			checklistState = newState;
			showChecklist = !isChecklistComplete();
		}
	});

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
	let isOverTarget = $derived(wordTarget > 0 && totalWords > wordTarget);
	let overshootPercent = $derived(
		wordTarget > 0 ? Math.round(((totalWords - wordTarget) / wordTarget) * 100) : 0
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

	// BC2: Chapter breakdown with word counts
	let chapterStats = $derived(appState.wordCounts?.by_chapter || []);
	let chapterStatsMap = $derived(new Map(chapterStats.map((c) => [c.chapter_id, c])));

	// BC2: Status percentages for visual bar
	let totalSceneCount = $derived(draftScenes + revisionScenes + doneScenes);
	let draftPercent = $derived(totalSceneCount > 0 ? (draftScenes / totalSceneCount) * 100 : 0);
	let revisionPercent = $derived(
		totalSceneCount > 0 ? (revisionScenes / totalSceneCount) * 100 : 0
	);
	let donePercent = $derived(totalSceneCount > 0 ? (doneScenes / totalSceneCount) * 100 : 0);

	function formatNumber(n: number): string {
		return n.toLocaleString();
	}

	function navigateToIssues() {
		appState.navigateTo('issue');
	}

	async function handleCreateFirstChapter() {
		await appState.createChapter('Chapter 1');
		appState.setViewMode('editor');
	}
</script>

<div class="dashboard">
	<div class="dashboard-header">
		<h2>{appState.project?.title || 'Project'} Dashboard</h2>
		{#if appState.project?.author}
			<span class="author">by {appState.project.author}</span>
		{/if}
	</div>

	<!-- UA2: Quick Start Writing Session -->
	{#if chapterCount > 0 && sceneCount > 0 && !appState.writingSessionActive}
		<div class="card quick-session-card">
			<div class="quick-session-header">
				<div>
					<h3>Start Writing Session</h3>
					<p class="session-description">
						Set a word goal and enter focus mode to minimize distractions.
					</p>
				</div>
				<!-- UA6: Jump to Last Scene -->
				{#if appState.lastEditedSceneId}
					<button class="jump-last-scene-btn" onclick={() => appState.jumpToLastScene()}>
						Continue Writing
					</button>
				{/if}
			</div>
			<div class="session-goals">
				<button class="session-goal-btn" onclick={() => appState.startWritingSession(250)}>
					<span class="goal-value">250</span>
					<span class="goal-label">words</span>
				</button>
				<button class="session-goal-btn" onclick={() => appState.startWritingSession(500)}>
					<span class="goal-value">500</span>
					<span class="goal-label">words</span>
				</button>
				<button class="session-goal-btn" onclick={() => appState.startWritingSession(1000)}>
					<span class="goal-value">1000</span>
					<span class="goal-label">words</span>
				</button>
				<button class="session-goal-btn custom" onclick={() => appState.startWritingSession()}>
					<span class="goal-value">Custom</span>
					<span class="goal-label">default: 500</span>
				</button>
			</div>
		</div>
	{/if}

	{#if chapterCount === 0 && sceneCount === 0}
		<div class="card wide-card">
			<EmptyState
				icon="book"
				title="Your project is empty"
				description="Create your first chapter to start writing. Chapters organize your scenes, and scenes hold your story."
				actionLabel="Create First Chapter"
				onaction={handleCreateFirstChapter}
			/>
		</div>
	{/if}

	<div class="dashboard-grid">
		<!-- Progress Overview Card -->
		<div class="card progress-card">
			<h3>Progress Overview</h3>
			<div class="stats-grid">
				<div class="stat-item primary" title="Total word count across all scenes">
					<span class="stat-value">{formatNumber(totalWords)}</span>
					<span class="stat-label">Total Words</span>
				</div>
				{#if wordTarget > 0}
					<div class="stat-item" title="Your manuscript word count goal">
						<span class="stat-value">{formatNumber(wordTarget)}</span>
						<span class="stat-label">Word Target</span>
					</div>
					<div class="stat-item" title="Percentage of word target reached">
						<span class="stat-value">{progressPercent}%</span>
						<span class="stat-label">Complete</span>
					</div>
				{/if}
				<div class="stat-item" title="Number of chapters in the project">
					<span class="stat-value">{chapterCount}</span>
					<span class="stat-label">Chapters</span>
				</div>
				<div class="stat-item" title="Number of scenes in the project">
					<span class="stat-value">{sceneCount}</span>
					<span class="stat-label">Scenes</span>
				</div>
				<div class="stat-item" title="Number of bible entries in the project">
					<span class="stat-value">{appState.bibleEntries.length}</span>
					<span class="stat-label">Bible Entries</span>
				</div>
				{#if sceneCount > 0}
					<div class="stat-item" title="Average words per scene">
						<span class="stat-value">{Math.round(totalWords / sceneCount)}</span>
						<span class="stat-label">Avg/Scene</span>
					</div>
				{/if}
			</div>

			{#if wordTarget > 0}
				<div class="progress-bar-container">
					<div class="progress-bar" class:overshoot={isOverTarget}>
						<div class="progress-fill" style="width: {progressPercent}%"></div>
					</div>
					<span class="progress-text">
						{progressPercent}% of target
						{#if isOverTarget}
							<span class="overshoot-text">+{overshootPercent}% over</span>
						{/if}
					</span>
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
				<LoadingState size="sm" />
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

		<!-- BC2: Manuscript Breakdown Card -->
		{#if appState.chapters.length > 0}
			<div class="card wide-card">
				<h3>Manuscript Breakdown</h3>
				<div class="breakdown-grid">
					{#each appState.chapters as chapter (chapter.id)}
						{@const stats = chapterStatsMap.get(chapter.id)}
						{@const chapterWordCount = stats?.word_count || 0}
						{@const chapterSceneCount = appState.scenes.get(chapter.id)?.length || 0}
						<div class="breakdown-row">
							<span class="chapter-title">{chapter.title}</span>
							<span class="chapter-words">{formatNumber(chapterWordCount)}</span>
							<span class="chapter-scenes"
								>{chapterSceneCount} scene{chapterSceneCount !== 1 ? 's' : ''}</span
							>
						</div>
					{/each}
				</div>

				<div class="status-breakdown">
					<div class="status-bar">
						{#if draftPercent > 0}
							<div
								class="bar-segment draft"
								style="width: {draftPercent}%"
								title="Draft: {draftScenes} scenes"
							></div>
						{/if}
						{#if revisionPercent > 0}
							<div
								class="bar-segment revision"
								style="width: {revisionPercent}%"
								title="Revision: {revisionScenes} scenes"
							></div>
						{/if}
						{#if donePercent > 0}
							<div
								class="bar-segment done"
								style="width: {donePercent}%"
								title="Done: {doneScenes} scenes"
							></div>
						{/if}
					</div>
					<div class="status-labels">
						<span class="status-label draft">Draft: {draftScenes}</span>
						<span class="status-label revision">Revision: {revisionScenes}</span>
						<span class="status-label done">Done: {doneScenes}</span>
					</div>
				</div>
			</div>
		{/if}

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
	</div>

	<!-- UA1: First Project Checklist -->
	{#if showChecklist && chapterCount > 0}
		<div class="checklist-card">
			<div class="checklist-header">
				<h3>Getting Started</h3>
				<button class="dismiss-checklist" onclick={() => (showChecklist = false)} title="Dismiss">
					<Icon name="close" size={14} />
				</button>
			</div>
			<div class="checklist-items">
				{#each firstProjectSteps as step (step.id)}
					{@const isComplete = checklistState[step.id]}
					<div class="checklist-item" class:complete={isComplete}>
						<span class="checklist-checkbox">
							{#if isComplete}
								<Icon name="check" size={12} />
							{/if}
						</span>
						<div class="checklist-content">
							<span class="checklist-title">{step.title}</span>
							{#if !isComplete}
								<span class="checklist-description">{step.description}</span>
							{/if}
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
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

	.progress-bar.overshoot .progress-fill {
		background-color: var(--color-success);
	}

	.overshoot-text {
		color: var(--color-success);
		font-weight: 500;
	}

	.progress-text {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		white-space: nowrap;
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
		padding: var(--spacing-xs);
		border-radius: var(--border-radius-sm);
	}

	.link-btn:hover {
		text-decoration: underline;
		background-color: var(--color-bg-hover);
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

	/* AZ11: Responsive layout for small screens */
	@media (max-width: 700px) {
		.dashboard-grid {
			grid-template-columns: 1fr;
		}

		.wide-card {
			grid-column: 1;
		}
	}

	@media (max-width: 500px) {
		.dashboard {
			padding: var(--spacing-sm);
		}

		.dashboard-header h2 {
			font-size: var(--font-size-lg);
		}

		.card {
			padding: var(--spacing-md);
		}

		.stats-grid {
			gap: var(--spacing-md);
		}

		.stat-item .stat-value {
			font-size: var(--font-size-lg);
		}
	}

	/* BC2: Manuscript breakdown */
	.breakdown-grid {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
		margin-bottom: var(--spacing-md);
	}

	.breakdown-row {
		display: grid;
		grid-template-columns: 1fr auto auto;
		gap: var(--spacing-md);
		padding: var(--spacing-xs) 0;
		border-bottom: 1px solid var(--color-border-light);
		align-items: center;
	}

	.breakdown-row:last-child {
		border-bottom: none;
	}

	.chapter-title {
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.chapter-words {
		font-size: var(--font-size-sm);
		font-weight: 600;
		color: var(--color-text-secondary);
		text-align: right;
	}

	.chapter-scenes {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-align: right;
		min-width: 60px;
	}

	.status-breakdown {
		margin-top: var(--spacing-md);
	}

	.status-bar {
		display: flex;
		height: 8px;
		border-radius: 4px;
		overflow: hidden;
		background-color: var(--color-bg-tertiary);
	}

	.bar-segment {
		transition: width 0.3s ease;
	}

	.bar-segment.draft {
		background-color: var(--color-text-muted);
	}

	.bar-segment.revision {
		background-color: var(--color-warning);
	}

	.bar-segment.done {
		background-color: var(--color-success);
	}

	.status-labels {
		display: flex;
		justify-content: space-between;
		margin-top: var(--spacing-xs);
	}

	.status-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.status-label.draft::before {
		content: '';
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: var(--color-text-muted);
		margin-right: var(--spacing-xs);
	}

	.status-label.revision::before {
		content: '';
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: var(--color-warning);
		margin-right: var(--spacing-xs);
	}

	.status-label.done::before {
		content: '';
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: var(--color-success);
		margin-right: var(--spacing-xs);
	}

	/* UA2: Quick Start Writing Session */
	.quick-session-card {
		grid-column: 1 / -1;
		background: linear-gradient(
			135deg,
			var(--color-accent-light) 0%,
			var(--color-bg-secondary) 100%
		);
	}

	.session-description {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		margin: 0 0 var(--spacing-md) 0;
	}

	.session-goals {
		display: flex;
		gap: var(--spacing-md);
		flex-wrap: wrap;
	}

	.session-goal-btn {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: var(--spacing-md) var(--spacing-lg);
		background-color: var(--color-bg-primary);
		border: 2px solid var(--color-border);
		border-radius: var(--border-radius-md);
		cursor: pointer;
		transition: all var(--transition-fast);
		min-width: 100px;
	}

	.session-goal-btn:hover {
		border-color: var(--color-accent);
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
	}

	.session-goal-btn .goal-value {
		font-size: var(--font-size-xl, 1.5rem);
		font-weight: 700;
		color: var(--color-accent);
	}

	.session-goal-btn .goal-label {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.session-goal-btn.custom {
		border-style: dashed;
	}

	.session-goal-btn.custom .goal-value {
		font-size: var(--font-size-md);
	}

	/* UA6: Jump to Last Scene */
	.quick-session-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: var(--spacing-md);
	}

	.quick-session-header > div {
		flex: 1;
	}

	.quick-session-header h3 {
		margin: 0;
	}

	.jump-last-scene-btn {
		padding: var(--spacing-sm) var(--spacing-md);
		background-color: var(--color-accent);
		color: var(--text-on-accent, #fff);
		border-radius: var(--border-radius-md);
		font-weight: 500;
		font-size: var(--font-size-sm);
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
	}

	.jump-last-scene-btn:hover {
		opacity: 0.9;
		transform: translateY(-1px);
	}

	/* UA1: First Project Checklist */
	.checklist-card {
		margin-top: var(--spacing-lg);
		padding: var(--spacing-md);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-lg);
	}

	.checklist-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: var(--spacing-md);
	}

	.checklist-header h3 {
		margin: 0;
		font-size: var(--font-size-base);
		font-weight: 600;
	}

	.dismiss-checklist {
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
		opacity: 0.6;
	}

	.dismiss-checklist:hover {
		opacity: 1;
		background-color: var(--color-bg-hover);
	}

	.checklist-items {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-sm);
	}

	.checklist-item {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-sm);
	}

	.checklist-checkbox {
		width: 18px;
		height: 18px;
		border: 2px solid var(--color-border);
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		margin-top: 2px;
	}

	.checklist-item.complete .checklist-checkbox {
		background-color: var(--color-success);
		border-color: var(--color-success);
		color: white;
	}

	.checklist-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.checklist-title {
		font-size: var(--font-size-sm);
		font-weight: 500;
	}

	.checklist-item.complete .checklist-title {
		color: var(--color-text-muted);
		text-decoration: line-through;
	}

	.checklist-description {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}
</style>
