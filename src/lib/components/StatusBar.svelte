<script lang="ts">
	import { appState } from '$lib/stores';
	import { countWords, formatWordCount } from '$lib/utils';

	import { Icon } from './ui';

	// Session tracking (in-memory only, resets on app restart)
	let sessionStartWordCount = $state(0);
	let sessionInitialized = $state(false);

	// Initialize session count when wordCounts becomes available
	$effect(() => {
		if (appState.wordCounts && !sessionInitialized) {
			sessionStartWordCount = appState.wordCounts.total;
			sessionInitialized = true;
		}
	});

	let sessionWordCount = $derived(
		appState.wordCounts ? Math.max(0, appState.wordCounts.total - sessionStartWordCount) : 0
	);
	let progressPercent = $derived(
		appState.project?.word_target && appState.wordCounts
			? Math.min(100, Math.round((appState.wordCounts.total / appState.project.word_target) * 100))
			: 0
	);

	// Session goal progress coloring
	let sessionGoal = $derived(appState.project?.daily_word_target ?? 0);
	let sessionGoalPercent = $derived(
		sessionGoal > 0 ? Math.min(100, Math.round((sessionWordCount / sessionGoal) * 100)) : 0
	);
	let sessionGoalClass = $derived(
		sessionGoal > 0
			? sessionGoalPercent >= 100
				? 'goal-complete'
				: sessionGoalPercent >= 50
					? 'goal-progress'
					: ''
			: ''
	);

	// AF1: Calculate unsaved duration for display
	let unsavedDuration = $derived.by(() => {
		if (!appState.unsavedSince) return '';
		const seconds = Math.floor((Date.now() - appState.unsavedSince.getTime()) / 1000);
		if (seconds < 60) return '';
		const minutes = Math.floor(seconds / 60);
		return `(${minutes}m)`;
	});

	// AO6: Only show "stale" warning after 30 seconds of unsaved changes
	let isStale = $derived.by(() => {
		if (!appState.unsavedSince) return false;
		return Date.now() - appState.unsavedSince.getTime() > 30000;
	});

	// AF7: Compute screen reader announcement
	let saveStatusAnnouncement = $derived.by(() => {
		if (appState.isSaving) return 'Saving your work';
		if (appState.saveFailed) return 'Save failed. Click to retry.';
		if (appState.isTyping) return 'Typing...';
		if (appState.hasUnsavedChanges) return 'You have unsaved changes';
		return 'All changes saved';
	});

	// BA3: Track if session word count just increased for flash animation
	let prevSessionWordCount = 0; // Non-reactive to avoid effect loops
	let sessionJustIncreased = $state(false);

	$effect(() => {
		const current = sessionWordCount;
		if (current > prevSessionWordCount && prevSessionWordCount > 0) {
			sessionJustIncreased = true;
			const timer = setTimeout(() => {
				sessionJustIncreased = false;
			}, 400);
			prevSessionWordCount = current;
			return () => clearTimeout(timer);
		}
		prevSessionWordCount = current;
		return undefined;
	});
</script>

<footer class="status-bar">
	<!-- AF7: aria-live region for screen reader announcements -->
	<div class="save-status-container" aria-live="polite" aria-atomic="true">
		<span class="sr-only">{saveStatusAnnouncement}</span>
	</div>

	<div class="status-left">
		<!-- CA7: Working indicator for pending operations -->
		{#if appState.isWorking}
			<span class="save-status working">
				<span class="save-spinner"></span>
				<span>Working...</span>
			</span>
			<span class="separator">|</span>
		{/if}
		{#if appState.isSaving}
			<!-- AF6: Icon + text for colorblind accessibility -->
			<span class="save-status saving">
				<span class="save-spinner"></span>
				<span>Saving...</span>
			</span>
		{:else if appState.saveFailed}
			<!-- AF6, AO2: Error icon + text with retry and backup options -->
			<div class="save-status save-failed">
				<Icon name="close" size={12} />
				<span>Save failed</span>
				<button
					class="save-action-btn"
					onclick={() => appState.retrySave()}
					title="Try saving again"
				>
					Retry
				</button>
				<button
					class="save-action-btn backup-btn"
					onclick={() => appState.emergencyExport()}
					title="Download current scene as backup"
				>
					Export backup
				</button>
			</div>
		{:else if appState.isTyping}
			<!-- BA1: Typing indicator with animated dots -->
			<span class="save-status typing">
				<span class="typing-indicator">
					<span class="typing-dot"></span>
					<span class="typing-dot"></span>
					<span class="typing-dot"></span>
				</span>
				<span>Typing</span>
			</span>
		{:else if appState.hasUnsavedChanges}
			<!-- AF1, AF6, AO6: More subtle unsaved indicator - only alert when stale -->
			<span class="save-status unsaved" class:stale={isStale}>
				{#if isStale}
					<Icon name="alert-triangle" size={12} />
					<span>Unsaved {unsavedDuration}</span>
				{:else}
					<span class="editing-dot"></span>
					<span>Editing</span>
				{/if}
			</span>
		{:else}
			<!-- AF5, AF6: Checkmark icon + text with flash animation -->
			<span
				class="save-status saved"
				class:just-saved={appState.justSaved}
				title="Autosave runs every 10 seconds"
			>
				<Icon name="check" size={12} />
				<span>
					{#if appState.lastSavedAt}
						Saved at {appState.lastSavedAt.toLocaleTimeString([], {
							hour: '2-digit',
							minute: '2-digit',
						})}
					{:else}
						Saved
					{/if}
				</span>
			</span>
		{/if}

		<!-- UB5: Auto-backup indicator -->
		{#if appState.lastSavedAt && !appState.hasUnsavedChanges && !appState.isSaving}
			<span
				class="auto-backup-indicator"
				title="Auto-saved at {appState.lastSavedAt.toLocaleTimeString()}"
			>
				<Icon name="save" size={11} />
			</span>
		{/if}

		<span class="separator">|</span>

		<!-- AX1: Standardized vocabulary - Writing/Revising instead of Drafting -->
		<span class="mode-indicator" class:revision={appState.workMode === 'revision'}>
			{appState.workMode === 'writing' ? 'Writing' : 'Revising'}
		</span>
	</div>

	<div class="status-center">
		{#if appState.statusMessage}
			<span class="status-message status-{appState.statusMessage.type}">
				{appState.statusMessage.text}
			</span>
		{:else if appState.selectedScene}
			{@const sceneWords = countWords(appState.selectedScene.text)}
			<span class="scene-info">
				{sceneWords} words in scene (~{Math.max(1, Math.ceil(sceneWords / 250))} min)
			</span>
		{/if}
	</div>

	<div class="status-right">
		<!-- AW5: Always show session counter, even when 0 -->
		<!-- BA3: Flash animation when session words increase -->
		<span
			class="word-stat session {sessionGoalClass}"
			class:just-saved={sessionJustIncreased && appState.justSaved}
		>
			<span class="label">Session:</span>
			<span class="value">{sessionWordCount > 0 ? '+' : ''}{formatWordCount(sessionWordCount)}</span
			>
		</span>
		<span class="separator">|</span>

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
		position: relative;
	}

	/* AF7: Hidden container for screen reader announcements */
	.save-status-container {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
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

	/* AO6: More subtle unsaved state - only warn when stale */
	.save-status.unsaved {
		color: var(--color-text-muted);
	}

	.save-status.unsaved.stale {
		color: var(--color-warning);
	}

	/* AO6: Subtle editing indicator */
	/* AV5, AZ1: More visible pulse animation with scale + opacity */
	.editing-dot {
		display: inline-block;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background-color: var(--color-accent);
		animation: editing-pulse 1.5s ease-in-out infinite;
	}

	@keyframes editing-pulse {
		0%,
		100% {
			opacity: 0.5;
			transform: scale(1);
		}
		50% {
			opacity: 1;
			transform: scale(1.2);
		}
	}

	.save-status.saving {
		color: var(--color-text-muted);
	}

	/* CA7: Working indicator */
	.save-status.working {
		color: var(--color-accent);
	}

	.save-spinner {
		display: inline-block;
		width: var(--spinner-size-sm);
		height: var(--spinner-size-sm);
		border: var(--spinner-width) solid var(--color-text-muted);
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin var(--spinner-speed) linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.save-dot {
		display: inline-block;
		width: 6px;
		height: 6px;
		border-radius: 50%;
	}

	/* AA5: More visible pulse animation for unsaved indicator */
	.unsaved-dot {
		background-color: var(--color-warning);
		animation: pulse-urgent 1.5s ease-in-out infinite;
	}

	@keyframes pulse-urgent {
		0%,
		100% {
			transform: scale(1);
			opacity: 0.7;
		}
		50% {
			transform: scale(1.4);
			opacity: 1;
		}
	}

	/* BA1: Typing indicator with animated dots */
	.save-status.typing {
		color: var(--color-accent);
	}

	.typing-indicator {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.typing-dot {
		width: 4px;
		height: 4px;
		border-radius: 50%;
		background-color: var(--color-accent);
		animation: typing-bounce 1.4s ease-in-out infinite;
	}

	.typing-dot:nth-child(1) {
		animation-delay: 0s;
	}

	.typing-dot:nth-child(2) {
		animation-delay: 0.2s;
	}

	.typing-dot:nth-child(3) {
		animation-delay: 0.4s;
	}

	@keyframes typing-bounce {
		0%,
		80%,
		100% {
			transform: scale(1);
			opacity: 0.5;
		}
		40% {
			transform: scale(1.3);
			opacity: 1;
		}
	}

	/* AF5, BA3: Amplified flash animation when save completes */
	.save-status.saved.just-saved {
		animation:
			save-flash 0.5s ease-out,
			save-pulse 0.3s ease-out;
	}

	@keyframes save-flash {
		0% {
			background-color: var(--color-success-light, rgba(34, 197, 94, 0.2));
			border-radius: var(--border-radius-sm);
		}
		100% {
			background-color: transparent;
		}
	}

	@keyframes save-pulse {
		0% {
			transform: scale(1);
		}
		50% {
			transform: scale(1.05);
		}
		100% {
			transform: scale(1);
		}
	}

	/* BA3: Flash on session word count when saving */
	.word-stat.session.just-saved .value {
		animation: count-flash 0.4s ease-out;
	}

	@keyframes count-flash {
		0%,
		100% {
			color: inherit;
		}
		50% {
			color: var(--color-success);
		}
	}

	.save-status.save-failed {
		color: var(--color-error);
		font-weight: 500;
		border-radius: var(--border-radius-sm);
		padding: 2px var(--spacing-xs);
	}

	/* AO2: Save action buttons for retry and backup */
	.save-action-btn {
		font-size: var(--font-size-xs);
		padding: 2px var(--spacing-sm);
		border-radius: var(--border-radius-sm);
		background-color: var(--color-bg-secondary);
		color: var(--color-text-secondary);
		border: 1px solid var(--color-border);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.save-action-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.save-action-btn.backup-btn {
		background-color: var(--color-error-light, rgba(239, 68, 68, 0.1));
		border-color: var(--color-error);
		color: var(--color-error);
	}

	.save-action-btn.backup-btn:hover {
		background-color: var(--color-error);
		color: white;
	}

	.failed-dot {
		background-color: var(--color-error);
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

	.word-stat.session.goal-progress .value {
		color: var(--color-accent);
		font-weight: 600;
	}

	.word-stat.session.goal-complete .value {
		color: var(--color-success);
		font-weight: 600;
		animation: goal-pulse 2s ease-in-out 3;
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

	.status-message {
		font-weight: 500;
	}

	.status-message.status-success {
		color: var(--color-success);
	}

	.status-message.status-warning {
		color: var(--color-warning);
	}

	.status-message.status-info {
		color: var(--color-text-secondary);
	}

	/* UB5: Auto-backup indicator */
	.auto-backup-indicator {
		display: flex;
		align-items: center;
		color: var(--color-success);
		opacity: 0.7;
	}

	.auto-backup-indicator:hover {
		opacity: 1;
	}
</style>
