<!--
  Onboarding welcome card for new/empty projects.

  Shown in the editor area when the project has 0 chapters.
  Guides the user through their first steps: create a chapter, add scenes, start writing.
  BB1: Progressive flow - advances through steps as user completes them.
  Dismissable via localStorage. Not shown for demo projects.
-->
<script lang="ts">
	import { tick, untrack } from 'svelte';

	import { appState } from '$lib/stores';
	import { showSuccess } from '$lib/toast';

	import { Button, Icon } from './ui';

	function getDismissedKey(): string {
		const path = appState.projectPath || 'default';
		return `cahnon-onboarding-dismissed:${path}`;
	}

	let dismissed = $state(false);
	// BB1: Track current onboarding step (1 = create chapter, 2 = add scene, 3 = start writing)
	let currentStep = $state(1);

	// Check localStorage on init (re-check when project changes)
	$effect(() => {
		const key = getDismissedKey();
		try {
			dismissed = localStorage.getItem(key) === 'true';
		} catch {
			dismissed = false;
		}
	});

	// BB1: Auto-advance steps based on project state
	$effect(() => {
		// Track data dependencies
		const chapterCount = appState.chapters.length;
		const hasScenes = Array.from(appState.scenes.values()).some((scenes) => scenes.length > 0);

		// Read currentStep without tracking to avoid infinite loop
		const step = untrack(() => currentStep);

		if (chapterCount > 0 && step === 1) {
			currentStep = 2;
		} else if (hasScenes && step === 2) {
			currentStep = 3;
		}
	});

	function dismiss() {
		dismissed = true;
		try {
			localStorage.setItem(getDismissedKey(), 'true');
		} catch {
			// localStorage unavailable
		}
	}

	async function handleCreateChapter() {
		await appState.createChapter('Chapter 1');
		showSuccess('Chapter created! Now add your first scene.');
		currentStep = 2;
	}

	async function handleCreateScene() {
		if (appState.chapters.length === 0) return;
		const chapterId = appState.chapters[0].id;
		await appState.createScene(chapterId, 'Scene 1');
		showSuccess('Scene created! Click below to start writing.');
		currentStep = 3;
	}

	function handleStartWriting() {
		dismiss();
		// Focus in the editor
		tick().then(() => {
			const editor = document.querySelector('.ProseMirror') as HTMLElement | null;
			if (editor) {
				editor.focus();
			}
		});
	}

	let isVisible = $derived(
		(!dismissed && !appState.isDemo && appState.chapters.length === 0) ||
			(!dismissed && !appState.isDemo && currentStep > 1 && currentStep <= 3)
	);
</script>

{#if isVisible}
	<div class="welcome-card">
		<button class="dismiss-btn" onclick={dismiss} aria-label="Dismiss welcome guide">
			<Icon name="close" size={16} />
		</button>

		<div class="welcome-header">
			<Icon name="book" size={36} strokeWidth={1.5} />
			<h2>Welcome to your new project</h2>
			<p class="subtitle">Here's how to get started in three steps.</p>
		</div>

		<!-- BB1: Progress indicator -->
		<div class="progress-bar">
			<div class="progress-fill" style="width: {((currentStep - 1) / 3) * 100}%"></div>
		</div>

		<ol class="steps">
			<li class="step" class:completed={currentStep > 1} class:active={currentStep === 1}>
				<span class="step-number">{currentStep > 1 ? '✓' : '1'}</span>
				<div class="step-content">
					<strong>Create a chapter</strong>
					<span class="step-desc">Chapters organize your manuscript into sections.</span>
				</div>
			</li>
			<li class="step" class:completed={currentStep > 2} class:active={currentStep === 2}>
				<span class="step-number">{currentStep > 2 ? '✓' : '2'}</span>
				<div class="step-content">
					<strong>Add scenes</strong>
					<span class="step-desc"
						>Each chapter contains scenes — the building blocks of your story.</span
					>
				</div>
			</li>
			<li class="step" class:active={currentStep === 3}>
				<span class="step-number">3</span>
				<div class="step-content">
					<strong>Start writing</strong>
					<span class="step-desc">Select a scene to open the editor and begin drafting.</span>
				</div>
			</li>
		</ol>

		<div class="welcome-action">
			{#if currentStep === 1}
				<Button variant="primary" size="lg" onclick={handleCreateChapter}>
					Create First Chapter
				</Button>
			{:else if currentStep === 2}
				<Button variant="primary" size="lg" onclick={handleCreateScene}>Add First Scene</Button>
			{:else}
				<Button variant="primary" size="lg" onclick={handleStartWriting}>Start Writing</Button>
			{/if}
		</div>
	</div>
{/if}

<style>
	.welcome-card {
		position: relative;
		max-width: 480px;
		margin: 0 auto;
		padding: var(--spacing-xl);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-lg);
		box-shadow: var(--shadow-sm);
	}

	.dismiss-btn {
		position: absolute;
		top: var(--spacing-md);
		right: var(--spacing-md);
		padding: var(--spacing-xs);
		color: var(--color-text-muted);
		border-radius: var(--border-radius-sm);
	}

	.dismiss-btn:hover {
		background-color: var(--color-bg-hover);
		color: var(--color-text-primary);
	}

	.welcome-header {
		text-align: center;
		margin-bottom: var(--spacing-lg);
		color: var(--color-text-muted);
	}

	.welcome-header h2 {
		margin: var(--spacing-sm) 0 var(--spacing-xs);
		font-size: var(--font-size-xl, 1.5rem);
		color: var(--color-text-primary);
	}

	.subtitle {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	/* BB1: Progress bar */
	.progress-bar {
		height: 4px;
		background-color: var(--color-bg-tertiary);
		border-radius: 2px;
		margin-bottom: var(--spacing-lg);
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: var(--color-accent);
		border-radius: 2px;
		transition: width 0.3s ease-out;
	}

	.steps {
		list-style: none;
		padding: 0;
		margin: 0 0 var(--spacing-lg);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.step {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-md);
	}

	.step-number {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: 50%;
		background-color: var(--color-bg-tertiary);
		color: var(--color-text-muted);
		font-size: var(--font-size-sm);
		font-weight: 600;
		flex-shrink: 0;
		transition: all 0.2s ease-out;
	}

	/* BB1: Active step styling */
	.step.active .step-number {
		background-color: var(--color-accent);
		color: var(--text-on-accent, #fff);
	}

	/* BB1: Completed step styling */
	.step.completed .step-number {
		background-color: var(--color-success);
		color: var(--text-on-accent, #fff);
	}

	.step.completed .step-content strong {
		color: var(--color-text-muted);
	}

	.step.completed .step-desc {
		color: var(--color-text-muted);
	}

	.step-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding-top: 3px;
	}

	.step-content strong {
		font-size: var(--font-size-base);
		color: var(--color-text-primary);
	}

	.step-desc {
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
	}

	.welcome-action {
		text-align: center;
	}
</style>
