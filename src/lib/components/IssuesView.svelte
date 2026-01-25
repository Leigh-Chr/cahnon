<script lang="ts">
	import { onMount } from 'svelte';
	import { issueApi, type Issue } from '$lib/api';

	let issues = $state<Issue[]>([]);
	let isLoading = $state(true);
	let showNewIssueForm = $state(false);
	let filterStatus = $state('all');

	let newIssueTitle = $state('');
	let newIssueDescription = $state('');
	let newIssueType = $state('continuity');
	let newIssueSeverity = $state('medium');

	const issueTypes = ['continuity', 'plot_hole', 'character', 'timeline', 'worldbuilding', 'other'];
	const issueSeverities = ['low', 'medium', 'high', 'critical'];
	const issueStatuses = ['open', 'in_progress', 'resolved', 'wont_fix'];

	// Use onMount for one-time initialization
	onMount(() => {
		loadIssues();
	});

	async function loadIssues() {
		isLoading = true;
		try {
			issues = await issueApi.getAll(filterStatus === 'all' ? undefined : filterStatus);
		} catch (e) {
			console.error('Failed to load issues:', e);
		}
		isLoading = false;
	}

	let filteredIssues = $derived(
		filterStatus === 'all' ? issues : issues.filter((i) => i.status === filterStatus)
	);

	async function createIssue() {
		if (!newIssueTitle.trim()) return;
		try {
			const issue = await issueApi.create({
				issue_type: newIssueType,
				title: newIssueTitle.trim(),
				description: newIssueDescription.trim() || undefined,
				severity: newIssueSeverity,
			});
			issues = [issue, ...issues];
			resetNewIssueForm();
		} catch (e) {
			console.error('Failed to create issue:', e);
		}
	}

	function resetNewIssueForm() {
		showNewIssueForm = false;
		newIssueTitle = '';
		newIssueDescription = '';
		newIssueType = 'continuity';
		newIssueSeverity = 'medium';
	}

	async function updateIssueStatus(issue: Issue, status: string) {
		try {
			const updated = await issueApi.update(issue.id, { status });
			issues = issues.map((i) => (i.id === updated.id ? updated : i));
		} catch (e) {
			console.error('Failed to update issue:', e);
		}
	}

	function getSeverityColor(severity: string): string {
		const colors: Record<string, string> = {
			low: 'var(--color-text-muted)',
			medium: 'var(--color-text-secondary)',
			high: 'var(--color-text-primary)',
			critical: 'var(--color-text-primary)',
		};
		return colors[severity] || 'var(--color-text-muted)';
	}

	function getStatusColor(status: string): string {
		const colors: Record<string, string> = {
			open: 'var(--color-text-primary)',
			in_progress: 'var(--color-text-secondary)',
			resolved: 'var(--color-text-muted)',
			wont_fix: 'var(--color-border)',
		};
		return colors[status] || 'var(--color-text-muted)';
	}

	function getTypeIcon(type: string): string {
		const icons: Record<string, string> = {
			continuity:
				'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z',
			plot_hole:
				'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z',
			character:
				'M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z',
			timeline:
				'M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z',
			worldbuilding:
				'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z',
			other:
				'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 17h-2v-2h2v2zm2.07-7.75l-.9.92C13.45 12.9 13 13.5 13 15h-2v-.5c0-1.1.45-2.1 1.17-2.83l1.24-1.26c.37-.36.59-.86.59-1.41 0-1.1-.9-2-2-2s-2 .9-2 2H8c0-2.21 1.79-4 4-4s4 1.79 4 4c0 .88-.36 1.68-.93 2.25z',
		};
		return icons[type] || icons['other'];
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString();
	}
</script>

<div class="issues-view">
	<div class="issues-header">
		<h2>Issues</h2>
		<div class="header-actions">
			<select bind:value={filterStatus} onchange={loadIssues} class="filter-select">
				<option value="all">All Issues</option>
				<option value="open">Open</option>
				<option value="in_progress">In Progress</option>
				<option value="resolved">Resolved</option>
				<option value="wont_fix">Won't Fix</option>
			</select>
			<button class="add-btn" onclick={() => (showNewIssueForm = true)}>
				<svg
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<line x1="12" y1="5" x2="12" y2="19" />
					<line x1="5" y1="12" x2="19" y2="12" />
				</svg>
				New Issue
			</button>
		</div>
	</div>

	{#if isLoading}
		<div class="loading">Loading issues...</div>
	{:else if filteredIssues.length === 0 && !showNewIssueForm}
		<div class="empty-state">
			<svg
				width="48"
				height="48"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.5"
			>
				<circle cx="12" cy="12" r="10" />
				<line x1="12" y1="8" x2="12" y2="12" />
				<line x1="12" y1="16" x2="12.01" y2="16" />
			</svg>
			<h3>No issues found</h3>
			<p>Track continuity errors, plot holes, and other problems to fix.</p>
			<button class="primary-btn" onclick={() => (showNewIssueForm = true)}
				>Report First Issue</button
			>
		</div>
	{:else}
		<div class="issues-list">
			{#if showNewIssueForm}
				<div class="issue-card new-issue-form">
					<div class="form-row">
						<div class="form-group flex-1">
							<label for="issue-title">Title</label>
							<!-- svelte-ignore a11y_autofocus -->
							<input
								id="issue-title"
								type="text"
								bind:value={newIssueTitle}
								placeholder="Brief description of the issue..."
								autofocus
							/>
						</div>
					</div>
					<div class="form-group">
						<label for="issue-description">Description</label>
						<textarea
							id="issue-description"
							bind:value={newIssueDescription}
							placeholder="Detailed explanation..."
							rows="3"
						></textarea>
					</div>
					<div class="form-row">
						<div class="form-group">
							<label for="issue-type">Type</label>
							<select id="issue-type" bind:value={newIssueType}>
								{#each issueTypes as type (type)}
									<option value={type}>{type.replace('_', ' ')}</option>
								{/each}
							</select>
						</div>
						<div class="form-group">
							<label for="issue-severity">Severity</label>
							<select id="issue-severity" bind:value={newIssueSeverity}>
								{#each issueSeverities as severity (severity)}
									<option value={severity}>{severity}</option>
								{/each}
							</select>
						</div>
					</div>
					<div class="form-actions">
						<button class="cancel-btn" onclick={resetNewIssueForm}>Cancel</button>
						<button class="save-btn" onclick={createIssue} disabled={!newIssueTitle.trim()}
							>Create Issue</button
						>
					</div>
				</div>
			{/if}

			{#each filteredIssues as issue (issue.id)}
				<div
					class="issue-card"
					class:resolved={issue.status === 'resolved' || issue.status === 'wont_fix'}
				>
					<div class="issue-header">
						<div class="issue-type" title={issue.issue_type}>
							<svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
								<path d={getTypeIcon(issue.issue_type)} />
							</svg>
						</div>
						<div class="issue-info">
							<h3>{issue.title}</h3>
							<div class="issue-meta">
								<span class="severity" style="color: {getSeverityColor(issue.severity)}"
									>{issue.severity}</span
								>
								<span class="type">{issue.issue_type.replace('_', ' ')}</span>
								<span class="date">{formatDate(issue.created_at)}</span>
							</div>
						</div>
						<div class="issue-status">
							<select
								value={issue.status}
								onchange={(e) => updateIssueStatus(issue, e.currentTarget.value)}
								style="color: {getStatusColor(issue.status)}"
							>
								{#each issueStatuses as status (status)}
									<option value={status}>{status.replace('_', ' ')}</option>
								{/each}
							</select>
						</div>
					</div>
					{#if issue.description}
						<p class="issue-description">{issue.description}</p>
					{/if}
					{#if issue.resolution_note}
						<div class="resolution-note">
							<strong>Resolution:</strong>
							{issue.resolution_note}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.issues-view {
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-primary);
	}

	.issues-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--spacing-md) var(--spacing-lg);
		border-bottom: 1px solid var(--color-border);
	}

	.issues-header h2 {
		font-size: var(--font-size-lg);
		font-weight: 600;
	}

	.header-actions {
		display: flex;
		gap: var(--spacing-sm);
		align-items: center;
	}

	.filter-select {
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.add-btn {
		display: flex;
		align-items: center;
		gap: var(--spacing-xs);
		padding: var(--spacing-xs) var(--spacing-md);
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-md);
		font-size: var(--font-size-sm);
		font-weight: 500;
	}

	.add-btn:hover {
		background-color: var(--color-accent-hover);
	}

	.loading,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		flex: 1;
		text-align: center;
		color: var(--color-text-muted);
		padding: var(--spacing-xl);
	}

	.empty-state svg {
		opacity: 0.5;
		margin-bottom: var(--spacing-md);
	}

	.empty-state h3 {
		font-size: var(--font-size-lg);
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-sm);
	}

	.primary-btn {
		margin-top: var(--spacing-lg);
		padding: var(--spacing-sm) var(--spacing-lg);
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-md);
		font-weight: 500;
	}

	.issues-list {
		flex: 1;
		overflow-y: auto;
		padding: var(--spacing-lg);
		display: flex;
		flex-direction: column;
		gap: var(--spacing-md);
	}

	.issue-card {
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-md);
		padding: var(--spacing-md);
	}

	.issue-card.resolved {
		opacity: 0.7;
	}

	.issue-header {
		display: flex;
		align-items: flex-start;
		gap: var(--spacing-md);
	}

	.issue-type {
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.issue-info {
		flex: 1;
	}

	.issue-info h3 {
		font-size: var(--font-size-base);
		font-weight: 500;
		margin-bottom: var(--spacing-xs);
	}

	.issue-meta {
		display: flex;
		gap: var(--spacing-md);
		font-size: var(--font-size-xs);
	}

	.severity {
		font-weight: 600;
		text-transform: uppercase;
	}

	.type {
		color: var(--color-text-secondary);
		text-transform: capitalize;
	}

	.date {
		color: var(--color-text-muted);
	}

	.issue-status select {
		padding: var(--spacing-xs) var(--spacing-sm);
		background-color: transparent;
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-xs);
		font-weight: 500;
		text-transform: capitalize;
		cursor: pointer;
	}

	.issue-description {
		margin-top: var(--spacing-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		padding-left: calc(20px + var(--spacing-md));
	}

	.resolution-note {
		margin-top: var(--spacing-sm);
		padding: var(--spacing-sm);
		background-color: var(--color-bg-tertiary);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-secondary);
		margin-left: calc(20px + var(--spacing-md));
	}

	.form-group {
		margin-bottom: var(--spacing-md);
	}

	.form-group.flex-1 {
		flex: 1;
	}

	.form-group label {
		display: block;
		font-size: var(--font-size-xs);
		font-weight: 500;
		color: var(--color-text-secondary);
		margin-bottom: var(--spacing-xs);
	}

	.form-group input[type='text'],
	.form-group textarea,
	.form-group select {
		width: 100%;
		padding: var(--spacing-sm);
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: var(--border-radius-sm);
		font-size: var(--font-size-sm);
		color: var(--color-text-primary);
	}

	.form-row {
		display: flex;
		gap: var(--spacing-md);
	}

	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--spacing-sm);
		margin-top: var(--spacing-md);
	}

	.cancel-btn {
		padding: var(--spacing-xs) var(--spacing-md);
		color: var(--color-text-secondary);
		border-radius: var(--border-radius-sm);
	}

	.cancel-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.save-btn {
		padding: var(--spacing-xs) var(--spacing-md);
		background-color: var(--color-accent);
		color: white;
		border-radius: var(--border-radius-sm);
		font-weight: 500;
	}

	.save-btn:hover:not(:disabled) {
		background-color: var(--color-accent-hover);
	}

	.save-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
