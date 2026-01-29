<script lang="ts">
	import { open, save } from '@tauri-apps/plugin-dialog';
	import { onMount } from 'svelte';

	import { projectApi, type RecentProject } from '$lib/api';
	import { appState } from '$lib/stores';
	import { formatRelativeTime } from '$lib/utils';

	let recentProjects = $state<RecentProject[]>([]);
	let newProjectTitle = $state('');
	let newProjectAuthor = $state('');
	let showNewProjectForm = $state(false);

	const MAX_TITLE_LENGTH = 200;
	const MAX_AUTHOR_LENGTH = 100;

	// Sanitize input: remove newlines and control characters, limit length
	function sanitizeInput(value: string, maxLength: number): string {
		return (
			value
				// eslint-disable-next-line no-control-regex
				.replace(/[\r\n\x00-\x1F\x7F]/g, '') // Remove newlines and control characters
				.substring(0, maxLength)
		);
	}

	function handleTitleInput(event: Event) {
		const input = event.target as HTMLInputElement;
		newProjectTitle = sanitizeInput(input.value, MAX_TITLE_LENGTH);
		input.value = newProjectTitle; // Update the input to reflect sanitized value
	}

	function handleAuthorInput(event: Event) {
		const input = event.target as HTMLInputElement;
		newProjectAuthor = sanitizeInput(input.value, MAX_AUTHOR_LENGTH);
		input.value = newProjectAuthor;
	}

	// Use onMount for one-time initialization
	onMount(() => {
		projectApi
			.getRecent()
			.then((recent) => {
				recentProjects = recent;
			})
			.catch((e) => {
				console.error('Failed to load recent projects:', e);
			});
	});

	async function handleCreateProject() {
		if (!newProjectTitle.trim()) return;

		const path = await save({
			title: 'Create New Project',
			filters: [{ name: 'Cahnon Project', extensions: ['cahnon'] }],
			defaultPath: `${newProjectTitle.replace(/[^a-zA-Z0-9]/g, '_')}.cahnon`,
		});

		if (path) {
			try {
				await appState.createProject(
					path,
					newProjectTitle.trim(),
					newProjectAuthor.trim() || undefined
				);
			} catch (e) {
				appState.error = e instanceof Error ? e.message : String(e);
			}
		}
	}

	async function handleOpenProject() {
		const path = await open({
			title: 'Open Project',
			filters: [{ name: 'Cahnon Project', extensions: ['cahnon'] }],
			multiple: false,
		});

		if (path && typeof path === 'string') {
			try {
				await appState.loadProject(path);
			} catch (e) {
				appState.error = e instanceof Error ? e.message : String(e);
			}
		}
	}

	async function handleOpenDemo() {
		try {
			await appState.loadDemoProject();
		} catch (e) {
			appState.error = e instanceof Error ? e.message : String(e);
		}
	}

	async function openRecentProject(project: RecentProject) {
		try {
			await appState.loadProject(project.path);
		} catch (e) {
			const errorMsg = e instanceof Error ? e.message : String(e);
			appState.error = `Could not open "${project.title}" — ${errorMsg}`;
			// Remove from recent if file doesn't exist
			recentProjects = recentProjects.filter((p) => p.path !== project.path);
		}
	}
</script>

<div class="welcome">
	<div class="welcome-content">
		<header class="welcome-header">
			<h1>Cahnon</h1>
			<p class="tagline">Write freely. Stay consistent.</p>
		</header>

		{#if appState.error}
			<div class="error-message">
				{appState.error}
				<button onclick={() => (appState.error = null)}>Dismiss</button>
			</div>
		{/if}

		<div class="welcome-actions">
			{#if showNewProjectForm}
				<div class="new-project-form">
					<h2>New Project</h2>
					<div class="form-field">
						<label for="title">Title</label>
						<input
							id="title"
							type="text"
							placeholder="My Novel"
							value={newProjectTitle}
							oninput={handleTitleInput}
							onkeydown={(e) => e.key === 'Enter' && handleCreateProject()}
							maxlength={MAX_TITLE_LENGTH}
						/>
						<span class="char-count" class:warning={newProjectTitle.length > MAX_TITLE_LENGTH - 20}>
							{newProjectTitle.length}/{MAX_TITLE_LENGTH}
						</span>
					</div>
					<div class="form-field">
						<label for="author">Author (optional)</label>
						<input
							id="author"
							type="text"
							placeholder="Your name"
							value={newProjectAuthor}
							oninput={handleAuthorInput}
							maxlength={MAX_AUTHOR_LENGTH}
						/>
					</div>
					<div class="form-actions">
						<button class="btn-secondary" onclick={() => (showNewProjectForm = false)}>
							Cancel
						</button>
						<button
							class="btn-primary"
							onclick={handleCreateProject}
							disabled={!newProjectTitle.trim() || appState.isLoading}
						>
							Create Project
						</button>
					</div>
				</div>
			{:else}
				<button class="action-card new-project" onclick={() => (showNewProjectForm = true)}>
					<div class="action-icon">
						<svg
							width="32"
							height="32"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<line x1="12" y1="5" x2="12" y2="19" />
							<line x1="5" y1="12" x2="19" y2="12" />
						</svg>
					</div>
					<div class="action-text">
						<h3>New Project</h3>
						<p>Start a new writing project</p>
					</div>
				</button>

				<button class="action-card open-project" onclick={handleOpenProject}>
					<div class="action-icon">
						<svg
							width="32"
							height="32"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<path
								d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
							/>
						</svg>
					</div>
					<div class="action-text">
						<h3>Open Project</h3>
						<p>Open an existing .cahnon file</p>
					</div>
				</button>

				<button class="action-card try-demo" onclick={handleOpenDemo} disabled={appState.isLoading}>
					<div class="action-icon demo-icon">
						<svg
							width="32"
							height="32"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
						>
							<polygon points="5 3 19 12 5 21 5 3" />
						</svg>
					</div>
					<div class="action-text">
						<h3>Try Demo</h3>
						<p>Explore a sample project</p>
					</div>
				</button>
			{/if}
		</div>

		{#if recentProjects.length > 0 && !showNewProjectForm}
			<div class="recent-projects">
				<h2>Recent Projects</h2>
				<div class="recent-list">
					{#each recentProjects as project (project.path)}
						<button class="recent-item" onclick={() => openRecentProject(project)}>
							<div class="recent-icon">
								<svg
									width="24"
									height="24"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
								>
									<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
									<polyline points="14 2 14 8 20 8" />
								</svg>
							</div>
							<div class="recent-info">
								<span class="recent-title">{project.title}</span>
								<span class="recent-path truncate">{project.path}</span>
							</div>
							<span class="recent-time">{formatRelativeTime(project.last_opened)}</span>
						</button>
					{/each}
				</div>
			</div>
		{/if}

		<footer class="welcome-footer">
			<p>Cahnon is free and open source software under the GPL-3.0 license.</p>
		</footer>
	</div>
</div>

<style>
	.welcome {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 100vh;
		padding: var(--spacing-xl);
		background-color: var(--color-bg-secondary);
	}

	.welcome-content {
		width: 100%;
		max-width: 600px;
	}

	.welcome-header {
		text-align: center;
		margin-bottom: var(--spacing-xl);
	}

	.welcome-header h1 {
		font-size: 48px;
		font-weight: 700;
		color: var(--color-text-primary);
		margin-bottom: var(--spacing-sm);
	}

	.tagline {
		font-size: var(--font-size-lg);
		color: var(--color-text-muted);
	}

	.error-message {
		background-color: var(--color-error);
		color: var(--text-on-accent);
		padding: var(--spacing-md);
		border-radius: var(--border-radius-md);
		margin-bottom: var(--spacing-lg);
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.error-message button {
		color: var(--text-on-accent);
		text-decoration: underline;
	}

	.welcome-actions {
		display: flex;
		gap: var(--spacing-md);
		margin-bottom: var(--spacing-xl);
	}

	.action-card {
		flex: 1;
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		padding: var(--spacing-lg);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-lg);
		text-align: left;
		transition: all var(--transition-fast);
	}

	.action-card:hover {
		border-color: var(--color-accent);
		box-shadow: var(--shadow-md);
	}

	.action-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 56px;
		height: 56px;
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-md);
		color: var(--color-accent);
		flex-shrink: 0;
	}

	.demo-icon {
		background-color: var(--color-accent-light);
		color: var(--color-accent);
	}

	.action-text h3 {
		font-size: var(--font-size-md);
		font-weight: 600;
		color: var(--color-text-primary);
		margin-bottom: var(--spacing-xs);
	}

	.action-text p {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	.new-project-form {
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-lg);
		padding: var(--spacing-lg);
		width: 100%;
	}

	.new-project-form h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
		margin-bottom: var(--spacing-lg);
	}

	.form-field {
		margin-bottom: var(--spacing-md);
	}

	.form-field label {
		display: block;
		font-size: var(--font-size-sm);
		font-weight: 500;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-xs);
	}

	.form-field input {
		width: 100%;
		padding: var(--spacing-sm) var(--spacing-md);
		font-size: var(--font-size-md);
	}

	.char-count {
		display: block;
		text-align: right;
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		margin-top: var(--spacing-xs);
	}

	.char-count.warning {
		color: var(--color-text-primary);
	}

	.form-actions {
		display: flex;
		gap: var(--spacing-md);
		justify-content: flex-end;
		margin-top: var(--spacing-lg);
	}

	.btn-secondary,
	.btn-primary {
		padding: var(--spacing-sm) var(--spacing-lg);
		font-size: var(--font-size-sm);
		font-weight: 500;
		border-radius: var(--border-radius-md);
		transition: all var(--transition-fast);
	}

	.btn-secondary {
		color: var(--color-text-secondary);
		background-color: var(--color-bg-tertiary);
	}

	.btn-secondary:hover {
		background-color: var(--color-bg-hover);
	}

	.btn-primary {
		background-color: var(--color-accent);
		color: var(--text-on-accent);
	}

	.btn-primary:hover {
		background-color: var(--color-accent-hover);
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.recent-projects h2 {
		font-size: var(--font-size-sm);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--color-text-muted);
		margin-bottom: var(--spacing-md);
	}

	.recent-list {
		display: flex;
		flex-direction: column;
		gap: var(--spacing-xs);
	}

	.recent-item {
		display: flex;
		align-items: center;
		gap: var(--spacing-md);
		padding: var(--spacing-md);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		text-align: left;
		transition: all var(--transition-fast);
	}

	.recent-item:hover {
		border-color: var(--color-accent);
		background-color: var(--color-accent-light);
	}

	.recent-icon {
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.recent-info {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.recent-title {
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.recent-path {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
	}

	.recent-time {
		font-size: var(--font-size-xs);
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.welcome-footer {
		text-align: center;
		margin-top: var(--spacing-xl);
	}

	.welcome-footer p {
		font-size: var(--font-size-sm);
		color: var(--color-text-muted);
	}

	/* Responsive adjustments */
	@media (max-width: 900px) {
		.welcome-content {
			max-width: 90%;
		}
		.welcome-actions {
			flex-wrap: wrap;
			justify-content: center;
		}
	}
</style>
