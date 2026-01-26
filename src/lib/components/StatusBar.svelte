<script lang="ts">
	import { appState } from '$lib/stores';
	import { countWords, formatWordCount } from '$lib/utils';

	// Session tracking
	let sessionStartWordCount = $state(0);
	let sessionInitialized = $state(false);

	// Today's tracking
	const TODAY_KEY = 'cahnon_today_progress';

	interface TodayProgress {
		date: string;
		startWordCount: number;
	}

	function getTodayKey(): string {
		return new Date().toISOString().split('T')[0];
	}

	function initTodayProgress(currentTotal: number) {
		try {
			const saved = localStorage.getItem(TODAY_KEY);
			if (saved) {
				const progress: TodayProgress = JSON.parse(saved);
				if (progress.date === getTodayKey()) {
					return progress.startWordCount;
				}
			}
			// New day or no data
			const newProgress: TodayProgress = {
				date: getTodayKey(),
				startWordCount: currentTotal,
			};
			localStorage.setItem(TODAY_KEY, JSON.stringify(newProgress));
			return currentTotal;
		} catch {
			return currentTotal;
		}
	}

	let todayStartWordCount = $state(0);

	// Initialize session and today counts when wordCounts becomes available
	$effect(() => {
		if (appState.wordCounts && !sessionInitialized) {
			sessionStartWordCount = appState.wordCounts.total;
			todayStartWordCount = initTodayProgress(appState.wordCounts.total);
			sessionInitialized = true;
		}
	});

	let sessionWordCount = $derived(
		appState.wordCounts ? Math.max(0, appState.wordCounts.total - sessionStartWordCount) : 0
	);
	let todayWordCount = $derived(
		appState.wordCounts ? Math.max(0, appState.wordCounts.total - todayStartWordCount) : 0
	);
	let progressPercent = $derived(
		appState.project?.word_target && appState.wordCounts
			? Math.min(100, Math.round((appState.wordCounts.total / appState.project.word_target) * 100))
			: 0
	);
	let dailyProgressPercent = $derived(
		appState.project?.daily_word_target && todayWordCount
			? Math.min(100, Math.round((todayWordCount / appState.project.daily_word_target) * 100))
			: 0
	);
</script>

<footer class="status-bar">
	<div class="status-left">
		{#if appState.hasUnsavedChanges}
			<span class="save-status unsaved">Unsaved</span>
		{:else}
			<span class="save-status saved">Saved</span>
		{/if}

		<span class="separator">|</span>

		<span class="mode-indicator" class:revision={appState.workMode === 'revision'}>
			{appState.workMode === 'writing' ? 'Writing Mode' : 'Revision Mode'}
		</span>
	</div>

	<div class="status-center">
		{#if appState.selectedScene}
			<span class="scene-info">
				{countWords(appState.selectedScene.text)} words in scene
			</span>
		{/if}
	</div>

	<div class="status-right">
		{#if sessionWordCount > 0}
			<span class="word-stat session">
				<span class="label">Session:</span>
				<span class="value">+{formatWordCount(sessionWordCount)}</span>
			</span>
			<span class="separator">|</span>
		{/if}

		{#if todayWordCount > 0 || appState.project?.daily_word_target}
			<span class="word-stat today">
				<span class="label">Today:</span>
				<span class="value">+{formatWordCount(todayWordCount)}</span>
				{#if appState.project?.daily_word_target}
					<span
						class="daily-progress"
						title="{todayWordCount} / {appState.project.daily_word_target} words"
					>
						<span class="mini-progress-bar">
							<span class="mini-progress-fill" style="width: {dailyProgressPercent}%"></span>
						</span>
						<span class="daily-percent">{dailyProgressPercent}%</span>
					</span>
				{/if}
			</span>
			<span class="separator">|</span>
		{/if}

		{#if appState.wordCounts}
			<span class="word-stat">
				<span class="label">Total:</span>
				<span class="value">{formatWordCount(appState.wordCounts.total)}</span>
			</span>
		{/if}

		{#if appState.project?.word_target}
			<span class="separator">|</span>
			<span
				class="progress-indicator"
				title="{appState.wordCounts?.total || 0} / {appState.project.word_target} words"
			>
				<span class="progress-bar">
					<span class="progress-fill" style="width: {progressPercent}%"></span>
				</span>
				<span class="progress-text">{progressPercent}%</span>
			</span>
		{/if}
	</div>
</footer>

<style>
	.status-bar {
		height: var(--statusbar-height);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 var(--spacing-md);
		background-color: var(--color-bg-tertiary);
		border-top: 1px solid var(--color-border);
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.status-left,
	.status-center,
	.status-right {
		display: flex;
		align-items: center;
		gap: var(--spacing-sm);
	}

	.status-left {
		flex: 1;
	}

	.status-center {
		flex: 1;
		justify-content: center;
	}

	.status-right {
		flex: 1;
		justify-content: flex-end;
	}

	.separator {
		color: var(--color-border);
	}

	.save-status {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
	}

	.save-status.saved {
		color: var(--color-success);
	}

	.save-status.unsaved {
		color: var(--color-warning);
	}

	.mode-indicator {
		padding: 2px var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
	}

	.mode-indicator.revision {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}

	.word-stat {
		display: flex;
		gap: var(--spacing-xs);
	}

	.word-stat .label {
		color: var(--color-text-muted);
	}

	.word-stat .value {
		font-weight: 500;
		color: var(--color-text-secondary);
	}

	.word-stat.session .value {
		color: var(--color-accent);
	}

	.word-stat.today .value {
		color: var(--color-success);
	}

	.daily-progress {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		margin-left: var(--spacing-xs);
	}

	.mini-progress-bar {
		width: 40px;
		height: 4px;
		background-color: var(--color-bg-secondary);
		border-radius: 2px;
		overflow: hidden;
	}

	.mini-progress-fill {
		height: 100%;
		background-color: var(--color-success);
		border-radius: 2px;
		transition: width var(--transition-normal);
	}

	.daily-percent {
		font-size: var(--font-size-xs);
		color: var(--color-success);
		min-width: 28px;
	}

	.progress-indicator {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
	}

	.progress-bar {
		width: 60px;
		height: 6px;
		background-color: var(--color-bg-secondary);
		border-radius: 3px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: var(--color-accent);
		border-radius: 3px;
		transition: width var(--transition-normal);
	}

	.progress-text {
		font-weight: 500;
		color: var(--color-text-secondary);
		min-width: 35px;
	}
</style>
